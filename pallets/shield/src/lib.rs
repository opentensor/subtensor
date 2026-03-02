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
use sp_runtime::traits::{Applyable, Block as BlockT, Checkable, Hash};
use stp_shield::{
    INHERENT_IDENTIFIER, InherentType, LOG_TARGET, ShieldPublicKey, ShieldedTransaction,
};

use alloc::vec;

pub use pallet::*;

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

const MLKEM768_PK_LEN: usize = 1184;
const MAX_EXTRINSIC_DEPTH: u32 = 8;

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The identifier type for an authority.
        type AuthorityId: Member + Parameter + MaybeSerializeDeserialize + MaxEncodedLen;

        /// A way to find the current and next block author.
        type FindAuthors: FindAuthors<Self>;
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    // Current block author ML‑KEM‑768 public key bytes.
    //
    // Note: Do not use this to encrypt transactions as this
    // is only used to validate transactions in the extension.
    // Use `NextKey` instead.
    #[pallet::storage]
    pub type CurrentKey<T> = StorageValue<_, ShieldPublicKey, OptionQuery>;

    // Next block author ML‑KEM‑768 public key bytes.
    //
    // This is the key that should be used to encrypt transactions.
    #[pallet::storage]
    pub type NextKey<T> = StorageValue<_, ShieldPublicKey, OptionQuery>;

    /// Latest announced ML‑KEM‑768 public key per block author.
    /// This is the key the author will use for decapsulation in their next slot.
    #[pallet::storage]
    pub type AuthorKeys<T: Config> =
        StorageMap<_, Twox64Concat, T::AuthorityId, ShieldPublicKey, OptionQuery>;

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
        /// The announced ML‑KEM public key length is invalid.
        BadPublicKeyLen,
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
        /// Announce the ML‑KEM public key that will become `CurrentKey` in
        /// the next block the current author will produce.
        ///
        /// Note: The public key can be `None` if the author failed to include the key in the
        /// inherent data (which should never happen except node failure). In that case, we
        /// store the next key as `None` to reflect that this author will not be able
        /// handle encrypted transactions in his next block.
        #[pallet::call_index(0)]
        #[pallet::weight(Weight::from_parts(23_190_000, 0)
        .saturating_add(T::DbWeight::get().reads(2_u64))
        .saturating_add(T::DbWeight::get().writes(3_u64)))]
        pub fn announce_next_key(
            origin: OriginFor<T>,
            public_key: Option<ShieldPublicKey>,
        ) -> DispatchResult {
            ensure_none(origin)?;

            let author = T::FindAuthors::find_current_author()
                // This should never happen as we are in an inherent.
                .ok_or(Error::<T>::Unreachable)?;

            // Shift the key chain: Current ← NextKey.
            // NextKey was set in the previous block to be the current author's key,
            // so this naturally tracks the last 2 keys users may have encrypted with.
            CurrentKey::<T>::set(NextKey::<T>::get());

            if let Some(public_key) = &public_key {
                ensure!(
                    public_key.len() == MLKEM768_PK_LEN,
                    Error::<T>::BadPublicKeyLen
                );
                AuthorKeys::<T>::insert(&author, public_key.clone());
            } else {
                // If the author did not announce a key, remove his old key from storage,
                // he will not be able to accept shielded transactions in his next block.
                AuthorKeys::<T>::remove(&author);
            }

            // Expose the next block author's key so users can encrypt for them.
            NextKey::<T>::kill();
            if let Some(next_author) = T::FindAuthors::find_next_author()
                && let Some(key) = AuthorKeys::<T>::get(&next_author)
            {
                NextKey::<T>::put(key);
            }

            Ok(())
        }

        /// Users submit an encrypted wrapper.
        ///
        /// Client‑side:
        ///
        ///   1. Read `NextKey` (ML‑KEM public key bytes) from storage.
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
        #[pallet::weight(Weight::from_parts(207_500_000, 0)
        .saturating_add(T::DbWeight::get().reads(0_u64))
        .saturating_add(T::DbWeight::get().writes(0_u64)))]
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
            let public_key = data
                .get_data::<InherentType>(&INHERENT_IDENTIFIER)
                .inspect_err(
                    |e| log::debug!(target: LOG_TARGET, "Failed to get shielded public key inherent data: {:?}", e),
                )
                .ok()??;
            Some(Call::announce_next_key { public_key })
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
    fn find_next_author() -> Option<T::AuthorityId>;
}

impl<T: Config> FindAuthors<T> for () {
    fn find_current_author() -> Option<T::AuthorityId> {
        None
    }
    fn find_next_author() -> Option<T::AuthorityId> {
        None
    }
}

/// Decrypt a shielded transaction using the raw decapsulation key bytes.
///
/// Performs ML-KEM-768 decapsulation followed by XChaCha20-Poly1305 AEAD decryption.
/// Runs entirely in WASM — no host functions needed.
fn unshield(dec_key_bytes: &[u8], shielded_tx: &ShieldedTransaction) -> Option<alloc::vec::Vec<u8>> {
    let dec_key =
        DecapsulationKey::<MlKem768Params>::from_bytes(dec_key_bytes.try_into().ok()?);
    let ciphertext = Ciphertext::<MlKem768>::try_from(shielded_tx.kem_ct.as_slice()).ok()?;
    let shared_secret = dec_key.decapsulate(&ciphertext).ok()?;

    let aead = XChaCha20Poly1305::new(shared_secret.as_slice().into());
    let nonce = XNonce::from_slice(&shielded_tx.nonce);
    aead.decrypt(nonce, Payload { msg: &shielded_tx.aead_ct, aad: &[] }).ok()
}
