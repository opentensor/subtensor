//! Compare two weights.rs files and report drift.
//!
//! Parses both files with `syn`, extracts per-function weight data including
//! base values and per-component (parameterized) slopes, then compares with
//! a configurable percentage threshold.
//!
//! Exit codes:
//!   0 — all within threshold
//!   1 — error
//!   2 — drift exceeds threshold

use anyhow::{Context, Result};
use clap::Parser;
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use syn::{Expr, File, ImplItem, Item, Lit, ReturnType, Stmt};

#[derive(Parser)]
#[command(about = "Compare two weights.rs files and report drift")]
struct Cli {
    #[arg(long)]
    old: PathBuf,
    #[arg(long)]
    new: PathBuf,
    #[arg(long, default_value = "40")]
    threshold: f64,
}

/// All weight data for a single function.
#[derive(Debug, Clone, Default)]
struct WeightValues {
    base_weight: u64,
    proof_size: u64,
    base_reads: u64,
    base_writes: u64,
    /// Per-component slopes, ordered by appearance in the chain.
    component_weights: Vec<u64>,
    component_reads: Vec<u64>,
    component_writes: Vec<u64>,
    component_proof: Vec<u64>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let old_weights = parse_file(&cli.old)?;
    let new_weights = parse_file(&cli.new)?;

    let mut any_drift = false;

    for (name, nw) in &new_weights {
        let ow = old_weights.get(name).cloned().unwrap_or_default();
        let mut reasons: Vec<String> = Vec::new();

        // Base weight: threshold-based
        check_pct(
            ow.base_weight,
            nw.base_weight,
            cli.threshold,
            "base_weight",
            &mut reasons,
        );

        // Base reads/writes: exact
        if nw.base_reads != ow.base_reads {
            reasons.push(format!("reads {} -> {}", ow.base_reads, nw.base_reads));
        }
        if nw.base_writes != ow.base_writes {
            reasons.push(format!("writes {} -> {}", ow.base_writes, nw.base_writes));
        }

        // Component slopes: threshold-based for weights, exact for reads/writes
        check_vec_pct(
            &ow.component_weights,
            &nw.component_weights,
            cli.threshold,
            "comp_weight",
            &mut reasons,
        );
        check_vec_exact(
            &ow.component_reads,
            &nw.component_reads,
            "comp_reads",
            &mut reasons,
        );
        check_vec_exact(
            &ow.component_writes,
            &nw.component_writes,
            "comp_writes",
            &mut reasons,
        );

        let drifted = !reasons.is_empty();
        if drifted {
            any_drift = true;
        }

        let pct = if ow.base_weight > 0 {
            (nw.base_weight as f64 - ow.base_weight as f64) / ow.base_weight as f64 * 100.0
        } else if nw.base_weight > 0 {
            100.0
        } else {
            0.0
        };

        let icon = if drifted { "\u{274c}" } else { "\u{2705}" };

        println!(
            "  {} {:<40} {:>12} -> {:<12} ({:>+.1}%)  reads {:>4} -> {:<4}  writes {:>4} -> {:<4}",
            icon,
            name,
            ow.base_weight,
            nw.base_weight,
            pct,
            ow.base_reads,
            nw.base_reads,
            ow.base_writes,
            nw.base_writes,
        );
    }

    for name in old_weights.keys() {
        if !new_weights.contains_key(name) {
            println!("  \u{274c} {:<40} REMOVED", name);
            any_drift = true;
        }
    }

    if any_drift {
        std::process::exit(2);
    }

    Ok(())
}

fn check_pct(old: u64, new: u64, threshold: f64, label: &str, reasons: &mut Vec<String>) {
    let drift = if old > 0 {
        ((new as f64 - old as f64) / old as f64 * 100.0).abs()
    } else if new > 0 {
        100.0
    } else {
        return;
    };
    if drift > threshold {
        reasons.push(format!("{label} {drift:.1}%"));
    }
}

fn check_vec_pct(old: &[u64], new: &[u64], threshold: f64, label: &str, reasons: &mut Vec<String>) {
    if old.len() != new.len() {
        reasons.push(format!("{label} count {} -> {}", old.len(), new.len()));
        return;
    }
    for (i, (o, n)) in old.iter().zip(new.iter()).enumerate() {
        check_pct(*o, *n, threshold, &format!("{label}[{i}]"), reasons);
    }
}

fn check_vec_exact(old: &[u64], new: &[u64], label: &str, reasons: &mut Vec<String>) {
    if old != new {
        reasons.push(format!("{label} {:?} -> {:?}", old, new));
    }
}

// ── Parsing ─────────────────────────────────────────────────────────────────

fn parse_file(path: &PathBuf) -> Result<BTreeMap<String, WeightValues>> {
    let src = fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    let file: File = syn::parse_str(&src).with_context(|| format!("parsing {}", path.display()))?;
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
            if !matches!(&method.sig.output, ReturnType::Type(..)) {
                continue;
            }
            let mut wv = WeightValues::default();
            for stmt in &method.block.stmts {
                if let Stmt::Expr(expr, _) = stmt {
                    walk(expr, &mut wv, false);
                }
            }
            result.insert(method.sig.ident.to_string(), wv);
        }
        break;
    }

    Ok(result)
}

fn is_substrate_weight_impl(impl_block: &syn::ItemImpl) -> bool {
    let self_ty = match impl_block.self_ty.as_ref() {
        syn::Type::Path(tp) => tp
            .path
            .segments
            .iter()
            .map(|s| s.ident.to_string())
            .collect::<String>(),
        _ => return false,
    };
    if !self_ty.contains("SubstrateWeight") {
        return false;
    }
    impl_block
        .trait_
        .as_ref()
        .and_then(|(_, path, _)| path.segments.last())
        .map(|s| s.ident == "WeightInfo")
        .unwrap_or(false)
}

/// Walk a method-call chain. `in_mul` tracks whether we're inside `.saturating_mul()`,
/// which means values are per-component slopes rather than base values.
fn walk(expr: &Expr, wv: &mut WeightValues, in_mul: bool) {
    match expr {
        Expr::Call(call) => {
            if is_from_parts(&call.func) && call.args.len() >= 2 {
                let v = expr_to_u64(&call.args[0]).unwrap_or(0);
                let p = expr_to_u64(&call.args[1]).unwrap_or(0);
                if in_mul {
                    if v > 0 {
                        wv.component_weights.push(v);
                    }
                    if p > 0 {
                        wv.component_proof.push(p);
                    }
                } else {
                    wv.base_weight = v;
                    wv.proof_size = p;
                }
            }
        }
        Expr::MethodCall(mc) => {
            match mc.method.to_string().as_str() {
                "saturating_add" => {
                    walk(&mc.receiver, wv, in_mul);
                    if let Some(arg) = mc.args.first() {
                        walk(arg, wv, in_mul);
                    }
                }
                "saturating_mul" => {
                    // Receiver is the component slope value
                    walk(&mc.receiver, wv, true);
                }
                "reads" => {
                    extract_rw(
                        mc.args.first(),
                        in_mul,
                        &mut wv.base_reads,
                        &mut wv.component_reads,
                    );
                    walk(&mc.receiver, wv, in_mul);
                }
                "writes" => {
                    extract_rw(
                        mc.args.first(),
                        in_mul,
                        &mut wv.base_writes,
                        &mut wv.component_writes,
                    );
                    walk(&mc.receiver, wv, in_mul);
                }
                _ => walk(&mc.receiver, wv, in_mul),
            }
        }
        Expr::Paren(p) => walk(&p.expr, wv, in_mul),
        _ => {}
    }
}

/// Extract a reads/writes value from the argument to `.reads()` / `.writes()`.
/// Handles both plain literals and `(N_u64).saturating_mul(k.into())` patterns.
fn extract_rw(arg: Option<&Expr>, in_mul: bool, base: &mut u64, components: &mut Vec<u64>) {
    let Some(arg) = arg else { return };

    // Direct literal: .reads(2_u64)
    if let Some(n) = expr_to_u64(arg) {
        if in_mul {
            components.push(n);
        } else {
            *base += n;
        }
        return;
    }

    // Component pattern: .reads((2_u64).saturating_mul(k.into()))
    if let Expr::MethodCall(inner_mc) = arg
        && inner_mc.method == "saturating_mul"
        && let Some(n) = expr_to_u64(&inner_mc.receiver)
    {
        components.push(n);
    }
}

fn is_from_parts(func: &Expr) -> bool {
    matches!(func, Expr::Path(p) if p.path.segments.last().map(|s| s.ident == "from_parts").unwrap_or(false))
}

fn expr_to_u64(expr: &Expr) -> Option<u64> {
    match expr {
        Expr::Lit(lit) => match &lit.lit {
            Lit::Int(i) => i.base10_parse::<u64>().ok(),
            _ => None,
        },
        Expr::Cast(c) => expr_to_u64(&c.expr),
        Expr::MethodCall(mc) if mc.method == "into" => expr_to_u64(&mc.receiver),
        Expr::Paren(p) => expr_to_u64(&p.expr),
        _ => None,
    }
}
