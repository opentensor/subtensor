//! Subtensor Primitives for IO
//!
//! This crate contains interfaces for the runtime to communicate with the outside world, ergo `io`.
//! In other context, such interfaces are referred to as "**host functions**".
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use codec::{Decode, Encode};
use sp_runtime_interface::{
    pass_by::{
        AllocateAndReturnByCodec, PassFatPointerAndRead, PassPointerAndRead, PassPointerAndWrite,
    },
    runtime_interface,
};
use stp_shield::ShieldKeystoreExt;

use alloc::{
    string::{String, ToString},
    vec::Vec,
};

#[cfg(not(substrate_runtime))]
use sp_externalities::ExternalitiesExt;

#[derive(Debug, Encode, Decode)]
pub enum Error {
    Crypto(String),
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::Crypto(e) => write!(f, "Crypto error: {}", e),
        }
    }
}

/// Interfaces for working with crypto withing the runtime.
#[runtime_interface]
pub trait Crypto {
    /// Decapsulate a ciphertext using the ML-KEM-768 algorithm.
    #[allow(clippy::expect_used)]
    fn mlkem768_decapsulate(
        &mut self,
        ciphertext: PassFatPointerAndRead<&[u8]>,
        buffer: PassPointerAndWrite<&mut [u8; 32], 32>,
    ) -> AllocateAndReturnByCodec<Result<(), Error>> {
        let result = &self
            .extension::<ShieldKeystoreExt>()
            .expect("No `shield keystore` associated for the current context!")
            .mlkem768_decapsulate(ciphertext)
            .map_err(|e| Error::Crypto(e.to_string()))?;
        buffer.copy_from_slice(result);
        Ok(())
    }

    /// Decrypt a ciphertext using the XChaCha20-Poly1305 AEAD scheme.
    #[allow(clippy::expect_used)]
    fn aead_decrypt(
        &mut self,
        key: PassPointerAndRead<&[u8; 32], 32>,
        nonce: PassPointerAndRead<&[u8; 24], 24>,
        msg: PassFatPointerAndRead<&[u8]>,
        aad: PassFatPointerAndRead<&[u8]>,
    ) -> AllocateAndReturnByCodec<Result<Vec<u8>, Error>> {
        self.extension::<ShieldKeystoreExt>()
            .expect("No `shield keystore` associated for the current context!")
            .aead_decrypt(*key, *nonce, msg, aad)
            .map_err(|e| Error::Crypto(e.to_string()))
    }
}

/// The host functions Subtensor provides for the Wasm runtime environment.
///
/// All these host functions will be callable from inside the Wasm environment.
#[cfg(not(substrate_runtime))]
pub type SubtensorHostFunctions = (crypto::HostFunctions,);
