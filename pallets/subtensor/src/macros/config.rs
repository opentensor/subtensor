#![allow(clippy::crate_in_macro_def)]

use frame_support::pallet_macros::pallet_section;
/// A [`pallet_section`] that defines the errors for a pallet.
/// This can later be imported into the pallet using [`import_section`].
#[pallet_section]
mod config {

    use subtensor_swap_interface::SwapHandler;

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config:
        frame_system::Config + pallet_drand::Config + pallet_crowdloan::Config
    {
        /// call type
        type RuntimeCall: Parameter
            + Dispatchable<RuntimeOrigin = Self::RuntimeOrigin>
            + From<Call<Self>>
            + IsType<<Self as frame_system::Config>::RuntimeCall>
            + From<frame_system::Call<Self>>;

        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// A sudo-able call.
        type SudoRuntimeCall: Parameter
            + UnfilteredDispatchable<RuntimeOrigin = Self::RuntimeOrigin>
            + GetDispatchInfo;

        /// Origin checking for council majority
        type CouncilOrigin: EnsureOrigin<Self::RuntimeOrigin>;

        ///  Currency type that will be used to place deposits on neurons
        type Currency: fungible::Balanced<Self::AccountId, Balance = u64>
            + fungible::Mutate<Self::AccountId>;

        /// Senate members with members management functions.
        type SenateMembers: crate::MemberManagement<Self::AccountId>;

        /// Interface to allow other pallets to control who can register identities
        type TriumvirateInterface: crate::CollectiveInterface<Self::AccountId, Self::Hash, u32>;

        /// The scheduler type used for scheduling delayed calls.
        type Scheduler: ScheduleAnon<
                BlockNumberFor<Self>,
                LocalCallOf<Self>,
                PalletsOriginOf<Self>,
                Hasher = Self::Hashing,
            >;

        /// the preimage to store the call data.
        type Preimages: QueryPreimage<H = Self::Hashing> + StorePreimage;

        /// Swap interface.
        type SwapInterface: SwapHandler<Self::AccountId>;

        /// Interface to allow interacting with the proxy pallet.
        type ProxyInterface: crate::ProxyInterface<Self::AccountId>;

        /// =================================
        /// ==== Initial Value Constants ====
        /// =================================

        /// Initial currency issuance.
        #[pallet::constant]
        type InitialIssuance: Get<u64>;
        /// Initial min allowed weights setting.
        #[pallet::constant]
        type InitialMinAllowedWeights: Get<u16>;
        /// Initial Emission Ratio.
        #[pallet::constant]
        type InitialEmissionValue: Get<u16>;
        /// Initial max weight limit.
        #[pallet::constant]
        type InitialMaxWeightsLimit: Get<u16>;
        /// Tempo for each network.
        #[pallet::constant]
        type InitialTempo: Get<u16>;
        /// Initial Difficulty.
        #[pallet::constant]
        type InitialDifficulty: Get<u64>;
        /// Initial Max Difficulty.
        #[pallet::constant]
        type InitialMaxDifficulty: Get<u64>;
        /// Initial Min Difficulty.
        #[pallet::constant]
        type InitialMinDifficulty: Get<u64>;
        /// Initial RAO Recycled.
        #[pallet::constant]
        type InitialRAORecycledForRegistration: Get<u64>;
        /// Initial Burn.
        #[pallet::constant]
        type InitialBurn: Get<u64>;
        /// Initial Max Burn.
        #[pallet::constant]
        type InitialMaxBurn: Get<u64>;
        /// Initial Min Burn.
        #[pallet::constant]
        type InitialMinBurn: Get<u64>;
        /// Initial adjustment interval.
        #[pallet::constant]
        type InitialAdjustmentInterval: Get<u16>;
        /// Initial bonds moving average.
        #[pallet::constant]
        type InitialBondsMovingAverage: Get<u64>;
        /// Initial bonds penalty.
        #[pallet::constant]
        type InitialBondsPenalty: Get<u16>;
        /// Initial bonds reset.
        #[pallet::constant]
        type InitialBondsResetOn: Get<bool>;
        /// Initial target registrations per interval.
        #[pallet::constant]
        type InitialTargetRegistrationsPerInterval: Get<u16>;
        /// Rho constant.
        #[pallet::constant]
        type InitialRho: Get<u16>;
        /// AlphaSigmoidSteepness constant.
        #[pallet::constant]
        type InitialAlphaSigmoidSteepness: Get<i16>;
        /// Kappa constant.
        #[pallet::constant]
        type InitialKappa: Get<u16>;
        /// Max UID constant.
        #[pallet::constant]
        type InitialMaxAllowedUids: Get<u16>;
        /// Initial validator context pruning length.
        #[pallet::constant]
        type InitialValidatorPruneLen: Get<u64>;
        /// Initial scaling law power.
        #[pallet::constant]
        type InitialScalingLawPower: Get<u16>;
        /// Immunity Period Constant.
        #[pallet::constant]
        type InitialImmunityPeriod: Get<u16>;
        /// Activity constant.
        #[pallet::constant]
        type InitialActivityCutoff: Get<u16>;
        /// Initial max registrations per block.
        #[pallet::constant]
        type InitialMaxRegistrationsPerBlock: Get<u16>;
        /// Initial pruning score for each neuron.
        #[pallet::constant]
        type InitialPruningScore: Get<u16>;
        /// Initial maximum allowed validators per network.
        #[pallet::constant]
        type InitialMaxAllowedValidators: Get<u16>;
        /// Initial default delegation take.
        #[pallet::constant]
        type InitialDefaultDelegateTake: Get<u16>;
        /// Initial minimum delegation take.
        #[pallet::constant]
        type InitialMinDelegateTake: Get<u16>;
        /// Initial default childkey take.
        #[pallet::constant]
        type InitialDefaultChildKeyTake: Get<u16>;
        /// Initial minimum childkey take.
        #[pallet::constant]
        type InitialMinChildKeyTake: Get<u16>;
        /// Initial maximum childkey take.
        #[pallet::constant]
        type InitialMaxChildKeyTake: Get<u16>;
        /// Initial weights version key.
        #[pallet::constant]
        type InitialWeightsVersionKey: Get<u64>;
        /// Initial serving rate limit.
        #[pallet::constant]
        type InitialServingRateLimit: Get<u64>;
        /// Initial transaction rate limit.
        #[pallet::constant]
        type InitialTxRateLimit: Get<u64>;
        /// Initial delegate take transaction rate limit.
        #[pallet::constant]
        type InitialTxDelegateTakeRateLimit: Get<u64>;
        /// Initial childkey take transaction rate limit.
        #[pallet::constant]
        type InitialTxChildKeyTakeRateLimit: Get<u64>;
        /// Initial percentage of total stake required to join senate.
        #[pallet::constant]
        type InitialSenateRequiredStakePercentage: Get<u64>;
        /// Initial adjustment alpha on burn and pow.
        #[pallet::constant]
        type InitialAdjustmentAlpha: Get<u64>;
        /// Initial network immunity period
        #[pallet::constant]
        type InitialNetworkImmunityPeriod: Get<u64>;
        /// Initial minimum allowed network UIDs
        #[pallet::constant]
        type InitialNetworkMinAllowedUids: Get<u16>;
        /// Initial network minimum burn cost
        #[pallet::constant]
        type InitialNetworkMinLockCost: Get<u64>;
        /// Initial network subnet cut.
        #[pallet::constant]
        type InitialSubnetOwnerCut: Get<u16>;
        /// Initial lock reduction interval.
        #[pallet::constant]
        type InitialNetworkLockReductionInterval: Get<u64>;
        /// Initial network creation rate limit
        #[pallet::constant]
        type InitialNetworkRateLimit: Get<u64>;
        /// Cost of swapping a hotkey.
        #[pallet::constant]
        type KeySwapCost: Get<u64>;
        /// The upper bound for the alpha parameter. Used for Liquid Alpha.
        #[pallet::constant]
        type AlphaHigh: Get<u16>;
        /// The lower bound for the alpha parameter. Used for Liquid Alpha.
        #[pallet::constant]
        type AlphaLow: Get<u16>;
        /// A flag to indicate if Liquid Alpha is enabled.
        #[pallet::constant]
        type LiquidAlphaOn: Get<bool>;
        /// A flag to indicate if Yuma3 is enabled.
        type Yuma3On: Get<bool>;
        // /// Initial hotkey emission tempo.
        // #[pallet::constant]
        // type InitialHotkeyEmissionTempo: Get<u64>;
        /// Coldkey swap schedule duartion.
        #[pallet::constant]
        type InitialColdkeySwapScheduleDuration: Get<BlockNumberFor<Self>>;
        /// Coldkey swap reschedule duration.
        #[pallet::constant]
        type InitialColdkeySwapRescheduleDuration: Get<BlockNumberFor<Self>>;
        /// Dissolve network schedule duration
        #[pallet::constant]
        type InitialDissolveNetworkScheduleDuration: Get<BlockNumberFor<Self>>;
        /// Initial TAO weight.
        #[pallet::constant]
        type InitialTaoWeight: Get<u64>;
        /// Initial EMA price halving period
        #[pallet::constant]
        type InitialEmaPriceHalvingPeriod: Get<u64>;
        /// Block number after a new subnet accept the start call extrinsic.
        #[pallet::constant]
        type DurationOfStartCall: Get<u64>;
        /// Cost of swapping a hotkey in a subnet.
        #[pallet::constant]
        type KeySwapOnSubnetCost: Get<u64>;
        /// Block number for a coldkey swap the hotkey in specific subnet.
        #[pallet::constant]
        type HotkeySwapOnSubnetInterval: Get<u64>;
        /// Number of blocks between dividends distribution.
        #[pallet::constant]
        type LeaseDividendsDistributionInterval: Get<BlockNumberFor<Self>>;
    }
}
