// pallets/mev-shield/src/lib.rs
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use frame_support::traits::IsSubType;
use frame_support::{pallet_prelude::*, sp_runtime::traits::Hash};
use frame_system::{ensure_none, ensure_signed, pallet_prelude::*};
use sp_runtime::Vec;
use sp_runtime::traits::Applyable;
use sp_runtime::traits::Block as BlockT;
use sp_runtime::traits::Checkable;

use alloc::vec;

pub use pallet::*;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[cfg(test)]
pub mod mock;

#[cfg(test)]
mod tests;

mod extension;

const KEY_HASH_LEN: usize = 16;

type PublicKey = BoundedVec<u8, ConstU32<2048>>;

type InherentType = Option<Vec<u8>>;

type ExtrinsicOf<Block> = <Block as BlockT>::Extrinsic;
type CheckedOf<T, Context> = <T as Checkable<Context>>::Checked;
type ApplyableCallOf<T> = <T as Applyable>::Call;

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

    // ----------------- Storage -----------------

    // Next block author ML‑KEM‑768 public key bytes.
    #[pallet::storage]
    pub type NextKey<T> = StorageValue<_, PublicKey, OptionQuery>;

    /// Current and next ML‑KEM‑768 public key bytes of all block authors.
    #[pallet::storage]
    pub type AuthorKeys<T: Config> = StorageMap<
        _,
        Twox64Concat,
        T::AuthorityId,
        (Option<PublicKey>, Option<PublicKey>),
        OptionQuery,
    >;

    // ----------------- Events & Errors -----------------

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

    // ----------------- Hooks -----------------

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_initialize(_n: BlockNumberFor<T>) -> Weight {
            let mut weight = Weight::zero();

            // We clear the next key no matter what happens next.
            NextKey::<T>::kill();
            weight = weight.saturating_add(T::DbWeight::get().writes(1_u64));

            weight = weight.saturating_add(T::DbWeight::get().reads(1_u64));
            let Some(author) = T::FindAuthors::find_next_author() else {
                return weight;
            };

            weight = weight.saturating_add(T::DbWeight::get().reads(1_u64));
            let Some((Some(key), _)) = AuthorKeys::<T>::get(author) else {
                return weight;
            };

            NextKey::<T>::put(key);
            weight = weight.saturating_add(T::DbWeight::get().writes(1_u64));

            weight
        }
    }

    // ----------------- Calls -----------------

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
        #[pallet::weight((
            Weight::from_parts(20_999_999_999, 0)
                .saturating_add(T::DbWeight::get().reads(1_u64))
                .saturating_add(T::DbWeight::get().writes(1_u64)),
            DispatchClass::Operational,
        ))]
        pub fn announce_next_key(
            origin: OriginFor<T>,
            public_key: Option<PublicKey>,
        ) -> DispatchResult {
            ensure_none(origin)?;

            let author = T::FindAuthors::find_current_author()
                // This should never happen as we are in an inherent.
                .ok_or_else(|| Error::<T>::Unreachable)?;

            if let Some(public_key) = &public_key {
                const MAX_KYBER768_PK_LENGTH: usize = 1184;
                ensure!(
                    public_key.len() == MAX_KYBER768_PK_LENGTH,
                    Error::<T>::BadPublicKeyLen
                );
            }

            AuthorKeys::<T>::mutate(author, |keys| {
                if let Some((current_key, next_key)) = keys {
                    *current_key = next_key.clone();
                    *next_key = public_key;
                } else {
                    // First time we see this author.
                    *keys = Some((None, public_key));
                }
            });

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
        ///
        ///      with ML‑KEM‑768 + XChaCha20‑Poly1305, producing
        ///
        ///        ciphertext = [u16 kem_len] || kem_ct || nonce24 || aead_ct
        ///
        #[pallet::call_index(1)]
        #[pallet::weight(Weight::from_parts(13_980_000, 0)
        .saturating_add(T::DbWeight::get().reads(1_u64))
        .saturating_add(T::DbWeight::get().writes(1_u64)))]
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

        const INHERENT_IDENTIFIER: [u8; 8] = *b"shieldpk";

        fn create_inherent(data: &InherentData) -> Option<Self::Call> {
            let public_key = data
                .get_data::<InherentType>(&Self::INHERENT_IDENTIFIER)
                .ok()??
                .map(|pk| BoundedVec::truncate_from(pk));

            Some(Call::announce_next_key { public_key })
        }

        fn is_inherent(call: &Self::Call) -> bool {
            matches!(call, Call::announce_next_key { .. })
        }
    }
}

impl<T: Config> Pallet<T> {
    pub fn try_decrypt_extrinsic<Block, Context>(
        uxt: ExtrinsicOf<Block>,
    ) -> Option<<Block as BlockT>::Extrinsic>
    where
        Block: BlockT<Header = HeaderFor<T>, Hash = <T as frame_system::Config>::Hash>,
        Block::Extrinsic: Checkable<Context>,
        CheckedOf<Block::Extrinsic, Context>: Applyable,
        ApplyableCallOf<CheckedOf<Block::Extrinsic, Context>>: IsSubType<Call<T>>,
        Context: Default,
    {
        const MAX_EXTRINSIC_DEPTH: u32 = 8;

        // Prevent stack overflows by limiting the depth of the extrinsic.
        let encoded = uxt.encode();
        let uxt = <Block::Extrinsic as codec::DecodeLimit>::decode_all_with_depth_limit(
            MAX_EXTRINSIC_DEPTH,
            &mut &encoded[..],
        )
        .inspect_err(|e| log::error!("Failed to decode extrinsic: {:?}", e))
        .ok()?;

        // Verify that the signature is good.
        let xt = ExtrinsicOf::<Block>::check(uxt, &Context::default())
            .inspect_err(|e| log::error!("Failed to check extrinsic: {:?}", e))
            .ok()?;
        let call = xt.call();

        let Some(Call::submit_encrypted { ciphertext, .. }) =
            IsSubType::<Call<T>>::is_sub_type(call)
        else {
            return None;
        };

        log::info!("Submit encrypted received: {}", ciphertext.len());

        if ciphertext.len() < 2 {
            return None;
        }

        let m = EncryptedMessage::parse(&ciphertext)?;
        let mut shared_secret = [0u8; 32];

        stp_io::crypto::mlkem768_decapsulate(&m.kem, &mut shared_secret).ok()?;
        let plaintext =
            stp_io::crypto::aead_decrypt(&shared_secret, &m.nonce, &m.aead, &[]).ok()?;

        if plaintext.is_empty() {
            return None;
        }

        let signed_xt = ExtrinsicOf::<Block>::decode(&mut &plaintext[..]).ok()?;
        log::info!("Decrypted extrinsic: {:?}", signed_xt);

        None
    }
}

#[derive(Debug)]
struct EncryptedMessage {
    kem: Vec<u8>,
    aead: Vec<u8>,
    nonce: [u8; 24],
}

impl EncryptedMessage {
    fn parse(ciphertext: &[u8]) -> Option<Self> {
        let mut cursor: usize = 0;

        let kem_len_end = cursor.checked_add(2)?;
        let kem_len_slice = ciphertext.get(cursor..kem_len_end)?;
        let kem_len_bytes: [u8; 2] = kem_len_slice.try_into().ok()?;
        let kem_len = u16::from_le_bytes(kem_len_bytes) as usize;
        cursor = kem_len_end;

        let kem_end = cursor.checked_add(kem_len)?;
        let kem = ciphertext.get(cursor..kem_end)?.to_vec();
        cursor = kem_end;

        const NONCE_LEN: usize = 24;
        let nonce_end = cursor.checked_add(NONCE_LEN)?;
        let nonce_bytes = ciphertext.get(cursor..nonce_end)?;
        let nonce: [u8; NONCE_LEN] = nonce_bytes.try_into().ok()?;
        cursor = nonce_end;

        let aead = ciphertext.get(cursor..)?.to_vec();

        Some(Self { kem, aead, nonce })
    }
}

pub trait FindAuthors<T: Config> {
    fn find_current_author() -> Option<T::AuthorityId>;
    fn find_next_author() -> Option<T::AuthorityId>;
}
