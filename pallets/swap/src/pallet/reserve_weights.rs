use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::pallet_prelude::*;
use safe_bigmath::*;
use safe_math::*;
use sp_arithmetic::Perquintill;
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

        let precision = 256;
        let x_safe = SafeInt::from(x);
        let delta_safe = SafeInt::from(dx);
        let w1_safe = SafeInt::from(w1);
        let w2_safe = SafeInt::from(w2);
        let perquintill_scale = SafeInt::from(ACCURACY as u128);
        let denominator = x_safe.clone() + delta_safe;
        let maybe_result_safe_int = if base_quote {
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
}

// cargo test --package pallet-subtensor-swap --lib -- pallet::reserve_weights::tests --nocapture
#[cfg(test)]
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
    fn test_exp_bae_quote_happy_path() {
        // Outer test cases: w_quote
        [
            Perquintill::from_rational(500_000_000_000_u128, 1_000_000_000_000_u128),
            Perquintill::from_rational(500_000_000_001_u128, 1_000_000_000_000_u128),
            // Perquintill::from_rational(500_000_000_100_u128, 1_000_000_000_000_u128),
            // Perquintill::from_rational(500_000_001_000_u128, 1_000_000_000_000_u128),
            // Perquintill::from_rational(500_000_010_000_u128, 1_000_000_000_000_u128),
            // Perquintill::from_rational(500_000_100_000_u128, 1_000_000_000_000_u128),
            // Perquintill::from_rational(500_001_000_000_u128, 1_000_000_000_000_u128),
            // Perquintill::from_rational(500_010_000_000_u128, 1_000_000_000_000_u128),
            // Perquintill::from_rational(500_100_000_000_u128, 1_000_000_000_000_u128),
            // Perquintill::from_rational(501_000_000_000_u128, 1_000_000_000_000_u128),
            // Perquintill::from_rational(510_000_000_000_u128, 1_000_000_000_000_u128),
            // Perquintill::from_rational(100_000_000_000_u128, 1_000_000_000_000_u128),
            // Perquintill::from_rational(100_000_000_001_u128, 1_000_000_000_000_u128),
            // Perquintill::from_rational(200_000_000_000_u128, 1_000_000_000_000_u128),
            // Perquintill::from_rational(300_000_000_000_u128, 1_000_000_000_000_u128),
            // Perquintill::from_rational(400_000_000_000_u128, 1_000_000_000_000_u128),
            // Perquintill::from_rational(600_000_000_000_u128, 1_000_000_000_000_u128),
            // Perquintill::from_rational(700_000_000_000_u128, 1_000_000_000_000_u128),
            // Perquintill::from_rational(800_000_000_000_u128, 1_000_000_000_000_u128),
            // Perquintill::from_rational(899_999_999_999_u128, 1_000_000_000_000_u128),
            // Perquintill::from_rational(900_000_000_000_u128, 1_000_000_000_000_u128),
        ]
        .into_iter()
        .for_each(|w_quote| {
            // Inner test cases: y, x, ∆x
            [
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
                (1_000_000_000_u64, 4_000_000_000_u64, 1_000_000_000_000_u64),
            ]
            .into_iter()
            .for_each(|(y, x, dx)| {
                let rw = ReserveWeight::new(w_quote).unwrap();
                let e = rw.exp_base_quote(x, dx);
                println!("e = {:?}", e);
                let one = U64F64::from_num(1);
                let y_fixed = U64F64::from_num(y);
                let dy = y_fixed * (one - e);

                let w1 = perquintill_to_f64(rw.get_base_weight());
                let w2 = perquintill_to_f64(rw.get_quote_weight());
                let e_expected = (x as f64 / (x as f64 + dx as f64)).powf(w1 / w2);
                println!("e_expected = {:?}", e_expected);
                let dy_expected = y as f64 * (1. - e_expected);

                let mut eps = dy_expected / 100000.;
                if eps > 0.1 {
                    eps = 0.1;
                }
                assert_abs_diff_eq!(dy.to_num::<f64>(), dy_expected, epsilon = eps,);
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
    fn test_exp_bae_quote_dy_precision() {
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

    // #[ignore]
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
}
