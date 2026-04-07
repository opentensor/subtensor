// pallets/mev-shield/src/lib.rs
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use chacha20poly1305::{
    KeyInit, XChaCha20Poly1305, XNonce,
    aead::{Aead, Payload},
};
use frame_support::{pallet_prelude::*, traits::IsSubType};
use frame_system::{ensure_none, ensure_signed, pallet_prelude::*};
use ml_kem::{
    Ciphertext, EncodedSizeUser, MlKem768, MlKem768Params,
    kem::{Decapsulate, DecapsulationKey},
};
use sp_io::hashing::twox_128;
use sp_runtime::traits::{Applyable, Block as BlockT, Checkable, Hash};
use stp_shield::{
    INHERENT_IDENTIFIER, InherentType, LOG_TARGET, MLKEM768_ENC_KEY_LEN, ShieldEncKey,
    ShieldedTransaction,
};

use alloc::vec;

pub use pallet::*;

pub mod weights;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[cfg(test)]
pub mod mock;

#[cfg(test)]
mod tests;

mod extension;
mod migrations;
pub use extension::CheckShieldedTxValidity;

type MigrationKeyMaxLen = ConstU32<128>;

type ExtrinsicOf<Block> = <Block as BlockT>::Extrinsic;
type CheckedOf<T, Context> = <T as Checkable<Context>>::Checked;
type ApplyableCallOf<T> = <T as Applyable>::Call;

const MAX_EXTRINSIC_DEPTH: u32 = 8;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use crate::weights::WeightInfo;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The identifier type for an authority.
        type AuthorityId: Member + Parameter + MaybeSerializeDeserialize + MaxEncodedLen;

        /// A way to find the current and next block author.
        type FindAuthors: FindAuthors<Self>;

        /// Weight information for extrinsics in this pallet.
        type WeightInfo: crate::weights::WeightInfo;
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    /// Current block author's ML-KEM-768 encapsulation key (internal, not for encryption).
    #[pallet::storage]
    pub type CurrentKey<T> = StorageValue<_, ShieldEncKey, OptionQuery>;

    /// Next block author's key, staged here before promoting to `CurrentKey`.
    #[pallet::storage]
    pub type PendingKey<T> = StorageValue<_, ShieldEncKey, OptionQuery>;

    /// Key users should encrypt with (N+2 author's key).
    #[pallet::storage]
    pub type NextKey<T> = StorageValue<_, ShieldEncKey, OptionQuery>;

    /// Per-author ML-KEM-768 encapsulation key, updated each time the author produces a block.
    #[pallet::storage]
    pub type AuthorKeys<T: Config> =
        StorageMap<_, Twox64Concat, T::AuthorityId, ShieldEncKey, OptionQuery>;

    /// Block number at which `PendingKey` is no longer valid (exclusive upper bound).
    /// Updated every block during rotation.
    #[pallet::storage]
    pub type PendingKeyExpiresAt<T: Config> = StorageValue<_, BlockNumberFor<T>, OptionQuery>;

    /// Block number at which `NextKey` is no longer valid (exclusive upper bound).
    /// Updated every block during rotation.
    #[pallet::storage]
    pub type NextKeyExpiresAt<T: Config> = StorageValue<_, BlockNumberFor<T>, OptionQuery>;

    /// Stores whether some migration has been run.
    #[pallet::storage]
    pub type HasMigrationRun<T: Config> =
        StorageMap<_, Identity, BoundedVec<u8, MigrationKeyMaxLen>, bool, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Encrypted wrapper accepted.
        EncryptedSubmitted { id: T::Hash, who: T::AccountId },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// The announced ML‑KEM encapsulation key length is invalid.
        BadEncKeyLen,
        /// Unreachable.
        Unreachable,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_runtime_upgrade() -> frame_support::weights::Weight {
            let mut weight = frame_support::weights::Weight::from_parts(0, 0);

            weight = weight.saturating_add(
                migrations::migrate_clear_v1_storage::migrate_clear_v1_storage::<T>(),
            );

            weight
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Rotate the key chain and announce the current author's ML-KEM encapsulation key.
        ///
        /// Called as an inherent every block. `enc_key` is `None` on node failure,
        /// which removes the author from future shielded tx eligibility.
        ///
        /// Key rotation order (using pre-update AuthorKeys):
        ///   1. CurrentKey  ← PendingKey
        ///   2. PendingKey  ← NextKey
        ///   3. NextKey     ← next-next author's key  (user-facing)
        ///   4. AuthorKeys[current] ← announced key
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::announce_next_key())]
        pub fn announce_next_key(
            origin: OriginFor<T>,
            enc_key: Option<ShieldEncKey>,
        ) -> DispatchResult {
            ensure_none(origin)?;

            let author = T::FindAuthors::find_current_author().ok_or(Error::<T>::Unreachable)?;
            let now = <frame_system::Pallet<T>>::block_number();

            // 1. CurrentKey ← PendingKey
            if let Some(pending_key) = PendingKey::<T>::take() {
                CurrentKey::<T>::put(pending_key);
            } else {
                CurrentKey::<T>::kill();
            }

            // 2. PendingKey ← NextKey (what was N+2 last block is now N+1)
            if let Some(next_key) = NextKey::<T>::take() {
                PendingKey::<T>::put(next_key);
            } else {
                PendingKey::<T>::kill();
            }

            // 3. NextKey ← next-next author's key
            if let Some(next_next_author) = T::FindAuthors::find_next_next_author()
                && let Some(key) = AuthorKeys::<T>::get(&next_next_author)
            {
                NextKey::<T>::put(key);
            } else {
                NextKey::<T>::kill();
            }

            // 4. Update AuthorKeys after rotations for consistent reads above.
            if let Some(enc_key) = &enc_key {
                ensure!(
                    enc_key.len() == MLKEM768_ENC_KEY_LEN,
                    Error::<T>::BadEncKeyLen
                );
                AuthorKeys::<T>::insert(&author, enc_key.clone());
            } else {
                AuthorKeys::<T>::remove(&author);
            }

            // 5. Set expiration blocks for user-facing keys.
            if PendingKey::<T>::get().is_some() {
                PendingKeyExpiresAt::<T>::put(now + 2u32.into());
            } else {
                PendingKeyExpiresAt::<T>::kill();
            }
            if NextKey::<T>::get().is_some() {
                NextKeyExpiresAt::<T>::put(now + 3u32.into());
            } else {
                NextKeyExpiresAt::<T>::kill();
            }

            Ok(())
        }

        /// Users submit an encrypted wrapper.
        ///
        /// Client‑side:
        ///
        ///   1. Read `NextKey` (ML‑KEM encapsulation key bytes) from storage.
        ///   2. Sign your extrinsic so that it can be executed when added to the pool,
        ///        i.e. you may need to increment the nonce if you submit using the same account.
        ///   3. Encrypt:
        ///
        ///        plaintext = signed_extrinsic
        ///        key_hash = xxhash128(NextKey)
        ///        kem_len = Length of kem_ct in bytes (u16)
        ///        kem_ct = Ciphertext from ML‑KEM‑768
        ///        nonce = Random 24 bytes used for XChaCha20‑Poly1305
        ///        aead_ct = Ciphertext from XChaCha20‑Poly1305
        ///
        ///      with ML‑KEM‑768 + XChaCha20‑Poly1305, producing
        ///
        ///        ciphertext = key_hash || kem_len || kem_ct || nonce || aead_ct
        ///
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::submit_encrypted())]
        pub fn submit_encrypted(
            origin: OriginFor<T>,
            ciphertext: BoundedVec<u8, ConstU32<8192>>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let id: T::Hash = T::Hashing::hash_of(&(who.clone(), &ciphertext));

            Self::deposit_event(Event::EncryptedSubmitted { id, who });
            Ok(())
        }
    }

    #[pallet::inherent]
    impl<T: Config> ProvideInherent for Pallet<T> {
        type Call = Call<T>;
        type Error = sp_inherents::MakeFatalError<()>;

        const INHERENT_IDENTIFIER: InherentIdentifier = INHERENT_IDENTIFIER;

        fn create_inherent(data: &InherentData) -> Option<Self::Call> {
            let enc_key = data
                .get_data::<InherentType>(&INHERENT_IDENTIFIER)
                .inspect_err(
                    |e| log::debug!(target: LOG_TARGET, "Failed to get shielded enc key inherent data: {:?}", e),
                )
                .ok()??;
            Some(Call::announce_next_key { enc_key })
        }

        fn is_inherent(call: &Self::Call) -> bool {
            matches!(call, Call::announce_next_key { .. })
        }
    }
}

impl<T: Config> Pallet<T> {
    pub fn try_decode_shielded_tx<Block: BlockT, Context: Default>(
        uxt: ExtrinsicOf<Block>,
    ) -> Option<ShieldedTransaction>
    where
        Block::Extrinsic: Checkable<Context>,
        CheckedOf<Block::Extrinsic, Context>: Applyable,
        ApplyableCallOf<CheckedOf<Block::Extrinsic, Context>>: IsSubType<Call<T>>,
    {
        // Prevent stack overflows by limiting the depth of the extrinsic.
        let encoded = uxt.encode();
        let uxt = <Block::Extrinsic as codec::DecodeLimit>::decode_all_with_depth_limit(
            MAX_EXTRINSIC_DEPTH,
            &mut &encoded[..],
        )
        .inspect_err(
            |e| log::debug!(target: LOG_TARGET, "Failed to decode shielded extrinsic: {:?}", e),
        )
        .ok()?;

        // Verify that the signature is correct.
        let xt = ExtrinsicOf::<Block>::check(uxt, &Context::default())
            .inspect_err(
                |e| log::debug!(target: LOG_TARGET, "Failed to check shielded extrinsic: {:?}", e),
            )
            .ok()?;
        let call = xt.call();

        let Some(Call::submit_encrypted { ciphertext }) = IsSubType::<Call<T>>::is_sub_type(call)
        else {
            return None;
        };

        ShieldedTransaction::parse(ciphertext)
    }

    pub fn is_shielded_using_current_key(key_hash: &[u8; 16]) -> bool {
        let pending = PendingKey::<T>::get();
        let pending_hash = pending.as_ref().map(|k| twox_128(&k[..]));
        pending_hash.as_ref() == Some(key_hash)
    }

    pub fn try_unshield_tx<Block: BlockT>(
        dec_key_bytes: alloc::vec::Vec<u8>,
        shielded_tx: ShieldedTransaction,
    ) -> Option<<Block as BlockT>::Extrinsic> {
        let plaintext = unshield(&dec_key_bytes, &shielded_tx).or_else(|| {
            log::debug!(target: LOG_TARGET, "Failed to unshield transaction");
            None
        })?;

        if plaintext.is_empty() {
            return None;
        }

        ExtrinsicOf::<Block>::decode(&mut &plaintext[..]).inspect_err(
            |e| log::debug!(target: LOG_TARGET, "Failed to decode shielded transaction: {:?}", e),
        ).ok()
    }
}

pub trait FindAuthors<T: Config> {
    fn find_current_author() -> Option<T::AuthorityId>;
    fn find_next_next_author() -> Option<T::AuthorityId>;
}

impl<T: Config> FindAuthors<T> for () {
    fn find_current_author() -> Option<T::AuthorityId> {
        None
    }
    fn find_next_next_author() -> Option<T::AuthorityId> {
        None
    }
}

/// Decrypt a shielded transaction using the raw decapsulation key bytes.
///
/// Performs ML-KEM-768 decapsulation followed by XChaCha20-Poly1305 AEAD decryption.
/// Runs entirely in WASM — no host functions needed.
fn unshield(
    dec_key_bytes: &[u8],
    shielded_tx: &ShieldedTransaction,
) -> Option<alloc::vec::Vec<u8>> {
    let dec_key = DecapsulationKey::<MlKem768Params>::from_bytes(dec_key_bytes.try_into().ok()?);
    let ciphertext = Ciphertext::<MlKem768>::try_from(shielded_tx.kem_ct.as_slice()).ok()?;
    let shared_secret = dec_key.decapsulate(&ciphertext).ok()?;

    let aead = XChaCha20Poly1305::new(shared_secret.as_slice().into());
    let nonce = XNonce::from_slice(&shielded_tx.nonce);
    aead.decrypt(
        nonce,
        Payload {
            msg: &shielded_tx.aead_ct,
            aad: &[],
        },
    )
    .ok()
}
