#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::result_unit_err)]

use codec::{Decode, Encode};
use lencode::io::Cursor;
use lencode::{Decode as LenDecode, Encode as LenEncode};
use safe_bigmath::*;
use scale_info::TypeInfo;
use sp_std::marker;
use sp_std::ops::Neg;
use substrate_fixed::types::U64F64;

// Maximum value that can be represented with SafeFloat
pub const SAFE_FLOAT_MAX: u128 = 1_000_000_000_000_000_000_000_u128;
pub const SAFE_FLOAT_MAX_EXP: i64 = 21_i64;

/// Controlled precision floating point number with efficient storage
///
/// Precision is controlled in a way that keeps enough mantissa digits so
/// that updating hotkey stake by 1 rao makes difference in the resulting shared
/// pool variables (both coldkey share and share pool denominator), but also
/// precision should be limited so that updating by 0.1 rao does not make the
/// difference (because there's no such thing as 0.1 rao, rao is integer).
#[derive(Clone, Debug)]
pub struct SafeFloat {
    mantissa: SafeInt,
    exponent: i64,
}

#[derive(Encode, Decode, Default, TypeInfo, Clone, PartialEq, Eq, Debug)]
pub struct SafeFloatSerializable {
    mantissa: Vec<u8>,
    exponent: i64,
}

/// Power of 10 in SafeInt
/// Uses SafeInt pow function that accepts u32 argument
/// and the formula: 10^(a*b) = (10^a)^b
fn pow10(e: u64) -> SafeInt {
    if e == 0 {
        return SafeInt::one();
    }
    let exp_high = ((e & 0xFFFFFFFF00000000) >> 32) as u32;
    let exp_low = (e & 0xFFFFFFFF) as u32;
    let ten_exp_low = SafeInt::from(10u32).pow(exp_low);
    let ten_exp_high = SafeInt::from(10u32).pow(exp_high);
    let two_exp_16 = 1u32 << 16;

    ten_exp_high.pow(two_exp_16).pow(two_exp_16) * ten_exp_low
}

fn intlog10(a: &SafeInt) -> u64 {
    let scale = SafeInt::from(1_000_000_000_000_000_000i128);
    let precision = 256u32;
    let max_iters = Some(4096);
    (a.log10(&scale, precision, max_iters))
        .unwrap_or_default()
        .to_u64()
        .unwrap_or_default()
}

impl SafeFloat {
    pub fn zero() -> Self {
        SafeFloat {
            mantissa: SafeInt::zero(),
            exponent: 0_i64,
        }
    }

    pub fn new(mantissa: SafeInt, exponent: i64) -> Option<Self> {
        // Cap at SAFE_FLOAT_MAX
        let max_value = SafeInt::from(SAFE_FLOAT_MAX) + SafeInt::one();
        if !(mantissa.clone() / max_value).unwrap_or_default().is_zero() {
            return None;
        }

        let mut safe_float = SafeFloat { mantissa, exponent };

        if safe_float.normalize() {
            Some(safe_float)
        } else {
            None
        }
    }

    /// Adjusts mantissa and exponent of this floating point number so that
    /// SAFE_FLOAT_MAX <= mantissa < 10 * SAFE_FLOAT_MAX
    ///
    /// Returns true in case of success or false if exponent over- or underflows
    pub(crate) fn normalize(&mut self) -> bool {
        let max_value = SafeInt::from(SAFE_FLOAT_MAX);
        let max_value_div10 = SafeInt::from(SAFE_FLOAT_MAX.checked_div(10).unwrap_or_default());
        let mantissa_abs = self.mantissa.clone().abs();

        let exponent_adjustment: i64 = if mantissa_abs.is_zero() {
            0i64
        } else if max_value_div10 >= mantissa_abs {
            // Mantissa is too low, upscale mantissa + reduce exponent
            let scale = (max_value_div10 / mantissa_abs).unwrap_or_default();
            ((intlog10(&scale).saturating_add(1)) as i64).neg()
        } else if max_value < mantissa_abs {
            // Mantissa is too high, downscale mantissa + increase exponent
            let scale = (mantissa_abs / max_value).unwrap_or_default();
            (intlog10(&scale).saturating_add(1)) as i64
        } else {
            0i64
        };

        // Check exponent over- or underflows
        let new_exponent_i128 = (self.exponent as i128).saturating_add(exponent_adjustment as i128);
        if (i64::MIN as i128 <= new_exponent_i128) && (new_exponent_i128 <= i64::MAX as i128) {
            self.exponent = new_exponent_i128 as i64;
        } else {
            return false;
        }

        if exponent_adjustment > 0 {
            let mantissa_adjustment = pow10(exponent_adjustment as u64);
            self.mantissa = (self.mantissa.clone() / mantissa_adjustment).unwrap_or_default();
        } else {
            let mantissa_adjustment = pow10(exponent_adjustment.neg() as u64);
            self.mantissa = self.mantissa.clone() * mantissa_adjustment
        }

        // Check if adjusted mantissa turned into zero, in which case set exponent to 0.
        if self.mantissa.is_zero() {
            self.exponent = 0;
        }

        true
    }

    /// Divide current value by a preserving precision (SAFE_FLOAT_MAX digits in mantissa)
    ///   result = m1 * 10^e1 / m2 * 10^e2
    pub fn div(&self, a: &SafeFloat) -> Option<Self> {
        // We need to offset exponent so that
        //   1. e1 - e2 is non-negative
        //   2. We have enough precision after division
        let redundant_exponent = SAFE_FLOAT_MAX_EXP.saturating_mul(2);

        let maybe_new_mantissa =
            self.mantissa.clone() * pow10(redundant_exponent as u64) / a.mantissa.clone();
        if let Some(new_mantissa) = maybe_new_mantissa {
            let mut safe_float = SafeFloat {
                mantissa: new_mantissa,
                exponent: self
                    .exponent
                    .saturating_sub(a.exponent)
                    .saturating_sub(redundant_exponent),
            };
            if safe_float.normalize() {
                Some(safe_float)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn add(&self, a: &SafeFloat) -> Option<Self> {
        // Multiply both operands by 10^exponent_offset so that both are above 1.
        // (lowest exponent becomes 0)
        let exponent_offset = self.exponent.min(a.exponent).neg();
        let unnormalized_mantissa = self.mantissa.clone()
            * pow10(self.exponent.saturating_add(exponent_offset) as u64)
            + a.mantissa.clone() * pow10(a.exponent.saturating_add(exponent_offset) as u64);

        let mut safe_float = SafeFloat {
            mantissa: unnormalized_mantissa,
            exponent: exponent_offset.neg(),
        };
        if safe_float.normalize() {
            Some(safe_float)
        } else {
            None
        }
    }

    /// Calculate self * a / b without loss of precision
    pub fn mul_div(&self, a: &SafeFloat, b: &SafeFloat) -> Option<Self> {
        let self_a_mantissa = self.mantissa.clone() * a.mantissa.clone();
        let self_a_exponent = self.exponent.saturating_add(a.exponent);

        // Divide by b without adjusting precision first (preserve higher precision
        // of multiplication result)
        SafeFloat {
            mantissa: self_a_mantissa,
            exponent: self_a_exponent,
        }
        .div(b)
    }

    pub fn is_zero(&self) -> bool {
        self.mantissa.is_zero()
    }

    /// Returns true if self > a
    pub fn gt(&self, a: &SafeFloat) -> bool {
        // Shortcut: same exponent → compare mantissas directly
        if self.exponent == a.exponent {
            return self.mantissa > a.mantissa;
        }

        // Bring both to the same exponent = max(exponents)
        let max_e = self.exponent.max(a.exponent);
        let k1 = max_e - self.exponent;
        let k2 = max_e - a.exponent;

        let scale1 = pow10(k1 as u64);
        let scale2 = pow10(k2 as u64);

        let lhs = &self.mantissa * &scale1;
        let rhs = &a.mantissa * &scale2;

        lhs - rhs > 0
    }
}

// Saturating conversion: negatives -> 0, overflow -> u64::MAX
impl From<&SafeFloat> for u64 {
    fn from(value: &SafeFloat) -> Self {
        // Negative values are clamped to 0
        if value.mantissa.is_negative() {
            return 0;
        }

        // If exponent is zero, it's just an integer mantissa
        if value.exponent == 0 {
            return value.mantissa.to_u64().unwrap_or(u64::MAX);
        }

        // scale = 10^exponent
        let scale = pow10(value.exponent.abs() as u64);

        // mantissa * 10^exponent
        let q: SafeInt = if value.exponent > 0 {
            &value.mantissa * &scale
        } else {
            (&value.mantissa / &scale).unwrap_or_else(SafeInt::zero)
        };

        // Convert quotient to u64, saturating on overflow
        if q.is_zero() {
            0
        } else {
            q.to_u64().unwrap_or(u64::MAX)
        }
    }
}

// Convenience impl for owning values
impl From<SafeFloat> for u64 {
    fn from(value: SafeFloat) -> Self {
        u64::from(&value)
    }
}

impl From<u64> for SafeFloat {
    fn from(value: u64) -> Self {
        SafeFloat::new(SafeInt::from(value), 0).unwrap_or_default()
    }
}

impl From<U64F64> for SafeFloat {
    fn from(value: U64F64) -> Self {
        let bits = value.to_bits();
        // High 64 bits = integer part
        let int = (bits >> 64) as u64;
        // Low 64 bits = fractional part
        let frac = (bits & 0xFFFF_FFFF_FFFF_FFFF) as u64;

        // If strictly zero, shortcut
        if bits == 0 {
            return SafeFloat::zero();
        }

        // SafeFloat for integer part: int * 10^0
        let safe_int = SafeFloat::new(SafeInt::from(int), 0).unwrap_or_default();

        // Numerator of fractional part: frac * 10^0
        let safe_frac_num = SafeFloat::new(SafeInt::from(frac), 0).unwrap_or_default();

        // Denominator = 2^64 as an integer SafeFloat: (2^64) * 10^0
        let two64: u128 = 1u128 << 64;
        let safe_two64 = SafeFloat::new(SafeInt::from(two64), 0).unwrap_or_default();

        // frac_part = frac / 2^64
        let safe_frac = safe_frac_num.div(&safe_two64).unwrap_or_default();

        // int + frac/2^64, with all mantissa/exponent normalization
        safe_int.add(&safe_frac).unwrap_or_default()
    }
}

impl From<&SafeFloat> for SafeFloatSerializable {
    fn from(value: &SafeFloat) -> Self {
        let mut mantissa_serializable = Vec::new();
        value
            .mantissa
            .encode(&mut mantissa_serializable)
            .unwrap_or_default();

        SafeFloatSerializable {
            mantissa: mantissa_serializable,
            exponent: value.exponent,
        }
    }
}

impl From<&SafeFloatSerializable> for SafeFloat {
    fn from(value: &SafeFloatSerializable) -> Self {
        let decoded = SafeInt::decode(&mut Cursor::new(&value.mantissa)).unwrap_or_default();
        SafeFloat {
            mantissa: decoded,
            exponent: value.exponent,
        }
    }
}

impl From<&SafeFloat> for f64 {
    fn from(value: &SafeFloat) -> Self {
        // Zero shortcut
        if value.mantissa.is_zero() {
            return 0.0;
        }

        // If you ever allow negative mantissas, handle sign here.
        // For now we assume mantissa >= 0 per your spec.
        let mut mant = value.mantissa.clone();
        let mut exp_i32 = value.exponent as i32;

        let ten = SafeInt::from(10);

        // Max integer exactly representable in f64: 2^53 - 1
        let max_exact = SafeInt::from((1u64 << 53) - 1);

        // While mantissa is too large to be exactly represented,
        // discard right decimal digits: mant /= 10, and adjust exponent
        // so that mant * 10^exp stays the same value.
        while mant > max_exact {
            mant = (&mant / &ten).unwrap_or_default();
            exp_i32 += 1; // because value = mant * 10^exp, and we did mant /= 10
        }

        // Now mant <= max_exact, so we can convert mant to u64 then to f64 exactly.
        let mant_u64 = mant.to_u64().unwrap_or_default();

        let mant_f = mant_u64 as f64;
        let scale = 10f64.powi(exp_i32);

        mant_f * scale
    }
}

impl From<SafeFloat> for f64 {
    fn from(value: SafeFloat) -> Self {
        f64::from(&value)
    }
}

impl Default for SafeFloat {
    fn default() -> Self {
        SafeFloat::zero()
    }
}

pub trait SharePoolDataOperations<Key> {
    /// Gets shared value (always "the real thing" measured in rao, not fractional)
    fn get_shared_value(&self) -> u64;
    /// Gets single share for a given key
    fn get_share(&self, key: &Key) -> SafeFloat;
    // Tries to get a single share for a given key, as a result.
    fn try_get_share(&self, key: &Key) -> Result<SafeFloat, ()>;
    /// Gets share pool denominator
    fn get_denominator(&self) -> SafeFloat;
    /// Updates shared value by provided signed value
    fn set_shared_value(&mut self, value: u64);
    /// Update single share for a given key by provided signed value
    fn set_share(&mut self, key: &Key, share: SafeFloat);
    /// Update share pool denominator by provided signed value
    fn set_denominator(&mut self, update: SafeFloat);
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
        let shared_value: SafeFloat =
            SafeFloat::new(SafeInt::from(self.state_ops.get_shared_value()), 0).unwrap_or_default();
        let current_share: SafeFloat = self.state_ops.get_share(key);
        let denominator: SafeFloat = self.state_ops.get_denominator();
        shared_value
            .mul_div(&current_share, &denominator)
            .unwrap_or(SafeFloat::zero())
            .into()
    }

    pub fn get_value_from_shares(&self, current_share: SafeFloat) -> u64 {
        let shared_value: SafeFloat =
            SafeFloat::new(SafeInt::from(self.state_ops.get_shared_value()), 0).unwrap_or_default();
        let denominator: SafeFloat = self.state_ops.get_denominator();
        shared_value
            .mul_div(&current_share, &denominator)
            .unwrap_or(SafeFloat::zero())
            .into()
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
        let shared_value: u64 = self.state_ops.get_shared_value();
        self.state_ops.set_shared_value(if update >= 0 {
            shared_value.saturating_add(update as u64)
        } else {
            shared_value.saturating_sub(update.neg() as u64)
        });
    }

    pub fn sim_update_value_for_one(&mut self, update: i64) -> bool {
        let shared_value: u64 = self.state_ops.get_shared_value();
        let denominator: SafeFloat = self.state_ops.get_denominator();

        // Then, update this key's share
        if denominator.mantissa == 0 {
            true
        } else {
            // There are already keys in the pool, set or update this key
            let shares_per_update = self.get_shares_per_update(update, shared_value, &denominator);

            !shares_per_update.is_zero()
        }
    }

    fn get_shares_per_update(
        &self,
        update: i64,
        shared_value: u64,
        denominator: &SafeFloat,
    ) -> SafeFloat {
        let shared_value: SafeFloat =
            SafeFloat::new(SafeInt::from(shared_value), 0).unwrap_or_default();
        let update: SafeFloat = SafeFloat::new(SafeInt::from(update), 0).unwrap_or_default();
        update
            .mul_div(denominator, &shared_value)
            .unwrap_or_default()
    }

    /// Update the value associated with an item identified by the Key
    /// Returns actual update
    ///
    pub fn update_value_for_one(&mut self, key: &K, update: i64) {
        let shared_value: u64 = self.state_ops.get_shared_value();
        let current_share: SafeFloat = self.state_ops.get_share(key);
        let denominator: SafeFloat = self.state_ops.get_denominator();

        // Then, update this key's share
        if denominator.is_zero() {
            // Initialize the pool. The first key gets all.
            let update_float: SafeFloat =
                SafeFloat::new(SafeInt::from(update), 0).unwrap_or_default();
            self.state_ops.set_denominator(update_float.clone());
            self.state_ops.set_share(key, update_float);
        } else {
            let shares_per_update: SafeFloat =
                self.get_shares_per_update(update, shared_value, &denominator);

            // Handle SafeFloat overflows quietly here because this overflow of i64 exponent
            // is extremely hypothetical and should never happen in practice.
            let new_denominator = match denominator.add(&shares_per_update) {
                Some(new_denominator) => new_denominator,
                None => {
                    log::error!(
                        "SafeFloat::add overflow when adding {:?} to {:?}; keeping old denominator",
                        shares_per_update,
                        denominator,
                    );
                    // Return the value as it was before the failed addition
                    denominator
                }
            };

            let new_current_share = match current_share.add(&shares_per_update) {
                Some(new_current_share) => new_current_share,
                None => {
                    log::error!(
                        "SafeFloat::add overflow when adding {:?} to {:?}; keeping old current_share",
                        shares_per_update,
                        current_share,
                    );
                    // Return the value as it was before the failed addition
                    current_share
                }
            };

            self.state_ops.set_denominator(new_denominator);
            self.state_ops.set_share(key, new_current_share);
        }

        // Update shared value
        self.update_value_for_all(update);
    }
}

// cargo test --package share-pool --lib -- tests --nocapture
#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;
    use lencode::io::Cursor;
    use lencode::{Decode, Encode};
    use std::collections::BTreeMap;
    use substrate_fixed::types::U64F64;

    struct MockSharePoolDataOperations {
        shared_value: u64,
        share: BTreeMap<u16, SafeFloat>,
        denominator: SafeFloat,
    }

    impl MockSharePoolDataOperations {
        fn new() -> Self {
            MockSharePoolDataOperations {
                shared_value: 0u64,
                share: BTreeMap::new(),
                denominator: SafeFloat::zero(),
            }
        }
    }

    impl SharePoolDataOperations<u16> for MockSharePoolDataOperations {
        fn get_shared_value(&self) -> u64 {
            self.shared_value
        }

        fn get_share(&self, key: &u16) -> SafeFloat {
            self.share.get(key).cloned().unwrap_or_else(SafeFloat::zero)
        }

        fn try_get_share(&self, key: &u16) -> Result<SafeFloat, ()> {
            match self.share.get(key).cloned() {
                Some(value) => Ok(value),
                None => Err(()),
            }
        }

        fn get_denominator(&self) -> SafeFloat {
            self.denominator.clone()
        }

        fn set_shared_value(&mut self, value: u64) {
            self.shared_value = value;
        }

        fn set_share(&mut self, key: &u16, share: SafeFloat) {
            self.share.insert(*key, share);
        }

        fn set_denominator(&mut self, update: SafeFloat) {
            self.denominator = update;
        }
    }

    #[test]
    fn test_get_value() {
        let mut mock_ops = MockSharePoolDataOperations::new();
        mock_ops.set_denominator(10u64.into());
        mock_ops.set_share(&1_u16, 3u64.into());
        mock_ops.set_share(&2_u16, 7u64.into());
        mock_ops.set_shared_value(100u64.into());
        let share_pool = SharePool::new(mock_ops);
        let result1 = share_pool.get_value(&1);
        let result2 = share_pool.get_value(&2);
        assert_eq!(result1, 30);
        assert_eq!(result2, 70);
    }

    #[test]
    fn test_division_by_zero() {
        let mut mock_ops = MockSharePoolDataOperations::new();
        mock_ops.set_denominator(SafeFloat::zero()); // Zero denominator
        let pool = SharePool::<u16, MockSharePoolDataOperations>::new(mock_ops);

        let value = pool.get_value(&1);
        assert_eq!(value, 0, "Value should be 0 when denominator is zero");
    }

    #[test]
    fn test_max_shared_value() {
        let mut mock_ops = MockSharePoolDataOperations::new();
        mock_ops.set_shared_value(u64::MAX.into());
        mock_ops.set_share(&1, 3u64.into()); // Use a neutral value for share
        mock_ops.set_share(&2, 7u64.into()); // Use a neutral value for share
        mock_ops.set_denominator(10u64.into()); // Neutral value to see max effect
        let pool = SharePool::<u16, MockSharePoolDataOperations>::new(mock_ops);

        let max_value = pool.get_value(&1) + pool.get_value(&2);
        assert!(u64::MAX - max_value <= 5, "Max value should map to u64 MAX");
    }

    #[test]
    fn test_max_share_value() {
        let mut mock_ops = MockSharePoolDataOperations::new();
        mock_ops.set_shared_value(1_000_000_000u64); // Use a neutral value for shared value
        mock_ops.set_share(&1, (u64::MAX / 2).into());
        mock_ops.set_share(&2, (u64::MAX / 2).into());
        mock_ops.set_denominator((u64::MAX).into());
        let pool = SharePool::<u16, MockSharePoolDataOperations>::new(mock_ops);

        let value1 = pool.get_value(&1) as i128;
        let value2 = pool.get_value(&2) as i128;

        assert_abs_diff_eq!(value1 as f64, 500_000_000 as f64, epsilon = 1.);
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

        // 50%/50% stakes consisting of 1 rao each
        pool.update_value_for_one(&1, 1);
        pool.update_value_for_one(&2, 1);

        // // Huge emission resulting in 1M Alpha
        // // Both stakers should have 500k Alpha each
        // pool.update_value_for_all(999_999_999_999_998);

        // // Everyone unstakes almost everything, leaving 10 rao in the stake
        // pool.update_value_for_one(&1, -499_999_999_999_990);
        // pool.update_value_for_one(&2, -499_999_999_999_990);

        // // Huge emission resulting in 1M Alpha
        // // Both stakers should have 500k Alpha each
        // pool.update_value_for_all(999_999_999_999_980);

        // // Stakers add 1k Alpha each
        // pool.update_value_for_one(&1, 1_000_000_000_000);
        // pool.update_value_for_one(&2, 1_000_000_000_000);

        // let value1 = pool.get_value(&1) as f64;
        // let value2 = pool.get_value(&2) as f64;
        // assert_abs_diff_eq!(value1, 501_000_000_000_000_f64, epsilon = 1.);
        // assert_abs_diff_eq!(value2, 501_000_000_000_000_f64, epsilon = 1.);
    }

    // cargo test --package share-pool --lib -- tests::test_denom_high_precision_many_small_unstakes --exact --show-output
    #[test]
    fn test_denom_high_precision_many_small_unstakes() {
        let mock_ops = MockSharePoolDataOperations::new();
        let mut pool = SharePool::<u16, MockSharePoolDataOperations>::new(mock_ops);

        // 50%/50% stakes consisting of 1 rao each
        pool.update_value_for_one(&1, 1);
        pool.update_value_for_one(&2, 1);

        // Huge emission resulting in 1M Alpha
        // Both stakers should have 500k Alpha + 1 rao each
        pool.update_value_for_all(1_000_000_000_000_000);

        // Run X number of small unstake transactions
        let tx_count = 1000;
        let unstake_amount = -500_000_000;
        for _ in 0..tx_count {
            pool.update_value_for_one(&1, unstake_amount);
            pool.update_value_for_one(&2, unstake_amount);
        }

        // Emit 1M - each gets 500k Alpha
        pool.update_value_for_all(1_000_000_000_000_000);

        // Each adds 1k Alpha
        pool.update_value_for_one(&1, 1_000_000_000_000);
        pool.update_value_for_one(&2, 1_000_000_000_000);

        // Result, each should get
        //   (500k+1) + tx_count * unstake_amount + 500k + 1k
        let value1 = pool.get_value(&1) as i128;
        let value2 = pool.get_value(&2) as i128;
        let expected = 1_001_000_000_000_000 + tx_count * unstake_amount;

        assert_abs_diff_eq!(value1 as f64, expected as f64, epsilon = 1.);
        assert_abs_diff_eq!(value2 as f64, expected as f64, epsilon = 1.);
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
        // Test case (update, shared_value, denominator_mantissa, denominator_exponent)
        [
            (1_i64, 1_u64, 1_u64, 0_i64),
            (1, 1_000_000_000_000_000_000, 1, 0),
            (1, 21_000_000_000_000_000, 1, 5),
            (1, 21_000_000_000_000_000, 1, -1_000_000),
            (1, 21_000_000_000_000_000, 1, -1_000_000_000),
            (1, 21_000_000_000_000_000, 1, -1_000_000_001),
            (1_000, 21_000_000_000_000_000, 1, 5),
            (21_000_000_000_000_000, 21_000_000_000_000_000, 1, 5),
            (21_000_000_000_000_000, 21_000_000_000_000_000, 1, -5),
            (21_000_000_000_000_000, 21_000_000_000_000_000, 1, -100),
            (21_000_000_000_000_000, 21_000_000_000_000_000, 1, 100),
            (210_000_000_000_000_000, 21_000_000_000_000_000, 1, 5),
            (1_000, 1_000, 21_000_000_000_000_000, 0),
            (1_000, 1_000, 21_000_000_000_000_000, -1),
        ]
        .into_iter()
        .for_each(
            |(update, shared_value, denominator_mantissa, denominator_exponent)| {
                let mock_ops = MockSharePoolDataOperations::new();
                let pool = SharePool::<u16, MockSharePoolDataOperations>::new(mock_ops);

                let denominator_float =
                    SafeFloat::new(SafeInt::from(denominator_mantissa), denominator_exponent)
                        .unwrap();
                let denominator_f64: f64 = denominator_float.clone().into();
                let spu: f64 = pool
                    .get_shares_per_update(update, shared_value, &denominator_float)
                    .into();
                let expected = update as f64 * denominator_f64 / shared_value as f64;
                let precision = 1000.;
                assert_abs_diff_eq!(expected, spu, epsilon = expected / precision);
            },
        );
    }

    #[test]
    fn test_safeint_serialization() {
        let safe_int = SafeInt::from(12345);
        let mut buf = Vec::new();
        safe_int.encode(&mut buf).unwrap();

        let decoded = SafeInt::decode(&mut Cursor::new(&buf)).unwrap();
        assert_eq!(decoded, safe_int);
    }

    #[test]
    fn test_safefloat_normalize() {
        // Test case: mantissa, exponent, expected mantissa, expected exponent
        [
            (1_u128, 0, 100_000_000_000_000_000_000_u128, -20_i64),
            (0, 0, 0, 0),
            (10_u128, 0, 100_000_000_000_000_000_000_u128, -19),
            (1_000_u128, 0, 100_000_000_000_000_000_000_u128, -17),
            (
                100_000_000_000_000_000_000_u128,
                0,
                1_000_000_000_000_000_000_000_u128,
                -1,
            ),
            (SAFE_FLOAT_MAX, 0, SAFE_FLOAT_MAX, 0),
        ]
        .into_iter()
        .for_each(|(m, e, expected_m, expected_e)| {
            let a = SafeFloat::new(SafeInt::from(m), e).unwrap();
            assert_eq!(a.mantissa, SafeInt::from(expected_m));
            assert_eq!(a.exponent, SafeInt::from(expected_e));
        });
    }

    #[test]
    fn test_safefloat_add() {
        // Test case: man_a, exp_a, man_b, exp_b, expected mantissa of a+b, expected exponent of a+b
        [
            // 1 + 1 = 2
            (
                1_u128,
                0,
                1_u128,
                0,
                200_000_000_000_000_000_000_u128,
                -20_i64,
            ),
            // SAFE_FLOAT_MAX + SAFE_FLOAT_MAX
            (
                SAFE_FLOAT_MAX,
                0,
                SAFE_FLOAT_MAX,
                0,
                SAFE_FLOAT_MAX * 2 / 10,
                1_i64,
            ),
            // Expected loss of precision: tiny + huge
            (
                1_u128,
                0,
                1_000_000_000_000_000_000_000_u128,
                1,
                1_000_000_000_000_000_000_000_u128,
                1_i64,
            ),
            (
                1_u128,
                0,
                1_u128,
                22,
                1_000_000_000_000_000_000_000_u128,
                1_i64,
            ),
            (
                1_u128,
                0,
                1_u128,
                23,
                1_000_000_000_000_000_000_000_u128,
                2_i64,
            ),
            (
                123_u128,
                0,
                1_u128,
                23,
                1_000_000_000_000_000_000_001_u128,
                2_i64,
            ),
            (
                123_u128,
                1,
                1_u128,
                23,
                1_000_000_000_000_000_000_012_u128,
                2_i64,
            ),
            // --- New tests start here ---

            // Small-ish + very large (10^22 + 42)
            // 42 * 10^0 + 1 * 10^22 ≈ 1e22 + 42
            // Normalized ≈ (1e21 + 4) * 10^1
            (
                42_u128,
                0,
                1_u128,
                22,
                1_000_000_000_000_000_000_004_u128,
                1_i64,
            ),
            // "Almost 10^21" + 10^22
            // (10^21 - 1) + 10^22 → floor((10^22 + 10^21 - 1) / 100) * 10^2
            (
                999_999_999_999_999_999_999_u128,
                0,
                1_u128,
                22,
                109_999_999_999_999_999_999_u128,
                2_i64,
            ),
            // Small-ish + 10^23 where the small part is completely lost
            // 42 + 10^23 -> floor((10^23 + 42)/100) * 10^2 ≈ 1e21 * 10^2
            (
                42_u128,
                0,
                1_u128,
                23,
                1_000_000_000_000_000_000_000_u128,
                2_i64,
            ),
            // Small-ish + 10^23 where tiny part slightly affects mantissa
            // 4200 + 10^23 -> floor((10^23 + 4200)/100) * 10^2 = (1e21 + 42) * 10^2
            (
                4_200_u128,
                0,
                1_u128,
                23,
                1_000_000_000_000_000_000_042_u128,
                2_i64,
            ),
            // (10^21 - 1) + 10^23
            // -> floor((10^23 + 10^21 - 1)/100) = 1e21 + 1e19 - 1
            (
                999_999_999_999_999_999_999_u128,
                0,
                1_u128,
                23,
                1_009_999_999_999_999_999_999_u128,
                2_i64,
            ),
            // Medium + 10^23 with exponent 1 on the smaller term
            // 999_999 * 10^1 + 1 * 10^23 -> (10^22 + 999_999) * 10^1
            // Normalized ≈ (1e21 + 99_999) * 10^2
            (
                999_999_u128,
                1,
                1_u128,
                23,
                1_000_000_000_000_000_099_999_u128,
                2_i64,
            ),
            // Check behaviour with exponent 24, tiny second term
            // 1 * 10^24 + 1 -> floor((10^24 + 1)/1000) * 10^3 ≈ 1e21 * 10^3
            (
                1_u128,
                24,
                1_u128,
                0,
                1_000_000_000_000_000_000_000_u128,
                3_i64,
            ),
            // 1 * 10^24 + a non-trivial small mantissa
            // 1e24 + 123456789012345678901 -> floor(/1000) = 1e21 + 123456789012345678
            (
                1_u128,
                24,
                123_456_789_012_345_678_901_u128,
                0,
                1_000_123_456_789_012_345_678_u128,
                3_i64,
            ),
            // 10^22 and 10^23 combined:
            // 1 * 10^22 + 1 * 10^23 = 11 * 10^22 = (1.1 * 10^23)
            // Normalized → (1.1e20) * 10^3
            (
                1_u128,
                22,
                1_u128,
                23,
                110_000_000_000_000_000_000_u128,
                3_i64,
            ),
            // Both operands already aligned at a huge scale:
            // (10^21 - 1) * 10^22 + 1 * 10^22 = 10^21 * 10^22 = 10^43
            // Canonical form: (1e21) * 10^22
            (
                999_999_999_999_999_999_999_u128,
                22,
                1_u128,
                22,
                1_000_000_000_000_000_000_000_u128,
                22_i64,
            ),
        ]
        .into_iter()
        .for_each(|(m_a, e_a, m_b, e_b, expected_m, expected_e)| {
            let a = SafeFloat::new(SafeInt::from(m_a), e_a).unwrap();
            let b = SafeFloat::new(SafeInt::from(m_b), e_b).unwrap();

            let a_plus_b = a.add(&b).unwrap();
            let b_plus_a = b.add(&a).unwrap();

            assert_eq!(a_plus_b.mantissa, SafeInt::from(expected_m));
            assert_eq!(a_plus_b.exponent, SafeInt::from(expected_e));
            assert_eq!(b_plus_a.mantissa, SafeInt::from(expected_m));
            assert_eq!(b_plus_a.exponent, SafeInt::from(expected_e));
        });
    }

    #[test]
    fn test_safefloat_div_by_zero_is_none() {
        let a = SafeFloat::new(SafeInt::from(1), 0).unwrap();
        assert!(a.div(&SafeFloat::zero()).is_none());
    }

    #[test]
    fn test_safefloat_div() {
        // Test case: man_a, exp_a, man_b, exp_b
        [
            (1_u128, 0_i64, 100_000_000_000_000_000_000_u128, -20_i64),
            (1_u128, 0, 1_u128, 0),
            (1_u128, 1, 1_u128, 0),
            (1_u128, 7, 1_u128, 0),
            (1_u128, 50, 1_u128, 0),
            (1_u128, 100, 1_u128, 0),
            (1_u128, 0, 7_u128, 0),
            (1_u128, 1, 7_u128, 0),
            (1_u128, 7, 7_u128, 0),
            (1_u128, 50, 7_u128, 0),
            (1_u128, 100, 7_u128, 0),
            (1_u128, 0, 3_u128, 0),
            (1_u128, 1, 3_u128, 0),
            (1_u128, 7, 3_u128, 0),
            (1_u128, 50, 3_u128, 0),
            (1_u128, 100, 3_u128, 0),
            (2_u128, 0, 3_u128, 0),
            (2_u128, 1, 3_u128, 0),
            (2_u128, 7, 3_u128, 0),
            (2_u128, 50, 3_u128, 0),
            (2_u128, 100, 3_u128, 0),
            (5_u128, 0, 3_u128, 0),
            (5_u128, 1, 3_u128, 0),
            (5_u128, 7, 3_u128, 0),
            (5_u128, 50, 3_u128, 0),
            (5_u128, 100, 3_u128, 0),
            (10_u128, 0, 100_000_000_000_000_000_000_u128, -19),
            (1_000_u128, 0, 100_000_000_000_000_000_000_u128, -17),
            (
                100_000_000_000_000_000_000_u128,
                0,
                1_000_000_000_000_000_000_000_u128,
                -1,
            ),
            (SAFE_FLOAT_MAX, 0, SAFE_FLOAT_MAX, 0),
            (SAFE_FLOAT_MAX, 100, SAFE_FLOAT_MAX, -100),
            (SAFE_FLOAT_MAX, 100, SAFE_FLOAT_MAX - 1, -100),
            (SAFE_FLOAT_MAX - 1, 100, SAFE_FLOAT_MAX, -100),
            (SAFE_FLOAT_MAX - 2, 100, SAFE_FLOAT_MAX, -100),
            (SAFE_FLOAT_MAX, 100, SAFE_FLOAT_MAX / 2 - 1, -100),
            (SAFE_FLOAT_MAX, 100, SAFE_FLOAT_MAX / 2 - 1, 100),
            (1_u128, 0, 100_000_000_000_000_000_000_u128, -20_i64),
            (
                123_456_789_123_456_789_123_u128,
                20_i64,
                87_654_321_987_654_321_987_u128,
                -20_i64,
            ),
            (
                123_456_789_123_456_789_123_u128,
                100_i64,
                87_654_321_987_654_321_987_u128,
                -100_i64,
            ),
            (
                123_456_789_123_456_789_123_u128,
                -100_i64,
                87_654_321_987_654_321_987_u128,
                100_i64,
            ),
            (
                123_456_789_123_456_789_123_u128,
                -99_i64,
                87_654_321_987_654_321_987_u128,
                99_i64,
            ),
            (
                123_456_789_123_456_789_123_u128,
                123_i64,
                87_654_321_987_654_321_987_u128,
                -32_i64,
            ),
            (
                123_456_789_123_456_789_123_u128,
                -123_i64,
                87_654_321_987_654_321_987_u128,
                32_i64,
            ),
        ]
        .into_iter()
        .for_each(|(ma, ea, mb, eb)| {
            let a = SafeFloat::new(SafeInt::from(ma), ea).unwrap();
            let b = SafeFloat::new(SafeInt::from(mb), eb).unwrap();

            let actual: f64 = a.div(&b).unwrap().into();
            let expected =
                ma as f64 * (10_f64).powi(ea as i32) / (mb as f64 * (10_f64).powi(eb as i32));

            assert_abs_diff_eq!(actual, expected, epsilon = actual / 100_000_000_000_000_f64);
        });
    }

    #[test]
    fn test_safefloat_mul_div() {
        // result = a * b / c
        // should not lose precision gained in a * b
        // Test case: man_a, exp_a, man_b, exp_b, man_c, exp_c
        [
            (1_u128, -20_i64, 1_u128, -20_i64, 1_u128, -20_i64),
            (123_u128, 20_i64, 123_u128, -20_i64, 321_u128, 0_i64),
            (
                123_123_123_123_123_123_u128,
                20_i64,
                321_321_321_321_321_321_u128,
                -20_i64,
                777_777_777_777_777_777_u128,
                0_i64,
            ),
            (
                11_111_111_111_111_111_111_u128,
                20_i64,
                99_321_321_321_321_321_321_u128,
                -20_i64,
                77_777_777_777_777_777_777_u128,
                0_i64,
            ),
        ]
        .into_iter()
        .for_each(|(ma, ea, mb, eb, mc, ec)| {
            let a = SafeFloat::new(SafeInt::from(ma), ea).unwrap();
            let b = SafeFloat::new(SafeInt::from(mb), eb).unwrap();
            let c = SafeFloat::new(SafeInt::from(mc), ec).unwrap();

            let actual: f64 = a.mul_div(&b, &c).unwrap().into();
            let expected = (ma as f64 * (10_f64).powi(ea as i32))
                * (mb as f64 * (10_f64).powi(eb as i32))
                / (mc as f64 * (10_f64).powi(ec as i32));

            assert_abs_diff_eq!(actual, expected, epsilon = actual / 100_000_000_000_000_f64);
        });
    }

    #[test]
    fn test_safefloat_from_u64f64() {
        [
            U64F64::from_num(1000.0),
            U64F64::from_num(10.0),
            U64F64::from_num(1.0),
            U64F64::from_num(0.1),
            U64F64::from_num(0.00000001),
            U64F64::from_num(123_456_789_123_456u128),
            // Exact zero
            U64F64::from_num(0.0),
            // Very small positive value (well above Q64.64 resolution)
            U64F64::from_num(1e-18),
            // Value just below 1
            U64F64::from_num(0.999_999_999_999_999_f64),
            // Value just above 1
            U64F64::from_num(1.000_000_000_000_001_f64),
            // "Random-looking" fractional with many digits
            U64F64::from_num(1.234_567_890_123_45_f64),
            // Large integer, but smaller than the max integer part of U64F64
            U64F64::from_num(999_999_999_999_999_999u128),
            // Very large integer near the upper bound of integer range
            U64F64::from_num(u64::MAX as u128),
            // Large number with fractional part
            U64F64::from_num(123_456_789_123_456.789_f64),
            // Medium-large with tiny fractional part to test precision on tail digits
            U64F64::from_num(1_000_000_000_000.000_001_f64),
            // Smallish with long fractional part
            U64F64::from_num(0.123_456_789_012_345_f64),
        ]
        .into_iter()
        .for_each(|f| {
            let safe_float: SafeFloat = f.into();
            let actual: f64 = safe_float.into();
            let expected = f.to_num::<f64>();

            // Relative epsilon ~1e-14 of the magnitude
            let epsilon = if actual == 0.0 {
                0.0
            } else {
                actual.abs() / 100_000_000_000_000_f64
            };

            assert_abs_diff_eq!(actual, expected, epsilon = epsilon);
        });
    }
}
