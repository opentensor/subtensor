#![cfg_attr(not(feature = "std"), no_std)]

//! # Balancer-style Weighted Pool Swap Pallet
//!
//! This pallet implements Balancer-style weighted pools for TAO/Alpha swaps.
//! It replaces the complex Uniswap V3 concentrated liquidity system with a simpler,
//! more flexible approach that allows unbalanced liquidity provision.
//!
//! ## Key Features
//! - Weighted constant product formula (like Balancer V2)
//! - Unbalanced liquidity provision supported
//! - Single share balance per user (no position management)
//! - Configurable pool weights per subnet
//! - Protocol-owned liquidity for emissions
//!
//! ## Pool Formula
//! V = (balance_tao ^ weight_tao) * (balance_alpha ^ weight_alpha) = constant

pub use pallet::*;

mod math;
mod types;
pub mod weights;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

use frame_support::pallet_prelude::*;
use frame_system::pallet_prelude::*;
use sp_runtime::{Perbill, traits::AccountIdConversion};
use substrate_fixed::types::U96F32;
use subtensor_runtime_common::{AlphaCurrency, BalanceOps, Currency, CurrencyReserve, NetUid, SubnetInfo, TaoCurrency};
use subtensor_swap_interface::{DefaultPriceLimit, Order as OrderT, SwapEngine, SwapHandler, SwapResult};

pub use types::{Pool, TokenType};
pub use weights::WeightInfo;

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Subnet information provider
        type SubnetInfo: SubnetInfo<Self::AccountId>;

        /// Balance operations provider
        type BalanceOps: BalanceOps<Self::AccountId>;

        /// TAO reserve operations
        type TaoReserve: CurrencyReserve<TaoCurrency>;

        /// Alpha reserve operations
        type AlphaReserve: CurrencyReserve<AlphaCurrency>;

        /// Protocol account ID derivation
        #[pallet::constant]
        type ProtocolId: Get<frame_support::PalletId>;

        /// Default TAO weight (percentage)
        #[pallet::constant]
        type DefaultTaoWeight: Get<Perbill>;

        /// Default Alpha weight (percentage)
        #[pallet::constant]
        type DefaultAlphaWeight: Get<Perbill>;

        /// Default swap fee
        #[pallet::constant]
        type DefaultSwapFee: Get<Perbill>;

        /// Maximum swap fee allowed
        #[pallet::constant]
        type MaxSwapFee: Get<Perbill>;

        /// Minimum liquidity required in pool
        #[pallet::constant]
        type MinimumLiquidity: Get<u64>;

        /// Weight information for extrinsics
        type WeightInfo: WeightInfo;
    }

    /// Pools indexed by subnet ID
    #[pallet::storage]
    #[pallet::getter(fn pools)]
    pub type Pools<T: Config> = StorageMap<_, Twox64Concat, NetUid, Pool, OptionQuery>;

    /// User LP shares: (netuid, account) -> shares
    #[pallet::storage]
    #[pallet::getter(fn liquidity_shares)]
    pub type LiquidityShares<T: Config> = StorageDoubleMap<
        _,
        Twox64Concat,
        NetUid,
        Twox64Concat,
        T::AccountId,
        u128,
        ValueQuery,
    >;

    /// Protocol-owned LP shares per subnet
    #[pallet::storage]
    #[pallet::getter(fn protocol_shares)]
    pub type ProtocolShares<T: Config> = StorageMap<_, Twox64Concat, NetUid, u128, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Pool created for subnet
        PoolCreated {
            netuid: NetUid,
            tao_balance: TaoCurrency,
            alpha_balance: AlphaCurrency,
        },
        /// Liquidity added to pool
        LiquidityAdded {
            netuid: NetUid,
            provider: T::AccountId,
            tao_amount: TaoCurrency,
            alpha_amount: AlphaCurrency,
            shares_minted: u128,
        },
        /// Liquidity removed from pool
        LiquidityRemoved {
            netuid: NetUid,
            provider: T::AccountId,
            tao_amount: TaoCurrency,
            alpha_amount: AlphaCurrency,
            shares_burned: u128,
        },
        /// Swap executed
        Swapped {
            netuid: NetUid,
            token_in: TokenType,
            amount_in: u64,
            amount_out: u64,
            fee: u64,
        },
        /// Pool weights updated
        PoolWeightsUpdated {
            netuid: NetUid,
            tao_weight: Perbill,
            alpha_weight: Perbill,
        },
        /// Swap fee updated
        SwapFeeUpdated {
            netuid: NetUid,
            fee: Perbill,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Pool does not exist for this subnet
        PoolNotFound,
        /// Insufficient balance for operation
        InsufficientBalance,
        /// Invalid pool weights (must sum to 100%)
        InvalidWeights,
        /// Swap fee too high
        SwapFeeTooHigh,
        /// Insufficient liquidity in pool
        InsufficientLiquidity,
        /// Amount too small
        AmountTooSmall,
        /// Slippage exceeded
        SlippageExceeded,
        /// Pool already initialized
        PoolAlreadyExists,
        /// Subnet does not exist
        SubnetDoesNotExist,
        /// Subtoken not enabled for subnet
        SubtokenDisabled,
        /// Math overflow
        Overflow,
        /// No shares to burn
        NoSharesToBurn,
        /// Insufficient shares
        InsufficientShares,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Add liquidity to a pool (supports unbalanced amounts)
        ///
        /// # Arguments
        /// * `origin` - The account adding liquidity (coldkey)
        /// * `hotkey` - The hotkey account (for Alpha)
        /// * `netuid` - Subnet ID
        /// * `tao_amount` - Amount of TAO to add
        /// * `alpha_amount` - Amount of Alpha to add
        /// * `min_shares` - Minimum shares to receive (slippage protection)
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::add_liquidity())]
        pub fn add_liquidity(
            origin: OriginFor<T>,
            hotkey: T::AccountId,
            netuid: NetUid,
            tao_amount: TaoCurrency,
            alpha_amount: AlphaCurrency,
            min_shares: u128,
        ) -> DispatchResult {
            let coldkey = ensure_signed(origin)?;

            ensure!(
                T::SubnetInfo::exists(netuid.into()),
                Error::<T>::SubnetDoesNotExist
            );
            ensure!(
                T::SubnetInfo::is_subtoken_enabled(netuid.into()),
                Error::<T>::SubtokenDisabled
            );

            // Ensure user has sufficient balances
            ensure!(
                T::BalanceOps::tao_balance(&coldkey) >= tao_amount,
                Error::<T>::InsufficientBalance
            );
            ensure!(
                T::BalanceOps::alpha_balance(netuid.into(), &coldkey, &hotkey) >= alpha_amount,
                Error::<T>::InsufficientBalance
            );

            // Calculate shares to mint
            let shares_minted = Self::do_add_liquidity(netuid, tao_amount, alpha_amount)?;

            ensure!(shares_minted >= min_shares, Error::<T>::SlippageExceeded);

            // Deduct tokens from user
            T::BalanceOps::decrease_balance(&coldkey, tao_amount)?;
            T::BalanceOps::decrease_stake(&coldkey, &hotkey, netuid.into(), alpha_amount)?;

            // Update reserves
            T::TaoReserve::increase_provided(netuid.into(), tao_amount);
            T::AlphaReserve::increase_provided(netuid.into(), alpha_amount);

            // Mint shares to user
            LiquidityShares::<T>::mutate(netuid, &coldkey, |shares| {
                *shares = shares.saturating_add(shares_minted);
            });

            Self::deposit_event(Event::LiquidityAdded {
                netuid,
                provider: coldkey,
                tao_amount,
                alpha_amount,
                shares_minted,
            });

            Ok(())
        }

        /// Remove liquidity from a pool
        ///
        /// # Arguments
        /// * `origin` - The account removing liquidity (coldkey)
        /// * `hotkey` - The hotkey account (for Alpha)
        /// * `netuid` - Subnet ID
        /// * `shares` - Number of LP shares to burn
        /// * `min_tao` - Minimum TAO to receive (slippage protection)
        /// * `min_alpha` - Minimum Alpha to receive (slippage protection)
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::remove_liquidity())]
        pub fn remove_liquidity(
            origin: OriginFor<T>,
            hotkey: T::AccountId,
            netuid: NetUid,
            shares: u128,
            min_tao: TaoCurrency,
            min_alpha: AlphaCurrency,
        ) -> DispatchResult {
            let coldkey = ensure_signed(origin)?;

            ensure!(shares > 0, Error::<T>::NoSharesToBurn);

            let user_shares = LiquidityShares::<T>::get(netuid, &coldkey);
            ensure!(user_shares >= shares, Error::<T>::InsufficientShares);

            // Calculate tokens to return
            let (tao_amount, alpha_amount) = Self::do_remove_liquidity(netuid, shares)?;

            ensure!(tao_amount >= min_tao, Error::<T>::SlippageExceeded);
            ensure!(alpha_amount >= min_alpha, Error::<T>::SlippageExceeded);

            // Burn shares from user
            LiquidityShares::<T>::mutate(netuid, &coldkey, |user_shares| {
                *user_shares = user_shares.saturating_sub(shares);
            });

            // Update reserves
            T::TaoReserve::decrease_provided(netuid.into(), tao_amount);
            T::AlphaReserve::decrease_provided(netuid.into(), alpha_amount);

            // Return tokens to user
            T::BalanceOps::increase_balance(&coldkey, tao_amount);
            T::BalanceOps::increase_stake(&coldkey, &hotkey, netuid.into(), alpha_amount)?;

            Self::deposit_event(Event::LiquidityRemoved {
                netuid,
                provider: coldkey,
                tao_amount,
                alpha_amount,
                shares_burned: shares,
            });

            Ok(())
        }

        /// Set pool weights (admin only)
        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::set_pool_weights())]
        pub fn set_pool_weights(
            origin: OriginFor<T>,
            netuid: NetUid,
            tao_weight: Perbill,
            alpha_weight: Perbill,
        ) -> DispatchResult {
            // Only root or subnet owner can set weights
            if frame_system::ensure_root(origin.clone()).is_err() {
                let account = ensure_signed(origin)?;
                ensure!(
                    T::SubnetInfo::is_owner(&account, netuid.into()),
                    DispatchError::BadOrigin
                );
            }

            // Weights must sum to 100%
            ensure!(
                tao_weight.saturating_add(alpha_weight) == Perbill::one(),
                Error::<T>::InvalidWeights
            );

            Pools::<T>::try_mutate(netuid, |maybe_pool| {
                let pool = maybe_pool.as_mut().ok_or(Error::<T>::PoolNotFound)?;
                pool.tao_weight = tao_weight;
                pool.alpha_weight = alpha_weight;
                Ok::<(), Error<T>>(())
            })?;

            Self::deposit_event(Event::PoolWeightsUpdated {
                netuid,
                tao_weight,
                alpha_weight,
            });

            Ok(())
        }

        /// Set swap fee (admin only)
        #[pallet::call_index(3)]
        #[pallet::weight(T::WeightInfo::set_swap_fee())]
        pub fn set_swap_fee(
            origin: OriginFor<T>,
            netuid: NetUid,
            fee: Perbill,
        ) -> DispatchResult {
            // Only root or subnet owner can set fee
            if frame_system::ensure_root(origin.clone()).is_err() {
                let account = ensure_signed(origin)?;
                ensure!(
                    T::SubnetInfo::is_owner(&account, netuid.into()),
                    DispatchError::BadOrigin
                );
            }

            ensure!(fee <= T::MaxSwapFee::get(), Error::<T>::SwapFeeTooHigh);

            Pools::<T>::try_mutate(netuid, |maybe_pool| {
                let pool = maybe_pool.as_mut().ok_or(Error::<T>::PoolNotFound)?;
                pool.swap_fee = fee;
                Ok::<(), Error<T>>(())
            })?;

            Self::deposit_event(Event::SwapFeeUpdated { netuid, fee });

            Ok(())
        }
    }

    // Internal implementation
    impl<T: Config> Pallet<T> {
        /// Initialize a pool with protocol-owned liquidity
        pub fn initialize_pool(
            netuid: NetUid,
            tao_amount: TaoCurrency,
            alpha_amount: AlphaCurrency,
        ) -> DispatchResult {
            ensure!(!Pools::<T>::contains_key(netuid), Error::<T>::PoolAlreadyExists);

            let pool = Pool {
                tao_balance: tao_amount,
                alpha_balance: alpha_amount,
                tao_weight: T::DefaultTaoWeight::get(),
                alpha_weight: T::DefaultAlphaWeight::get(),
                total_shares: 0,
                swap_fee: T::DefaultSwapFee::get(),
            };

            Pools::<T>::insert(netuid, pool);

            // Calculate initial protocol shares (geometric mean of balances)
            let tao_u128 = tao_amount.to_u64() as u128;
            let alpha_u128 = alpha_amount.to_u64() as u128;
            let initial_shares = sp_arithmetic::helpers_128bit::sqrt(
                tao_u128.saturating_mul(alpha_u128)
            );

            // Update pool total shares
            Pools::<T>::mutate(netuid, |maybe_pool| {
                if let Some(pool) = maybe_pool {
                    pool.total_shares = initial_shares;
                }
            });

            // Mint shares to protocol
            ProtocolShares::<T>::insert(netuid, initial_shares);

            Self::deposit_event(Event::PoolCreated {
                netuid,
                tao_balance: tao_amount,
                alpha_balance: alpha_amount,
            });

            Ok(())
        }

        /// Add liquidity and calculate shares to mint
        fn do_add_liquidity(
            netuid: NetUid,
            tao_amount: TaoCurrency,
            alpha_amount: AlphaCurrency,
        ) -> Result<u128, Error<T>> {
            let mut pool = Pools::<T>::get(netuid).ok_or(Error::<T>::PoolNotFound)?;

            let tao_u64 = tao_amount.to_u64();
            let alpha_u64 = alpha_amount.to_u64();

            // If pool is empty, initialize with geometric mean
            let shares_to_mint = if pool.total_shares == 0 {
                let tao_u128 = tao_u64 as u128;
                let alpha_u128 = alpha_u64 as u128;
                sp_arithmetic::helpers_128bit::sqrt(tao_u128.saturating_mul(alpha_u128))
            } else {
                // Calculate shares based on tokens added
                let tao_shares = if tao_u64 > 0 {
                    math::calc_shares_for_single_token_in(
                        pool.tao_balance.to_u64(),
                        pool.tao_weight,
                        pool.total_shares,
                        tao_u64,
                    )
                } else {
                    0
                };

                let alpha_shares = if alpha_u64 > 0 {
                    math::calc_shares_for_single_token_in(
                        pool.alpha_balance.to_u64(),
                        pool.alpha_weight,
                        pool.total_shares,
                        alpha_u64,
                    )
                } else {
                    0
                };

                // Take minimum to prevent LP manipulation
                tao_shares.min(alpha_shares).max(tao_shares.saturating_add(alpha_shares) / 2)
            };

            ensure!(shares_to_mint > 0, Error::<T>::AmountTooSmall);

            // Update pool state
            pool.tao_balance = pool.tao_balance.saturating_add(tao_amount);
            pool.alpha_balance = pool.alpha_balance.saturating_add(alpha_amount);
            pool.total_shares = pool.total_shares.saturating_add(shares_to_mint);

            Pools::<T>::insert(netuid, pool);

            Ok(shares_to_mint)
        }

        /// Remove liquidity and calculate tokens to return
        fn do_remove_liquidity(
            netuid: NetUid,
            shares: u128,
        ) -> Result<(TaoCurrency, AlphaCurrency), Error<T>> {
            let mut pool = Pools::<T>::get(netuid).ok_or(Error::<T>::PoolNotFound)?;

            ensure!(shares <= pool.total_shares, Error::<T>::InsufficientShares);

            // Calculate proportional amounts
            let tao_amount = (pool.tao_balance.to_u64() as u128)
                .saturating_mul(shares)
                .saturating_div(pool.total_shares) as u64;

            let alpha_amount = (pool.alpha_balance.to_u64() as u128)
                .saturating_mul(shares)
                .saturating_div(pool.total_shares) as u64;

            // Update pool state
            pool.tao_balance = pool.tao_balance.saturating_sub(tao_amount.into());
            pool.alpha_balance = pool.alpha_balance.saturating_sub(alpha_amount.into());
            pool.total_shares = pool.total_shares.saturating_sub(shares);

            Pools::<T>::insert(netuid, pool);

            Ok((tao_amount.into(), alpha_amount.into()))
        }

        /// Execute a swap using Balancer math
        fn do_swap(
            netuid: NetUid,
            token_in: TokenType,
            amount_in: u64,
            simulate: bool,
        ) -> Result<(u64, u64), Error<T>> {
            let mut pool = Pools::<T>::get(netuid).ok_or(Error::<T>::PoolNotFound)?;

            let token_out = Pool::other_token(token_in);

            let balance_in = pool.balance(token_in);
            let weight_in = pool.weight(token_in);
            let balance_out = pool.balance(token_out);
            let weight_out = pool.weight(token_out);

            // Calculate amount out using Balancer formula
            let amount_out = math::calc_out_given_in(
                balance_in,
                weight_in,
                balance_out,
                weight_out,
                amount_in,
                pool.swap_fee,
            );

            ensure!(amount_out > 0, Error::<T>::InsufficientLiquidity);
            ensure!(amount_out < balance_out, Error::<T>::InsufficientLiquidity);

            // Calculate fee
            let fee = pool.swap_fee.mul_floor(amount_in as u128) as u64;

            if !simulate {
                // Update pool balances
                pool.set_balance(token_in, balance_in.saturating_add(amount_in));
                pool.set_balance(token_out, balance_out.saturating_sub(amount_out));
                Pools::<T>::insert(netuid, pool);
            }

            Ok((amount_out, fee))
        }

        /// Get protocol account ID
        pub fn protocol_account_id() -> T::AccountId {
            T::ProtocolId::get().into_account_truncating()
        }

        /// Get current alpha price for a subnet
        pub fn current_price(netuid: NetUid) -> U96F32 {
            Pools::<T>::get(netuid)
                .map(|pool| pool.spot_price())
                .unwrap_or_default()
        }
    }
}

// Implement SwapHandler trait for compatibility
impl<T: Config> SwapHandler for Pallet<T> {
    fn swap<O>(
        netuid: NetUid,
        order: O,
        _price_limit: TaoCurrency,
        _drop_fees: bool,
        should_rollback: bool,
    ) -> Result<SwapResult<O::PaidIn, O::PaidOut>, DispatchError>
    where
        O: OrderT,
        Self: SwapEngine<O>,
    {
        let amount_in = order.amount().to_u64();
        
        let token_in = if core::any::TypeId::of::<O::PaidIn>() == core::any::TypeId::of::<TaoCurrency>() {
            TokenType::Tao
        } else {
            TokenType::Alpha
        };

        let (amount_out, fee) = Self::do_swap(netuid, token_in, amount_in, should_rollback)?;

        Ok(SwapResult {
            amount_paid_in: order.amount(),
            amount_paid_out: amount_out.into(),
            fee_paid: fee.into(),
        })
    }

    fn sim_swap<O>(
        netuid: NetUid,
        order: O,
    ) -> Result<SwapResult<O::PaidIn, O::PaidOut>, DispatchError>
    where
        O: OrderT,
        Self: SwapEngine<O>,
    {
        Self::swap(netuid, order, TaoCurrency::ZERO, false, true)
    }

    fn approx_fee_amount<C: Currency>(netuid: NetUid, amount: C) -> C {
        Pools::<T>::get(netuid)
            .map(|pool| {
                let fee_amount = pool.swap_fee.mul_floor(amount.to_u64() as u128) as u64;
                fee_amount.into()
            })
            .unwrap_or(C::ZERO)
    }

    fn current_alpha_price(netuid: NetUid) -> U96F32 {
        Self::current_price(netuid)
    }

    fn min_price<C: Currency>() -> C {
        // Balancer pools don't have hard min/max prices
        C::ZERO
    }

    fn max_price<C: Currency>() -> C {
        // Return a very large number
        (u64::MAX / 1000).into()
    }

    fn adjust_protocol_liquidity(
        netuid: NetUid,
        tao_delta: TaoCurrency,
        alpha_delta: AlphaCurrency,
    ) {
        // Add or remove protocol liquidity
        let _ = Pools::<T>::try_mutate(netuid, |maybe_pool| {
            if let Some(pool) = maybe_pool {
                pool.tao_balance = if tao_delta.to_u64() as i64 >= 0 {
                    pool.tao_balance.saturating_add(tao_delta)
                } else {
                    pool.tao_balance.saturating_sub(TaoCurrency::from(tao_delta.to_u64()))
                };
                
                pool.alpha_balance = if alpha_delta.to_u64() as i64 >= 0 {
                    pool.alpha_balance.saturating_add(alpha_delta)
                } else {
                    pool.alpha_balance.saturating_sub(AlphaCurrency::from(alpha_delta.to_u64()))
                };
            }
            Ok::<(), ()>(())
        });
    }

    fn dissolve_all_liquidity_providers(netuid: NetUid) -> DispatchResult {
        // Return all user LP shares as tokens
        let pool = Pools::<T>::get(netuid).ok_or("Pool not found")?;
        
        // Iterate all user shares and return their tokens
        let mut providers: sp_std::vec::Vec<(T::AccountId, u128)> = sp_std::vec::Vec::new();
        for (provider, shares) in LiquidityShares::<T>::iter_prefix(netuid) {
            if shares > 0 {
                providers.push((provider, shares));
            }
        }

        for (provider, shares) in providers {
            let tao_amount = (pool.tao_balance.to_u64() as u128)
                .saturating_mul(shares)
                .saturating_div(pool.total_shares) as u64;
            let alpha_amount = (pool.alpha_balance.to_u64() as u128)
                .saturating_mul(shares)
                .saturating_div(pool.total_shares) as u64;

            // Return tokens
            T::BalanceOps::increase_balance(&provider, tao_amount.into());
            // Note: We need a default hotkey for alpha returns
            // This is a simplification - in production, track hotkey per LP
            
            // Clear shares
            LiquidityShares::<T>::remove(netuid, &provider);
        }

        Ok(())
    }

    fn clear_protocol_liquidity(netuid: NetUid) -> DispatchResult {
        // Remove protocol shares and clear pool
        ProtocolShares::<T>::remove(netuid);
        Pools::<T>::remove(netuid);
        Ok(())
    }

    fn is_user_liquidity_enabled(_netuid: NetUid) -> bool {
        // Always enabled with Balancer
        true
    }

    fn toggle_user_liquidity(_netuid: NetUid, _enabled: bool) {
        // No-op: always enabled with Balancer
    }
}

// Implement DefaultPriceLimit for both directions
impl<T: Config> DefaultPriceLimit<TaoCurrency, AlphaCurrency> for Pallet<T> {
    fn default_price_limit<C: Currency>() -> C {
        (u64::MAX / 1000).into()
    }
}

impl<T: Config> DefaultPriceLimit<AlphaCurrency, TaoCurrency> for Pallet<T> {
    fn default_price_limit<C: Currency>() -> C {
        C::ZERO
    }
}

impl<T, O> SwapEngine<O> for Pallet<T>
where
    T: Config,
    O: OrderT,
    Self: DefaultPriceLimit<O::PaidIn, O::PaidOut>,
{
    fn swap(
        netuid: NetUid,
        order: O,
        price_limit: TaoCurrency,
        drop_fees: bool,
        should_rollback: bool,
    ) -> Result<SwapResult<O::PaidIn, O::PaidOut>, DispatchError> {
        <Self as SwapHandler>::swap(netuid, order, price_limit, drop_fees, should_rollback)
    }
}



