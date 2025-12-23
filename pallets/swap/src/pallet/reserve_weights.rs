use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::pallet_prelude::*;
use safe_bigmath::*;
use safe_math::*;
use sp_arithmetic::Perquintill;
use sp_core::U256;
use sp_runtime::Saturating;
use substrate_fixed::types::U64F64;
use subtensor_macros::freeze_struct;

#[freeze_struct("8c6bbe52ef752203")]
#[derive(Clone, Encode, Decode, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct ReserveWeight {
    quote: Perquintill,
}

// Lower imit of weights is 0.01
pub const ACCURACY: u64 = 1_000_000_000_000_000_000_u64;
pub const MIN_WEIGHT: Perquintill = Perquintill::from_parts(ACCURACY / 100);
pub const ONE: Perquintill = Perquintill::from_parts(ACCURACY);

#[derive(Debug)]
pub enum ReserveWeightError {
    InvalidValue,
}

impl Default for ReserveWeight {
    fn default() -> Self {
        Self {
            quote: Perquintill::from_rational(1u128, 2u128),
        }
    }
}

impl ReserveWeight {
    pub fn new(quote: Perquintill) -> Result<Self, ReserveWeightError> {
        if Self::check_constraints(quote) {
            Ok(ReserveWeight { quote })
        } else {
            Err(ReserveWeightError::InvalidValue)
        }
    }

    fn check_constraints(quote: Perquintill) -> bool {
        let base = ONE.saturating_sub(quote);
        (base >= MIN_WEIGHT) && (quote >= MIN_WEIGHT)
    }

    pub fn get_quote_weight(&self) -> Perquintill {
        self.quote
    }

    pub fn get_base_weight(&self) -> Perquintill {
        ONE.saturating_sub(self.quote)
    }

    pub fn set_quote_weight(&self, new_value: Perquintill) -> Result<(), ReserveWeightError> {
        if Self::check_constraints(new_value) {
            Ok(())
        } else {
            Err(ReserveWeightError::InvalidValue)
        }
    }

    fn exp_scaled(&self, x: u64, dx: u64, base_quote: bool) -> U64F64 {
        let den = x.saturating_add(dx);
        if den == 0 {
            return U64F64::saturating_from_num(0);
        }
        let w1: u128 = self.get_base_weight().deconstruct() as u128;
        let w2: u128 = self.get_quote_weight().deconstruct() as u128;
        let x_plus_dx = x.saturating_add(dx);

        let precision = 1024;
        let x_safe = SafeInt::from(x);
        let w1_safe = SafeInt::from(w1);
        let w2_safe = SafeInt::from(w2);
        let perquintill_scale = SafeInt::from(ACCURACY as u128);
        let denominator = SafeInt::from(x_plus_dx);
        let maybe_result_safe_int = if base_quote {

            println!("x = {:?}", x);
            println!("dx = {:?}", dx);
            println!("x_safe = {:?}", x_safe);
            println!("denominator = {:?}", denominator);
            println!("w1_safe = {:?}", w1_safe);
            println!("w2_safe = {:?}", w2_safe);
            println!("precision = {:?}", precision);
            println!("perquintill_scale = {:?}", perquintill_scale);
            
            SafeInt::pow_ratio_scaled(
                &x_safe,
                &denominator,
                &w1_safe,
                &w2_safe,
                precision,
                &perquintill_scale,
            )
        } else {
            SafeInt::pow_ratio_scaled(
                &x_safe,
                &denominator,
                &w2_safe,
                &w1_safe,
                precision,
                &perquintill_scale,
            )
        };

        if let Some(result_safe_int) = maybe_result_safe_int {
            if let Some(result_u64) = result_safe_int.to_u64() {
                return U64F64::saturating_from_num(result_u64)
                    .safe_div(U64F64::saturating_from_num(ACCURACY));
            }
        }
        return U64F64::saturating_from_num(0);
    }

    /// Calculates exponent of (x / (x + ∆x)) ^ (w_base/w_quote)
    pub fn exp_base_quote(&self, x: u64, dx: u64) -> U64F64 {
        self.exp_scaled(x, dx, true)
    }

    /// Calculates exponent of (y / (y + ∆y)) ^ (w_quote/w_base)
    pub fn exp_quote_base(&self, y: u64, dy: u64) -> U64F64 {
        self.exp_scaled(y, dy, false)
    }

    /// Calculates price as (w1/w2) * (y/x)
    pub fn calculate_price(&self, x: u64, y: u64) -> U64F64 {
        let w2_fixed = U64F64::saturating_from_num(self.get_quote_weight().deconstruct());
        let w1_fixed = U64F64::saturating_from_num(self.get_base_weight().deconstruct());
        let x_fixed = U64F64::saturating_from_num(x);
        let y_fixed = U64F64::saturating_from_num(y);
        w1_fixed
            .safe_div(w2_fixed)
            .saturating_mul(y_fixed.safe_div(x_fixed))
    }

    /// Multiply a u128 value by a Perquintill with u128 result rounded to the
    /// nearest integer
    fn mul_perquintill_round(p: Perquintill, value: u128) -> u128 {
        let parts = p.deconstruct() as u128;
        let acc = ACCURACY as u128;

        let num = U256::from(value) * U256::from(parts);
        let den = U256::from(acc);

        // add 0.5 ulp before integer division → round-to-nearest
        let res = (num + den / U256::from(2u8)) / den;
        res.min(U256::from(u128::MAX)).as_u128()
    }

    pub fn update_weights_for_added_liquidity(
        &mut self,
        tao_reserve: u64,
        alpha_reserve: u64,
        tao_delta: u64,
        alpha_delta: u64,
    ) -> Result<(), ReserveWeightError> {
        // Calculate new to-be reserves (do not update here)
        let tao_reserve_u128 = u64::from(tao_reserve) as u128;
        let alpha_reserve_u128 = u64::from(alpha_reserve) as u128;
        let tao_delta_u128 = u64::from(tao_delta) as u128;
        let alpha_delta_u128 = u64::from(alpha_delta) as u128;
        let new_tao_reserve_u128 = tao_reserve_u128.saturating_add(tao_delta_u128);
        let new_alpha_reserve_u128 = alpha_reserve_u128.saturating_add(alpha_delta_u128);

        // Calculate new weights
        let quantity_1: u128 = Self::mul_perquintill_round(
            self.get_base_weight(),
            tao_reserve_u128.saturating_mul(new_alpha_reserve_u128),
        );
        let quantity_2: u128 = Self::mul_perquintill_round(
            self.get_quote_weight(),
            alpha_reserve_u128.saturating_mul(new_tao_reserve_u128),
        );
        let q_sum = quantity_1.saturating_add(quantity_2);

        // Calculate new reserve weights
        let new_reserve_weight = if q_sum != 0 {
            // Both TAO and Alpha are non-zero, normal case
            Perquintill::from_rational(quantity_2, q_sum)
        } else {
            // Either TAO or Alpha reserve were and/or remain zero => Initialize weights to 0.5
            Perquintill::from_rational(1u128, 2u128)
        };

        self.set_quote_weight(new_reserve_weight)
    }

    /// Calculates quote delta needed to reach the price up when byuing
    pub fn calculate_quote_delta_in(
        &self,
        current_price: U64F64,
        target_price: U64F64,
        reserve: u64,
    ) -> u64 {
        let base_numerator: u128 = target_price.to_bits();
        let base_denominator: u128 = current_price.to_bits();
        let w1_fixed: u128 = self.get_base_weight().deconstruct() as u128;
        let scale: u128 = 10u128.pow(18);

        let maybe_exp_result = SafeInt::pow_ratio_scaled(
            &SafeInt::from(base_numerator),
            &SafeInt::from(base_denominator),
            // &SafeInt::from(3u128),
            // &SafeInt::from(4u128),
            &SafeInt::from(w1_fixed),
            &SafeInt::from(ACCURACY),
            160,
            &SafeInt::from(scale),
        );

        if let Some(exp_result_safe_int) = maybe_exp_result {
            if let Some(exp_result_u64) = exp_result_safe_int.to_u64() {
                let reserve_fixed = U64F64::saturating_from_num(reserve);
                let exp_result_fixed = U64F64::saturating_from_num(exp_result_u64);
                let one = U64F64::saturating_from_num(1);
                let scale_fixed = U64F64::saturating_from_num(scale);
                return reserve_fixed
                    .saturating_mul(exp_result_fixed.safe_div(scale_fixed).saturating_sub(one))
                    .saturating_to_num::<u64>();
            }
        }
        return 0u64;
    }

    /// Calculates base delta needed to reach the price down when selling
    pub fn calculate_base_delta_in(
        &self,
        current_price: U64F64,
        target_price: U64F64,
        reserve: u64,
    ) -> u64 {
        let base_numerator: u128 = current_price.to_bits();
        let base_denominator: u128 = target_price.to_bits();
        let w2_fixed: u128 = self.get_quote_weight().deconstruct() as u128;
        let scale: u128 = 10u128.pow(18);

        let maybe_exp_result = SafeInt::pow_ratio_scaled(
            &SafeInt::from(base_numerator),
            &SafeInt::from(base_denominator),
            &SafeInt::from(w2_fixed),
            &SafeInt::from(ACCURACY),
            160,
            &SafeInt::from(scale),
        );

        if let Some(exp_result_safe_int) = maybe_exp_result {
            if let Some(exp_result_u64) = exp_result_safe_int.to_u64() {
                let reserve_fixed = U64F64::saturating_from_num(reserve);
                let exp_result_fixed = U64F64::saturating_from_num(exp_result_u64);
                let one = U64F64::saturating_from_num(1);
                let scale_fixed = U64F64::saturating_from_num(scale);
                return reserve_fixed
                    .saturating_mul(exp_result_fixed.safe_div(scale_fixed).saturating_sub(one))
                    .saturating_to_num::<u64>();
            }
        }
        return 0u64;
    }
}

// cargo test --package pallet-subtensor-swap --lib -- pallet::reserve_weights::tests --nocapture
#[cfg(test)]
#[cfg(feature = "std")]
mod tests {
    use crate::pallet::ReserveWeight;
    use crate::pallet::reserve_weights::*;
    use approx::assert_abs_diff_eq;
    use sp_arithmetic::Perquintill;

    fn perquintill_to_f64(p: Perquintill) -> f64 {
        let parts = p.deconstruct() as f64;
        parts / ACCURACY as f64
    }

    #[test]
    fn test_perquintill_power() {
        const PRECISION: u32 = 4096;
        const PERQUINTILL: u128 = ACCURACY as u128;

        let x = SafeInt::from(21_000_000_000_000_000u64);
        let delta = SafeInt::from(7_000_000_000_000_000u64);
        let w1 = SafeInt::from(600_000_000_000_000_000u128);
        let w2 = SafeInt::from(400_000_000_000_000_000u128);
        let denominator = &x + &delta;
        assert_eq!(w1.clone() + w2.clone(), SafeInt::from(PERQUINTILL));

        let perquintill_result = SafeInt::pow_ratio_scaled(
            &x,
            &denominator,
            &w1,
            &w2,
            PRECISION,
            &SafeInt::from(PERQUINTILL),
        )
        .expect("perquintill integer result");

        assert_eq!(
            perquintill_result,
            SafeInt::from(649_519_052_838_328_985u128)
        );
        let readable = safe_bigmath::SafeDec::<18>::from_raw(perquintill_result);
        assert_eq!(format!("{}", readable), "0.649519052838328985");
    }

    /// Validate realistic values that can be calculated with f64 precision
    #[test]
    fn test_exp_base_quote_happy_path() {
        // Outer test cases: w_quote
        [
            // Perquintill::from_rational(500_000_000_000_u128, 1_000_000_000_000_u128),
            // Perquintill::from_rational(500_000_000_001_u128, 1_000_000_000_000_u128),
            // Perquintill::from_rational(499_999_999_999_u128, 1_000_000_000_000_u128),
            // Perquintill::from_rational(500_000_000_100_u128, 1_000_000_000_000_u128),
            // Perquintill::from_rational(500_000_001_000_u128, 1_000_000_000_000_u128),
            // Perquintill::from_rational(500_000_010_000_u128, 1_000_000_000_000_u128),
            // Perquintill::from_rational(500_000_100_000_u128, 1_000_000_000_000_u128),
            // Perquintill::from_rational(500_001_000_000_u128, 1_000_000_000_000_u128),
            // Perquintill::from_rational(500_010_000_000_u128, 1_000_000_000_000_u128),
            // Perquintill::from_rational(500_100_000_000_u128, 1_000_000_000_000_u128),
            // Perquintill::from_rational(501_000_000_000_u128, 1_000_000_000_000_u128),
            // Perquintill::from_rational(510_000_000_000_u128, 1_000_000_000_000_u128),
            Perquintill::from_rational(100_000_000_000_u128, 1_000_000_000_000_u128),
            // Perquintill::from_rational(100_000_000_001_u128, 1_000_000_000_000_u128),
            // Perquintill::from_rational(200_000_000_000_u128, 1_000_000_000_000_u128),
            // Perquintill::from_rational(300_000_000_000_u128, 1_000_000_000_000_u128),
            // Perquintill::from_rational(400_000_000_000_u128, 1_000_000_000_000_u128),
            // Perquintill::from_rational(600_000_000_000_u128, 1_000_000_000_000_u128),
            // Perquintill::from_rational(700_000_000_000_u128, 1_000_000_000_000_u128),
            // Perquintill::from_rational(800_000_000_000_u128, 1_000_000_000_000_u128),
            // Perquintill::from_rational(899_999_999_999_u128, 1_000_000_000_000_u128),
            // Perquintill::from_rational(900_000_000_000_u128, 1_000_000_000_000_u128),
            // Perquintill::from_rational(
            //     102_337_248_363_782_924_u128,
            //     1_000_000_000_000_000_000_u128,
            // ),
        ]
        .into_iter()
        .for_each(|w_quote| {
            // Inner test cases: y, x, ∆x
            [
                // (1_000_u64, 1_000_u64, 0_u64),
                // (1_000_u64, 1_000_u64, 1_u64),
                (1_500_u64, 1_000_u64, 1_u64),
                // (
                //     1_000_000_000_000_u64,
                //     100_000_000_000_000_u64,
                //     100_000_000_u64,
                // ),
                // (
                //     1_000_000_000_000_u64,
                //     100_000_000_000_000_u64,
                //     100_000_000_u64,
                // ),
                // (
                //     100_000_000_000_u64,
                //     100_000_000_000_000_u64,
                //     100_000_000_u64,
                // ),
                // (100_000_000_000_u64, 100_000_000_000_000_u64, 1_000_000_u64),
                // (
                //     100_000_000_000_u64,
                //     100_000_000_000_000_u64,
                //     1_000_000_000_000_u64,
                // ),
                // (
                //     1_000_000_000_u64,
                //     100_000_000_000_000_u64,
                //     1_000_000_000_000_u64,
                // ),
                // (
                //     1_000_000_u64,
                //     100_000_000_000_000_u64,
                //     1_000_000_000_000_u64,
                // ),
                // (1_000_u64, 100_000_000_000_000_u64, 1_000_000_000_000_u64),
                // (1_000_u64, 100_000_000_000_000_u64, 1_000_000_000_u64),
                // (1_000_u64, 100_000_000_000_000_u64, 1_000_000_u64),
                // (1_000_u64, 100_000_000_000_000_u64, 1_000_u64),
                // (1_000_u64, 100_000_000_000_000_u64, 100_000_000_000_000_u64),
                // (10_u64, 100_000_000_000_000_u64, 100_000_000_000_000_u64),
                // // Extreme values of ∆x for small x
                // (1_000_000_000_u64, 4_000_000_000_u64, 1_000_000_000_000_u64),
                // (1_000_000_000_000_u64, 1_000_u64, 1_000_000_000_000_u64),
                // (
                //     5_628_038_062_729_553_u64,
                //     400_775_553_u64,
                //     14_446_633_907_665_582_u64,
                // ),
                // (
                //     5_600_000_000_000_000_u64,
                //     400_000_000_u64,
                //     14_000_000_000_000_000_u64,
                // ),
            ]
            .into_iter()
            .for_each(|(y, x, dx)| {
                let rw = ReserveWeight::new(w_quote).unwrap();
                let e = rw.exp_base_quote(x, dx);
                let one = U64F64::from_num(1);
                let y_fixed = U64F64::from_num(y);
                println!("debug 1: e = {:?}", e);
                let dy = y_fixed * (one - e);
                println!("debug 2: dy = {:?}", dy);

                let w1 = perquintill_to_f64(rw.get_base_weight());
                let w2 = perquintill_to_f64(rw.get_quote_weight());
                let e_expected = (x as f64 / (x as f64 + dx as f64)).powf(w1 / w2);
                let dy_expected = y as f64 * (1. - e_expected);
                
                println!("debug 3: dy_expected = {:?}", dy_expected);

                let mut eps = dy_expected / 100000.;
                if eps > 1.0 {
                    eps = 1.0;
                }
                assert_abs_diff_eq!(dy.to_num::<f64>(), dy_expected, epsilon = eps);
            })
        });
    }

    /// This test exercises practical application edge cases of exp_base_quote
    /// The practical formula where this function is used:
    ///    ∆y = y * (exp_base_quote(x, ∆x) - 1)
    ///
    /// The test validates that two different sets of parameters produce (sensibly)
    /// different results
    ///
    #[test]
    fn test_exp_base_quote_dy_precision() {
        // Test cases: y, x1, ∆x1, w_quote1, x2, ∆x2, w_quote2
        // Realized dy1 should be greater than dy2
        [
            (
                1_000_000_000_u64,
                21_000_000_000_000_000_u64,
                21_000_000_000_u64,
                Perquintill::from_rational(1_000_000_000_000_u128, 2_000_000_000_000_u128),
                21_000_000_000_000_000_u64,
                21_000_000_000_u64,
                Perquintill::from_rational(1_000_000_000_001_u128, 2_000_000_000_000_u128),
            ),
            (
                1_000_000_000_u64,
                21_000_000_000_000_000_u64,
                21_000_000_000_u64,
                Perquintill::from_rational(1_000_000_000_000_u128, 2_000_000_000_001_u128),
                21_000_000_000_000_000_u64,
                21_000_000_000_u64,
                Perquintill::from_rational(1_000_000_000_000_u128, 2_000_000_000_000_u128),
            ),
            (
                1_000_000_000_u64,
                21_000_000_000_000_000_u64,
                2_u64,
                Perquintill::from_rational(1_000_000_000_000_u128, 2_000_000_000_000_u128),
                21_000_000_000_000_000_u64,
                1_u64,
                Perquintill::from_rational(1_000_000_000_000_u128, 2_000_000_000_000_u128),
            ),
            (
                1_000_000_000_u64,
                21_000_000_000_000_000_u64,
                1_u64,
                Perquintill::from_rational(1_000_000_000_000_u128, 2_000_000_000_000_u128),
                21_000_000_000_000_000_u64,
                1_u64,
                Perquintill::from_rational(1_010_000_000_000_u128, 2_000_000_000_000_u128),
            ),
            (
                1_000_000_000_u64,
                21_000_000_000_000_000_u64,
                1_u64,
                Perquintill::from_rational(1_000_000_000_000_u128, 2_010_000_000_000_u128),
                21_000_000_000_000_000_u64,
                1_u64,
                Perquintill::from_rational(1_000_000_000_000_u128, 2_000_000_000_000_u128),
            ),
        ]
        .into_iter()
        .for_each(|(y, x1, dx1, w_quote1, x2, dx2, w_quote2)| {
            let rw1 = ReserveWeight::new(w_quote1).unwrap();
            let rw2 = ReserveWeight::new(w_quote2).unwrap();

            let exp1 = rw1.exp_base_quote(x1, dx1);
            let exp2 = rw2.exp_base_quote(x2, dx2);

            let one = U64F64::from_num(1);
            let y_fixed = U64F64::from_num(y);
            let dy1 = y_fixed * (one - exp1);
            let dy2 = y_fixed * (one - exp2);

            assert!(dy1 > dy2);

            let zero = U64F64::from_num(0);
            assert!(dy1 != zero);
            assert!(dy2 != zero);
        })
    }

    /// Test the broad range of w_quote values, usually should be ignored
    #[ignore]
    #[test]
    fn test_exp_quote_broad_range() {
        let y = 1_000_000_000_000_u64;
        let x = 100_000_000_000_000_u64;
        let dx = 10_000_000_u64;

        let mut prev = U64F64::from_num(1_000_000_000);
        let mut last_progress = 0.;
        let start = 100_000_000_000_u128;
        let stop = 900_000_000_000_u128;
        for num in (start..=stop).step_by(1000 as usize) {
            let w_quote = Perquintill::from_rational(num, 1_000_000_000_000_u128);
            let rw = ReserveWeight::new(w_quote).unwrap();
            let e = rw.exp_base_quote(x, dx);

            let one = U64F64::from_num(1);
            // println!("e = {:?}", e);
            // println!("1 - e = {:?}", one - e);

            let dy = U64F64::from_num(y) * (one - e);
            // println!("dy = {:?}", dy);

            let progress = (num as f64 - start as f64) / (stop as f64 - start as f64);

            if progress - last_progress >= 0.0001 {
                println!("progress = {:?}%", progress * 100.);
                println!("dy = {:?}", dy);
                last_progress = progress;
            }

            assert!(dy != U64F64::from_num(0));
            assert!(dy <= prev);
            prev = dy;
        }
    }

    #[ignore]
    #[test]
    fn test_exp_quote_fuzzy() {
        use rand::rngs::StdRng;
        use rand::{Rng, SeedableRng};
        use rayon::prelude::*;
        use std::sync::Arc;
        use std::sync::atomic::{AtomicUsize, Ordering};

        const ITERATIONS: usize = 1_000_000_000;
        let counter = Arc::new(AtomicUsize::new(0));

        (0..ITERATIONS)
        .into_par_iter()
        .for_each(|i| {
            // Each iteration gets its own deterministic RNG.
            // Seed depends on i, so runs are reproducible.
            let mut rng = StdRng::seed_from_u64(42 + i as u64);
            let max_supply: u64 = 21_000_000_000_000_000;
            let full_range = true;

            let x: u64 = rng.gen_range(1_000..=max_supply); // Alpha reserve
            let y: u64 = if full_range {
                // TAO reserve (allow huge prices)
                rng.gen_range(1_000..=max_supply)
            } else {
                // TAO reserve (limit prices with 0-1000)
                rng.gen_range(1_000..x.saturating_mul(1000).min(max_supply))
            };
            let dx: u64 = if full_range {
                // Alhpa sold (allow huge values)
                rng.gen_range(1_000..=21_000_000_000_000_000)
            } else {
                // Alhpa sold (do not sell more than 100% of what's in alpha reserve)
                rng.gen_range(1_000..=x)
            };
            let w_numerator: u64 = rng.gen_range(ACCURACY / 10..=ACCURACY / 10 * 9);
            let w_quote = Perquintill::from_rational(w_numerator, ACCURACY);

            let rw = ReserveWeight::new(w_quote).unwrap();
            let e = rw.exp_base_quote(x, dx);

            let one = U64F64::from_num(1);
            let dy = U64F64::from_num(y) * (one - e);

            // Calculate expected in f64 and approx-assert
            let w1 = perquintill_to_f64(rw.get_base_weight());
            let w2 = perquintill_to_f64(rw.get_quote_weight());
            let e_expected = (x as f64 / (x as f64 + dx as f64)).powf(w1 / w2);
            let dy_expected = y as f64 * (1. - e_expected);

            let actual = dy.to_num::<f64>();
            let mut eps = dy_expected / 1_000_000.;
            if eps > 1000.0 {
                eps = 1000.0;
            }
            if eps < 1.0 {
                eps = 1.0;
            }

            assert!(
                (actual - dy_expected).abs() <= eps,
                "dy mismatch:\n actual:   {}\n expected: {}\n eps: {}\nParameters:\n x:  {}\n y:  {}\n dx: {}\n w_numerator: {}\n",
                actual, dy_expected, eps, x, y, dx, w_numerator,
            );

            // Assert that we aren't giving out more than reserve y
            assert!(dy <= y, "dy = {},\ny =  {}", dy, y,);            

            // Print progress
            let done = counter.fetch_add(1, Ordering::Relaxed) + 1;
            if done % 100_000_000 == 0 {
                let progress = done as f64 / ITERATIONS as f64 * 100.0;
                println!("progress = {progress:.4}%");
            }
        });
    }

    #[test]
    fn test_calculate_quote_delta_in() {
        let num = 250_000_000_000_u128; // w1 = 0.75 
        let w_quote = Perquintill::from_rational(num, 1_000_000_000_000_u128);
        let rw = ReserveWeight::new(w_quote).unwrap();

        let current_price: U64F64 = U64F64::from_num(0.1);
        let target_price: U64F64 = U64F64::from_num(0.2);
        let tao_reserve: u64 = 1_000_000_000;

        let dy = rw.calculate_quote_delta_in(current_price, target_price, tao_reserve);

        // ∆y = y•[(p'/p)^w1 - 1]
        let dy_expected = tao_reserve as f64
            * ((target_price.to_num::<f64>() / current_price.to_num::<f64>()).powf(0.75) - 1.0);

        assert_eq!(dy, dy_expected as u64,);
    }

    #[test]
    fn test_calculate_base_delta_in() {
        let num = 250_000_000_000_u128; // w2 = 0.25 
        let w_quote = Perquintill::from_rational(num, 1_000_000_000_000_u128);
        let rw = ReserveWeight::new(w_quote).unwrap();

        let current_price: U64F64 = U64F64::from_num(0.2);
        let target_price: U64F64 = U64F64::from_num(0.1);
        let alpha_reserve: u64 = 1_000_000_000;

        let dx = rw.calculate_base_delta_in(current_price, target_price, alpha_reserve);

        // ∆x = x•[(p/p')^w2 - 1]
        let dx_expected = alpha_reserve as f64
            * ((current_price.to_num::<f64>() / target_price.to_num::<f64>()).powf(0.25) - 1.0);

        assert_eq!(dx, dx_expected as u64,);
    }

    #[test]
    fn test_calculate_quote_delta_in_impossible() {
        let num = 250_000_000_000_u128; // w1 = 0.75 
        let w_quote = Perquintill::from_rational(num, 1_000_000_000_000_u128);
        let rw = ReserveWeight::new(w_quote).unwrap();

        // Impossible price (lower)
        let current_price: U64F64 = U64F64::from_num(0.1);
        let target_price: U64F64 = U64F64::from_num(0.05);
        let tao_reserve: u64 = 1_000_000_000;

        let dy = rw.calculate_quote_delta_in(current_price, target_price, tao_reserve);
        let dy_expected = 0u64;

        assert_eq!(dy, dy_expected as u64,);
    }

    #[test]
    fn test_calculate_base_delta_in_impossible() {
        let num = 250_000_000_000_u128; // w2 = 0.25 
        let w_quote = Perquintill::from_rational(num, 1_000_000_000_000_u128);
        let rw = ReserveWeight::new(w_quote).unwrap();

        // Impossible price (higher)
        let current_price: U64F64 = U64F64::from_num(0.1);
        let target_price: U64F64 = U64F64::from_num(0.2);
        let alpha_reserve: u64 = 1_000_000_000;

        let dx = rw.calculate_base_delta_in(current_price, target_price, alpha_reserve);
        let dx_expected = 0u64;

        assert_eq!(dx, dx_expected as u64,);
    }

    #[test]
    fn test_calculate_delta_in_reverse_swap() {
        let num = 500_000_000_000_u128;
        let w_quote = Perquintill::from_rational(num, 1_000_000_000_000_u128);
        let rw = ReserveWeight::new(w_quote).unwrap();

        let current_price: U64F64 = U64F64::from_num(0.1);
        let target_price: U64F64 = U64F64::from_num(0.2);
        let tao_reserve: u64 = 1_000_000_000;

        // Here is the simple case of w1 = w2 = 0.5, so alpha = tao / price
        let alpha_reserve: u64 = (tao_reserve as f64 / current_price.to_num::<f64>()) as u64;

        let dy = rw.calculate_quote_delta_in(current_price, target_price, tao_reserve);
        let dx = alpha_reserve as f64
            * (1.0
                - (tao_reserve as f64 / (tao_reserve as f64 + dy as f64))
                    .powf(num as f64 / (1_000_000_000_000 - num) as f64));

        // Verify that buying with dy will in fact bring the price to target_price
        let actual_price = rw.calculate_price(alpha_reserve - dx as u64, tao_reserve + dy);
        assert_abs_diff_eq!(
            actual_price.to_num::<f64>(),
            target_price.to_num::<f64>(),
            epsilon = target_price.to_num::<f64>() / 1_000_000_000.
        );
    }
}
