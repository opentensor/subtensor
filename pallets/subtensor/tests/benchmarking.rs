#![recursion_limit = "512"]

use pallet_subtensor::Error;
use frame_support::assert_ok;
use frame_system::Config;
use crate::mock::*;

mod mock;

/********************************************
	test benchmarking extrinsics are enabled/disabled with flag
*********************************************/

#[test]
fn test_benchmark_epoch_with_weights() {
	new_test_ext().execute_with(|| {
        let result = SubtensorModule::benchmark_epoch_with_weights(<<Test as Config>::RuntimeOrigin>::root());

        if cfg!(feature = "runtime-benchmarks") {
            // benchmarking enabled
            assert_ok!(result);
        } else {
            // benchmarking disabled
            // check error
            assert_eq!(result, Err(Error::<Test>::InvalidEmissionValues.into()));
        }
	});
}

#[test]
fn test_benchmark_epoch_without_weights() {
	new_test_ext().execute_with(|| {
        let result = SubtensorModule::benchmark_epoch_without_weights(<<Test as Config>::RuntimeOrigin>::root());

        if cfg!(feature = "runtime-benchmarks") {
            // benchmarking enabled
            assert_ok!(result);
        } else {
            // benchmarking disabled
            // check error
            assert_eq!(result, Err(Error::<Test>::InvalidEmissionValues.into()));
        }
	});
}

#[test]
fn test_benchmark_drain_emission() {
	new_test_ext().execute_with(|| {
        let result = SubtensorModule::benchmark_drain_emission(<<Test as Config>::RuntimeOrigin>::root());

        if cfg!(feature = "runtime-benchmarks") {
            // benchmarking enabled
            assert_ok!(result);
        } else {
            // benchmarking disabled
            // check error
            assert_eq!(result, Err(Error::<Test>::InvalidEmissionValues.into()));
        }
	});
}
    
    