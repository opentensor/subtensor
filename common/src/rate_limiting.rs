//! Shared rate-limiting types.

/// Identifier type for rate-limiting groups.
pub type GroupId = u32;

/// Group id for serving-related calls.
pub const GROUP_SERVE: GroupId = 0;
/// Group id for delegate-take related calls.
pub const GROUP_DELEGATE_TAKE: GroupId = 1;
/// Group id for subnet weight-setting calls.
pub const GROUP_WEIGHTS_SUBNET: GroupId = 2;
/// Group id for network registration calls.
pub const GROUP_REGISTER_NETWORK: GroupId = 3;
/// Group id for owner hyperparameter calls.
pub const GROUP_OWNER_HPARAMS: GroupId = 4;
/// Group id for staking operations.
pub const GROUP_STAKING_OPS: GroupId = 5;
/// Group id for key swap calls.
pub const GROUP_SWAP_KEYS: GroupId = 6;
