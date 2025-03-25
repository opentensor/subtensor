use super::*;
extern crate alloc;
use crate::epoch::math::*;
use codec::Compact;
use frame_support::pallet_prelude::{Decode, Encode};
use substrate_fixed::types::I64F64;
use substrate_fixed::types::I96F32;
use subtensor_macros::freeze_struct;

#[freeze_struct("cb3ff125c0c35c9e")]
#[derive(Decode, Encode, PartialEq, Eq, Clone, Debug, TypeInfo)]
pub struct Metagraph<AccountId: TypeInfo + Encode + Decode> {
    // Subnet index
    netuid: Compact<u16>,

    // Name and symbol
    name: Vec<Compact<u8>>,              // name
    symbol: Vec<Compact<u8>>,            // token symbol
    identity: Option<SubnetIdentityV2>,  // identity information.
    network_registered_at: Compact<u64>, // block at registration

    // Keys for owner.
    owner_hotkey: AccountId,  // hotkey
    owner_coldkey: AccountId, // coldkey.

    // Tempo terms.
    block: Compact<u64>,                  // block at call.
    tempo: Compact<u16>,                  // epoch tempo
    last_step: Compact<u64>,              // last epoch
    blocks_since_last_step: Compact<u64>, // blocks since last epoch.

    // Subnet emission terms
    subnet_emission: Compact<u64>,        // subnet emission via stao
    alpha_in: Compact<u64>,               // amount of alpha in reserve
    alpha_out: Compact<u64>,              // amount of alpha outstanding
    tao_in: Compact<u64>,                 // amount of tao injected per block
    alpha_out_emission: Compact<u64>,     // amount injected in alpha reserves per block
    alpha_in_emission: Compact<u64>,      // amount injected outstanding per block
    tao_in_emission: Compact<u64>,        // amount of tao injected per block
    pending_alpha_emission: Compact<u64>, // pending alpha to be distributed
    pending_root_emission: Compact<u64>,  // panding tao for root divs to be distributed
    subnet_volume: Compact<u128>,         // volume of the subnet in TAO
    moving_price: I96F32,                 // subnet moving price.

    // Hparams for epoch
    rho: Compact<u16>,   // subnet rho param
    kappa: Compact<u16>, // subnet kappa param

    // Validator params
    min_allowed_weights: Compact<u16>, // min allowed weights per val
    max_weights_limit: Compact<u16>,   // max allowed weights per val
    weights_version: Compact<u64>,     // allowed weights version
    weights_rate_limit: Compact<u64>,  // rate limit on weights.
    activity_cutoff: Compact<u16>,     // validator weights cut off period in blocks
    max_validators: Compact<u16>,      // max allowed validators.

    // Registration
    num_uids: Compact<u16>,
    max_uids: Compact<u16>,
    burn: Compact<u64>,                     // current burn cost..
    difficulty: Compact<u64>,               // current difficulty.
    registration_allowed: bool,             // allows registrations.
    pow_registration_allowed: bool,         // pow registration enabled.
    immunity_period: Compact<u16>,          // subnet miner immunity period
    min_difficulty: Compact<u64>,           // min pow difficulty
    max_difficulty: Compact<u64>,           // max pow difficulty
    min_burn: Compact<u64>,                 // min tao burn
    max_burn: Compact<u64>,                 // max tao burn
    adjustment_alpha: Compact<u64>,         // adjustment speed for registration params.
    adjustment_interval: Compact<u16>,      // pow and burn adjustment interval
    target_regs_per_interval: Compact<u16>, // target registrations per interval
    max_regs_per_block: Compact<u16>,       // max registrations per block.
    serving_rate_limit: Compact<u64>,       // axon serving rate limit

    // CR
    commit_reveal_weights_enabled: bool, // Is CR enabled.
    commit_reveal_period: Compact<u64>,  // Commit reveal interval

    // Bonds
    liquid_alpha_enabled: bool,     // Bonds liquid enabled.
    alpha_high: Compact<u16>,       // Alpha param high
    alpha_low: Compact<u16>,        // Alpha param low
    bonds_moving_avg: Compact<u64>, // Bonds moving avg

    // Metagraph info.
    hotkeys: Vec<AccountId>,                    // hotkey per UID
    coldkeys: Vec<AccountId>,                   // coldkey per UID
    identities: Vec<Option<ChainIdentityOfV2>>, // coldkeys identities
    axons: Vec<AxonInfo>,                       // UID axons.
    active: Vec<bool>,                          // Avtive per UID
    validator_permit: Vec<bool>,                // Val permit per UID
    pruning_score: Vec<Compact<u16>>,           // Pruning per UID
    last_update: Vec<Compact<u64>>,             // Last update per UID
    emission: Vec<Compact<u64>>,                // Emission per UID
    dividends: Vec<Compact<u16>>,               // Dividends per UID
    incentives: Vec<Compact<u16>>,              // Mining incentives per UID
    consensus: Vec<Compact<u16>>,               // Consensus per UID
    trust: Vec<Compact<u16>>,                   // Trust per UID
    rank: Vec<Compact<u16>>,                    // Rank per UID
    block_at_registration: Vec<Compact<u64>>,   // Reg block per UID
    alpha_stake: Vec<Compact<u64>>,             // Alpha staked per UID
    tao_stake: Vec<Compact<u64>>,               // TAO staked per UID
    total_stake: Vec<Compact<u64>>,             // Total stake per UID

    // Dividend break down.
    tao_dividends_per_hotkey: Vec<(AccountId, Compact<u64>)>, // List of dividend payouts in tao via root.
    alpha_dividends_per_hotkey: Vec<(AccountId, Compact<u64>)>, // List of dividend payout in alpha via subnet.
}

#[freeze_struct("d745be982b4d29ea")]
#[derive(Decode, Encode, PartialEq, Eq, Clone, Debug, TypeInfo)]
pub struct SelectiveMetagraph<AccountId: TypeInfo + Encode + Decode> {
    // Subnet index
    netuid: Compact<u16>,

    // Name and symbol
    name: Option<Vec<Compact<u8>>>,              // name
    symbol: Option<Vec<Compact<u8>>>,            // token symbol
    identity: Option<Option<SubnetIdentityV2>>,  // identity information.
    network_registered_at: Option<Compact<u64>>, // block at registration

    // Keys for owner.
    owner_hotkey: Option<AccountId>,  // hotkey
    owner_coldkey: Option<AccountId>, // coldkey.

    // Tempo terms.
    block: Option<Compact<u64>>,                  // block at call.
    tempo: Option<Compact<u16>>,                  // epoch tempo
    last_step: Option<Compact<u64>>,              // last epoch
    blocks_since_last_step: Option<Compact<u64>>, // blocks since last epoch.

    // Subnet emission terms
    subnet_emission: Option<Compact<u64>>, // subnet emission via stao
    alpha_in: Option<Compact<u64>>,        // amount of alpha in reserve
    alpha_out: Option<Compact<u64>>,       // amount of alpha outstanding
    tao_in: Option<Compact<u64>>,          // amount of tao injected per block
    alpha_out_emission: Option<Compact<u64>>, // amount injected in alpha reserves per block
    alpha_in_emission: Option<Compact<u64>>, // amount injected outstanding per block
    tao_in_emission: Option<Compact<u64>>, // amount of tao injected per block
    pending_alpha_emission: Option<Compact<u64>>, // pending alpha to be distributed
    pending_root_emission: Option<Compact<u64>>, // panding tao for root divs to be distributed
    subnet_volume: Option<Compact<u128>>,  // volume of the subnet in TAO
    moving_price: Option<I96F32>,          // subnet moving price.

    // Hparams for epoch
    rho: Option<Compact<u16>>,   // subnet rho param
    kappa: Option<Compact<u16>>, // subnet kappa param

    // Validator params
    min_allowed_weights: Option<Compact<u16>>, // min allowed weights per val
    max_weights_limit: Option<Compact<u16>>,   // max allowed weights per val
    weights_version: Option<Compact<u64>>,     // allowed weights version
    weights_rate_limit: Option<Compact<u64>>,  // rate limit on weights.
    activity_cutoff: Option<Compact<u16>>,     // validator weights cut off period in blocks
    max_validators: Option<Compact<u16>>,      // max allowed validators.

    // Registration
    num_uids: Option<Compact<u16>>,
    max_uids: Option<Compact<u16>>,
    burn: Option<Compact<u64>>,                     // current burn cost..
    difficulty: Option<Compact<u64>>,               // current difficulty.
    registration_allowed: Option<bool>,             // allows registrations.
    pow_registration_allowed: Option<bool>,         // pow registration enabled.
    immunity_period: Option<Compact<u16>>,          // subnet miner immunity period
    min_difficulty: Option<Compact<u64>>,           // min pow difficulty
    max_difficulty: Option<Compact<u64>>,           // max pow difficulty
    min_burn: Option<Compact<u64>>,                 // min tao burn
    max_burn: Option<Compact<u64>>,                 // max tao burn
    adjustment_alpha: Option<Compact<u64>>,         // adjustment speed for registration params.
    adjustment_interval: Option<Compact<u16>>,      // pow and burn adjustment interval
    target_regs_per_interval: Option<Compact<u16>>, // target registrations per interval
    max_regs_per_block: Option<Compact<u16>>,       // max registrations per block.
    serving_rate_limit: Option<Compact<u64>>,       // axon serving rate limit

    // CR
    commit_reveal_weights_enabled: Option<bool>, // Is CR enabled.
    commit_reveal_period: Option<Compact<u64>>,  // Commit reveal interval

    // Bonds
    liquid_alpha_enabled: Option<bool>, // Bonds liquid enabled.
    alpha_high: Option<Compact<u16>>,   // Alpha param high
    alpha_low: Option<Compact<u16>>,    // Alpha param low
    bonds_moving_avg: Option<Compact<u64>>, // Bonds moving avg

    // Metagraph info.
    hotkeys: Option<Vec<AccountId>>,  // hotkey per UID
    coldkeys: Option<Vec<AccountId>>, // coldkey per UID
    identities: Option<Vec<Option<ChainIdentityOfV2>>>, // coldkeys identities
    axons: Option<Vec<AxonInfo>>,     // UID axons.
    active: Option<Vec<bool>>,        // Avtive per UID
    validator_permit: Option<Vec<bool>>, // Val permit per UID
    pruning_score: Option<Vec<Compact<u16>>>, // Pruning per UID
    last_update: Option<Vec<Compact<u64>>>, // Last update per UID
    emission: Option<Vec<Compact<u64>>>, // Emission per UID
    dividends: Option<Vec<Compact<u16>>>, // Dividends per UID
    incentives: Option<Vec<Compact<u16>>>, // Mining incentives per UID
    consensus: Option<Vec<Compact<u16>>>, // Consensus per UID
    trust: Option<Vec<Compact<u16>>>, // Trust per UID
    rank: Option<Vec<Compact<u16>>>,  // Rank per UID
    block_at_registration: Option<Vec<Compact<u64>>>, // Reg block per UID
    alpha_stake: Option<Vec<Compact<u64>>>, // Alpha staked per UID
    tao_stake: Option<Vec<Compact<u64>>>, // TAO staked per UID
    total_stake: Option<Vec<Compact<u64>>>, // Total stake per UID

    // Dividend break down.
    tao_dividends_per_hotkey: Option<Vec<(AccountId, Compact<u64>)>>, // List of dividend payouts in tao via root.
    alpha_dividends_per_hotkey: Option<Vec<(AccountId, Compact<u64>)>>, // List of dividend payout in alpha via subnet.
}

impl<AccountId> Default for SelectiveMetagraph<AccountId>
where
    AccountId: TypeInfo + Encode + Decode,
{
    fn default() -> Self {
        Self {
            netuid: 0.into(),
            name: None,
            symbol: None,
            identity: None,
            network_registered_at: None,
            owner_hotkey: None,
            owner_coldkey: None,
            block: None,
            tempo: None,
            last_step: None,
            blocks_since_last_step: None,
            subnet_emission: None,
            alpha_in: None,
            alpha_out: None,
            tao_in: None,
            alpha_out_emission: None,
            alpha_in_emission: None,
            tao_in_emission: None,
            pending_alpha_emission: None,
            pending_root_emission: None,
            subnet_volume: None,
            moving_price: None,
            rho: None,
            kappa: None,
            min_allowed_weights: None,
            max_weights_limit: None,
            weights_version: None,
            weights_rate_limit: None,
            activity_cutoff: None,
            max_validators: None,
            num_uids: None,
            max_uids: None,
            burn: None,
            difficulty: None,
            registration_allowed: None,
            pow_registration_allowed: None,
            immunity_period: None,
            min_difficulty: None,
            max_difficulty: None,
            min_burn: None,
            max_burn: None,
            adjustment_alpha: None,
            adjustment_interval: None,
            target_regs_per_interval: None,
            max_regs_per_block: None,
            serving_rate_limit: None,
            commit_reveal_weights_enabled: None,
            commit_reveal_period: None,
            liquid_alpha_enabled: None,
            alpha_high: None,
            alpha_low: None,
            bonds_moving_avg: None,
            hotkeys: None,
            coldkeys: None,
            identities: None,
            axons: None,
            active: None,
            validator_permit: None,
            pruning_score: None,
            last_update: None,
            emission: None,
            dividends: None,
            incentives: None,
            consensus: None,
            trust: None,
            rank: None,
            block_at_registration: None,
            alpha_stake: None,
            tao_stake: None,
            total_stake: None,
            tao_dividends_per_hotkey: None,
            alpha_dividends_per_hotkey: None,
        }
    }
}

pub enum SelectiveMetagraphIndex {
    Name,
    Symbol,
    Identity,
    NetworkRegisteredAt,
    OwnerHotkey,
    OwnerColdkey,
    Block,
    Tempo,
    LastStep,
    BlocksSinceLastStep,
    SubnetEmission,
    AlphaIn,
    AlphaOut,
    TaoIn,
    AlphaOutEmission,
    AlphaInEmission,
    TaoInEmission,
    PendingAlphaEmission,
    PendingRootEmission,
    SubnetVolume,
    MovingPrice,
    Rho,
    Kappa,
    MinAllowedWeights,
    MaxWeightsLimit,
    WeightsVersion,
    WeightsRateLimit,
    ActivityCutoff,
    MaxValidators,
    NumUids,
    MaxUids,
    Burn,
    Difficulty,
    RegistrationAllowed,
    PowRegistrationAllowed,
    ImmunityPeriod,
    MinDifficulty,
    MaxDifficulty,
    MinBurn,
    MaxBurn,
    AdjustmentAlpha,
    AdjustmentInterval,
    TargetRegsPerInterval,
    MaxRegsPerBlock,
    ServingRateLimit,
    CommitRevealWeightsEnabled,
    CommitRevealPeriod,
    LiquidAlphaEnabled,
    AlphaHigh,
    AlphaLow,
    BondsMovingAvg,
    Hotkeys,
    Coldkeys,
    Identities,
    Axons,
    Active,
    ValidatorPermit,
    PruningScore,
    LastUpdate,
    Emission,
    Dividends,
    Incentives,
    Consensus,
    Trust,
    Rank,
    BlockAtRegistration,
    AlphaStake,
    TaoStake,
    TotalStake,
    TaoDividendsPerHotkey,
    AlphaDividendsPerHotkey,
}
impl<T: Config> Pallet<T> {
    pub fn get_metagraph(netuid: u16) -> Option<Metagraph<T::AccountId>> {
        if !Self::if_subnet_exist(netuid) {
            return None;
        }

        let n: u16 = Self::get_subnetwork_n(netuid);
        let mut hotkeys: Vec<T::AccountId> = vec![];
        let mut coldkeys: Vec<T::AccountId> = vec![];
        let mut block_at_registration: Vec<Compact<u64>> = vec![];
        let mut identities: Vec<Option<ChainIdentityOfV2>> = vec![];
        let mut axons: Vec<AxonInfo> = vec![];
        for uid in 0..n {
            let hotkey = Keys::<T>::get(netuid, uid);
            let coldkey = Owner::<T>::get(hotkey.clone());
            hotkeys.push(hotkey.clone());
            coldkeys.push(coldkey.clone());
            block_at_registration.push(BlockAtRegistration::<T>::get(netuid, uid).into());
            identities.push(IdentitiesV2::<T>::get(coldkey.clone()));
            axons.push(Self::get_axon_info(netuid, &hotkey));
        }
        let mut tao_dividends_per_hotkey: Vec<(T::AccountId, Compact<u64>)> = vec![];
        let mut alpha_dividends_per_hotkey: Vec<(T::AccountId, Compact<u64>)> = vec![];
        for hotkey in hotkeys.clone() {
            let tao_divs = TaoDividendsPerSubnet::<T>::get(netuid, hotkey.clone());
            let alpha_divs = AlphaDividendsPerSubnet::<T>::get(netuid, hotkey.clone());
            tao_dividends_per_hotkey.push((hotkey.clone(), tao_divs.into()));
            alpha_dividends_per_hotkey.push((hotkey.clone(), alpha_divs.into()));
        }
        let current_block: u64 = Pallet::<T>::get_current_block_as_u64();
        let last_step = LastMechansimStepBlock::<T>::get(netuid);
        let blocks_since_last_step: u64 = current_block.saturating_sub(last_step);
        let (total_stake_fl, alpha_stake_fl, tao_stake_fl): (
            Vec<I64F64>,
            Vec<I64F64>,
            Vec<I64F64>,
        ) = Self::get_stake_weights_for_network(netuid);

        let subnet_volume = SubnetVolume::<T>::get(netuid);
        Some(Metagraph {
            // Subnet index
            netuid: netuid.into(), // subnet index.

            // Name and symbol
            name: Self::get_name_for_subnet(netuid)
                .into_iter()
                .map(Compact)
                .collect(), // Name
            symbol: Self::get_symbol_for_subnet(netuid)
                .into_iter()
                .map(Compact)
                .collect(), // Symbol.
            identity: SubnetIdentitiesV2::<T>::get(netuid), // identity information.
            network_registered_at: NetworkRegisteredAt::<T>::get(netuid).into(), // block at registration

            // Keys for owner.
            owner_hotkey: SubnetOwnerHotkey::<T>::get(netuid), // Owner hotkey
            owner_coldkey: SubnetOwner::<T>::get(netuid),      // Owner Coldkey

            // Tempo terms.
            block: current_block.into(),           // Block at call.
            tempo: Self::get_tempo(netuid).into(), // epoch tempo
            last_step: LastMechansimStepBlock::<T>::get(netuid).into(), // last epoch
            blocks_since_last_step: blocks_since_last_step.into(), // blocks since last epoch.

            // Subnet emission terms
            subnet_emission: 0.into(),                        // DEPRECATED
            alpha_in: SubnetAlphaIn::<T>::get(netuid).into(), // amount of alpha in reserve
            alpha_out: SubnetAlphaOut::<T>::get(netuid).into(), // amount of alpha outstanding
            tao_in: SubnetTAO::<T>::get(netuid).into(),       // amount of tao injected per block
            alpha_out_emission: SubnetAlphaOutEmission::<T>::get(netuid).into(), // amount injected in alpha reserves per block
            alpha_in_emission: SubnetAlphaInEmission::<T>::get(netuid).into(), // amount injected outstanding per block
            tao_in_emission: SubnetTaoInEmission::<T>::get(netuid).into(), // amount of tao injected per block
            pending_alpha_emission: PendingEmission::<T>::get(netuid).into(), // pending alpha to be distributed
            pending_root_emission: PendingRootDivs::<T>::get(netuid).into(), // panding tao for root divs to be distributed
            subnet_volume: subnet_volume.into(),
            moving_price: SubnetMovingPrice::<T>::get(netuid),

            // Hparams for epoch
            rho: Self::get_rho(netuid).into(), // subnet rho param
            kappa: Self::get_kappa(netuid).into(), // subnet kappa param

            // Validator params
            min_allowed_weights: Self::get_min_allowed_weights(netuid).into(), // min allowed weights per val
            max_weights_limit: Self::get_max_weight_limit(netuid).into(),      // max allowed weight
            weights_version: Self::get_weights_version_key(netuid).into(), // allowed weights version
            weights_rate_limit: Self::get_weights_set_rate_limit(netuid).into(), // rate limit on weights.
            activity_cutoff: Self::get_activity_cutoff(netuid).into(), // validator weights cut off period in blocks
            max_validators: Self::get_max_allowed_validators(netuid).into(), // max allowed validators.

            // Registration
            num_uids: Self::get_subnetwork_n(netuid).into(),
            max_uids: Self::get_max_allowed_uids(netuid).into(),
            registration_allowed: Self::get_network_registration_allowed(netuid), // allows registrations.
            pow_registration_allowed: Self::get_network_pow_registration_allowed(netuid), // allows pow registrations.
            difficulty: Self::get_difficulty_as_u64(netuid).into(), // current difficulty.
            burn: Self::get_burn_as_u64(netuid).into(),
            immunity_period: Self::get_immunity_period(netuid).into(), // subnet miner immunity period
            min_difficulty: Self::get_min_difficulty(netuid).into(),   // min pow difficulty
            max_difficulty: Self::get_max_difficulty(netuid).into(),   // max pow difficulty
            min_burn: Self::get_min_burn_as_u64(netuid).into(),        // min tao burn
            max_burn: Self::get_max_burn_as_u64(netuid).into(),        // max tao burn
            adjustment_alpha: Self::get_adjustment_alpha(netuid).into(), // adjustment speed for registration params.
            adjustment_interval: Self::get_adjustment_interval(netuid).into(), // pow and burn adjustment interval
            target_regs_per_interval: Self::get_target_registrations_per_interval(netuid).into(), // target registrations per interval
            max_regs_per_block: Self::get_max_registrations_per_block(netuid).into(), // max registrations per block.
            serving_rate_limit: Self::get_serving_rate_limit(netuid).into(), // axon serving rate limit

            // CR
            commit_reveal_weights_enabled: Self::get_commit_reveal_weights_enabled(netuid), // Is CR enabled.
            commit_reveal_period: Self::get_reveal_period(netuid).into(), // Commit reveal interval

            // Bonds
            liquid_alpha_enabled: Self::get_liquid_alpha_enabled(netuid), // Bonds liquid enabled.
            alpha_high: Self::get_alpha_values(netuid).1.into(),          // Alpha param high
            alpha_low: Self::get_alpha_values(netuid).0.into(),           // Alpha param low
            bonds_moving_avg: Self::get_bonds_moving_average(netuid).into(), // Bonds moving avg

            // Metagraph info.
            hotkeys,  // hotkey per UID
            coldkeys, // coldkey per UID
            axons,    // Axon information per UID.
            identities,
            active: Active::<T>::get(netuid), // Avtive per UID
            validator_permit: ValidatorPermit::<T>::get(netuid), // Val permit per UID
            pruning_score: PruningScores::<T>::get(netuid)
                .into_iter()
                .map(Compact::from)
                .collect(), // Pruning per UID
            last_update: LastUpdate::<T>::get(netuid)
                .into_iter()
                .map(Compact::from)
                .collect(), // Last update per UID
            emission: Emission::<T>::get(netuid)
                .into_iter()
                .map(Compact::from)
                .collect(), // Emission per UID
            dividends: Dividends::<T>::get(netuid)
                .into_iter()
                .map(Compact::from)
                .collect(), // Dividends per UID
            incentives: Incentive::<T>::get(netuid)
                .into_iter()
                .map(Compact::from)
                .collect(), // Mining incentives per UID
            consensus: Consensus::<T>::get(netuid)
                .into_iter()
                .map(Compact::from)
                .collect(), // Consensus per UID
            trust: Trust::<T>::get(netuid)
                .into_iter()
                .map(Compact::from)
                .collect(), // Trust per UID
            rank: Rank::<T>::get(netuid)
                .into_iter()
                .map(Compact::from)
                .collect(), // Rank per UID
            block_at_registration,            // Reg block per UID
            alpha_stake: alpha_stake_fl
                .iter()
                .map(|xi| Compact::from(fixed64_to_u64(*xi)))
                .collect::<Vec<Compact<u64>>>(), // Alpha staked per UID
            tao_stake: tao_stake_fl
                .iter()
                .map(|xi| Compact::from(fixed64_to_u64(*xi)))
                .collect::<Vec<Compact<u64>>>(), // TAO staked per UID
            total_stake: total_stake_fl
                .iter()
                .map(|xi| Compact::from(fixed64_to_u64(*xi)))
                .collect::<Vec<Compact<u64>>>(), // Total stake per UID

            // Dividend break down.
            tao_dividends_per_hotkey,
            alpha_dividends_per_hotkey,
        })
    }
    pub fn get_all_metagraphs() -> Vec<Option<Metagraph<T::AccountId>>> {
        let netuids: Vec<u16> = Self::get_all_subnet_netuids();
        let mut metagraphs = Vec::<Option<Metagraph<T::AccountId>>>::new();
        for netuid in netuids.clone().iter() {
            metagraphs.push(Self::get_metagraph(*netuid));
        }
        metagraphs
    }

    pub fn get_selective_metagraph(
        netuid: u16,
        metagraph_name: &str,
    ) -> Option<SelectiveMetagraph<T::AccountId>> {
        if !Self::if_subnet_exist(netuid) {
            return None;
        }

        match metagraph_name {
            // Name and symbol
            "name" => Some(SelectiveMetagraph {
                netuid: netuid.into(),
                name: Some(
                    Self::get_name_for_subnet(netuid)
                        .into_iter()
                        .map(Compact)
                        .collect(),
                ),
                ..Default::default()
            }),
            "symbol" => Some(SelectiveMetagraph {
                netuid: netuid.into(),
                symbol: Some(
                    Self::get_symbol_for_subnet(netuid)
                        .into_iter()
                        .map(Compact)
                        .collect(),
                ),
                ..Default::default()
            }),
            "identity" => Some(SelectiveMetagraph {
                netuid: netuid.into(),
                identity: Some(SubnetIdentitiesV2::<T>::get(netuid)),
                ..Default::default()
            }),
            "network_registered_at" => Some(SelectiveMetagraph {
                netuid: netuid.into(),
                network_registered_at: Some(NetworkRegisteredAt::<T>::get(netuid).into()),
                ..Default::default()
            }),

            // Keys for owner.
            "owner_hotkey" => Some(SelectiveMetagraph {
                netuid: netuid.into(),
                owner_hotkey: Some(SubnetOwnerHotkey::<T>::get(netuid)),
                ..Default::default()
            }),
            "owner_coldkey" => Some(SelectiveMetagraph {
                netuid: netuid.into(),
                owner_coldkey: Some(SubnetOwner::<T>::get(netuid)),
                ..Default::default()
            }),

            // Tempo terms.
            "block" => Some(SelectiveMetagraph {
                netuid: netuid.into(),
                block: Some(Pallet::<T>::get_current_block_as_u64().into()),
                ..Default::default()
            }),
            "tempo" => Some(SelectiveMetagraph {
                netuid: netuid.into(),
                tempo: Some(Self::get_tempo(netuid).into()),
                ..Default::default()
            }),
            "last_step" => Some(SelectiveMetagraph {
                netuid: netuid.into(),
                last_step: Some(LastMechansimStepBlock::<T>::get(netuid).into()),
                ..Default::default()
            }),
            "blocks_since_last_step" => {
                let current_block: u64 = Pallet::<T>::get_current_block_as_u64();
                let last_step = LastMechansimStepBlock::<T>::get(netuid);
                let blocks_since_last_step: u64 = current_block.saturating_sub(last_step);
                Some(SelectiveMetagraph {
                    netuid: netuid.into(),
                    blocks_since_last_step: Some(blocks_since_last_step.into()),
                    ..Default::default()
                })
            }

            // Subnet emission terms
            "subnet_emission" => Some(SelectiveMetagraph {
                netuid: netuid.into(),
                subnet_emission: Some(0.into()),
                ..Default::default()
            }),
            "alpha_in" => Some(SelectiveMetagraph {
                netuid: netuid.into(),
                alpha_in: Some(SubnetAlphaIn::<T>::get(netuid).into()),
                ..Default::default()
            }),
            "alpha_out" => Some(SelectiveMetagraph {
                netuid: netuid.into(),
                alpha_out: Some(SubnetAlphaOut::<T>::get(netuid).into()),
                ..Default::default()
            }),
            "tao_in" => Some(SelectiveMetagraph {
                netuid: netuid.into(),
                tao_in: Some(SubnetTAO::<T>::get(netuid).into()),
                ..Default::default()
            }),
            "alpha_out_emission" => Some(SelectiveMetagraph {
                netuid: netuid.into(),
                alpha_out_emission: Some(SubnetAlphaOutEmission::<T>::get(netuid).into()),
                ..Default::default()
            }),
            "alpha_in_emission" => Some(SelectiveMetagraph {
                netuid: netuid.into(),
                alpha_in_emission: Some(SubnetAlphaInEmission::<T>::get(netuid).into()),
                ..Default::default()
            }),
            "tao_in_emission" => Some(SelectiveMetagraph {
                netuid: netuid.into(),
                tao_in_emission: Some(SubnetTaoInEmission::<T>::get(netuid).into()),
                ..Default::default()
            }),
            "pending_alpha_emission" => Some(SelectiveMetagraph {
                netuid: netuid.into(),
                pending_alpha_emission: Some(PendingEmission::<T>::get(netuid).into()),
                ..Default::default()
            }),
            "pending_root_emission" => Some(SelectiveMetagraph {
                netuid: netuid.into(),
                pending_root_emission: Some(PendingRootDivs::<T>::get(netuid).into()),
                ..Default::default()
            }),
            "subnet_volume" => Some(SelectiveMetagraph {
                netuid: netuid.into(),
                subnet_volume: Some(SubnetVolume::<T>::get(netuid).into()),
                ..Default::default()
            }),
            "moving_price" => Some(SelectiveMetagraph {
                netuid: netuid.into(),
                moving_price: Some(SubnetMovingPrice::<T>::get(netuid)),
                ..Default::default()
            }),

            // Hparams for epoch
            "rho" => Some(SelectiveMetagraph {
                netuid: netuid.into(),
                rho: Some(Self::get_rho(netuid).into()),
                ..Default::default()
            }),
            "kappa" => Some(SelectiveMetagraph {
                netuid: netuid.into(),
                kappa: Some(Self::get_kappa(netuid).into()),
                ..Default::default()
            }),

            // Validator params
            "min_allowed_weights" => Some(SelectiveMetagraph {
                netuid: netuid.into(),
                min_allowed_weights: Some(Self::get_min_allowed_weights(netuid).into()),
                ..Default::default()
            }),
            "max_weights_limit" => Some(SelectiveMetagraph {
                netuid: netuid.into(),
                max_weights_limit: Some(Self::get_max_weight_limit(netuid).into()),
                ..Default::default()
            }),
            "weights_version" => Some(SelectiveMetagraph {
                netuid: netuid.into(),
                weights_version: Some(Self::get_weights_version_key(netuid).into()),
                ..Default::default()
            }),
            "weights_rate_limit" => Some(SelectiveMetagraph {
                netuid: netuid.into(),
                weights_rate_limit: Some(Self::get_weights_set_rate_limit(netuid).into()),
                ..Default::default()
            }),
            "activity_cutoff" => Some(SelectiveMetagraph {
                netuid: netuid.into(),
                activity_cutoff: Some(Self::get_activity_cutoff(netuid).into()),
                ..Default::default()
            }),
            "max_validators" => Some(SelectiveMetagraph {
                netuid: netuid.into(),
                max_validators: Some(Self::get_max_allowed_validators(netuid).into()),
                ..Default::default()
            }),

            // Registration
            "num_uids" => Some(SelectiveMetagraph {
                netuid: netuid.into(),
                num_uids: Some(Self::get_subnetwork_n(netuid).into()),
                ..Default::default()
            }),
            "max_uids" => Some(SelectiveMetagraph {
                netuid: netuid.into(),
                max_uids: Some(Self::get_max_allowed_uids(netuid).into()),
                ..Default::default()
            }),
            "registration_allowed" => Some(SelectiveMetagraph {
                netuid: netuid.into(),
                registration_allowed: Some(Self::get_network_registration_allowed(netuid).into()),
                ..Default::default()
            }),
            "pow_registration_allowed" => Some(SelectiveMetagraph {
                netuid: netuid.into(),
                pow_registration_allowed: Some(
                    Self::get_network_pow_registration_allowed(netuid).into(),
                ),
                ..Default::default()
            }),
            "difficulty" => Some(SelectiveMetagraph {
                netuid: netuid.into(),
                difficulty: Some(Self::get_difficulty_as_u64(netuid).into()),
                ..Default::default()
            }),

            "burn" => Some(SelectiveMetagraph {
                netuid: netuid.into(),
                burn: Some(Self::get_burn_as_u64(netuid).into()),
                ..Default::default()
            }),

            "immunity_period" => Some(SelectiveMetagraph {
                netuid: netuid.into(),
                immunity_period: Some(Self::get_immunity_period(netuid).into()),
                ..Default::default()
            }),
            "min_difficulty" => Some(SelectiveMetagraph {
                netuid: netuid.into(),
                min_difficulty: Some(Self::get_min_difficulty(netuid).into()),
                ..Default::default()
            }),
            "max_difficulty" => Some(SelectiveMetagraph {
                netuid: netuid.into(),
                max_difficulty: Some(Self::get_max_difficulty(netuid).into()),
                ..Default::default()
            }),
            "min_burn" => Some(SelectiveMetagraph {
                netuid: netuid.into(),
                min_burn: Some(Self::get_min_burn_as_u64(netuid).into()),
                ..Default::default()
            }),
            "max_burn" => Some(SelectiveMetagraph {
                netuid: netuid.into(),
                max_burn: Some(Self::get_max_burn_as_u64(netuid).into()),
                ..Default::default()
            }),
            "adjustment_alpha" => Some(SelectiveMetagraph {
                netuid: netuid.into(),
                adjustment_alpha: Some(Self::get_adjustment_alpha(netuid).into()),
                ..Default::default()
            }),
            "adjustment_interval" => Some(SelectiveMetagraph {
                netuid: netuid.into(),
                adjustment_interval: Some(Self::get_adjustment_interval(netuid).into()),
                ..Default::default()
            }),
            "target_regs_per_interval" => Some(SelectiveMetagraph {
                netuid: netuid.into(),
                target_regs_per_interval: Some(
                    Self::get_target_registrations_per_interval(netuid).into(),
                ),
                ..Default::default()
            }),
            "max_regs_per_block" => Some(SelectiveMetagraph {
                netuid: netuid.into(),
                max_regs_per_block: Some(Self::get_max_registrations_per_block(netuid).into()),
                ..Default::default()
            }),
            "serving_rate_limit" => Some(SelectiveMetagraph {
                netuid: netuid.into(),
                serving_rate_limit: Some(Self::get_serving_rate_limit(netuid).into()),
                ..Default::default()
            }),

            // CR
            "commit_reveal_weights_enabled" => Some(SelectiveMetagraph {
                netuid: netuid.into(),
                commit_reveal_weights_enabled: Some(Self::get_commit_reveal_weights_enabled(
                    netuid,
                )),
                ..Default::default()
            }),
            "commit_reveal_period" => Some(SelectiveMetagraph {
                netuid: netuid.into(),
                commit_reveal_period: Some(Self::get_reveal_period(netuid).into()),
                ..Default::default()
            }),

            // Bonds
            "liquid_alpha_enabled" => Some(SelectiveMetagraph {
                netuid: netuid.into(),
                liquid_alpha_enabled: Some(Self::get_liquid_alpha_enabled(netuid)),
                ..Default::default()
            }),
            "alpha_high" => Some(SelectiveMetagraph {
                netuid: netuid.into(),
                alpha_high: Some(Self::get_alpha_values(netuid).1.into()),
                ..Default::default()
            }),
            "alpha_low" => Some(SelectiveMetagraph {
                netuid: netuid.into(),
                alpha_low: Some(Self::get_alpha_values(netuid).0.into()),
                ..Default::default()
            }),
            "bonds_moving_avg" => Some(SelectiveMetagraph {
                netuid: netuid.into(),
                bonds_moving_avg: Some(Self::get_bonds_moving_average(netuid).into()),
                ..Default::default()
            }),

            // Metagraph info.
            "hotkeys" => {
                let n: u16 = Self::get_subnetwork_n(netuid);
                let mut hotkeys: Vec<T::AccountId> = vec![];
                for uid in 0..n {
                    let hotkey = Keys::<T>::get(netuid, uid);
                    hotkeys.push(hotkey.clone());
                }

                Some(SelectiveMetagraph {
                    netuid: netuid.into(),
                    hotkeys: Some(hotkeys),
                    ..Default::default()
                })
            }
            "coldkeys" => {
                let n: u16 = Self::get_subnetwork_n(netuid);
                let mut coldkeys: Vec<T::AccountId> = vec![];
                for uid in 0..n {
                    let hotkey = Keys::<T>::get(netuid, uid);
                    let coldkey = Owner::<T>::get(hotkey.clone());
                    coldkeys.push(coldkey.clone());
                }
                Some(SelectiveMetagraph {
                    netuid: netuid.into(),
                    coldkeys: Some(coldkeys),
                    ..Default::default()
                })
            }
            "identities" => {
                let n: u16 = Self::get_subnetwork_n(netuid);
                let mut identities: Vec<Option<ChainIdentityOfV2>> = vec![];
                for uid in 0..n {
                    let hotkey = Keys::<T>::get(netuid, uid);
                    let coldkey = Owner::<T>::get(hotkey.clone());
                    identities.push(IdentitiesV2::<T>::get(coldkey.clone()));
                }
                Some(SelectiveMetagraph {
                    netuid: netuid.into(),
                    identities: Some(identities),
                    ..Default::default()
                })
            }
            "axons" => {
                let n: u16 = Self::get_subnetwork_n(netuid);
                let mut axons: Vec<AxonInfo> = vec![];
                for uid in 0..n {
                    let hotkey = Keys::<T>::get(netuid, uid);
                    axons.push(Self::get_axon_info(netuid, &hotkey));
                }
                Some(SelectiveMetagraph {
                    netuid: netuid.into(),
                    axons: Some(axons),
                    ..Default::default()
                })
            }
            "active" => Some(SelectiveMetagraph {
                netuid: netuid.into(),
                active: Some(Active::<T>::get(netuid)),
                ..Default::default()
            }),
            "validator_permit" => Some(SelectiveMetagraph {
                netuid: netuid.into(),
                active: Some(ValidatorPermit::<T>::get(netuid)),
                ..Default::default()
            }),

            "pruning_score" => Some(SelectiveMetagraph {
                netuid: netuid.into(),
                pruning_score: Some(
                    PruningScores::<T>::get(netuid)
                        .into_iter()
                        .map(Compact::from)
                        .collect(),
                ),
                ..Default::default()
            }),

            "last_update" => Some(SelectiveMetagraph {
                netuid: netuid.into(),
                last_update: Some(
                    LastUpdate::<T>::get(netuid)
                        .into_iter()
                        .map(Compact::from)
                        .collect(),
                ),
                ..Default::default()
            }),

            "emission" => Some(SelectiveMetagraph {
                netuid: netuid.into(),
                emission: Some(
                    Emission::<T>::get(netuid)
                        .into_iter()
                        .map(Compact::from)
                        .collect(),
                ),
                ..Default::default()
            }),

            "dividends" => Some(SelectiveMetagraph {
                netuid: netuid.into(),
                dividends: Some(
                    Dividends::<T>::get(netuid)
                        .into_iter()
                        .map(Compact::from)
                        .collect(),
                ),
                ..Default::default()
            }),

            "incentives" => Some(SelectiveMetagraph {
                netuid: netuid.into(),
                incentives: Some(
                    Incentive::<T>::get(netuid)
                        .into_iter()
                        .map(Compact::from)
                        .collect(),
                ),
                ..Default::default()
            }),

            "consensus" => Some(SelectiveMetagraph {
                netuid: netuid.into(),
                consensus: Some(
                    Consensus::<T>::get(netuid)
                        .into_iter()
                        .map(Compact::from)
                        .collect(),
                ),
                ..Default::default()
            }),

            "trust" => Some(SelectiveMetagraph {
                netuid: netuid.into(),
                trust: Some(
                    Trust::<T>::get(netuid)
                        .into_iter()
                        .map(Compact::from)
                        .collect(),
                ),
                ..Default::default()
            }),
            "rank" => Some(SelectiveMetagraph {
                netuid: netuid.into(),
                rank: Some(
                    Rank::<T>::get(netuid)
                        .into_iter()
                        .map(Compact::from)
                        .collect(),
                ),
                ..Default::default()
            }),
            "block_at_registration" => {
                let n: u16 = Self::get_subnetwork_n(netuid);
                let mut block_at_registration: Vec<Compact<u64>> = vec![];
                for uid in 0..n {
                    block_at_registration.push(BlockAtRegistration::<T>::get(netuid, uid).into());
                }
                Some(SelectiveMetagraph {
                    netuid: netuid.into(),
                    block_at_registration: Some(block_at_registration),
                    ..Default::default()
                })
            }
            "alpha_stake" => {
                let (_, alpha_stake_fl, _): (Vec<I64F64>, Vec<I64F64>, Vec<I64F64>) =
                    Self::get_stake_weights_for_network(netuid);
                Some(SelectiveMetagraph {
                    netuid: netuid.into(),
                    alpha_stake: Some(
                        alpha_stake_fl
                            .iter()
                            .map(|xi| Compact::from(fixed64_to_u64(*xi)))
                            .collect::<Vec<Compact<u64>>>(),
                    ),
                    ..Default::default()
                })
            }
            "tao_stake" => {
                let (_, _, tao_stake_fl): (Vec<I64F64>, Vec<I64F64>, Vec<I64F64>) =
                    Self::get_stake_weights_for_network(netuid);
                Some(SelectiveMetagraph {
                    netuid: netuid.into(),
                    tao_stake: Some(
                        tao_stake_fl
                            .iter()
                            .map(|xi| Compact::from(fixed64_to_u64(*xi)))
                            .collect::<Vec<Compact<u64>>>(),
                    ),
                    ..Default::default()
                })
            }
            "total_stake" => {
                let (total_stake_fl, _, _): (Vec<I64F64>, Vec<I64F64>, Vec<I64F64>) =
                    Self::get_stake_weights_for_network(netuid);
                Some(SelectiveMetagraph {
                    netuid: netuid.into(),
                    total_stake: Some(
                        total_stake_fl
                            .iter()
                            .map(|xi| Compact::from(fixed64_to_u64(*xi)))
                            .collect::<Vec<Compact<u64>>>(),
                    ),
                    ..Default::default()
                })
            }

            // Dividend break down.
            "tao_dividends_per_hotkey" => {
                let n: u16 = Self::get_subnetwork_n(netuid);
                let mut hotkeys: Vec<T::AccountId> = vec![];
                for uid in 0..n {
                    let hotkey = Keys::<T>::get(netuid, uid);
                    hotkeys.push(hotkey.clone());
                }
                let mut tao_dividends_per_hotkey: Vec<(T::AccountId, Compact<u64>)> = vec![];
                for hotkey in hotkeys.clone() {
                    let tao_divs = TaoDividendsPerSubnet::<T>::get(netuid, hotkey.clone());
                    tao_dividends_per_hotkey.push((hotkey.clone(), tao_divs.into()));
                }
                Some(SelectiveMetagraph {
                    netuid: netuid.into(),
                    tao_dividends_per_hotkey: Some(tao_dividends_per_hotkey),
                    ..Default::default()
                })
            }
            "alpha_dividends_per_hotkey" => {
                let mut alpha_dividends_per_hotkey: Vec<(T::AccountId, Compact<u64>)> = vec![];
                let n: u16 = Self::get_subnetwork_n(netuid);
                let mut hotkeys: Vec<T::AccountId> = vec![];

                for uid in 0..n {
                    let hotkey = Keys::<T>::get(netuid, uid);
                    hotkeys.push(hotkey.clone());
                }

                for hotkey in hotkeys.clone() {
                    let alpha_divs = AlphaDividendsPerSubnet::<T>::get(netuid, hotkey.clone());
                    alpha_dividends_per_hotkey.push((hotkey.clone(), alpha_divs.into()));
                }
                Some(SelectiveMetagraph {
                    netuid: netuid.into(),
                    alpha_dividends_per_hotkey: Some(alpha_dividends_per_hotkey),
                    ..Default::default()
                })
            }
            _ => {
                Some(SelectiveMetagraph {
                    // Subnet index
                    netuid: netuid.into(),
                    ..Default::default()
                })
            }
        }
    }
}
