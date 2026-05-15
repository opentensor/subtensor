use core::num::NonZeroU64;

use frame_support::{PalletId, pallet_prelude::*, traits::Get};
use frame_system::pallet_prelude::*;
use sp_arithmetic::Perbill;
use substrate_fixed::types::U64F64;
use subtensor_runtime_common::{
    AlphaBalance, BalanceOps, NetUid, SubnetInfo, TaoBalance, TokenReserve,
};

use crate::{
    position::{Position, PositionId},
    tick::{LayerLevel, Tick, TickIndex},
    weights::WeightInfo,
};

pub use pallet::*;

mod impls;
mod swap_step;
#[cfg(test)]
mod tests;

#[allow(clippy::module_inception)]
#[frame_support::pallet]
#[allow(clippy::expect_used)]
mod pallet {
    use super::*;
    use frame_system::{ensure_root, ensure_signed};

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Implementor of
        /// [`SubnetInfo`](subtensor_swap_interface::SubnetInfo).
        type SubnetInfo: SubnetInfo<Self::AccountId>;

        /// Tao reserves info.
        type TaoReserve: TokenReserve<TaoBalance>;

        /// Alpha reserves info.
        type AlphaReserve: TokenReserve<AlphaBalance>;

        /// Implementor of
        /// [`BalanceOps`](subtensor_swap_interface::BalanceOps).
        type BalanceOps: BalanceOps<Self::AccountId>;

        /// This type is used to derive protocol accoun ID.
        #[pallet::constant]
        type ProtocolId: Get<PalletId>;

        /// The maximum fee rate that can be set
        #[pallet::constant]
        type MaxFeeRate: Get<u16>;

        /// The maximum number of positions a user can have
        #[pallet::constant]
        type MaxPositions: Get<u32>;

        /// Minimum liquidity that is safe for rounding and integer math.
        #[pallet::constant]
        type MinimumLiquidity: Get<u64>;

        /// Minimum reserve for tao and alpha
        #[pallet::constant]
        type MinimumReserve: Get<NonZeroU64>;

        /// Weight information for extrinsics in this pallet.
        type WeightInfo: WeightInfo;

        /// Helper for setting up cross-pallet state needed by benchmarks.
        #[cfg(feature = "runtime-benchmarks")]
        type BenchmarkHelper: BenchmarkHelper<Self::AccountId>;
    }

    /// Benchmark setup helper — the runtime wires this to set state in other pallets.
    #[cfg(feature = "runtime-benchmarks")]
    pub trait BenchmarkHelper<AccountId> {
        fn setup_subnet(netuid: NetUid);
        fn register_hotkey(hotkey: &AccountId, coldkey: &AccountId);
    }

    #[cfg(feature = "runtime-benchmarks")]
    impl<AccountId> BenchmarkHelper<AccountId> for () {
        fn setup_subnet(_netuid: NetUid) {}
        fn register_hotkey(_hotkey: &AccountId, _coldkey: &AccountId) {}
    }

    /// Default fee rate if not set
    #[pallet::type_value]
    pub fn DefaultFeeRate() -> u16 {
        33 // ~0.05 %
    }

    /// Fee split between pool and block builder.
    /// Pool receives the portion returned by this function
    #[pallet::type_value]
    pub fn DefaultFeeSplit() -> Perbill {
        Perbill::zero()
    }

    /// The fee rate applied to swaps per subnet, normalized value between 0 and u16::MAX
    #[pallet::storage]
    pub type FeeRate<T> = StorageMap<_, Twox64Concat, NetUid, u16, ValueQuery, DefaultFeeRate>;

    // Global accrued fees in tao per subnet
    #[pallet::storage]
    pub type FeeGlobalTao<T> = StorageMap<_, Twox64Concat, NetUid, U64F64, ValueQuery>;

    // Global accrued fees in alpha per subnet
    #[pallet::storage]
    pub type FeeGlobalAlpha<T> = StorageMap<_, Twox64Concat, NetUid, U64F64, ValueQuery>;

    /// Storage for all ticks, using subnet ID as the primary key and tick index as the secondary key
    #[pallet::storage]
    pub type Ticks<T> = StorageDoubleMap<_, Twox64Concat, NetUid, Twox64Concat, TickIndex, Tick>;

    /// Storage to determine whether swap V3 was initialized for a specific subnet.
    #[pallet::storage]
    pub type SwapV3Initialized<T> = StorageMap<_, Twox64Concat, NetUid, bool, ValueQuery>;

    /// Storage for the square root price of Alpha token for each subnet.
    #[pallet::storage]
    pub type AlphaSqrtPrice<T> = StorageMap<_, Twox64Concat, NetUid, U64F64, ValueQuery>;

    /// Storage for the current price tick.
    #[pallet::storage]
    pub type CurrentTick<T> = StorageMap<_, Twox64Concat, NetUid, TickIndex, ValueQuery>;

    /// Storage for the current liquidity amount for each subnet.
    #[pallet::storage]
    pub type CurrentLiquidity<T> = StorageMap<_, Twox64Concat, NetUid, u64, ValueQuery>;

    /// Indicates whether a subnet has been switched to V3 swap from V2.
    /// If `true`, the subnet is permanently on V3 swap mode allowing add/remove liquidity
    /// operations. Once set to `true` for a subnet, it cannot be changed back to `false`.
    #[pallet::storage]
    pub type EnabledUserLiquidity<T> = StorageMap<_, Twox64Concat, NetUid, bool, ValueQuery>;

    /// Storage for user positions, using subnet ID and account ID as keys
    /// The value is a bounded vector of Position structs with details about the liquidity positions
    #[pallet::storage]
    pub type Positions<T: Config> = StorageNMap<
        _,
        (
            NMapKey<Twox64Concat, NetUid>,       // Subnet ID
            NMapKey<Twox64Concat, T::AccountId>, // Account ID
            NMapKey<Twox64Concat, PositionId>,   // Position ID
        ),
        Position<T>,
        OptionQuery,
    >;

    /// Position ID counter.
    #[pallet::storage]
    pub type LastPositionId<T> = StorageValue<_, u128, ValueQuery>;

    /// Tick index bitmap words storage
    #[pallet::storage]
    pub type TickIndexBitmapWords<T: Config> = StorageNMap<
        _,
        (
            NMapKey<Twox64Concat, NetUid>,     // Subnet ID
            NMapKey<Twox64Concat, LayerLevel>, // Layer level
            NMapKey<Twox64Concat, u32>,        // word index
        ),
        u128,
        ValueQuery,
    >;

    /// TAO reservoir for scraps of protocol claimed fees.
    #[pallet::storage]
    pub type ScrapReservoirTao<T> = StorageMap<_, Twox64Concat, NetUid, TaoBalance, ValueQuery>;

    /// Alpha reservoir for scraps of protocol claimed fees.
    #[pallet::storage]
    pub type ScrapReservoirAlpha<T> = StorageMap<_, Twox64Concat, NetUid, AlphaBalance, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Event emitted when the fee rate has been updated for a subnet
        FeeRateSet { netuid: NetUid, rate: u16 },

        /// Event emitted when user liquidity operations are enabled for a subnet.
        /// First enable even indicates a switch from V2 to V3 swap.
        UserLiquidityToggled { netuid: NetUid, enable: bool },

        /// Event emitted when a liquidity position is added to a subnet's liquidity pool.
        LiquidityAdded {
            /// The coldkey account that owns the position
            coldkey: T::AccountId,
            /// The hotkey account where Alpha comes from
            hotkey: T::AccountId,
            /// The subnet identifier
            netuid: NetUid,
            /// Unique identifier for the liquidity position
            position_id: PositionId,
            /// The amount of liquidity added to the position
            liquidity: u64,
            /// The amount of TAO tokens committed to the position
            tao: TaoBalance,
            /// The amount of Alpha tokens committed to the position
            alpha: AlphaBalance,
            /// the lower tick
            tick_low: TickIndex,
            /// the upper tick
            tick_high: TickIndex,
        },

        /// Event emitted when a liquidity position is removed from a subnet's liquidity pool.
        LiquidityRemoved {
            /// The coldkey account that owns the position
            coldkey: T::AccountId,
            /// The hotkey account where Alpha goes to
            hotkey: T::AccountId,
            /// The subnet identifier
            netuid: NetUid,
            /// Unique identifier for the liquidity position
            position_id: PositionId,
            /// The amount of liquidity removed from the position
            liquidity: u64,
            /// The amount of TAO tokens returned to the user
            tao: TaoBalance,
            /// The amount of Alpha tokens returned to the user
            alpha: AlphaBalance,
            /// The amount of TAO fees earned from the position
            fee_tao: TaoBalance,
            /// The amount of Alpha fees earned from the position
            fee_alpha: AlphaBalance,
            /// the lower tick
            tick_low: TickIndex,
            /// the upper tick
            tick_high: TickIndex,
        },

        /// Event emitted when a liquidity position is modified in a subnet's liquidity pool.
        /// Modifying causes the fees to be claimed.
        LiquidityModified {
            /// The coldkey account that owns the position
            coldkey: T::AccountId,
            /// The hotkey account where Alpha comes from or goes to
            hotkey: T::AccountId,
            /// The subnet identifier
            netuid: NetUid,
            /// Unique identifier for the liquidity position
            position_id: PositionId,
            /// The amount of liquidity added to or removed from the position
            liquidity: i64,
            /// The amount of TAO tokens returned to the user
            tao: i64,
            /// The amount of Alpha tokens returned to the user
            alpha: i64,
            /// The amount of TAO fees earned from the position
            fee_tao: TaoBalance,
            /// The amount of Alpha fees earned from the position
            fee_alpha: AlphaBalance,
            /// the lower tick
            tick_low: TickIndex,
            /// the upper tick
            tick_high: TickIndex,
        },
    }

    #[pallet::error]
    #[derive(PartialEq)]
    pub enum Error<T> {
        /// The fee rate is too high
        FeeRateTooHigh,

        /// The provided amount is insufficient for the swap.
        InsufficientInputAmount,

        /// The provided liquidity is insufficient for the operation.
        InsufficientLiquidity,

        /// The operation would exceed the price limit.
        PriceLimitExceeded,

        /// The caller does not have enough balance for the operation.
        InsufficientBalance,

        /// Attempted to remove liquidity that does not exist.
        LiquidityNotFound,

        /// The provided tick range is invalid.
        InvalidTickRange,

        /// Maximum user positions exceeded
        MaxPositionsExceeded,

        /// Too many swap steps
        TooManySwapSteps,

        /// Provided liquidity parameter is invalid (likely too small)
        InvalidLiquidityValue,

        /// Reserves too low for operation.
        ReservesTooLow,

        /// The subnet does not exist.
        MechanismDoesNotExist,

        /// User liquidity operations are disabled for this subnet
        UserLiquidityDisabled,

        /// The subnet does not have subtoken enabled
        SubtokenDisabled,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #![deny(clippy::expect_used)]

        /// Set the fee rate for swaps on a specific subnet (normalized value).
        /// For example, 0.3% is approximately 196.
        ///
        /// Only callable by the admin origin
        #[pallet::call_index(0)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::set_fee_rate())]
        pub fn set_fee_rate(origin: OriginFor<T>, netuid: NetUid, rate: u16) -> DispatchResult {
            ensure_root(origin)?;

            // Ensure that the subnet exists.
            ensure!(
                T::SubnetInfo::exists(netuid.into()),
                Error::<T>::MechanismDoesNotExist
            );

            ensure!(rate <= T::MaxFeeRate::get(), Error::<T>::FeeRateTooHigh);

            FeeRate::<T>::insert(netuid, rate);

            Self::deposit_event(Event::FeeRateSet { netuid, rate });

            Ok(())
        }

        /// Enable user liquidity operations for a specific subnet. This switches the
        /// subnet from V2 to V3 swap mode. Thereafter, adding new user liquidity can be disabled
        /// by toggling this flag to false, but the swap mode will remain V3 because of existing
        /// user liquidity until all users withdraw their liquidity.
        ///
        /// Only sudo or subnet owner can enable user liquidity.
        /// Only sudo can disable user liquidity.
        #[pallet::call_index(4)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::toggle_user_liquidity())]
        pub fn toggle_user_liquidity(
            origin: OriginFor<T>,
            netuid: NetUid,
            enable: bool,
        ) -> DispatchResult {
            if ensure_root(origin.clone()).is_err() {
                let account_id: T::AccountId = ensure_signed(origin)?;
                // Only enabling is allowed to subnet owner
                ensure!(
                    T::SubnetInfo::is_owner(&account_id, netuid.into()) && enable,
                    DispatchError::BadOrigin
                );
            }

            ensure!(
                T::SubnetInfo::exists(netuid.into()),
                Error::<T>::MechanismDoesNotExist
            );

            // EnabledUserLiquidity::<T>::insert(netuid, enable);

            // Self::deposit_event(Event::UserLiquidityToggled { netuid, enable });

            Ok(())
        }

        /// Add liquidity to a specific price range for a subnet.
        ///
        /// Parameters:
        /// - origin: The origin of the transaction
        /// - netuid: Subnet ID
        /// - tick_low: Lower bound of the price range
        /// - tick_high: Upper bound of the price range
        /// - liquidity: Amount of liquidity to add
        ///
        /// Emits `Event::LiquidityAdded` on success
        #[pallet::call_index(1)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::add_liquidity())]
        pub fn add_liquidity(
            origin: OriginFor<T>,
            _hotkey: T::AccountId,
            _netuid: NetUid,
            _tick_low: TickIndex,
            _tick_high: TickIndex,
            _liquidity: u64,
        ) -> DispatchResult {
            ensure_signed(origin)?;

            // Extrinsic should have no effect. This fix may have to be reverted later,
            // so leaving the code in for now.

            // // Ensure that the subnet exists.
            // ensure!(
            //     T::SubnetInfo::exists(netuid.into()),
            //     Error::<T>::MechanismDoesNotExist
            // );

            // ensure!(
            //     T::SubnetInfo::is_subtoken_enabled(netuid.into()),
            //     Error::<T>::SubtokenDisabled
            // );

            // let (position_id, tao, alpha) = Self::do_add_liquidity(
            //     netuid.into(),
            //     &coldkey,
            //     &hotkey,
            //     tick_low,
            //     tick_high,
            //     liquidity,
            // )?;
            // let alpha = AlphaBalance::from(alpha);
            // let tao = TaoBalance::from(tao);

            // // Remove TAO and Alpha balances or fail transaction if they can't be removed exactly
            // let tao_provided = T::BalanceOps::decrease_balance(&coldkey, tao)?;
            // ensure!(tao_provided == tao, Error::<T>::InsufficientBalance);

            // let alpha_provided =
            //     T::BalanceOps::decrease_stake(&coldkey, &hotkey, netuid.into(), alpha)?;
            // ensure!(alpha_provided == alpha, Error::<T>::InsufficientBalance);

            // // Add provided liquidity to user-provided reserves
            // T::TaoReserve::increase_provided(netuid.into(), tao_provided);
            // T::AlphaReserve::increase_provided(netuid.into(), alpha_provided);

            // // Emit an event
            // Self::deposit_event(Event::LiquidityAdded {
            //     coldkey,
            //     hotkey,
            //     netuid,
            //     position_id,
            //     liquidity,
            //     tao,
            //     alpha,
            //     tick_low,
            //     tick_high,
            // });

            // Ok(())

            Err(Error::<T>::UserLiquidityDisabled.into())
        }

        /// Remove liquidity from a specific position.
        ///
        /// Parameters:
        /// - origin: The origin of the transaction
        /// - netuid: Subnet ID
        /// - position_id: ID of the position to remove
        ///
        /// Emits `Event::LiquidityRemoved` on success
        #[pallet::call_index(2)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::remove_liquidity())]
        pub fn remove_liquidity(
            _origin: OriginFor<T>,
            _hotkey: T::AccountId,
            _netuid: NetUid,
            _position_id: PositionId,
        ) -> DispatchResult {
            // Deprecated by balancer. We don't have any active liquidity providers either.
            Ok(())
        }

        /// Modify a liquidity position.
        ///
        /// Parameters:
        /// - origin: The origin of the transaction
        /// - netuid: Subnet ID
        /// - position_id: ID of the position to remove
        /// - liquidity_delta: Liquidity to add (if positive) or remove (if negative)
        ///
        /// Emits `Event::LiquidityRemoved` on success
        #[pallet::call_index(3)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::modify_position())]
        pub fn modify_position(
            _origin: OriginFor<T>,
            _hotkey: T::AccountId,
            _netuid: NetUid,
            _position_id: PositionId,
            _liquidity_delta: i64,
        ) -> DispatchResult {
            // Deprecated by balancer. We don't have any active liquidity providers either.
            Ok(())
        }

        /// Disable user liquidity in all subnets.
        ///
        /// Emits `Event::UserLiquidityToggled` on success
        #[pallet::call_index(5)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::disable_lp())]
        pub fn disable_lp(origin: OriginFor<T>) -> DispatchResult {
            ensure_root(origin)?;

            for netuid in 1..=128 {
                let netuid = NetUid::from(netuid as u16);
                if EnabledUserLiquidity::<T>::get(netuid) {
                    EnabledUserLiquidity::<T>::insert(netuid, false);
                    Self::deposit_event(Event::UserLiquidityToggled {
                        netuid,
                        enable: false,
                    });
                }

                // Remove provided liquidity unconditionally because the network may have
                // user liquidity previously disabled
                // Ignore result to avoid early stopping
                let _ = Self::do_dissolve_all_liquidity_providers(netuid);
            }

            Ok(())
        }
    }
}
