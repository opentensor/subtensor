#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "512"]
#![allow(clippy::too_many_arguments)]
// Edit this file to define custom logic or remove it if it is not needed.
// Learn more about FRAME and the core library of Substrate FRAME pallets:
// <https://docs.substrate.io/reference/frame-pallets/>

use frame_system::{self as system, ensure_signed};
pub use pallet::*;

use codec::{Decode, Encode};
use frame_support::sp_runtime::transaction_validity::InvalidTransaction;
use frame_support::{
    dispatch::{self, DispatchResult, DispatchResultWithPostInfo},
    ensure,
    pallet_macros::import_section,
    pallet_prelude::*,
    traits::tokens::fungible,
};
use pallet_balances::Call as BalancesCall;
// use pallet_scheduler as Scheduler;
use scale_info::TypeInfo;
use sp_core::Get;
use sp_runtime::{DispatchError, transaction_validity::TransactionValidityError};
use sp_std::marker::PhantomData;
use subtensor_runtime_common::{AlphaCurrency, Currency, NetUid, TaoCurrency};

// ============================
//	==== Benchmark Imports =====
// ============================
mod benchmarks;

// =========================
//	==== Pallet Imports =====
// =========================
pub mod coinbase;
pub mod epoch;
pub mod macros;
pub mod migrations;
pub mod rpc_info;
pub mod staking;
pub mod subnets;
pub mod swap;
pub mod utils;
use crate::utils::rate_limiting::{Hyperparameter, TransactionType};
use macros::{config, dispatches, errors, events, genesis, hooks};

#[cfg(test)]
mod tests;
pub mod transaction_extension;

// apparently this is stabilized since rust 1.36
extern crate alloc;

pub const MAX_CRV3_COMMIT_SIZE_BYTES: u32 = 5000;

#[allow(deprecated)]
#[deny(missing_docs)]
#[import_section(errors::errors)]
#[import_section(events::events)]
#[import_section(dispatches::dispatches)]
#[import_section(genesis::genesis)]
#[import_section(hooks::hooks)]
#[import_section(config::config)]
#[frame_support::pallet]
pub mod pallet {
    use crate::RateLimitKey;
    use crate::migrations;
    use crate::subnets::leasing::{LeaseId, SubnetLeaseOf};
    use frame_support::Twox64Concat;
    use frame_support::{
        BoundedVec,
        dispatch::GetDispatchInfo,
        pallet_prelude::{DispatchResult, StorageMap, ValueQuery, *},
        traits::{
            OriginTrait, QueryPreimage, StorePreimage, UnfilteredDispatchable, tokens::fungible,
        },
    };
    use frame_system::pallet_prelude::*;
    use pallet_drand::types::RoundNumber;
    use runtime_common::prod_or_fast;
    use sp_core::{ConstU32, H160, H256};
    use sp_runtime::traits::{Dispatchable, TrailingZeroInput};
    use sp_std::collections::vec_deque::VecDeque;
    use sp_std::vec;
    use sp_std::vec::Vec;
    use substrate_fixed::types::{I96F32, U64F64};
    use subtensor_macros::freeze_struct;
    use subtensor_runtime_common::{
        AlphaCurrency, Currency, NetUid, NetUidStorageIndex, SubId, TaoCurrency,
    };

    #[cfg(not(feature = "std"))]
    use alloc::boxed::Box;
    #[cfg(feature = "std")]
    use sp_std::prelude::Box;

    /// Origin for the pallet
    pub type PalletsOriginOf<T> =
        <<T as frame_system::Config>::RuntimeOrigin as OriginTrait>::PalletsOrigin;

    /// Call type for the pallet
    pub type CallOf<T> = <T as frame_system::Config>::RuntimeCall;

    /// Tracks version for migrations. Should be monotonic with respect to the
    /// order of migrations. (i.e. always increasing)
    const STORAGE_VERSION: StorageVersion = StorageVersion::new(7);

    /// Minimum balance required to perform a coldkey swap
    pub const MIN_BALANCE_TO_PERFORM_COLDKEY_SWAP: TaoCurrency = TaoCurrency::new(100_000_000); // 0.1 TAO in RAO

    /// Minimum commit reveal periods
    pub const MIN_COMMIT_REVEAL_PEROIDS: u64 = 1;
    /// Maximum commit reveal periods
    pub const MAX_COMMIT_REVEAL_PEROIDS: u64 = 100;

    #[pallet::pallet]
    #[pallet::without_storage_info]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(_);

    /// Alias for the account ID.
    pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;

    /// Struct for Axon.
    pub type AxonInfoOf = AxonInfo;

    /// local one
    pub type LocalCallOf<T> = <T as Config>::RuntimeCall;

    /// Data structure for Axon information.
    #[crate::freeze_struct("3545cfb0cac4c1f5")]
    #[derive(Encode, Decode, Default, TypeInfo, Clone, PartialEq, Eq, Debug)]
    pub struct AxonInfo {
        ///  Axon serving block.
        pub block: u64,
        ///  Axon version
        pub version: u32,
        ///  Axon u128 encoded ip address of type v6 or v4.
        pub ip: u128,
        ///  Axon u16 encoded port.
        pub port: u16,
        ///  Axon ip type, 4 for ipv4 and 6 for ipv6.
        pub ip_type: u8,
        ///  Axon protocol. TCP, UDP, other.
        pub protocol: u8,
        ///  Axon proto placeholder 1.
        pub placeholder1: u8,
        ///  Axon proto placeholder 2.
        pub placeholder2: u8,
    }

    /// Struct for NeuronCertificate.
    pub type NeuronCertificateOf = NeuronCertificate;
    /// Data structure for NeuronCertificate information.
    #[freeze_struct("1c232be200d9ec6c")]
    #[derive(Decode, Encode, Default, TypeInfo, PartialEq, Eq, Clone, Debug)]
    pub struct NeuronCertificate {
        ///  The neuron TLS public key
        pub public_key: BoundedVec<u8, ConstU32<64>>,
        ///  The algorithm used to generate the public key
        pub algorithm: u8,
    }

    impl TryFrom<Vec<u8>> for NeuronCertificate {
        type Error = ();

        fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
            if value.len() > 65 {
                return Err(());
            }
            // take the first byte as the algorithm
            let algorithm = value.first().ok_or(())?;
            // and the rest as the public_key
            let certificate = value.get(1..).ok_or(())?.to_vec();
            Ok(Self {
                public_key: BoundedVec::try_from(certificate).map_err(|_| ())?,
                algorithm: *algorithm,
            })
        }
    }

    ///  Struct for Prometheus.
    pub type PrometheusInfoOf = PrometheusInfo;

    /// Data structure for Prometheus information.
    #[crate::freeze_struct("5dde687e63baf0cd")]
    #[derive(Encode, Decode, Default, TypeInfo, Clone, PartialEq, Eq, Debug)]
    pub struct PrometheusInfo {
        /// Prometheus serving block.
        pub block: u64,
        /// Prometheus version.
        pub version: u32,
        ///  Prometheus u128 encoded ip address of type v6 or v4.
        pub ip: u128,
        ///  Prometheus u16 encoded port.
        pub port: u16,
        /// Prometheus ip type, 4 for ipv4 and 6 for ipv6.
        pub ip_type: u8,
    }

    ///  Struct for ChainIdentities. (DEPRECATED for V2)
    pub type ChainIdentityOf = ChainIdentity;

    /// Data structure for Chain Identities. (DEPRECATED for V2)
    #[crate::freeze_struct("bbfd00438dbe2b58")]
    #[derive(Encode, Decode, Default, TypeInfo, Clone, PartialEq, Eq, Debug)]
    pub struct ChainIdentity {
        /// The name of the chain identity
        pub name: Vec<u8>,
        /// The URL associated with the chain identity
        pub url: Vec<u8>,
        /// The image representation of the chain identity
        pub image: Vec<u8>,
        /// The Discord information for the chain identity
        pub discord: Vec<u8>,
        /// A description of the chain identity
        pub description: Vec<u8>,
        /// Additional information about the chain identity
        pub additional: Vec<u8>,
    }

    ///  Struct for ChainIdentities.
    pub type ChainIdentityOfV2 = ChainIdentityV2;

    /// Data structure for Chain Identities.
    #[crate::freeze_struct("ad72a270be7b59d7")]
    #[derive(Encode, Decode, Default, TypeInfo, Clone, PartialEq, Eq, Debug)]
    pub struct ChainIdentityV2 {
        /// The name of the chain identity
        pub name: Vec<u8>,
        /// The URL associated with the chain identity
        pub url: Vec<u8>,
        /// The github repository associated with the identity
        pub github_repo: Vec<u8>,
        /// The image representation of the chain identity
        pub image: Vec<u8>,
        /// The Discord information for the chain identity
        pub discord: Vec<u8>,
        /// A description of the chain identity
        pub description: Vec<u8>,
        /// Additional information about the chain identity
        pub additional: Vec<u8>,
    }

    ///  Struct for SubnetIdentities. (DEPRECATED for V2)
    pub type SubnetIdentityOf = SubnetIdentity;
    /// Data structure for Subnet Identities. (DEPRECATED for V2)
    #[crate::freeze_struct("f448dc3dad763108")]
    #[derive(Encode, Decode, Default, TypeInfo, Clone, PartialEq, Eq, Debug)]
    pub struct SubnetIdentity {
        /// The name of the subnet
        pub subnet_name: Vec<u8>,
        /// The github repository associated with the chain identity
        pub github_repo: Vec<u8>,
        /// The subnet's contact
        pub subnet_contact: Vec<u8>,
    }

    ///  Struct for SubnetIdentitiesV2. (DEPRECATED for V3)
    pub type SubnetIdentityOfV2 = SubnetIdentityV2;
    /// Data structure for Subnet Identities (DEPRECATED for V3)
    #[crate::freeze_struct("e002be4cd05d7b3e")]
    #[derive(Encode, Decode, Default, TypeInfo, Clone, PartialEq, Eq, Debug)]
    pub struct SubnetIdentityV2 {
        /// The name of the subnet
        pub subnet_name: Vec<u8>,
        /// The github repository associated with the subnet
        pub github_repo: Vec<u8>,
        /// The subnet's contact
        pub subnet_contact: Vec<u8>,
        /// The subnet's website
        pub subnet_url: Vec<u8>,
        /// The subnet's discord
        pub discord: Vec<u8>,
        /// The subnet's description
        pub description: Vec<u8>,
        /// Additional information about the subnet
        pub additional: Vec<u8>,
    }

    ///  Struct for SubnetIdentitiesV3.
    pub type SubnetIdentityOfV3 = SubnetIdentityV3;
    /// Data structure for Subnet Identities
    #[crate::freeze_struct("6a441335f985a0b")]
    #[derive(
        Encode, Decode, DecodeWithMemTracking, Default, TypeInfo, Clone, PartialEq, Eq, Debug,
    )]
    pub struct SubnetIdentityV3 {
        /// The name of the subnet
        pub subnet_name: Vec<u8>,
        /// The github repository associated with the subnet
        pub github_repo: Vec<u8>,
        /// The subnet's contact
        pub subnet_contact: Vec<u8>,
        /// The subnet's website
        pub subnet_url: Vec<u8>,
        /// The subnet's discord
        pub discord: Vec<u8>,
        /// The subnet's description
        pub description: Vec<u8>,
        /// The subnet's logo
        pub logo_url: Vec<u8>,
        /// Additional information about the subnet
        pub additional: Vec<u8>,
    }

    /// Enum for recycle or burn for the owner_uid(s)
    ///
    /// Can specify
    #[derive(TypeInfo, Clone, PartialEq, Eq, Debug)]
    #[default = Self::Burn(U16::MAX)] // default to burn everything
    pub enum RecycleOrBurnEnum {
        Burn(u16), // u16-normalized weight
        Recycle(u16),
    }
    impl codec::EncodeLike for RecycleOrBurnEnum {
        fn encode_to<E: codec::Encoder>(&self, e: &mut E) -> Result<(), E::Error> {
            match self {
                Self::Burn(weight) => {
                    e.encode_u8(0)?;
                    e.encode_u16(*weight)
                }
                Self::Recycle(weight) => {
                    e.encode_u8(1)?;
                    e.encode_u16(*weight)
                }
            }
        }
    }
    impl codec::DecodeLike for RecycleOrBurnEnum {
        fn decode<D: codec::Decoder>(d: &mut D) -> Result<Self, D::Error> {
            let tag = d.read_byte()?;
            match tag {
                0 => {
                    let weight = d.read_u16()?;
                    Ok(Self::Burn(weight))
                }
                1 => {
                    let weight = d.read_u16()?;
                    Ok(Self::Recycle(weight))
                }
                _ => Err(codec::Error::from("invalid tag")),
            }
        }
    }

    /// ============================
    /// ==== Staking + Accounts ====
    /// ============================

    #[pallet::type_value]
    /// Default value for zero.
    pub fn DefaultZeroU64<T: Config>() -> u64 {
        0
    }
    /// Default value for Alpha currency.
    #[pallet::type_value]
    pub fn DefaultZeroAlpha<T: Config>() -> AlphaCurrency {
        AlphaCurrency::ZERO
    }
    /// Default value for Tao currency.
    #[pallet::type_value]
    pub fn DefaultZeroTao<T: Config>() -> TaoCurrency {
        TaoCurrency::ZERO
    }
    #[pallet::type_value]
    /// Default value for zero.
    pub fn DefaultZeroU128<T: Config>() -> u128 {
        0
    }
    #[pallet::type_value]
    /// Default value for zero.
    pub fn DefaultZeroU16<T: Config>() -> u16 {
        0
    }
    #[pallet::type_value]
    /// Default value for false.
    pub fn DefaultFalse<T: Config>() -> bool {
        false
    }
    #[pallet::type_value]
    /// Default value for false.
    pub fn DefaultTrue<T: Config>() -> bool {
        true
    }
    #[pallet::type_value]
    /// Total Rao in circulation.
    pub fn TotalSupply<T: Config>() -> u64 {
        21_000_000_000_000_000
    }
    #[pallet::type_value]
    /// Default Delegate Take.
    pub fn DefaultDelegateTake<T: Config>() -> u16 {
        T::InitialDefaultDelegateTake::get()
    }

    #[pallet::type_value]
    /// Default childkey take.
    pub fn DefaultChildKeyTake<T: Config>() -> u16 {
        T::InitialDefaultChildKeyTake::get()
    }
    #[pallet::type_value]
    /// Default minimum delegate take.
    pub fn DefaultMinDelegateTake<T: Config>() -> u16 {
        T::InitialMinDelegateTake::get()
    }

    #[pallet::type_value]
    /// Default minimum childkey take.
    pub fn DefaultMinChildKeyTake<T: Config>() -> u16 {
        T::InitialMinChildKeyTake::get()
    }

    #[pallet::type_value]
    /// Default maximum childkey take.
    pub fn DefaultMaxChildKeyTake<T: Config>() -> u16 {
        T::InitialMaxChildKeyTake::get()
    }

    #[pallet::type_value]
    /// Default account take.
    pub fn DefaultAccountTake<T: Config>() -> u64 {
        0
    }
    #[pallet::type_value]
    /// Default value for global weight.
    pub fn DefaultTaoWeight<T: Config>() -> u64 {
        T::InitialTaoWeight::get()
    }
    #[pallet::type_value]
    /// Default emission per block.
    pub fn DefaultBlockEmission<T: Config>() -> u64 {
        1_000_000_000
    }
    #[pallet::type_value]
    /// Default allowed delegation.
    pub fn DefaultAllowsDelegation<T: Config>() -> bool {
        false
    }
    #[pallet::type_value]
    /// Default total issuance.
    pub fn DefaultTotalIssuance<T: Config>() -> TaoCurrency {
        T::InitialIssuance::get().into()
    }
    #[pallet::type_value]
    /// Default account, derived from zero trailing bytes.
    pub fn DefaultAccount<T: Config>() -> T::AccountId {
        T::AccountId::decode(&mut TrailingZeroInput::zeroes())
            .expect("trailing zeroes always produce a valid account ID; qed")
    }
    // pub fn DefaultStakeInterval<T: Config>() -> u64 {
    //     360
    // } (DEPRECATED)
    #[pallet::type_value]
    /// Default account linkage
    pub fn DefaultAccountLinkage<T: Config>() -> Vec<(u64, T::AccountId)> {
        vec![]
    }
    #[pallet::type_value]
    /// Default pending childkeys
    pub fn DefaultPendingChildkeys<T: Config>() -> (Vec<(u64, T::AccountId)>, u64) {
        (vec![], 0)
    }
    #[pallet::type_value]
    /// Default account linkage
    pub fn DefaultProportion<T: Config>() -> u64 {
        0
    }
    #[pallet::type_value]
    /// Default accumulated emission for a hotkey
    pub fn DefaultAccumulatedEmission<T: Config>() -> u64 {
        0
    }
    #[pallet::type_value]
    /// Default last adjustment block.
    pub fn DefaultLastAdjustmentBlock<T: Config>() -> u64 {
        0
    }
    #[pallet::type_value]
    /// Default last adjustment block.
    pub fn DefaultRegistrationsThisBlock<T: Config>() -> u16 {
        0
    }
    #[pallet::type_value]
    /// Default EMA price halving blocks
    pub fn DefaultEMAPriceMovingBlocks<T: Config>() -> u64 {
        T::InitialEmaPriceHalvingPeriod::get()
    }
    #[pallet::type_value]
    /// Default registrations this block.
    pub fn DefaultBurn<T: Config>() -> TaoCurrency {
        T::InitialBurn::get().into()
    }
    #[pallet::type_value]
    /// Default burn token.
    pub fn DefaultMinBurn<T: Config>() -> TaoCurrency {
        T::InitialMinBurn::get().into()
    }
    #[pallet::type_value]
    /// Default min burn token.
    pub fn DefaultMaxBurn<T: Config>() -> TaoCurrency {
        T::InitialMaxBurn::get().into()
    }
    #[pallet::type_value]
    /// Default max burn token.
    pub fn DefaultDifficulty<T: Config>() -> u64 {
        T::InitialDifficulty::get()
    }
    #[pallet::type_value]
    /// Default difficulty value.
    pub fn DefaultMinDifficulty<T: Config>() -> u64 {
        T::InitialMinDifficulty::get()
    }
    #[pallet::type_value]
    /// Default min difficulty value.
    pub fn DefaultMaxDifficulty<T: Config>() -> u64 {
        T::InitialMaxDifficulty::get()
    }
    #[pallet::type_value]
    /// Default max difficulty value.
    pub fn DefaultMaxRegistrationsPerBlock<T: Config>() -> u16 {
        T::InitialMaxRegistrationsPerBlock::get()
    }
    #[pallet::type_value]
    /// Default max registrations per block.
    pub fn DefaultRAORecycledForRegistration<T: Config>() -> TaoCurrency {
        T::InitialRAORecycledForRegistration::get().into()
    }
    #[pallet::type_value]
    /// Default number of networks.
    pub fn DefaultN<T: Config>() -> u16 {
        0
    }
    #[pallet::type_value]
    /// Default value for hotkeys.
    pub fn DefaultHotkeys<T: Config>() -> Vec<u16> {
        vec![]
    }
    #[pallet::type_value]
    /// Default value if network is added.
    pub fn DefaultNeworksAdded<T: Config>() -> bool {
        false
    }
    #[pallet::type_value]
    /// Default value for network member.
    pub fn DefaultIsNetworkMember<T: Config>() -> bool {
        false
    }
    #[pallet::type_value]
    /// Default value for registration allowed.
    pub fn DefaultRegistrationAllowed<T: Config>() -> bool {
        true
    }
    #[pallet::type_value]
    /// Default value for network registered at.
    pub fn DefaultNetworkRegisteredAt<T: Config>() -> u64 {
        0
    }
    #[pallet::type_value]
    /// Default value for network immunity period.
    pub fn DefaultNetworkImmunityPeriod<T: Config>() -> u64 {
        T::InitialNetworkImmunityPeriod::get()
    }
    #[pallet::type_value]
    /// Default value for network min allowed UIDs.
    pub fn DefaultNetworkMinAllowedUids<T: Config>() -> u16 {
        T::InitialNetworkMinAllowedUids::get()
    }
    #[pallet::type_value]
    /// Default value for network min lock cost.
    pub fn DefaultNetworkMinLockCost<T: Config>() -> TaoCurrency {
        T::InitialNetworkMinLockCost::get().into()
    }
    #[pallet::type_value]
    /// Default value for network lock reduction interval.
    pub fn DefaultNetworkLockReductionInterval<T: Config>() -> u64 {
        T::InitialNetworkLockReductionInterval::get()
    }
    #[pallet::type_value]
    /// Default value for subnet owner cut.
    pub fn DefaultSubnetOwnerCut<T: Config>() -> u16 {
        T::InitialSubnetOwnerCut::get()
    }
    #[pallet::type_value]
    /// Default value for recycle or burn.
    pub fn DefaultRecycleOrBurn<T: Config>() -> RecycleOrBurnEnum {
        RecycleOrBurnEnum::Burn(U16::MAX) // default to burn
    }
    #[pallet::type_value]
    /// Default value for network rate limit.
    pub fn DefaultNetworkRateLimit<T: Config>() -> u64 {
        if cfg!(feature = "pow-faucet") {
            return 0;
        }
        T::InitialNetworkRateLimit::get()
    }
    #[pallet::type_value]
    /// Default value for weights version key rate limit.
    /// In units of tempos.
    pub fn DefaultWeightsVersionKeyRateLimit<T: Config>() -> u64 {
        5 // 5 tempos
    }
    #[pallet::type_value]
    /// Default value for pending emission.
    pub fn DefaultPendingEmission<T: Config>() -> AlphaCurrency {
        0.into()
    }
    #[pallet::type_value]
    /// Default value for blocks since last step.
    pub fn DefaultBlocksSinceLastStep<T: Config>() -> u64 {
        0
    }
    #[pallet::type_value]
    /// Default value for last mechanism step block.
    pub fn DefaultLastMechanismStepBlock<T: Config>() -> u64 {
        0
    }
    #[pallet::type_value]
    /// Default value for subnet owner.
    pub fn DefaultSubnetOwner<T: Config>() -> T::AccountId {
        T::AccountId::decode(&mut sp_runtime::traits::TrailingZeroInput::zeroes())
            .expect("trailing zeroes always produce a valid account ID; qed")
    }
    #[pallet::type_value]
    /// Default value for subnet locked.
    pub fn DefaultSubnetLocked<T: Config>() -> u64 {
        0
    }
    #[pallet::type_value]
    /// Default value for network tempo
    pub fn DefaultTempo<T: Config>() -> u16 {
        T::InitialTempo::get()
    }
    #[pallet::type_value]
    /// Default value for weights set rate limit.
    pub fn DefaultWeightsSetRateLimit<T: Config>() -> u64 {
        100
    }
    #[pallet::type_value]
    /// Default block number at registration.
    pub fn DefaultBlockAtRegistration<T: Config>() -> u64 {
        0
    }
    #[pallet::type_value]
    /// Default value for rho parameter.
    pub fn DefaultRho<T: Config>() -> u16 {
        T::InitialRho::get()
    }
    #[pallet::type_value]
    /// Default value for alpha sigmoid steepness.
    pub fn DefaultAlphaSigmoidSteepness<T: Config>() -> i16 {
        T::InitialAlphaSigmoidSteepness::get()
    }
    #[pallet::type_value]
    /// Default value for kappa parameter.
    pub fn DefaultKappa<T: Config>() -> u16 {
        T::InitialKappa::get()
    }
    #[pallet::type_value]
    /// Default maximum allowed UIDs.
    pub fn DefaultMaxAllowedUids<T: Config>() -> u16 {
        T::InitialMaxAllowedUids::get()
    }
    #[pallet::type_value]
    /// Default immunity period.
    pub fn DefaultImmunityPeriod<T: Config>() -> u16 {
        T::InitialImmunityPeriod::get()
    }
    #[pallet::type_value]
    /// Default activity cutoff.
    pub fn DefaultActivityCutoff<T: Config>() -> u16 {
        T::InitialActivityCutoff::get()
    }
    #[pallet::type_value]
    /// Default maximum weights limit.
    pub fn DefaultMaxWeightsLimit<T: Config>() -> u16 {
        T::InitialMaxWeightsLimit::get()
    }
    #[pallet::type_value]
    /// Default weights version key.
    pub fn DefaultWeightsVersionKey<T: Config>() -> u64 {
        T::InitialWeightsVersionKey::get()
    }
    #[pallet::type_value]
    /// Default minimum allowed weights.
    pub fn DefaultMinAllowedWeights<T: Config>() -> u16 {
        T::InitialMinAllowedWeights::get()
    }
    #[pallet::type_value]
    /// Default maximum allowed validators.
    pub fn DefaultMaxAllowedValidators<T: Config>() -> u16 {
        T::InitialMaxAllowedValidators::get()
    }
    #[pallet::type_value]
    /// Default adjustment interval.
    pub fn DefaultAdjustmentInterval<T: Config>() -> u16 {
        T::InitialAdjustmentInterval::get()
    }
    #[pallet::type_value]
    /// Default bonds moving average.
    pub fn DefaultBondsMovingAverage<T: Config>() -> u64 {
        T::InitialBondsMovingAverage::get()
    }
    /// Default bonds penalty.
    #[pallet::type_value]
    pub fn DefaultBondsPenalty<T: Config>() -> u16 {
        T::InitialBondsPenalty::get()
    }
    /// Default value for bonds reset - will not reset bonds
    #[pallet::type_value]
    pub fn DefaultBondsResetOn<T: Config>() -> bool {
        T::InitialBondsResetOn::get()
    }
    /// Default validator prune length.
    #[pallet::type_value]
    pub fn DefaultValidatorPruneLen<T: Config>() -> u64 {
        T::InitialValidatorPruneLen::get()
    }
    #[pallet::type_value]
    /// Default scaling law power.
    pub fn DefaultScalingLawPower<T: Config>() -> u16 {
        T::InitialScalingLawPower::get()
    }
    #[pallet::type_value]
    /// Default target registrations per interval.
    pub fn DefaultTargetRegistrationsPerInterval<T: Config>() -> u16 {
        T::InitialTargetRegistrationsPerInterval::get()
    }
    #[pallet::type_value]
    /// Default adjustment alpha.
    pub fn DefaultAdjustmentAlpha<T: Config>() -> u64 {
        T::InitialAdjustmentAlpha::get()
    }
    #[pallet::type_value]
    /// Default minimum stake for weights.
    pub fn DefaultStakeThreshold<T: Config>() -> u64 {
        0
    }
    #[pallet::type_value]
    /// Default Reveal Period Epochs
    pub fn DefaultRevealPeriodEpochs<T: Config>() -> u64 {
        1
    }
    #[pallet::type_value]
    /// Value definition for vector of u16.
    pub fn EmptyU16Vec<T: Config>() -> Vec<u16> {
        vec![]
    }
    #[pallet::type_value]
    /// Value definition for vector of u64.
    pub fn EmptyU64Vec<T: Config>() -> Vec<u64> {
        vec![]
    }
    #[pallet::type_value]
    /// Value definition for vector of bool.
    pub fn EmptyBoolVec<T: Config>() -> Vec<bool> {
        vec![]
    }
    #[pallet::type_value]
    /// Value definition for bonds with type vector of (u16, u16).
    pub fn DefaultBonds<T: Config>() -> Vec<(u16, u16)> {
        vec![]
    }
    #[pallet::type_value]
    /// Value definition for weights with vector of (u16, u16).
    pub fn DefaultWeights<T: Config>() -> Vec<(u16, u16)> {
        vec![]
    }
    #[pallet::type_value]
    /// Default value for key with type T::AccountId derived from trailing zeroes.
    pub fn DefaultKey<T: Config>() -> T::AccountId {
        T::AccountId::decode(&mut sp_runtime::traits::TrailingZeroInput::zeroes())
            .expect("trailing zeroes always produce a valid account ID; qed")
    }
    // pub fn DefaultHotkeyEmissionTempo<T: Config>() -> u64 {
    //     T::InitialHotkeyEmissionTempo::get()
    // } (DEPRECATED)
    #[pallet::type_value]
    /// Default value for rate limiting
    pub fn DefaultTxRateLimit<T: Config>() -> u64 {
        T::InitialTxRateLimit::get()
    }
    #[pallet::type_value]
    /// Default value for delegate take rate limiting
    pub fn DefaultTxDelegateTakeRateLimit<T: Config>() -> u64 {
        T::InitialTxDelegateTakeRateLimit::get()
    }
    #[pallet::type_value]
    /// Default value for chidlkey take rate limiting
    pub fn DefaultTxChildKeyTakeRateLimit<T: Config>() -> u64 {
        T::InitialTxChildKeyTakeRateLimit::get()
    }
    #[pallet::type_value]
    /// Default value for last extrinsic block.
    pub fn DefaultLastTxBlock<T: Config>() -> u64 {
        0
    }
    #[pallet::type_value]
    /// Default value for serving rate limit.
    pub fn DefaultServingRateLimit<T: Config>() -> u64 {
        T::InitialServingRateLimit::get()
    }
    #[pallet::type_value]
    /// Default value for weight commit/reveal enabled.
    pub fn DefaultCommitRevealWeightsEnabled<T: Config>() -> bool {
        true
    }
    #[pallet::type_value]
    /// Default value for weight commit/reveal version.
    pub fn DefaultCommitRevealWeightsVersion<T: Config>() -> u16 {
        4
    }
    #[pallet::type_value]
    /// Senate requirements
    pub fn DefaultSenateRequiredStakePercentage<T: Config>() -> u64 {
        T::InitialSenateRequiredStakePercentage::get()
    }
    #[pallet::type_value]
    /// -- ITEM (switches liquid alpha on)
    pub fn DefaultLiquidAlpha<T: Config>() -> bool {
        false
    }
    #[pallet::type_value]
    /// -- ITEM (switches liquid alpha on)
    pub fn DefaultYuma3<T: Config>() -> bool {
        false
    }
    #[pallet::type_value]
    /// (alpha_low: 0.7, alpha_high: 0.9)
    pub fn DefaultAlphaValues<T: Config>() -> (u16, u16) {
        (45875, 58982)
    }
    #[pallet::type_value]
    /// Default value for coldkey swap schedule duration
    pub fn DefaultColdkeySwapScheduleDuration<T: Config>() -> BlockNumberFor<T> {
        T::InitialColdkeySwapScheduleDuration::get()
    }

    #[pallet::type_value]
    /// Default value for coldkey swap reschedule duration
    pub fn DefaultColdkeySwapRescheduleDuration<T: Config>() -> BlockNumberFor<T> {
        T::InitialColdkeySwapRescheduleDuration::get()
    }

    #[pallet::type_value]
    /// Default value for applying pending items (e.g. childkeys).
    pub fn DefaultPendingCooldown<T: Config>() -> u64 {
        prod_or_fast!(7_200, 15)
    }

    #[pallet::type_value]
    /// Default minimum stake.
    pub fn DefaultMinStake<T: Config>() -> TaoCurrency {
        2_000_000.into()
    }

    #[pallet::type_value]
    /// Default unicode vector for tau symbol.
    pub fn DefaultUnicodeVecU8<T: Config>() -> Vec<u8> {
        b"\xF0\x9D\x9C\x8F".to_vec() // Unicode for tau (𝜏)
    }

    #[pallet::type_value]
    /// Default value for dissolve network schedule duration
    pub fn DefaultDissolveNetworkScheduleDuration<T: Config>() -> BlockNumberFor<T> {
        T::InitialDissolveNetworkScheduleDuration::get()
    }

    #[pallet::type_value]
    /// Default moving alpha for the moving price.
    pub fn DefaultMovingAlpha<T: Config>() -> I96F32 {
        // Moving average take 30 days to reach 50% of the price
        // and 3.5 months to reach 90%.
        I96F32::saturating_from_num(0.000003)
    }
    #[pallet::type_value]
    /// Default subnet moving price.
    pub fn DefaultMovingPrice<T: Config>() -> I96F32 {
        I96F32::saturating_from_num(0.0)
    }
    #[pallet::type_value]
    /// Default value for Share Pool variables
    pub fn DefaultSharePoolZero<T: Config>() -> U64F64 {
        U64F64::saturating_from_num(0)
    }

    #[pallet::type_value]
    /// Default value for minimum activity cutoff
    pub fn DefaultMinActivityCutoff<T: Config>() -> u16 {
        360
    }

    #[pallet::type_value]
    /// Default value for coldkey swap scheduled
    pub fn DefaultColdkeySwapScheduled<T: Config>() -> (BlockNumberFor<T>, T::AccountId) {
        let default_account = T::AccountId::decode(&mut TrailingZeroInput::zeroes())
            .expect("trailing zeroes always produce a valid account ID; qed");
        (BlockNumberFor::<T>::from(0_u32), default_account)
    }

    #[pallet::type_value]
    /// Default value for setting subnet owner hotkey rate limit
    pub fn DefaultSetSNOwnerHotkeyRateLimit<T: Config>() -> u64 {
        50400
    }

    #[pallet::type_value]
    /// Default number of terminal blocks in a tempo during which admin operations are prohibited
    pub fn DefaultAdminFreezeWindow<T: Config>() -> u16 {
        10
    }

    #[pallet::type_value]
    /// Default number of tempos for owner hyperparameter update rate limit
    pub fn DefaultOwnerHyperparamRateLimit<T: Config>() -> u16 {
        2
    }

    #[pallet::type_value]
    /// Default value for ck burn, 18%.
    pub fn DefaultCKBurn<T: Config>() -> u64 {
        0
    }

    #[pallet::storage]
    pub type MinActivityCutoff<T: Config> =
        StorageValue<_, u16, ValueQuery, DefaultMinActivityCutoff<T>>;

    #[pallet::storage]
    /// Global window (in blocks) at the end of each tempo where admin ops are disallowed
    pub type AdminFreezeWindow<T: Config> =
        StorageValue<_, u16, ValueQuery, DefaultAdminFreezeWindow<T>>;

    #[pallet::storage]
    /// Global number of epochs used to rate limit subnet owner hyperparameter updates
    pub type OwnerHyperparamRateLimit<T: Config> =
        StorageValue<_, u16, ValueQuery, DefaultOwnerHyperparamRateLimit<T>>;

    #[pallet::storage]
    pub type ColdkeySwapScheduleDuration<T: Config> =
        StorageValue<_, BlockNumberFor<T>, ValueQuery, DefaultColdkeySwapScheduleDuration<T>>;

    #[pallet::storage]
    pub type ColdkeySwapRescheduleDuration<T: Config> =
        StorageValue<_, BlockNumberFor<T>, ValueQuery, DefaultColdkeySwapRescheduleDuration<T>>;

    #[pallet::storage]
    pub type DissolveNetworkScheduleDuration<T: Config> =
        StorageValue<_, BlockNumberFor<T>, ValueQuery, DefaultDissolveNetworkScheduleDuration<T>>;

    #[pallet::storage]
    pub type SenateRequiredStakePercentage<T> =
        StorageValue<_, u64, ValueQuery, DefaultSenateRequiredStakePercentage<T>>;

    #[pallet::storage]
    /// --- DMap ( netuid, coldkey ) --> blocknumber | last hotkey swap on network.
    pub type LastHotkeySwapOnNetuid<T: Config> = StorageDoubleMap<
        _,
        Identity,
        NetUid,
        Blake2_128Concat,
        T::AccountId,
        u64,
        ValueQuery,
        DefaultZeroU64<T>,
    >;

    #[pallet::storage]
    /// Ensures unique IDs for StakeJobs storage map
    pub type NextStakeJobId<T> = StorageValue<_, u64, ValueQuery, DefaultZeroU64<T>>;

    /// ============================
    /// ==== Staking Variables ====
    /// ============================
    /// The Subtensor [`TotalIssuance`] represents the total issuance of tokens on the Bittensor network.
    ///
    /// It is comprised of three parts:
    /// - The total amount of issued tokens, tracked in the TotalIssuance of the Balances pallet
    /// - The total amount of tokens staked in the system, tracked in [`TotalStake`]
    /// - The total amount of tokens locked up for subnet reg, tracked in [`TotalSubnetLocked`] attained by iterating over subnet lock.
    ///
    /// Eventually, Bittensor should migrate to using Holds afterwhich time we will not require this
    /// separate accounting.
    #[pallet::storage]
    /// --- ITEM --> Global weight
    pub type TaoWeight<T> = StorageValue<_, u64, ValueQuery, DefaultTaoWeight<T>>;
    #[pallet::storage]
    /// --- ITEM --> CK burn
    pub type CKBurn<T> = StorageValue<_, u64, ValueQuery, DefaultCKBurn<T>>;
    #[pallet::storage]
    /// --- ITEM ( default_delegate_take )
    pub type MaxDelegateTake<T> = StorageValue<_, u16, ValueQuery, DefaultDelegateTake<T>>;
    #[pallet::storage]
    /// --- ITEM ( min_delegate_take )
    pub type MinDelegateTake<T> = StorageValue<_, u16, ValueQuery, DefaultMinDelegateTake<T>>;
    #[pallet::storage]
    /// --- ITEM ( default_childkey_take )
    pub type MaxChildkeyTake<T> = StorageValue<_, u16, ValueQuery, DefaultMaxChildKeyTake<T>>;
    #[pallet::storage]
    /// --- ITEM ( min_childkey_take )
    pub type MinChildkeyTake<T> = StorageValue<_, u16, ValueQuery, DefaultMinChildKeyTake<T>>;
    #[pallet::storage]
    /// MAP ( hot ) --> cold | Returns the controlling coldkey for a hotkey.
    pub type Owner<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, T::AccountId, ValueQuery, DefaultAccount<T>>;
    #[pallet::storage]
    /// MAP ( hot ) --> take | Returns the hotkey delegation take. And signals that this key is open for delegation.
    pub type Delegates<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, u16, ValueQuery, DefaultDelegateTake<T>>;
    #[pallet::storage]
    /// DMAP ( hot, netuid ) --> take | Returns the hotkey childkey take for a specific subnet
    pub type ChildkeyTake<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId, // First key: hotkey
        Identity,
        NetUid, // Second key: netuid
        u16,    // Value: take
        ValueQuery,
    >;
    #[pallet::storage]
    /// DMAP ( netuid, parent ) --> (Vec<(proportion,child)>, cool_down_block)
    pub type PendingChildKeys<T: Config> = StorageDoubleMap<
        _,
        Identity,
        NetUid,
        Blake2_128Concat,
        T::AccountId,
        (Vec<(u64, T::AccountId)>, u64),
        ValueQuery,
        DefaultPendingChildkeys<T>,
    >;
    #[pallet::storage]
    /// DMAP ( parent, netuid ) --> Vec<(proportion,child)>
    pub type ChildKeys<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Identity,
        NetUid,
        Vec<(u64, T::AccountId)>,
        ValueQuery,
        DefaultAccountLinkage<T>,
    >;
    #[pallet::storage]
    /// DMAP ( child, netuid ) --> Vec<(proportion,parent)>
    pub type ParentKeys<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Identity,
        NetUid,
        Vec<(u64, T::AccountId)>,
        ValueQuery,
        DefaultAccountLinkage<T>,
    >;
    #[pallet::storage] // --- DMAP ( netuid, hotkey ) --> u64 | Last total dividend this hotkey got on tempo.
    pub type AlphaDividendsPerSubnet<T: Config> = StorageDoubleMap<
        _,
        Identity,
        NetUid,
        Blake2_128Concat,
        T::AccountId,
        AlphaCurrency,
        ValueQuery,
        DefaultZeroAlpha<T>,
    >;
    #[pallet::storage] // --- DMAP ( netuid, hotkey ) --> u64 | Last total root dividend paid to this hotkey on this subnet.
    pub type TaoDividendsPerSubnet<T: Config> = StorageDoubleMap<
        _,
        Identity,
        NetUid,
        Blake2_128Concat,
        T::AccountId,
        TaoCurrency,
        ValueQuery,
        DefaultZeroTao<T>,
    >;

    /// ==================
    /// ==== Coinbase ====
    /// ==================
    #[pallet::storage]
    /// --- ITEM ( global_block_emission )
    pub type BlockEmission<T> = StorageValue<_, u64, ValueQuery, DefaultBlockEmission<T>>;
    #[pallet::storage]
    /// --- DMap ( hot, netuid ) --> emission | last hotkey emission on network.
    pub type LastHotkeyEmissionOnNetuid<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Identity,
        NetUid,
        AlphaCurrency,
        ValueQuery,
        DefaultZeroAlpha<T>,
    >;

    /// ==========================
    /// ==== Staking Counters ====
    /// ==========================
    /// The Subtensor [`TotalIssuance`] represents the total issuance of tokens on the Bittensor network.
    ///
    /// It is comprised of three parts:
    /// - The total amount of issued tokens, tracked in the TotalIssuance of the Balances pallet
    /// - The total amount of tokens staked in the system, tracked in [`TotalStake`]
    /// - The total amount of tokens locked up for subnet reg, tracked in [`TotalSubnetLocked`] attained by iterating over subnet lock.
    ///
    /// Eventually, Bittensor should migrate to using Holds afterwhich time we will not require this
    /// separate accounting.
    #[pallet::storage] // --- ITEM ( total_issuance )
    pub type TotalIssuance<T> = StorageValue<_, TaoCurrency, ValueQuery, DefaultTotalIssuance<T>>;
    #[pallet::storage] // --- ITEM ( total_stake )
    pub type TotalStake<T> = StorageValue<_, TaoCurrency, ValueQuery>;
    #[pallet::storage] // --- ITEM ( moving_alpha ) -- subnet moving alpha.
    pub type SubnetMovingAlpha<T> = StorageValue<_, I96F32, ValueQuery, DefaultMovingAlpha<T>>;
    #[pallet::storage] // --- MAP ( netuid ) --> moving_price | The subnet moving price.
    pub type SubnetMovingPrice<T: Config> =
        StorageMap<_, Identity, NetUid, I96F32, ValueQuery, DefaultMovingPrice<T>>;
    #[pallet::storage] // --- MAP ( netuid ) --> total_volume | The total amount of TAO bought and sold since the start of the network.
    pub type SubnetVolume<T: Config> =
        StorageMap<_, Identity, NetUid, u128, ValueQuery, DefaultZeroU128<T>>;
    #[pallet::storage] // --- MAP ( netuid ) --> tao_in_subnet | Returns the amount of TAO in the subnet.
    pub type SubnetTAO<T: Config> =
        StorageMap<_, Identity, NetUid, TaoCurrency, ValueQuery, DefaultZeroTao<T>>;
    #[pallet::storage] // --- MAP ( netuid ) --> tao_in_user_subnet | Returns the amount of TAO in the subnet reserve provided by users as liquidity.
    pub type SubnetTaoProvided<T: Config> =
        StorageMap<_, Identity, NetUid, TaoCurrency, ValueQuery, DefaultZeroTao<T>>;
    #[pallet::storage] // --- MAP ( netuid ) --> alpha_in_emission | Returns the amount of alph in  emission into the pool per block.
    pub type SubnetAlphaInEmission<T: Config> =
        StorageMap<_, Identity, NetUid, AlphaCurrency, ValueQuery, DefaultZeroAlpha<T>>;
    #[pallet::storage] // --- MAP ( netuid ) --> alpha_out_emission | Returns the amount of alpha out emission into the network per block.
    pub type SubnetAlphaOutEmission<T: Config> =
        StorageMap<_, Identity, NetUid, AlphaCurrency, ValueQuery, DefaultZeroAlpha<T>>;
    #[pallet::storage] // --- MAP ( netuid ) --> tao_in_emission | Returns the amount of tao emitted into this subent on the last block.
    pub type SubnetTaoInEmission<T: Config> =
        StorageMap<_, Identity, NetUid, TaoCurrency, ValueQuery, DefaultZeroTao<T>>;
    #[pallet::storage] // --- MAP ( netuid ) --> alpha_supply_in_pool | Returns the amount of alpha in the pool.
    pub type SubnetAlphaIn<T: Config> =
        StorageMap<_, Identity, NetUid, AlphaCurrency, ValueQuery, DefaultZeroAlpha<T>>;
    #[pallet::storage] // --- MAP ( netuid ) --> alpha_supply_user_in_pool | Returns the amount of alpha in the pool provided by users as liquidity.
    pub type SubnetAlphaInProvided<T: Config> =
        StorageMap<_, Identity, NetUid, AlphaCurrency, ValueQuery, DefaultZeroAlpha<T>>;
    #[pallet::storage]
    /// --- MAP ( netuid ) --> alpha_supply_in_subnet | Returns the amount of alpha in the subnet.
    pub type SubnetAlphaOut<T: Config> =
        StorageMap<_, Identity, NetUid, AlphaCurrency, ValueQuery, DefaultZeroAlpha<T>>;
    #[pallet::storage] // --- MAP ( cold ) --> Vec<hot> | Maps coldkey to hotkeys that stake to it
    pub type StakingHotkeys<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, Vec<T::AccountId>, ValueQuery>;
    #[pallet::storage] // --- MAP ( cold ) --> Vec<hot> | Returns the vector of hotkeys controlled by this coldkey.
    pub type OwnedHotkeys<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, Vec<T::AccountId>, ValueQuery>;
    #[pallet::storage] // --- MAP ( cold ) --> hot | Returns the hotkey a coldkey will autostake to with mining rewards.
    pub type AutoStakeDestination<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, T::AccountId, OptionQuery>;

    #[pallet::storage] // --- DMAP ( cold ) --> (block_expected, new_coldkey) | Maps coldkey to the block to swap at and new coldkey.
    pub type ColdkeySwapScheduled<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        (BlockNumberFor<T>, T::AccountId),
        ValueQuery,
        DefaultColdkeySwapScheduled<T>,
    >;

    #[pallet::storage] // --- DMAP ( hot, netuid ) --> alpha | Returns the total amount of alpha a hotkey owns.
    pub type TotalHotkeyAlpha<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Identity,
        NetUid,
        AlphaCurrency,
        ValueQuery,
        DefaultZeroAlpha<T>,
    >;
    #[pallet::storage] // --- DMAP ( hot, netuid ) --> alpha | Returns the total amount of alpha a hotkey owned in the last epoch.
    pub type TotalHotkeyAlphaLastEpoch<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Identity,
        NetUid,
        AlphaCurrency,
        ValueQuery,
        DefaultZeroAlpha<T>,
    >;
    #[pallet::storage]
    /// DMAP ( hot, netuid ) --> total_alpha_shares | Returns the number of alpha shares for a hotkey on a subnet.
    pub type TotalHotkeyShares<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Identity,
        NetUid,
        U64F64,
        ValueQuery,
        DefaultSharePoolZero<T>,
    >;
    #[pallet::storage] // --- NMAP ( hot, cold, netuid ) --> alpha | Returns the alpha shares for a hotkey, coldkey, netuid triplet.
    pub type Alpha<T: Config> = StorageNMap<
        _,
        (
            NMapKey<Blake2_128Concat, T::AccountId>, // hot
            NMapKey<Blake2_128Concat, T::AccountId>, // cold
            NMapKey<Identity, NetUid>,               // subnet
        ),
        U64F64, // Shares
        ValueQuery,
    >;
    #[pallet::storage] // --- MAP ( netuid ) --> token_symbol | Returns the token symbol for a subnet.
    pub type TokenSymbol<T: Config> =
        StorageMap<_, Identity, NetUid, Vec<u8>, ValueQuery, DefaultUnicodeVecU8<T>>;

    /// ============================
    /// ==== Global Parameters =====
    /// ============================
    #[pallet::storage]
    /// --- StorageItem Global Used Work.
    pub type UsedWork<T: Config> = StorageMap<_, Identity, Vec<u8>, u64, ValueQuery>;
    #[pallet::storage]
    /// --- ITEM( global_max_registrations_per_block )
    pub type MaxRegistrationsPerBlock<T> =
        StorageMap<_, Identity, NetUid, u16, ValueQuery, DefaultMaxRegistrationsPerBlock<T>>;
    #[pallet::storage]
    /// --- ITEM( total_number_of_existing_networks )
    pub type TotalNetworks<T> = StorageValue<_, u16, ValueQuery>;
    #[pallet::storage]
    /// ITEM( network_immunity_period )
    pub type NetworkImmunityPeriod<T> =
        StorageValue<_, u64, ValueQuery, DefaultNetworkImmunityPeriod<T>>;
    #[pallet::storage]
    /// ITEM( min_network_lock_cost )
    pub type NetworkMinLockCost<T> =
        StorageValue<_, TaoCurrency, ValueQuery, DefaultNetworkMinLockCost<T>>;
    #[pallet::storage]
    /// ITEM( last_network_lock_cost )
    pub type NetworkLastLockCost<T> =
        StorageValue<_, TaoCurrency, ValueQuery, DefaultNetworkMinLockCost<T>>;
    #[pallet::storage]
    /// ITEM( network_lock_reduction_interval )
    pub type NetworkLockReductionInterval<T> =
        StorageValue<_, u64, ValueQuery, DefaultNetworkLockReductionInterval<T>>;
    #[pallet::storage]
    /// ITEM( subnet_owner_cut )
    pub type SubnetOwnerCut<T> = StorageValue<_, u16, ValueQuery, DefaultSubnetOwnerCut<T>>;
    #[pallet::storage]
    /// ITEM( network_rate_limit )
    pub type NetworkRateLimit<T> = StorageValue<_, u64, ValueQuery, DefaultNetworkRateLimit<T>>;
    #[pallet::storage]
    /// --- ITEM( nominator_min_required_stake ) --- Factor of DefaultMinStake in per-mill format.
    pub type NominatorMinRequiredStake<T> = StorageValue<_, u64, ValueQuery, DefaultZeroU64<T>>;
    #[pallet::storage]
    /// ITEM( weights_version_key_rate_limit ) --- Rate limit in tempos.
    pub type WeightsVersionKeyRateLimit<T> =
        StorageValue<_, u64, ValueQuery, DefaultWeightsVersionKeyRateLimit<T>>;

    /// ============================
    /// ==== Rate Limiting =====
    /// ============================

    #[pallet::storage]
    /// --- MAP ( RateLimitKey ) --> Block number in which the last rate limited operation occured
    pub type LastRateLimitedBlock<T: Config> =
        StorageMap<_, Identity, RateLimitKey<T::AccountId>, u64, ValueQuery, DefaultZeroU64<T>>;

    /// ============================
    /// ==== Subnet Locks =====
    /// ============================
    #[pallet::storage] // --- MAP ( netuid ) --> transfer_toggle
    pub type TransferToggle<T: Config> =
        StorageMap<_, Identity, NetUid, bool, ValueQuery, DefaultTrue<T>>;
    #[pallet::storage] // --- MAP ( netuid ) --> total_subnet_locked
    pub type SubnetLocked<T: Config> =
        StorageMap<_, Identity, NetUid, TaoCurrency, ValueQuery, DefaultZeroTao<T>>;
    #[pallet::storage] // --- MAP ( netuid ) --> largest_locked
    pub type LargestLocked<T: Config> =
        StorageMap<_, Identity, NetUid, u64, ValueQuery, DefaultZeroU64<T>>;

    /// =================
    /// ==== Tempos =====
    /// =================
    #[pallet::storage] // --- MAP ( netuid ) --> tempo
    pub type Tempo<T> = StorageMap<_, Identity, NetUid, u16, ValueQuery, DefaultTempo<T>>;

    /// ============================
    /// ==== Subnet Parameters =====
    /// ============================
    /// --- MAP ( netuid ) --> block number of first emission
    #[pallet::storage]
    pub type FirstEmissionBlockNumber<T: Config> =
        StorageMap<_, Identity, NetUid, u64, OptionQuery>;
    /// --- MAP ( netuid ) --> subnet mechanism
    #[pallet::storage]
    pub type SubnetMechanism<T: Config> =
        StorageMap<_, Identity, NetUid, u16, ValueQuery, DefaultZeroU16<T>>;
    #[pallet::storage]
    /// --- MAP ( netuid ) --> subnetwork_n (Number of UIDs in the network).
    pub type SubnetworkN<T: Config> = StorageMap<_, Identity, NetUid, u16, ValueQuery, DefaultN<T>>;
    #[pallet::storage]
    /// --- MAP ( netuid ) --> network_is_added
    pub type NetworksAdded<T: Config> =
        StorageMap<_, Identity, NetUid, bool, ValueQuery, DefaultNeworksAdded<T>>;
    #[pallet::storage]
    /// --- DMAP ( hotkey, netuid ) --> bool
    pub type IsNetworkMember<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Identity,
        NetUid,
        bool,
        ValueQuery,
        DefaultIsNetworkMember<T>,
    >;
    #[pallet::storage]
    /// --- MAP ( netuid ) --> network_registration_allowed
    pub type NetworkRegistrationAllowed<T: Config> =
        StorageMap<_, Identity, NetUid, bool, ValueQuery, DefaultRegistrationAllowed<T>>;
    #[pallet::storage]
    /// --- MAP ( netuid ) --> network_pow_allowed
    pub type NetworkPowRegistrationAllowed<T: Config> =
        StorageMap<_, Identity, NetUid, bool, ValueQuery, DefaultRegistrationAllowed<T>>;
    #[pallet::storage]
    /// --- MAP ( netuid ) --> block_created
    pub type NetworkRegisteredAt<T: Config> =
        StorageMap<_, Identity, NetUid, u64, ValueQuery, DefaultNetworkRegisteredAt<T>>;
    #[pallet::storage]
    /// --- MAP ( netuid ) --> pending_emission
    pub type PendingEmission<T> =
        StorageMap<_, Identity, NetUid, AlphaCurrency, ValueQuery, DefaultPendingEmission<T>>;
    #[pallet::storage]
    /// --- MAP ( netuid ) --> pending_root_emission
    pub type PendingRootDivs<T> =
        StorageMap<_, Identity, NetUid, TaoCurrency, ValueQuery, DefaultZeroTao<T>>;
    #[pallet::storage]
    /// --- MAP ( netuid ) --> pending_alpha_swapped
    pub type PendingAlphaSwapped<T> =
        StorageMap<_, Identity, NetUid, AlphaCurrency, ValueQuery, DefaultZeroAlpha<T>>;
    #[pallet::storage]
    /// --- MAP ( netuid ) --> pending_owner_cut
    pub type PendingOwnerCut<T> =
        StorageMap<_, Identity, NetUid, AlphaCurrency, ValueQuery, DefaultZeroAlpha<T>>;
    #[pallet::storage]
    /// --- MAP ( netuid ) --> blocks_since_last_step
    pub type BlocksSinceLastStep<T> =
        StorageMap<_, Identity, NetUid, u64, ValueQuery, DefaultBlocksSinceLastStep<T>>;
    #[pallet::storage]
    /// --- MAP ( netuid ) --> last_mechanism_step_block
    pub type LastMechansimStepBlock<T> =
        StorageMap<_, Identity, NetUid, u64, ValueQuery, DefaultLastMechanismStepBlock<T>>;
    #[pallet::storage]
    /// --- MAP ( netuid ) --> subnet_owner
    pub type SubnetOwner<T: Config> =
        StorageMap<_, Identity, NetUid, T::AccountId, ValueQuery, DefaultSubnetOwner<T>>;
    #[pallet::storage]
    /// --- MAP ( netuid ) --> subnet_owner_hotkey
    pub type SubnetOwnerHotkey<T: Config> =
        StorageMap<_, Identity, NetUid, T::AccountId, ValueQuery, DefaultSubnetOwner<T>>;
    #[pallet::storage]
    /// --- MAP ( netuid ) --> recycle_or_burn
    pub type RecycleOrBurn<T: Config> =
        StorageMap<_, Identity, NetUid, RecycleOrBurnEnum, ValueQuery, DefaultRecycleOrBurn<T>>;
    #[pallet::storage]
    /// --- MAP ( netuid ) --> serving_rate_limit
    pub type ServingRateLimit<T> =
        StorageMap<_, Identity, NetUid, u64, ValueQuery, DefaultServingRateLimit<T>>;
    #[pallet::storage]
    /// --- MAP ( netuid ) --> Rho
    pub type Rho<T> = StorageMap<_, Identity, NetUid, u16, ValueQuery, DefaultRho<T>>;
    #[pallet::storage]
    /// --- MAP ( netuid ) --> AlphaSigmoidSteepness
    pub type AlphaSigmoidSteepness<T> =
        StorageMap<_, Identity, NetUid, i16, ValueQuery, DefaultAlphaSigmoidSteepness<T>>;
    #[pallet::storage]
    /// --- MAP ( netuid ) --> Kappa
    pub type Kappa<T> = StorageMap<_, Identity, NetUid, u16, ValueQuery, DefaultKappa<T>>;
    #[pallet::storage]
    /// --- MAP ( netuid ) --> registrations_this_interval
    pub type RegistrationsThisInterval<T: Config> =
        StorageMap<_, Identity, NetUid, u16, ValueQuery>;
    #[pallet::storage]
    /// --- MAP ( netuid ) --> pow_registrations_this_interval
    pub type POWRegistrationsThisInterval<T: Config> =
        StorageMap<_, Identity, NetUid, u16, ValueQuery>;
    #[pallet::storage]
    /// --- MAP ( netuid ) --> burn_registrations_this_interval
    pub type BurnRegistrationsThisInterval<T: Config> =
        StorageMap<_, Identity, NetUid, u16, ValueQuery>;
    #[pallet::storage]
    /// --- MAP ( netuid ) --> max_allowed_uids
    pub type MaxAllowedUids<T> =
        StorageMap<_, Identity, NetUid, u16, ValueQuery, DefaultMaxAllowedUids<T>>;
    #[pallet::storage]
    /// --- MAP ( netuid ) --> immunity_period
    pub type ImmunityPeriod<T> =
        StorageMap<_, Identity, NetUid, u16, ValueQuery, DefaultImmunityPeriod<T>>;
    #[pallet::storage]
    /// --- MAP ( netuid ) --> activity_cutoff
    pub type ActivityCutoff<T> =
        StorageMap<_, Identity, NetUid, u16, ValueQuery, DefaultActivityCutoff<T>>;
    #[pallet::storage]
    /// --- MAP ( netuid ) --> max_weight_limit
    pub type MaxWeightsLimit<T> =
        StorageMap<_, Identity, NetUid, u16, ValueQuery, DefaultMaxWeightsLimit<T>>;
    #[pallet::storage]
    /// --- MAP ( netuid ) --> weights_version_key
    pub type WeightsVersionKey<T> =
        StorageMap<_, Identity, NetUid, u64, ValueQuery, DefaultWeightsVersionKey<T>>;
    #[pallet::storage]
    /// --- MAP ( netuid ) --> min_allowed_weights
    pub type MinAllowedWeights<T> =
        StorageMap<_, Identity, NetUid, u16, ValueQuery, DefaultMinAllowedWeights<T>>;
    #[pallet::storage]
    /// --- MAP ( netuid ) --> max_allowed_validators
    pub type MaxAllowedValidators<T> =
        StorageMap<_, Identity, NetUid, u16, ValueQuery, DefaultMaxAllowedValidators<T>>;
    #[pallet::storage]
    /// --- MAP ( netuid ) --> adjustment_interval
    pub type AdjustmentInterval<T> =
        StorageMap<_, Identity, NetUid, u16, ValueQuery, DefaultAdjustmentInterval<T>>;
    #[pallet::storage]
    /// --- MAP ( netuid ) --> bonds_moving_average
    pub type BondsMovingAverage<T> =
        StorageMap<_, Identity, NetUid, u64, ValueQuery, DefaultBondsMovingAverage<T>>;
    #[pallet::storage]
    /// --- MAP ( netuid ) --> bonds_penalty
    pub type BondsPenalty<T> =
        StorageMap<_, Identity, NetUid, u16, ValueQuery, DefaultBondsPenalty<T>>;
    #[pallet::storage]
    /// --- MAP ( netuid ) --> bonds_reset
    pub type BondsResetOn<T> =
        StorageMap<_, Identity, NetUid, bool, ValueQuery, DefaultBondsResetOn<T>>;
    /// --- MAP ( netuid ) --> weights_set_rate_limit
    #[pallet::storage]
    pub type WeightsSetRateLimit<T> =
        StorageMap<_, Identity, NetUid, u64, ValueQuery, DefaultWeightsSetRateLimit<T>>;
    #[pallet::storage]
    /// --- MAP ( netuid ) --> validator_prune_len
    pub type ValidatorPruneLen<T> =
        StorageMap<_, Identity, NetUid, u64, ValueQuery, DefaultValidatorPruneLen<T>>;
    #[pallet::storage]
    /// --- MAP ( netuid ) --> scaling_law_power
    pub type ScalingLawPower<T> =
        StorageMap<_, Identity, NetUid, u16, ValueQuery, DefaultScalingLawPower<T>>;
    #[pallet::storage]
    /// --- MAP ( netuid ) --> target_registrations_this_interval
    pub type TargetRegistrationsPerInterval<T> =
        StorageMap<_, Identity, NetUid, u16, ValueQuery, DefaultTargetRegistrationsPerInterval<T>>;
    #[pallet::storage]
    /// --- MAP ( netuid ) --> adjustment_alpha
    pub type AdjustmentAlpha<T: Config> =
        StorageMap<_, Identity, NetUid, u64, ValueQuery, DefaultAdjustmentAlpha<T>>;
    #[pallet::storage]
    /// --- MAP ( netuid ) --> commit reveal v2 weights are enabled
    pub type CommitRevealWeightsEnabled<T> =
        StorageMap<_, Identity, NetUid, bool, ValueQuery, DefaultCommitRevealWeightsEnabled<T>>;
    #[pallet::storage]
    /// --- MAP ( netuid ) --> Burn
    pub type Burn<T> = StorageMap<_, Identity, NetUid, TaoCurrency, ValueQuery, DefaultBurn<T>>;
    #[pallet::storage]
    /// --- MAP ( netuid ) --> Difficulty
    pub type Difficulty<T> = StorageMap<_, Identity, NetUid, u64, ValueQuery, DefaultDifficulty<T>>;
    #[pallet::storage]
    /// --- MAP ( netuid ) --> MinBurn
    pub type MinBurn<T> =
        StorageMap<_, Identity, NetUid, TaoCurrency, ValueQuery, DefaultMinBurn<T>>;
    #[pallet::storage]
    /// --- MAP ( netuid ) --> MaxBurn
    pub type MaxBurn<T> =
        StorageMap<_, Identity, NetUid, TaoCurrency, ValueQuery, DefaultMaxBurn<T>>;
    #[pallet::storage]
    /// --- MAP ( netuid ) --> MinDifficulty
    pub type MinDifficulty<T> =
        StorageMap<_, Identity, NetUid, u64, ValueQuery, DefaultMinDifficulty<T>>;
    #[pallet::storage]
    /// --- MAP ( netuid ) --> MaxDifficulty
    pub type MaxDifficulty<T> =
        StorageMap<_, Identity, NetUid, u64, ValueQuery, DefaultMaxDifficulty<T>>;
    #[pallet::storage]
    /// --- MAP ( netuid ) -->  Block at last adjustment.
    pub type LastAdjustmentBlock<T> =
        StorageMap<_, Identity, NetUid, u64, ValueQuery, DefaultLastAdjustmentBlock<T>>;
    #[pallet::storage]
    /// --- MAP ( netuid ) --> Registrations of this Block.
    pub type RegistrationsThisBlock<T> =
        StorageMap<_, Identity, NetUid, u16, ValueQuery, DefaultRegistrationsThisBlock<T>>;
    #[pallet::storage]
    /// --- MAP ( netuid ) --> Halving time of average moving price.
    pub type EMAPriceHalvingBlocks<T> =
        StorageMap<_, Identity, NetUid, u64, ValueQuery, DefaultEMAPriceMovingBlocks<T>>;
    #[pallet::storage]
    /// --- MAP ( netuid ) --> global_RAO_recycled_for_registration
    pub type RAORecycledForRegistration<T> = StorageMap<
        _,
        Identity,
        NetUid,
        TaoCurrency,
        ValueQuery,
        DefaultRAORecycledForRegistration<T>,
    >;
    #[pallet::storage]
    /// --- ITEM ( tx_rate_limit )
    pub type TxRateLimit<T> = StorageValue<_, u64, ValueQuery, DefaultTxRateLimit<T>>;
    #[pallet::storage]
    /// --- ITEM ( tx_delegate_take_rate_limit )
    pub type TxDelegateTakeRateLimit<T> =
        StorageValue<_, u64, ValueQuery, DefaultTxDelegateTakeRateLimit<T>>;
    #[pallet::storage]
    /// --- ITEM ( tx_childkey_take_rate_limit )
    pub type TxChildkeyTakeRateLimit<T> =
        StorageValue<_, u64, ValueQuery, DefaultTxChildKeyTakeRateLimit<T>>;
    #[pallet::storage]
    /// --- MAP ( netuid ) --> Whether or not Liquid Alpha is enabled
    pub type LiquidAlphaOn<T> =
        StorageMap<_, Blake2_128Concat, NetUid, bool, ValueQuery, DefaultLiquidAlpha<T>>;
    #[pallet::storage]
    /// --- MAP ( netuid ) --> Whether or not Yuma3 is enabled
    pub type Yuma3On<T> =
        StorageMap<_, Blake2_128Concat, NetUid, bool, ValueQuery, DefaultYuma3<T>>;
    #[pallet::storage]
    ///  MAP ( netuid ) --> (alpha_low, alpha_high)
    pub type AlphaValues<T> =
        StorageMap<_, Identity, NetUid, (u16, u16), ValueQuery, DefaultAlphaValues<T>>;
    #[pallet::storage]
    /// --- MAP ( netuid ) --> If subtoken trading enabled
    pub type SubtokenEnabled<T> =
        StorageMap<_, Identity, NetUid, bool, ValueQuery, DefaultFalse<T>>;

    #[pallet::type_value]
    /// Default value for burn keys limit
    pub fn DefaultImmuneOwnerUidsLimit<T: Config>() -> u16 {
        1
    }
    #[pallet::type_value]
    /// Maximum value for burn keys limit
    pub fn MaxImmuneOwnerUidsLimit<T: Config>() -> u16 {
        10
    }
    #[pallet::type_value]
    /// Minimum value for burn keys limit
    pub fn MinImmuneOwnerUidsLimit<T: Config>() -> u16 {
        1
    }
    #[pallet::storage]
    /// --- MAP ( netuid ) --> Burn key limit
    pub type ImmuneOwnerUidsLimit<T> =
        StorageMap<_, Identity, NetUid, u16, ValueQuery, DefaultImmuneOwnerUidsLimit<T>>;

    /// =======================================
    /// ==== Subnetwork Consensus Storage  ====
    /// =======================================
    #[pallet::storage] // --- DMAP ( netuid ) --> stake_weight | weight for stake used in YC.
    pub(super) type StakeWeight<T: Config> =
        StorageMap<_, Identity, NetUid, Vec<u16>, ValueQuery, EmptyU16Vec<T>>;
    #[pallet::storage]
    /// --- DMAP ( netuid, hotkey ) --> uid
    pub type Uids<T: Config> =
        StorageDoubleMap<_, Identity, NetUid, Blake2_128Concat, T::AccountId, u16, OptionQuery>;
    #[pallet::storage]
    /// --- DMAP ( netuid, uid ) --> hotkey
    pub type Keys<T: Config> = StorageDoubleMap<
        _,
        Identity,
        NetUid,
        Identity,
        u16,
        T::AccountId,
        ValueQuery,
        DefaultKey<T>,
    >;
    #[pallet::storage]
    /// --- MAP ( netuid ) --> (hotkey, se, ve)
    pub type LoadedEmission<T: Config> =
        StorageMap<_, Identity, NetUid, Vec<(T::AccountId, u64, u64)>, OptionQuery>;
    #[pallet::storage]
    /// --- MAP ( netuid ) --> active
    pub type Active<T: Config> =
        StorageMap<_, Identity, NetUid, Vec<bool>, ValueQuery, EmptyBoolVec<T>>;
    #[pallet::storage]
    /// --- MAP ( netuid ) --> rank
    pub type Rank<T: Config> =
        StorageMap<_, Identity, NetUid, Vec<u16>, ValueQuery, EmptyU16Vec<T>>;
    #[pallet::storage]
    /// --- MAP ( netuid ) --> trust
    pub type Trust<T: Config> =
        StorageMap<_, Identity, NetUid, Vec<u16>, ValueQuery, EmptyU16Vec<T>>;
    #[pallet::storage]
    /// --- MAP ( netuid ) --> consensus
    pub type Consensus<T: Config> =
        StorageMap<_, Identity, NetUid, Vec<u16>, ValueQuery, EmptyU16Vec<T>>;
    #[pallet::storage]
    /// --- MAP ( netuid ) --> incentive
    pub type Incentive<T: Config> =
        StorageMap<_, Identity, NetUidStorageIndex, Vec<u16>, ValueQuery, EmptyU16Vec<T>>;
    #[pallet::storage]
    /// --- MAP ( netuid ) --> dividends
    pub type Dividends<T: Config> =
        StorageMap<_, Identity, NetUid, Vec<u16>, ValueQuery, EmptyU16Vec<T>>;
    #[pallet::storage]
    /// --- MAP ( netuid ) --> emission
    pub type Emission<T: Config> = StorageMap<_, Identity, NetUid, Vec<AlphaCurrency>, ValueQuery>;
    #[pallet::storage]
    /// --- MAP ( netuid ) --> last_update
    pub type LastUpdate<T: Config> =
        StorageMap<_, Identity, NetUidStorageIndex, Vec<u64>, ValueQuery, EmptyU64Vec<T>>;
    #[pallet::storage]
    /// --- MAP ( netuid ) --> validator_trust
    pub type ValidatorTrust<T: Config> =
        StorageMap<_, Identity, NetUid, Vec<u16>, ValueQuery, EmptyU16Vec<T>>;
    #[pallet::storage]
    /// --- MAP ( netuid ) --> pruning_scores
    pub type PruningScores<T: Config> =
        StorageMap<_, Identity, NetUid, Vec<u16>, ValueQuery, EmptyU16Vec<T>>;
    #[pallet::storage]
    /// --- MAP ( netuid ) --> validator_permit
    pub type ValidatorPermit<T: Config> =
        StorageMap<_, Identity, NetUid, Vec<bool>, ValueQuery, EmptyBoolVec<T>>;
    #[pallet::storage]
    /// --- DMAP ( netuid, uid ) --> weights
    pub type Weights<T: Config> = StorageDoubleMap<
        _,
        Identity,
        NetUidStorageIndex,
        Identity,
        u16,
        Vec<(u16, u16)>,
        ValueQuery,
        DefaultWeights<T>,
    >;
    #[pallet::storage]
    /// --- DMAP ( netuid, uid ) --> bonds
    pub type Bonds<T: Config> = StorageDoubleMap<
        _,
        Identity,
        NetUidStorageIndex,
        Identity,
        u16,
        Vec<(u16, u16)>,
        ValueQuery,
        DefaultBonds<T>,
    >;
    #[pallet::storage]
    /// --- DMAP ( netuid, uid ) --> block_at_registration
    pub type BlockAtRegistration<T: Config> = StorageDoubleMap<
        _,
        Identity,
        NetUid,
        Identity,
        u16,
        u64,
        ValueQuery,
        DefaultBlockAtRegistration<T>,
    >;
    #[pallet::storage]
    /// --- MAP ( netuid, hotkey ) --> axon_info
    pub type Axons<T: Config> = StorageDoubleMap<
        _,
        Identity,
        NetUid,
        Blake2_128Concat,
        T::AccountId,
        AxonInfoOf,
        OptionQuery,
    >;
    /// --- MAP ( netuid, hotkey ) --> certificate
    #[pallet::storage]
    pub type NeuronCertificates<T: Config> = StorageDoubleMap<
        _,
        Identity,
        NetUid,
        Blake2_128Concat,
        T::AccountId,
        NeuronCertificateOf,
        OptionQuery,
    >;
    #[pallet::storage]
    /// --- MAP ( netuid, hotkey ) --> prometheus_info
    pub type Prometheus<T: Config> = StorageDoubleMap<
        _,
        Identity,
        NetUid,
        Blake2_128Concat,
        T::AccountId,
        PrometheusInfoOf,
        OptionQuery,
    >;
    #[pallet::storage] // --- MAP ( coldkey ) --> identity. (DEPRECATED for V2)
    pub type Identities<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, ChainIdentityOf, OptionQuery>;

    #[pallet::storage] // --- MAP ( coldkey ) --> identity
    pub type IdentitiesV2<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, ChainIdentityOfV2, OptionQuery>;

    #[pallet::storage] // --- MAP ( netuid ) --> identity. (DEPRECATED for V2)
    pub type SubnetIdentities<T: Config> =
        StorageMap<_, Blake2_128Concat, NetUid, SubnetIdentityOf, OptionQuery>;

    #[pallet::storage] // --- MAP ( netuid ) --> identityV2 (DEPRECATED for V3)
    pub type SubnetIdentitiesV2<T: Config> =
        StorageMap<_, Blake2_128Concat, NetUid, SubnetIdentityOfV2, OptionQuery>;

    #[pallet::storage] // --- MAP ( netuid ) --> SubnetIdentityOfV3
    pub type SubnetIdentitiesV3<T: Config> =
        StorageMap<_, Blake2_128Concat, NetUid, SubnetIdentityOfV3, OptionQuery>;

    /// =================================
    /// ==== Axon / Promo Endpoints =====
    /// =================================
    #[pallet::storage] // --- NMAP ( hot, netuid, name ) --> last_block | Returns the last block of a transaction for a given key, netuid, and name.
    pub type TransactionKeyLastBlock<T: Config> = StorageNMap<
        _,
        (
            NMapKey<Blake2_128Concat, T::AccountId>, // hot
            NMapKey<Identity, NetUid>,               // netuid
            NMapKey<Identity, u16>,                  // extrinsic enum.
        ),
        u64,
        ValueQuery,
    >;
    #[deprecated]
    #[pallet::storage]
    /// --- MAP ( key ) --> last_block
    pub type LastTxBlock<T: Config> =
        StorageMap<_, Identity, T::AccountId, u64, ValueQuery, DefaultLastTxBlock<T>>;
    #[deprecated]
    #[pallet::storage]
    /// --- MAP ( key ) --> last_tx_block_childkey_take
    pub type LastTxBlockChildKeyTake<T: Config> =
        StorageMap<_, Identity, T::AccountId, u64, ValueQuery, DefaultLastTxBlock<T>>;
    #[deprecated]
    #[pallet::storage]
    /// --- MAP ( key ) --> last_tx_block_delegate_take
    pub type LastTxBlockDelegateTake<T: Config> =
        StorageMap<_, Identity, T::AccountId, u64, ValueQuery, DefaultLastTxBlock<T>>;
    // FIXME: this storage is used interchangably for alpha/tao
    #[pallet::storage]
    /// ITEM( weights_min_stake )
    pub type StakeThreshold<T> = StorageValue<_, u64, ValueQuery, DefaultStakeThreshold<T>>;
    #[pallet::storage]
    /// --- MAP (netuid, who) --> VecDeque<(hash, commit_block, first_reveal_block, last_reveal_block)> | Stores a queue of commits for an account on a given netuid.
    pub type WeightCommits<T: Config> = StorageDoubleMap<
        _,
        Twox64Concat,
        NetUidStorageIndex,
        Twox64Concat,
        T::AccountId,
        VecDeque<(H256, u64, u64, u64)>,
        OptionQuery,
    >;
    #[pallet::storage]
    /// MAP (netuid, epoch) → VecDeque<(who, commit_block, ciphertext, reveal_round)>
    /// Stores a queue of weight commits for an account on a given subnet.
    pub type TimelockedWeightCommits<T: Config> = StorageDoubleMap<
        _,
        Twox64Concat,
        NetUidStorageIndex,
        Twox64Concat,
        u64, // epoch key
        VecDeque<(
            T::AccountId,
            u64, // commit_block
            BoundedVec<u8, ConstU32<MAX_CRV3_COMMIT_SIZE_BYTES>>,
            RoundNumber,
        )>,
        ValueQuery,
    >;
    #[pallet::storage]
    /// MAP (netuid, epoch) → VecDeque<(who, ciphertext, reveal_round)>
    /// DEPRECATED for CRV3WeightCommitsV2
    pub type CRV3WeightCommits<T: Config> = StorageDoubleMap<
        _,
        Twox64Concat,
        NetUidStorageIndex,
        Twox64Concat,
        u64, // epoch key
        VecDeque<(
            T::AccountId,
            BoundedVec<u8, ConstU32<MAX_CRV3_COMMIT_SIZE_BYTES>>,
            RoundNumber,
        )>,
        ValueQuery,
    >;
    #[pallet::storage]
    /// MAP (netuid, epoch) → VecDeque<(who, commit_block, ciphertext, reveal_round)>
    /// DEPRECATED for TimelockedWeightCommits
    pub type CRV3WeightCommitsV2<T: Config> = StorageDoubleMap<
        _,
        Twox64Concat,
        NetUidStorageIndex,
        Twox64Concat,
        u64, // epoch key
        VecDeque<(
            T::AccountId,
            u64, // commit_block
            BoundedVec<u8, ConstU32<MAX_CRV3_COMMIT_SIZE_BYTES>>,
            RoundNumber,
        )>,
        ValueQuery,
    >;
    #[pallet::storage]
    /// --- Map (netuid) --> Number of epochs allowed for commit reveal periods
    pub type RevealPeriodEpochs<T: Config> =
        StorageMap<_, Twox64Concat, NetUid, u64, ValueQuery, DefaultRevealPeriodEpochs<T>>;

    #[pallet::storage]
    /// --- Map (coldkey, hotkey) --> u64 the last block at which stake was added/removed.
    pub type LastColdkeyHotkeyStakeBlock<T: Config> = StorageDoubleMap<
        _,
        Twox64Concat,
        T::AccountId,
        Twox64Concat,
        T::AccountId,
        u64,
        OptionQuery,
    >;

    #[pallet::storage]
    /// DMAP ( hot, cold, netuid ) --> rate limits for staking operations
    /// Value contains just a marker: we use this map as a set.
    pub type StakingOperationRateLimiter<T: Config> = StorageNMap<
        _,
        (
            NMapKey<Blake2_128Concat, T::AccountId>, // hot
            NMapKey<Blake2_128Concat, T::AccountId>, // cold
            NMapKey<Identity, NetUid>,               // subnet
        ),
        bool,
        ValueQuery,
    >;

    /// =============================
    /// ==== EVM related storage ====
    /// =============================
    #[pallet::storage]
    /// --- DMAP (netuid, uid) --> (H160, last_block_where_ownership_was_proven)
    pub type AssociatedEvmAddress<T: Config> =
        StorageDoubleMap<_, Twox64Concat, NetUid, Twox64Concat, u16, (H160, u64), OptionQuery>;

    /// ========================
    /// ==== Subnet Leasing ====
    /// ========================
    #[pallet::storage]
    /// --- MAP ( lease_id ) --> subnet lease | The subnet lease for a given lease id.
    pub type SubnetLeases<T: Config> =
        StorageMap<_, Twox64Concat, LeaseId, SubnetLeaseOf<T>, OptionQuery>;

    #[pallet::storage]
    /// --- DMAP ( lease_id, contributor ) --> shares | The shares of a contributor for a given lease.
    pub type SubnetLeaseShares<T: Config> =
        StorageDoubleMap<_, Twox64Concat, LeaseId, Identity, T::AccountId, U64F64, ValueQuery>;

    #[pallet::storage]
    // --- MAP ( netuid ) --> lease_id | The lease id for a given netuid.
    pub type SubnetUidToLeaseId<T: Config> =
        StorageMap<_, Twox64Concat, NetUid, LeaseId, OptionQuery>;

    #[pallet::storage]
    /// --- ITEM ( next_lease_id ) | The next lease id.
    pub type NextSubnetLeaseId<T: Config> = StorageValue<_, LeaseId, ValueQuery, ConstU32<0>>;

    #[pallet::storage]
    /// --- MAP ( lease_id ) --> accumulated_dividends | The accumulated dividends for a given lease that needs to be distributed.
    pub type AccumulatedLeaseDividends<T: Config> =
        StorageMap<_, Twox64Concat, LeaseId, AlphaCurrency, ValueQuery, DefaultZeroAlpha<T>>;

    #[pallet::storage]
    /// --- ITEM ( CommitRevealWeightsVersion )
    pub type CommitRevealWeightsVersion<T> =
        StorageValue<_, u16, ValueQuery, DefaultCommitRevealWeightsVersion<T>>;

    /// ======================
    /// ==== Sub-subnets =====
    /// ======================
    #[pallet::type_value]
    /// -- ITEM (Default number of sub-subnets)
    pub fn DefaultSubsubnetCount<T: Config>() -> SubId {
        SubId::from(1)
    }
    #[pallet::type_value]
    /// -- ITEM (Maximum number of sub-subnets)
    pub fn MaxSubsubnetCount<T: Config>() -> SubId {
        SubId::from(8)
    }
    #[pallet::type_value]
    /// -- ITEM (Rate limit for subsubnet count updates)
    pub fn SubsubnetCountSetRateLimit<T: Config>() -> u64 {
        prod_or_fast!(7_200, 1)
    }
    #[pallet::type_value]
    /// -- ITEM (Rate limit for subsubnet emission distribution updates)
    pub fn SubsubnetEmissionRateLimit<T: Config>() -> u64 {
        prod_or_fast!(7_200, 1)
    }
    #[pallet::storage]
    /// --- MAP ( netuid ) --> Current number of sub-subnets
    pub type SubsubnetCountCurrent<T: Config> =
        StorageMap<_, Twox64Concat, NetUid, SubId, ValueQuery, DefaultSubsubnetCount<T>>;
    #[pallet::storage]
    /// --- MAP ( netuid ) --> Normalized vector of emission split proportion between subsubnets
    pub type SubsubnetEmissionSplit<T: Config> =
        StorageMap<_, Twox64Concat, NetUid, Vec<u16>, OptionQuery>;

    /// ==================
    /// ==== Genesis =====
    /// ==================
    #[pallet::storage] // --- Storage for migration run status
    pub type HasMigrationRun<T: Config> = StorageMap<_, Identity, Vec<u8>, bool, ValueQuery>;

    #[pallet::type_value]
    /// Default value for pending childkey cooldown (settable by root, default 0)
    pub fn DefaultPendingChildKeyCooldown<T: Config>() -> u64 {
        0
    }

    #[pallet::storage]
    /// Storage value for pending childkey cooldown, settable by root.
    pub type PendingChildKeyCooldown<T: Config> =
        StorageValue<_, u64, ValueQuery, DefaultPendingChildKeyCooldown<T>>;

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        /// Stakes record in genesis.
        pub stakes: Vec<(T::AccountId, Vec<(T::AccountId, (u64, u16))>)>,
        /// The total issued balance in genesis
        pub balances_issuance: TaoCurrency,
    }

    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                stakes: Default::default(),
                balances_issuance: TaoCurrency::ZERO,
            }
        }
    }

    // ---- Subtensor helper functions.
    impl<T: Config> Pallet<T> {
        /// Is the caller allowed to set weights
        pub fn check_weights_min_stake(hotkey: &T::AccountId, netuid: NetUid) -> bool {
            // Blacklist weights transactions for low stake peers.
            let (total_stake, _, _) = Self::get_stake_weights_for_hotkey_on_subnet(hotkey, netuid);
            total_stake >= Self::get_stake_threshold()
        }

        /// Helper function to check if register is allowed
        pub fn checked_allowed_register(netuid: NetUid) -> bool {
            if netuid.is_root() {
                return false;
            }
            if !Self::if_subnet_exist(netuid) {
                return false;
            }
            if !Self::get_network_registration_allowed(netuid) {
                return false;
            }
            if Self::get_registrations_this_block(netuid)
                >= Self::get_max_registrations_per_block(netuid)
            {
                return false;
            }
            if Self::get_registrations_this_interval(netuid)
                >= Self::get_target_registrations_per_interval(netuid).saturating_mul(3)
            {
                return false;
            }
            true
        }

        /// Ensure subtoken enalbed
        pub fn ensure_subtoken_enabled(subnet: NetUid) -> Result<(), Error<T>> {
            ensure!(
                SubtokenEnabled::<T>::get(subnet),
                Error::<T>::SubtokenDisabled
            );
            Ok(())
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum CustomTransactionError {
    ColdkeyInSwapSchedule,
    StakeAmountTooLow,
    BalanceTooLow,
    SubnetDoesntExist,
    HotkeyAccountDoesntExist,
    NotEnoughStakeToWithdraw,
    RateLimitExceeded,
    InsufficientLiquidity,
    SlippageTooHigh,
    TransferDisallowed,
    HotKeyNotRegisteredInNetwork,
    InvalidIpAddress,
    ServingRateLimitExceeded,
    InvalidPort,
    BadRequest,
    ZeroMaxAmount,
    InvalidRevealRound,
    CommitNotFound,
    CommitBlockNotInRevealRange,
    InputLengthsUnequal,
}

impl From<CustomTransactionError> for u8 {
    fn from(variant: CustomTransactionError) -> u8 {
        match variant {
            CustomTransactionError::ColdkeyInSwapSchedule => 0,
            CustomTransactionError::StakeAmountTooLow => 1,
            CustomTransactionError::BalanceTooLow => 2,
            CustomTransactionError::SubnetDoesntExist => 3,
            CustomTransactionError::HotkeyAccountDoesntExist => 4,
            CustomTransactionError::NotEnoughStakeToWithdraw => 5,
            CustomTransactionError::RateLimitExceeded => 6,
            CustomTransactionError::InsufficientLiquidity => 7,
            CustomTransactionError::SlippageTooHigh => 8,
            CustomTransactionError::TransferDisallowed => 9,
            CustomTransactionError::HotKeyNotRegisteredInNetwork => 10,
            CustomTransactionError::InvalidIpAddress => 11,
            CustomTransactionError::ServingRateLimitExceeded => 12,
            CustomTransactionError::InvalidPort => 13,
            CustomTransactionError::BadRequest => 255,
            CustomTransactionError::ZeroMaxAmount => 14,
            CustomTransactionError::InvalidRevealRound => 15,
            CustomTransactionError::CommitNotFound => 16,
            CustomTransactionError::CommitBlockNotInRevealRange => 17,
            CustomTransactionError::InputLengthsUnequal => 18,
        }
    }
}

impl From<CustomTransactionError> for TransactionValidityError {
    fn from(variant: CustomTransactionError) -> Self {
        TransactionValidityError::Invalid(InvalidTransaction::Custom(variant.into()))
    }
}

use sp_std::vec;

// TODO: unravel this rats nest, for some reason rustc thinks this is unused even though it's
// used not 25 lines below
#[allow(unused)]
use sp_std::vec::Vec;
use subtensor_macros::freeze_struct;

/// Trait for managing a membership pallet instance in the runtime
pub trait MemberManagement<AccountId> {
    /// Add member
    fn add_member(account: &AccountId) -> DispatchResultWithPostInfo;

    /// Remove a member
    fn remove_member(account: &AccountId) -> DispatchResultWithPostInfo;

    /// Swap member
    fn swap_member(remove: &AccountId, add: &AccountId) -> DispatchResultWithPostInfo;

    /// Get all members
    fn members() -> Vec<AccountId>;

    /// Check if an account is apart of the set
    fn is_member(account: &AccountId) -> bool;

    /// Get our maximum member count
    fn max_members() -> u32;
}

impl<T> MemberManagement<T> for () {
    /// Add member
    fn add_member(_: &T) -> DispatchResultWithPostInfo {
        Ok(().into())
    }

    // Remove a member
    fn remove_member(_: &T) -> DispatchResultWithPostInfo {
        Ok(().into())
    }

    // Swap member
    fn swap_member(_: &T, _: &T) -> DispatchResultWithPostInfo {
        Ok(().into())
    }

    // Get all members
    fn members() -> Vec<T> {
        vec![]
    }

    // Check if an account is apart of the set
    fn is_member(_: &T) -> bool {
        false
    }

    fn max_members() -> u32 {
        0
    }
}

/// Trait for interacting with collective pallets
pub trait CollectiveInterface<AccountId, Hash, ProposalIndex> {
    /// Remove vote
    fn remove_votes(hotkey: &AccountId) -> Result<bool, DispatchError>;

    fn add_vote(
        hotkey: &AccountId,
        proposal: Hash,
        index: ProposalIndex,
        approve: bool,
    ) -> Result<bool, DispatchError>;
}

impl<T, H, P> CollectiveInterface<T, H, P> for () {
    fn remove_votes(_: &T) -> Result<bool, DispatchError> {
        Ok(true)
    }

    fn add_vote(_: &T, _: H, _: P, _: bool) -> Result<bool, DispatchError> {
        Ok(true)
    }
}

impl<T: Config + pallet_balances::Config<Balance = u64>>
    subtensor_runtime_common::SubnetInfo<T::AccountId> for Pallet<T>
{
    fn tao_reserve(netuid: NetUid) -> TaoCurrency {
        SubnetTAO::<T>::get(netuid).saturating_add(SubnetTaoProvided::<T>::get(netuid))
    }

    fn alpha_reserve(netuid: NetUid) -> AlphaCurrency {
        SubnetAlphaIn::<T>::get(netuid).saturating_add(SubnetAlphaInProvided::<T>::get(netuid))
    }

    fn exists(netuid: NetUid) -> bool {
        Self::if_subnet_exist(netuid)
    }

    fn mechanism(netuid: NetUid) -> u16 {
        SubnetMechanism::<T>::get(netuid)
    }

    fn is_owner(account_id: &T::AccountId, netuid: NetUid) -> bool {
        SubnetOwner::<T>::get(netuid) == *account_id
    }

    fn is_subtoken_enabled(netuid: NetUid) -> bool {
        SubtokenEnabled::<T>::get(netuid)
    }
}

impl<T: Config + pallet_balances::Config<Balance = u64>>
    subtensor_runtime_common::BalanceOps<T::AccountId> for Pallet<T>
{
    fn tao_balance(account_id: &T::AccountId) -> TaoCurrency {
        pallet_balances::Pallet::<T>::free_balance(account_id).into()
    }

    fn alpha_balance(
        netuid: NetUid,
        coldkey: &T::AccountId,
        hotkey: &T::AccountId,
    ) -> AlphaCurrency {
        Self::get_stake_for_hotkey_and_coldkey_on_subnet(hotkey, coldkey, netuid)
    }

    fn increase_balance(coldkey: &T::AccountId, tao: TaoCurrency) {
        Self::add_balance_to_coldkey_account(coldkey, tao.into())
    }

    fn decrease_balance(
        coldkey: &T::AccountId,
        tao: TaoCurrency,
    ) -> Result<TaoCurrency, DispatchError> {
        Self::remove_balance_from_coldkey_account(coldkey, tao.into())
    }

    fn increase_stake(
        coldkey: &T::AccountId,
        hotkey: &T::AccountId,
        netuid: NetUid,
        alpha: AlphaCurrency,
    ) -> Result<(), DispatchError> {
        ensure!(
            Self::hotkey_account_exists(hotkey),
            Error::<T>::HotKeyAccountNotExists
        );

        // Increse alpha out counter
        SubnetAlphaOut::<T>::mutate(netuid, |total| {
            *total = total.saturating_add(alpha);
        });

        Self::increase_stake_for_hotkey_and_coldkey_on_subnet(hotkey, coldkey, netuid, alpha);

        Ok(())
    }

    fn decrease_stake(
        coldkey: &T::AccountId,
        hotkey: &T::AccountId,
        netuid: NetUid,
        alpha: AlphaCurrency,
    ) -> Result<AlphaCurrency, DispatchError> {
        ensure!(
            Self::hotkey_account_exists(hotkey),
            Error::<T>::HotKeyAccountNotExists
        );

        // Decrese alpha out counter
        SubnetAlphaOut::<T>::mutate(netuid, |total| {
            *total = total.saturating_sub(alpha);
        });

        Ok(Self::decrease_stake_for_hotkey_and_coldkey_on_subnet(
            hotkey, coldkey, netuid, alpha,
        ))
    }

    fn increase_provided_tao_reserve(netuid: NetUid, tao: TaoCurrency) {
        Self::increase_provided_tao_reserve(netuid, tao);
    }

    fn decrease_provided_tao_reserve(netuid: NetUid, tao: TaoCurrency) {
        Self::decrease_provided_tao_reserve(netuid, tao);
    }

    fn increase_provided_alpha_reserve(netuid: NetUid, alpha: AlphaCurrency) {
        Self::increase_provided_alpha_reserve(netuid, alpha);
    }

    fn decrease_provided_alpha_reserve(netuid: NetUid, alpha: AlphaCurrency) {
        Self::decrease_provided_alpha_reserve(netuid, alpha);
    }
}

/// Enum that defines types of rate limited operations for
/// storing last block when this operation occured
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo)]
pub enum RateLimitKey<AccountId> {
    // The setting sn owner hotkey operation is rate limited per netuid
    SetSNOwnerHotkey(NetUid),
    // Generic rate limit for subnet-owner hyperparameter updates (per netuid)
    OwnerHyperparamUpdate(NetUid, Hyperparameter),
    // Subnet registration rate limit
    NetworkLastRegistered,
    // Last tx block limit per account ID
    LastTxBlock(AccountId),
    // Last tx block child key limit per account ID
    LastTxBlockChildKeyTake(AccountId),
    // Last tx block delegate key limit per account ID
    LastTxBlockDelegateTake(AccountId),
}

pub trait ProxyInterface<AccountId> {
    fn add_lease_beneficiary_proxy(beneficiary: &AccountId, lease: &AccountId) -> DispatchResult;
    fn remove_lease_beneficiary_proxy(beneficiary: &AccountId, lease: &AccountId)
    -> DispatchResult;
}

impl<T> ProxyInterface<T> for () {
    fn add_lease_beneficiary_proxy(_: &T, _: &T) -> DispatchResult {
        Ok(())
    }

    fn remove_lease_beneficiary_proxy(_: &T, _: &T) -> DispatchResult {
        Ok(())
    }
}
