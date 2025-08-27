#![allow(
    clippy::arithmetic_side_effects,
    clippy::indexing_slicing,
    clippy::unwrap_used
)]

/// Test plan:
///   - [ ] Netuid index math (with SubsubnetCountCurrent limiting)
///   - [ ] Emissions are split proportionally
///   - [ ] Sum of split emissions is equal to rao_emission passed to epoch
///   - [ ] Weights can be set/commited/revealed by subsubnet
///   - [ ] Rate limiting is enforced by subsubnet
///   - [ ] Bonds are applied per subsubnet
///   - [ ] Incentives are per subsubnet
///   - [ ] Subsubnet limit can be set up to 8 (with admin pallet)
///   - [ ] When subsubnet limit is reduced, reduction is GlobalSubsubnetDecreasePerSuperblock per super-block
///   - [ ] When reduction of subsubnet limit occurs, Weights, Incentive, LastUpdate, Bonds, and WeightCommits are cleared
///   - [ ] Epoch terms of subnet are weighted sum (or logical OR) of all subsubnet epoch terms
///   - [ ] Subnet epoch terms persist in state
///   - [ ] Subsubnet epoch terms persist in state

use super::mock::*;

#[test]
fn test_index_from_netuid_and_subnet() {
    new_test_ext(1).execute_with(|| {});
}

#[test]
fn test_subsubnet_emission_proportions() {
    new_test_ext(1).execute_with(|| {});
}
