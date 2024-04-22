use codec::Compact;
mod mock;
use frame_support::assert_ok;
use frame_system::Config;
use mock::*;
use sp_core::U256;

// #[test]
// fn test_get_dynamic_pool_info_ok() {
//     new_test_ext(1).execute_with(|| {
//         let netuid: u16 = 1;
//         let tempo: u16 = 13;
//         let mut dynamic_pool_info = SubtensorModule::get_dynamic_pool_info(netuid);     
//         assert_ok!(SubtensorModule::register_network(
//             <<Test as Config>::RuntimeOrigin>::signed(U256::from(0)),
//             U256::from(0),
//             0,
//             0,
//             0
//         ));
//         assert_ok!(SubtensorModule::add_subnet_stake(
//             <<Test as Config>::RuntimeOrigin>::signed(U256::from(0)),
//             U256::from(0),
//             netuid,
//             100_000_000_000_000
//         ));
//         assert_ok!(SubtensorModule::add_weighted_stake(
//             <<Test as Config>::RuntimeOrigin>::signed(U256::from(0)),
//             U256::from(0),
//             netuid,
//             vec![(U256::from(0), U256::from(0))],
//             vec![(U256::from(0), U256::from(0))]
//         ));
//     });
// }

// #[test]
// fn test_get_dynamic_pool_infos_ok() {
//     new_test_ext(1).execute_with(|| {
//         let netuid: u16 = 1;                
//         let mut dynamic_pool_infos = SubtensorModule::get_dynamic_pool_infos();
//         assert_ok!(SubtensorModule::register_network(
//             <<Test as Config>::RuntimeOrigin>::signed(U256::from(0)),
//             U256::from(0),
//             0,
//             0,
//             0
//         ));
//         assert_ok!(SubtensorModule::add_subnet_stake(
//             <<Test as Config>::RuntimeOrigin>::signed(U256::from(0)),
//             U256::from(0),
//             netuid,
//             100_000_000_000_000
//         ));
//         assert_ok!(SubtensorModule::add_weighted_stake(
//             <<Test as Config>::RuntimeOrigin>::signed(U256::from(0)),
//             U256::from(0),
//             netuid,
//             vec![(U256::from(0), U256::from(0))],
//             vec![(U256::from(0), U256::from(0))]
//         ));
//     });
// }