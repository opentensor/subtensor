//! Generate or update a placeholder weights.rs from a pallet's benchmarking file.
//!
//! Parses the benchmarking.rs with `syn` to discover `#[benchmark]` functions,
//! then reconciles with an existing weights.rs (if any):
//!
//!   - Benchmark exists + weight exists  → keep existing weight values
//!   - Benchmark exists + no weight      → add with placeholder values
//!   - No benchmark    + weight exists   → remove
//!
//! Usage:
//!   weight-stub --benchmarks pallets/foo/src/benchmarking.rs \
//!               --output pallets/foo/src/weights.rs \
//!               --pallet pallet_foo

use anyhow::{Context, Result};
use clap::Parser;
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use syn::{Expr, File, ImplItem, Item, ItemMod, Lit, ReturnType, Stmt};

#[derive(Parser)]
#[command(about = "Generate or update placeholder weights.rs from a benchmarking file")]
struct Cli {
    /// Path to the benchmarking.rs file
    #[arg(long)]
    benchmarks: PathBuf,

    /// Output path for the weights.rs (also read as the existing file if present)
    #[arg(long)]
    output: PathBuf,

    /// Pallet name (e.g., pallet_subtensor)
    #[arg(long)]
    pallet: String,
}

// ── Benchmark extraction ────────────────────────────────────────────────────

#[derive(Debug, Clone)]
struct BenchmarkFn {
    name: String,
    params: Vec<String>,
}

fn extract_benchmarks(src: &str) -> Result<Vec<BenchmarkFn>> {
    let file: File = syn::parse_str(src).context("syn parse failed")?;
    let mut results = Vec::new();

    for item in &file.items {
        match item {
            Item::Fn(f) => {
                if has_benchmark_attr(&f.attrs)
                    && let Some(b) = extract_benchmark_fn_info(&f.sig)
                {
                    results.push(b);
                }
            }
            Item::Mod(m) => collect_benchmarks_from_mod(m, &mut results),
            _ => {}
        }
    }

    Ok(results)
}

fn collect_benchmarks_from_mod(m: &ItemMod, results: &mut Vec<BenchmarkFn>) {
    let Some((_, items)) = &m.content else {
        return;
    };
    for item in items {
        match item {
            Item::Fn(f) => {
                if has_benchmark_attr(&f.attrs)
                    && let Some(b) = extract_benchmark_fn_info(&f.sig)
                {
                    results.push(b);
                }
            }
            Item::Mod(inner) => collect_benchmarks_from_mod(inner, results),
            _ => {}
        }
    }
}

fn has_benchmark_attr(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|attr| {
        attr.path()
            .segments
            .last()
            .map(|s| s.ident == "benchmark")
            .unwrap_or(false)
    })
}

fn extract_benchmark_fn_info(sig: &syn::Signature) -> Option<BenchmarkFn> {
    let name = sig.ident.to_string();

    let mut params = Vec::new();
    for input in &sig.inputs {
        if let syn::FnArg::Typed(pat_type) = input
            && let syn::Pat::Ident(pat_ident) = pat_type.pat.as_ref()
        {
            let param_name = pat_ident.ident.to_string();
            let ty_str = type_to_string(&pat_type.ty);
            if ty_str.contains("Linear") || ty_str.contains("ParamRange") {
                params.push(param_name);
            }
        }
    }

    Some(BenchmarkFn { name, params })
}

fn type_to_string(ty: &syn::Type) -> String {
    match ty {
        syn::Type::Path(tp) => tp
            .path
            .segments
            .iter()
            .map(|s| s.ident.to_string())
            .collect::<Vec<_>>()
            .join("::"),
        _ => String::new(),
    }
}

// ── Existing weight extraction (reused from weight_compare logic) ───────────

#[derive(Debug, Clone)]
struct WeightValues {
    base_weight: u64,
    proof_size: u64,
    reads: u64,
    writes: u64,
}

impl Default for WeightValues {
    fn default() -> Self {
        Self {
            base_weight: 10_000_000,
            proof_size: 0,
            reads: 1,
            writes: 1,
        }
    }
}

fn parse_existing_weights(src: &str) -> Result<BTreeMap<String, WeightValues>> {
    let file: File = syn::parse_str(src).context("syn parse failed")?;
    let mut result = BTreeMap::new();

    for item in &file.items {
        let Item::Impl(impl_block) = item else {
            continue;
        };

        if !is_substrate_weight_impl(impl_block) {
            continue;
        }

        for impl_item in &impl_block.items {
            let ImplItem::Fn(method) = impl_item else {
                continue;
            };

            let fn_name = method.sig.ident.to_string();

            if !matches!(&method.sig.output, ReturnType::Type(..)) {
                continue;
            }

            let mut values = WeightValues {
                base_weight: 0,
                proof_size: 0,
                reads: 0,
                writes: 0,
            };
            extract_from_block(&method.block.stmts, &mut values);
            result.insert(fn_name, values);
        }

        break;
    }

    Ok(result)
}

fn is_substrate_weight_impl(impl_block: &syn::ItemImpl) -> bool {
    let self_ty = display_type(&impl_block.self_ty);
    if !self_ty.contains("SubstrateWeight") {
        return false;
    }
    if let Some((_, path, _)) = &impl_block.trait_ {
        return path
            .segments
            .last()
            .map(|s| s.ident == "WeightInfo")
            .unwrap_or(false);
    }
    false
}

fn extract_from_block(stmts: &[Stmt], values: &mut WeightValues) {
    for stmt in stmts {
        match stmt {
            Stmt::Expr(expr, _) => extract_from_expr(expr, values),
            Stmt::Local(local) => {
                if let Some(init) = &local.init {
                    extract_from_expr(&init.expr, values);
                }
            }
            _ => {}
        }
    }
}

fn extract_from_expr(expr: &Expr, values: &mut WeightValues) {
    match expr {
        Expr::Call(call) => {
            if is_from_parts_call(&call.func) && call.args.len() >= 2 {
                if let Some(base) = expr_to_u64(&call.args[0]) {
                    values.base_weight = base;
                }
                if let Some(proof) = expr_to_u64(&call.args[1]) {
                    values.proof_size = proof;
                }
            }
            for arg in &call.args {
                extract_from_expr(arg, values);
            }
        }
        Expr::MethodCall(mc) => {
            match mc.method.to_string().as_str() {
                "reads" => {
                    if let Some(n) = mc.args.first().and_then(expr_to_u64) {
                        values.reads += n;
                    }
                }
                "writes" => {
                    if let Some(n) = mc.args.first().and_then(expr_to_u64) {
                        values.writes += n;
                    }
                }
                _ => {}
            }
            extract_from_expr(&mc.receiver, values);
            for arg in &mc.args {
                extract_from_expr(arg, values);
            }
        }
        Expr::Paren(p) => extract_from_expr(&p.expr, values),
        Expr::Block(b) => extract_from_block(&b.block.stmts, values),
        _ => {}
    }
}

fn is_from_parts_call(func: &Expr) -> bool {
    match func {
        Expr::Path(p) => p
            .path
            .segments
            .last()
            .map(|s| s.ident == "from_parts")
            .unwrap_or(false),
        _ => false,
    }
}

fn expr_to_u64(expr: &Expr) -> Option<u64> {
    match expr {
        Expr::Lit(lit) => match &lit.lit {
            Lit::Int(int_lit) => int_lit.base10_parse::<u64>().ok(),
            _ => None,
        },
        Expr::Cast(cast) => expr_to_u64(&cast.expr),
        Expr::MethodCall(mc) if mc.method == "into" => expr_to_u64(&mc.receiver),
        _ => None,
    }
}

fn display_type(ty: &syn::Type) -> String {
    match ty {
        syn::Type::Path(tp) => tp
            .path
            .segments
            .iter()
            .map(|s| s.ident.to_string())
            .collect::<Vec<_>>()
            .join(""),
        _ => String::from("?"),
    }
}

// ── Code generation ─────────────────────────────────────────────────────────

fn fmt_weight(w: u64) -> String {
    let s = w.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push('_');
        }
        result.push(c);
    }
    result.chars().rev().collect()
}

fn generate_weights_rs(
    pallet: &str,
    benchmarks: &[BenchmarkFn],
    existing: &BTreeMap<String, WeightValues>,
) -> String {
    let mut out = String::new();

    out.push_str(&format!(
        r#"// Weights for `{pallet}`.
//
// Stubbed entries were auto-generated by weight-stub.
// Re-generate with real values by running benchmarks:
//   ./scripts/benchmark_all.sh {pallet}

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(missing_docs)]
#![allow(dead_code)]

use frame_support::{{traits::Get, weights::{{Weight, constants::RocksDbWeight}}}};
use core::marker::PhantomData;

/// Weight functions needed for `{pallet}`.
pub trait WeightInfo {{
"#
    ));

    for b in benchmarks {
        out.push('\t');
        out.push_str(&fn_signature(b, false));
        out.push_str(";\n");
    }

    out.push_str("}\n\n");

    // SubstrateWeight<T> impl
    out.push_str(&format!(
        "/// Weights for `{pallet}` using the Substrate node and recommended hardware.\n\
         pub struct SubstrateWeight<T>(PhantomData<T>);\n\
         impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {{\n"
    ));

    for b in benchmarks {
        let w = existing.get(&b.name).cloned().unwrap_or_default();
        write_fn_body(&mut out, b, &w, "T::DbWeight::get()");
    }

    out.push_str("}\n\n");

    // () impl
    out.push_str("// For backwards compatibility and tests.\nimpl WeightInfo for () {\n");

    for b in benchmarks {
        let w = existing.get(&b.name).cloned().unwrap_or_default();
        write_fn_body(&mut out, b, &w, "RocksDbWeight::get()");
    }

    out.push_str("}\n");
    out
}

fn fn_signature(b: &BenchmarkFn, prefix_unused: bool) -> String {
    let mut s = format!("fn {}(", b.name);
    for (i, p) in b.params.iter().enumerate() {
        if i > 0 {
            s.push_str(", ");
        }
        if prefix_unused {
            s.push_str(&format!("_{p}: u32"));
        } else {
            s.push_str(&format!("{p}: u32"));
        }
    }
    s.push_str(") -> Weight");
    s
}

fn write_fn_body(out: &mut String, b: &BenchmarkFn, w: &WeightValues, db_weight: &str) {
    out.push('\t');
    out.push_str(&fn_signature(b, true));
    out.push_str(" {\n");

    out.push_str(&format!(
        "\t\tWeight::from_parts({}, {})\n",
        fmt_weight(w.base_weight),
        w.proof_size
    ));

    if w.reads > 0 {
        out.push_str(&format!(
            "\t\t\t.saturating_add({db_weight}.reads({}_u64))\n",
            w.reads
        ));
    }
    if w.writes > 0 {
        out.push_str(&format!(
            "\t\t\t.saturating_add({db_weight}.writes({}_u64))\n",
            w.writes
        ));
    }

    out.push_str("\t}\n");
}

// ── Main ────────────────────────────────────────────────────────────────────

fn main() -> Result<()> {
    let cli = Cli::parse();

    let bench_src = fs::read_to_string(&cli.benchmarks)
        .with_context(|| format!("reading {}", cli.benchmarks.display()))?;

    let benchmarks = extract_benchmarks(&bench_src)
        .with_context(|| format!("parsing {}", cli.benchmarks.display()))?;

    if benchmarks.is_empty() {
        anyhow::bail!(
            "no #[benchmark] functions found in {}",
            cli.benchmarks.display()
        );
    }

    // Load existing weights if the output file already exists
    let existing: BTreeMap<String, WeightValues> = if cli.output.exists() {
        let old_src = fs::read_to_string(&cli.output)
            .with_context(|| format!("reading existing {}", cli.output.display()))?;
        parse_existing_weights(&old_src).unwrap_or_default()
    } else {
        BTreeMap::new()
    };

    // Report what will happen
    let mut added = Vec::new();
    let mut kept = Vec::new();
    let mut removed = Vec::new();

    for b in &benchmarks {
        if existing.contains_key(&b.name) {
            kept.push(&b.name);
        } else {
            added.push(&b.name);
        }
    }

    let bench_names: std::collections::HashSet<&str> =
        benchmarks.iter().map(|b| b.name.as_str()).collect();
    for name in existing.keys() {
        if !bench_names.contains(name.as_str()) {
            removed.push(name.clone());
        }
    }

    let output = generate_weights_rs(&cli.pallet, &benchmarks, &existing);

    if let Some(parent) = cli.output.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&cli.output, &output).with_context(|| format!("writing {}", cli.output.display()))?;

    // Summary
    println!("Wrote {} to {}", benchmarks.len(), cli.output.display());
    if !kept.is_empty() {
        println!("  kept:    {} (existing values preserved)", kept.len());
    }
    if !added.is_empty() {
        println!("  added:   {} (placeholder values)", added.len());
        for name in &added {
            println!("           + {name}");
        }
    }
    if !removed.is_empty() {
        println!("  removed: {} (no longer in benchmarks)", removed.len());
        for name in &removed {
            println!("           - {name}");
        }
    }

    Ok(())
}
