#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::result_unit_err)]

use sp_std::marker;
use sp_std::ops::Neg;
use substrate_fixed::types::{I64F64, U64F64};

pub trait SharePoolDataOperations<Key> {
    /// Gets shared value
    fn get_shared_value(&self) -> U64F64;
    /// Gets single share for a given key
    fn get_share(&self, key: &Key) -> U64F64;
    // Tries to get a single share for a given key, as a result.
    fn try_get_share(&self, key: &Key) -> Result<U64F64, ()>;
    /// Gets share pool denominator
    fn get_denominator(&self) -> U64F64;
    /// Updates shared value by provided signed value
    fn set_shared_value(&mut self, value: U64F64);
    /// Update single share for a given key by provided signed value
    fn set_share(&mut self, key: &Key, share: U64F64);
    /// Update share pool denominator by provided signed value
    fn set_denominator(&mut self, update: U64F64);
}

/// SharePool struct that depends on the Key type and uses the SharePoolDataOperations
#[derive(Debug)]
pub struct SharePool<K, Ops>
where
    K: Eq,
    Ops: SharePoolDataOperations<K>,
{
    state_ops: Ops,
    phantom_key: marker::PhantomData<K>,
}

impl<K, Ops> SharePool<K, Ops>
where
    K: Eq,
    Ops: SharePoolDataOperations<K>,
{
    pub fn new(ops: Ops) -> Self {
        SharePool {
            state_ops: ops,
            phantom_key: marker::PhantomData,
        }
    }

    pub fn get_value(&self, key: &K) -> u64 {
        let shared_value: U64F64 = self.state_ops.get_shared_value();
        let current_share: U64F64 = self.state_ops.get_share(key);
        let denominator: U64F64 = self.state_ops.get_denominator();

        let maybe_value_per_share = shared_value.checked_div(denominator);
        (if let Some(value_per_share) = maybe_value_per_share {
            value_per_share.saturating_mul(current_share)
        } else {
            shared_value
                .saturating_mul(current_share)
                .checked_div(denominator)
                .unwrap_or(U64F64::saturating_from_num(0))
        })
        .saturating_to_num::<u64>()
    }

    pub fn get_value_from_shares(&self, current_share: U64F64) -> u64 {
        let shared_value: U64F64 = self.state_ops.get_shared_value();
        let denominator: U64F64 = self.state_ops.get_denominator();

        let maybe_value_per_share = shared_value.checked_div(denominator);
        (if let Some(value_per_share) = maybe_value_per_share {
            value_per_share.saturating_mul(current_share)
        } else {
            shared_value
                .saturating_mul(current_share)
                .checked_div(denominator)
                .unwrap_or(U64F64::saturating_from_num(0))
        })
        .saturating_to_num::<u64>()
    }

    pub fn try_get_value(&self, key: &K) -> Result<u64, ()> {
        match self.state_ops.try_get_share(key) {
            Ok(_) => Ok(self.get_value(key)),
            Err(i) => Err(i),
        }
    }

    /// Update the total shared value.
    /// Every key's associated value effectively updates with this operation
    pub fn update_value_for_all(&mut self, update: i64) {
        let shared_value: U64F64 = self.state_ops.get_shared_value();
        self.state_ops.set_shared_value(if update >= 0 {
            shared_value.saturating_add(U64F64::saturating_from_num(update))
        } else {
            shared_value.saturating_sub(U64F64::saturating_from_num(update.neg()))
        });
    }

    pub fn sim_update_value_for_one(&mut self, update: i64) -> bool {
        let shared_value: U64F64 = self.state_ops.get_shared_value();
        let denominator: U64F64 = self.state_ops.get_denominator();

        // Then, update this key's share
        if denominator == 0 {
            true
        } else {
            // There are already keys in the pool, set or update this key
            let shares_per_update: I64F64 =
                self.get_shares_per_update(update, &shared_value, &denominator);

            shares_per_update != 0
        }
    }

    fn get_shares_per_update(
        &self,
        update: i64,
        shared_value: &U64F64,
        denominator: &U64F64,
    ) -> I64F64 {
        let maybe_value_per_share = shared_value.checked_div(*denominator);
        if let Some(value_per_share) = maybe_value_per_share {
            I64F64::saturating_from_num(update)
                .checked_div(I64F64::saturating_from_num(value_per_share))
                .unwrap_or(I64F64::saturating_from_num(0))
        } else {
            I64F64::saturating_from_num(update)
                .checked_div(I64F64::saturating_from_num(*shared_value))
                .unwrap_or(I64F64::saturating_from_num(0))
                .saturating_mul(I64F64::saturating_from_num(*denominator))
        }
    }

    /// Update the value associated with an item identified by the Key
    /// Returns actual update
    ///
    pub fn update_value_for_one(&mut self, key: &K, update: i64) -> i64 {
        let shared_value: U64F64 = self.state_ops.get_shared_value();
        let current_share: U64F64 = self.state_ops.get_share(key);
        let denominator: U64F64 = self.state_ops.get_denominator();
        let initial_value: i64 = self.get_value(key) as i64;
        let mut actual_update: i64 = update;

        // Then, update this key's share
        if denominator == 0 {
            // Initialize the pool. The first key gets all.
            let update_fixed: U64F64 = U64F64::saturating_from_num(update);
            self.state_ops.set_denominator(update_fixed);
            self.state_ops.set_share(key, update_fixed);
        } else {
            let shares_per_update: I64F64 =
                self.get_shares_per_update(update, &shared_value, &denominator);

            if shares_per_update >= 0 {
                self.state_ops.set_denominator(
                    denominator.saturating_add(U64F64::saturating_from_num(shares_per_update)),
                );
                self.state_ops.set_share(
                    key,
                    current_share.saturating_add(U64F64::saturating_from_num(shares_per_update)),
                );
            } else {
                // Calculate new share and denominator after the decrease
                let shares_decrease = U64F64::saturating_from_num(shares_per_update.neg());
                let new_denominator = denominator.saturating_sub(shares_decrease);
                let new_share = current_share.saturating_sub(shares_decrease);

                // Only force full unstake if the remaining share would be essentially zero
                // (less than 1 unit in the smallest representable value)
                // This preserves precision for partial unstakes while still cleaning up
                // truly negligible remainders
                if new_share < U64F64::saturating_from_num(1u64)
                    || new_denominator < U64F64::saturating_from_num(1u64)
                {
                    // Remaining share is negligible, remove all to clean up storage
                    self.state_ops
                        .set_denominator(denominator.saturating_sub(current_share));
                    self.state_ops
                        .set_share(key, U64F64::saturating_from_num(0));
                    actual_update = initial_value.neg();
                } else {
                    self.state_ops.set_denominator(new_denominator);
                    self.state_ops.set_share(key, new_share);
                }
            }
        }

        // Update shared value
        self.update_value_for_all(actual_update);

        // Return actual udate
        actual_update
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    struct MockSharePoolDataOperations {
        shared_value: U64F64,
        share: BTreeMap<u16, U64F64>,
        denominator: U64F64,
    }

    impl MockSharePoolDataOperations {
        fn new() -> Self {
            MockSharePoolDataOperations {
                shared_value: U64F64::saturating_from_num(0),
                share: BTreeMap::new(),
                denominator: U64F64::saturating_from_num(0),
            }
        }
    }

    impl SharePoolDataOperations<u16> for MockSharePoolDataOperations {
        fn get_shared_value(&self) -> U64F64 {
            self.shared_value
        }

        fn get_share(&self, key: &u16) -> U64F64 {
            *self
                .share
                .get(key)
                .unwrap_or(&U64F64::saturating_from_num(0))
        }

        fn try_get_share(&self, key: &u16) -> Result<U64F64, ()> {
            match self.share.get(key) {
                Some(&value) => Ok(value),
                None => Err(()),
            }
        }

        fn get_denominator(&self) -> U64F64 {
            self.denominator
        }

        fn set_shared_value(&mut self, value: U64F64) {
            self.shared_value = value;
        }

        fn set_share(&mut self, key: &u16, share: U64F64) {
            self.share.insert(*key, share);
        }

        fn set_denominator(&mut self, update: U64F64) {
            self.denominator = update;
        }
    }

    #[test]
    fn test_get_value() {
        let mut mock_ops = MockSharePoolDataOperations::new();
        mock_ops.set_denominator(U64F64::saturating_from_num(10));
        mock_ops.set_share(&1_u16, U64F64::saturating_from_num(3));
        mock_ops.set_share(&2_u16, U64F64::saturating_from_num(7));
        mock_ops.set_shared_value(U64F64::saturating_from_num(100));
        let share_pool = SharePool::new(mock_ops);
        let result1 = share_pool.get_value(&1);
        let result2 = share_pool.get_value(&2);
        assert_eq!(result1, 30);
        assert_eq!(result2, 70);
    }

    #[test]
    fn test_division_by_zero() {
        let mut mock_ops = MockSharePoolDataOperations::new();
        mock_ops.set_denominator(U64F64::saturating_from_num(0)); // Zero denominator
        let pool = SharePool::<u16, MockSharePoolDataOperations>::new(mock_ops);

        let value = pool.get_value(&1);
        assert_eq!(value, 0, "Value should be 0 when denominator is zero");
    }

    #[test]
    fn test_max_shared_value() {
        let mut mock_ops = MockSharePoolDataOperations::new();
        mock_ops.set_shared_value(U64F64::saturating_from_num(u64::MAX));
        mock_ops.set_share(&1, U64F64::saturating_from_num(3)); // Use a neutral value for share
        mock_ops.set_share(&2, U64F64::saturating_from_num(7)); // Use a neutral value for share
        mock_ops.set_denominator(U64F64::saturating_from_num(10)); // Neutral value to see max effect
        let pool = SharePool::<u16, MockSharePoolDataOperations>::new(mock_ops);

        let max_value = pool.get_value(&1) + pool.get_value(&2);
        assert!(u64::MAX - max_value <= 5, "Max value should map to u64 MAX");
    }

    #[test]
    fn test_max_share_value() {
        let mut mock_ops = MockSharePoolDataOperations::new();
        mock_ops.set_shared_value(U64F64::saturating_from_num(1_000_000_000)); // Use a neutral value for shared value
        mock_ops.set_share(&1, U64F64::saturating_from_num(u64::MAX / 2));
        mock_ops.set_share(&2, U64F64::saturating_from_num(u64::MAX / 2));
        mock_ops.set_denominator(U64F64::saturating_from_num(u64::MAX));
        let pool = SharePool::<u16, MockSharePoolDataOperations>::new(mock_ops);

        let value1 = pool.get_value(&1) as i128;
        let value2 = pool.get_value(&2) as i128;

        assert!((value1 - 500_000_000).abs() <= 1);
        assert!((value2 - 500_000_000).abs() <= 1);
    }

    #[test]
    fn test_denom_precision() {
        let mock_ops = MockSharePoolDataOperations::new();
        let mut pool = SharePool::<u16, MockSharePoolDataOperations>::new(mock_ops);

        pool.update_value_for_one(&1, 1000);

        let value_tmp = pool.get_value(&1) as i128;
        assert_eq!(value_tmp, 1000);

        pool.update_value_for_one(&1, -990);
        pool.update_value_for_one(&2, 1000);
        pool.update_value_for_one(&2, -990);

        let value1 = pool.get_value(&1) as i128;
        let value2 = pool.get_value(&2) as i128;

        assert_eq!(value1, 10);
        assert_eq!(value2, 10);
    }

    // cargo test --package share-pool --lib -- tests::test_denom_high_precision --exact --show-output
    #[test]
    fn test_denom_high_precision() {
        let mock_ops = MockSharePoolDataOperations::new();
        let mut pool = SharePool::<u16, MockSharePoolDataOperations>::new(mock_ops);

        pool.update_value_for_one(&1, 1);
        pool.update_value_for_one(&2, 1);

        pool.update_value_for_all(999_999_999_999_998);

        pool.update_value_for_one(&1, -499_999_999_999_990);
        pool.update_value_for_one(&2, -499_999_999_999_990);

        pool.update_value_for_all(999_999_999_999_980);

        pool.update_value_for_one(&1, 1_000_000_000_000);
        pool.update_value_for_one(&2, 1_000_000_000_000);

        let value1 = pool.get_value(&1) as i128;
        let value2 = pool.get_value(&2) as i128;

        // First to stake gets all accumulated emission if there are no other stakers
        // (which is artificial situation because there will be no emissions if there is no stake)
        assert!((value1 - 1_001_000_000_000_000).abs() < 100);
        assert!((value2 - 1_000_000_000_000).abs() < 100);
    }

    // cargo test --package share-pool --lib -- tests::test_denom_high_precision_many_small_unstakes --exact --show-output
    #[test]
    fn test_denom_high_precision_many_small_unstakes() {
        let mock_ops = MockSharePoolDataOperations::new();
        let mut pool = SharePool::<u16, MockSharePoolDataOperations>::new(mock_ops);

        pool.update_value_for_one(&1, 1);
        pool.update_value_for_one(&2, 1);

        pool.update_value_for_all(1_000_000_000_000_000);

        for _ in 0..1_000_000 {
            pool.update_value_for_one(&1, -500_000_000);
            pool.update_value_for_one(&2, -500_000_000);
        }

        pool.update_value_for_all(1_000_000_000_000_000);

        pool.update_value_for_one(&1, 1_000_000_000_000);
        pool.update_value_for_one(&2, 1_000_000_000_000);

        let value1 = pool.get_value(&1) as i128;
        let value2 = pool.get_value(&2) as i128;

        assert!((value1 - 1_001_000_000_000_000).abs() < 10);
        assert!((value2 - 1_000_000_000_000).abs() < 10);
    }

    #[test]
    fn test_update_value_for_one() {
        let mock_ops = MockSharePoolDataOperations::new();
        let mut pool = SharePool::<u16, MockSharePoolDataOperations>::new(mock_ops);

        pool.update_value_for_one(&1, 1000);

        let value = pool.get_value(&1) as i128;
        assert_eq!(value, 1000);
    }

    #[test]
    fn test_update_value_for_all() {
        let mock_ops = MockSharePoolDataOperations::new();
        let mut pool = SharePool::<u16, MockSharePoolDataOperations>::new(mock_ops);

        pool.update_value_for_all(1000);
        assert_eq!(
            pool.state_ops.shared_value,
            U64F64::saturating_from_num(1000)
        );
    }

    // cargo test --package share-pool --lib -- tests::test_get_shares_per_update --exact --show-output
    #[test]
    fn test_get_shares_per_update() {
        [
            (1_i64, 1_u64, 1.0, 1.0),
            (
                1_000,
                21_000_000_000_000_000,
                0.00001,
                0.00000000000000000043,
            ),
            (
                21_000_000_000_000_000,
                21_000_000_000_000_000,
                0.00001,
                0.00001,
            ),
            (
                210_000_000_000_000_000,
                21_000_000_000_000_000,
                0.00001,
                0.0001,
            ),
            (
                1_000,
                1_000,
                21_000_000_000_000_000_f64,
                21_000_000_000_000_000_f64,
            ),
        ]
        .iter()
        .for_each(|(update, shared_value, denominator, expected)| {
            let mock_ops = MockSharePoolDataOperations::new();
            let pool = SharePool::<u16, MockSharePoolDataOperations>::new(mock_ops);

            let shared_fixed = U64F64::from_num(*shared_value);
            let denominator_fixed = U64F64::from_num(*denominator);
            let expected_fixed = I64F64::from_num(*expected);

            let spu: I64F64 =
                pool.get_shares_per_update(*update, &shared_fixed, &denominator_fixed);
            let precision: I64F64 = I64F64::from_num(1000.);
            assert!((spu - expected_fixed).abs() <= expected_fixed / precision,);
        });
    }
}
