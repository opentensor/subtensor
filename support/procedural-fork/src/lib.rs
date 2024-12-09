//! This crate is a fork of the `frame-support-procedural` crate from
//! `substrate/frame/support/procedural` in `polkadot-sdk`. The purpose of this fork is to
//! re-export all parsing code from the original crate to make it accessible to other crates,
//! since the original crate is a `proc-macro` crate and therefore cannot have any non-macro
//! public exports. If Parity ever decides to move the parsing code to a separate crate, this
//! fork will no longer need to exist, but right now this is the only reliable way to get
//! access to the core parsing logic of substrate.
//!
//! Tags will be created for each major version of `polkadot-sdk` that `subtensor` relies on,
//! on an as-needed, ad-hoc basis, and versions will matched the corresponding `polkadot-sdk`
//! version/tag name.
#![recursion_limit = "512"]
#![allow(warnings)]
#![allow(clippy::all)]
#![ignore] // ensure procedural-fork tests are not run

extern crate proc_macro;

mod benchmark;
mod construct_runtime;
mod crate_version;
mod derive_impl;
mod dummy_part_checker;
mod dynamic_params;
mod key_prefix;
mod match_and_insert;
mod no_bound;
mod pallet;
mod pallet_error;
mod runtime;
mod storage_alias;
mod transactional;
mod tt_macro;

use std::{
    cell::RefCell,
    env::{set_var, var},
    path::PathBuf,
    str::FromStr,
    sync::Mutex,
};

pub(crate) const INHERENT_INSTANCE_NAME: &str = "__InherentHiddenInstance";

/// The number of module instances supported by the runtime, starting at index 1,
/// and up to `NUMBER_OF_INSTANCE`.
pub(crate) const NUMBER_OF_INSTANCE: u8 = 16;

thread_local! {
    /// A global counter, can be used to generate a relatively unique identifier.
    static COUNTER: RefCell<Counter> = const { RefCell::new(Counter(0)) };
}

/// Counter to generate a relatively unique identifier for macros. This is necessary because
/// declarative macros gets hoisted to the crate root, which shares the namespace with other pallets
/// containing the very same macros.
struct Counter(u64);

impl Counter {
    fn inc(&mut self) -> u64 {
        let ret = self.0;
        self.0 += 1;
        ret
    }
}

/// Get the value from the given environment variable set by cargo.
///
/// The value is parsed into the requested destination type.
fn get_cargo_env_var<T: FromStr>(version_env: &str) -> std::result::Result<T, ()> {
    let version = std::env::var(version_env)
        .unwrap_or_else(|_| panic!("`{}` is always set by cargo; qed", version_env));

    T::from_str(&version).map_err(drop)
}

/// Generate the counter_prefix related to the storage.
/// counter_prefix is used by counted storage map.
fn counter_prefix(prefix: &str) -> String {
    format!("CounterFor{}", prefix)
}

/// Improvement on [`exports::simulate_manifest_dir`] that allows for an arbitrary return type
pub fn simulate_manifest_dir<P, F, R>(path: P, closure: F) -> R
where
    P: AsRef<std::path::Path>,
    F: FnOnce() -> R + std::panic::UnwindSafe,
{
    static MANIFEST_DIR_LOCK: Mutex<()> = Mutex::new(());
    let guard = MANIFEST_DIR_LOCK.lock().unwrap();
    let orig = PathBuf::from(
        var("CARGO_MANIFEST_DIR").expect("failed to read ENV var `CARGO_MANIFEST_DIR`"),
    );
    set_var("CARGO_MANIFEST_DIR", orig.join(path.as_ref()));
    let result = std::panic::catch_unwind(closure);
    set_var("CARGO_MANIFEST_DIR", &orig);
    drop(guard);
    result.unwrap()
}

#[cfg(not(doc))]
pub mod exports {
    pub use crate::pallet::parse::tests::simulate_manifest_dir;

    pub mod benchmark {
        pub use crate::benchmark::*;
    }

    pub mod crate_version {
        pub use crate::crate_version::*;
    }

    pub mod derive_impl {
        pub use crate::derive_impl::*;
    }

    pub mod dummy_part_checker {
        pub use crate::dummy_part_checker::*;
    }

    pub mod dynamic_params {
        pub use crate::dynamic_params::*;
    }

    pub mod key_prefix {
        pub use crate::key_prefix::*;
    }

    pub mod match_and_insert {
        pub use crate::match_and_insert::*;
    }

    pub mod pallet_error {
        pub use crate::pallet_error::*;
    }

    pub mod storage_alias {
        pub use crate::storage_alias::*;
    }

    pub mod transactional {
        pub use crate::transactional::*;
    }

    pub mod tt_macro {
        pub use crate::tt_macro::*;
    }

    pub mod construct_runtime {
        pub use crate::construct_runtime::*;

        pub mod parse {
            pub use crate::construct_runtime::parse::*;
        }

        pub mod expand {
            pub use crate::construct_runtime::expand::*;
        }
    }

    pub mod no_bound {
        pub mod clone {
            pub use crate::no_bound::clone::*;
        }

        pub mod debug {
            pub use crate::no_bound::debug::*;
        }

        pub mod default {
            pub use crate::no_bound::default::*;
        }

        pub mod ord {
            pub use crate::no_bound::ord::*;
        }

        pub mod partial_eq {
            pub use crate::no_bound::partial_eq::*;
        }

        pub mod partial_ord {
            pub use crate::no_bound::partial_ord::*;
        }
    }

    pub mod pallet {
        pub use crate::pallet::*;

        pub mod parse {
            pub use crate::pallet::parse::*;
        }
    }
}
