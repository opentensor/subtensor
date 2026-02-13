use super::*;
pub mod evm;
pub mod identity;
pub mod misc;
pub mod rate_limiting;
#[cfg(feature = "try-runtime")]
pub mod try_state;
pub mod voting_power;

#[macro_export]
macro_rules! WeightMeterWrapper {
    ( $meter:expr, $weight:expr, $body:expr ) => {{
        if !$meter.can_consume($weight) {
            return $meter.consumed();
        }
        $body;
        $meter.consume($weight);
    }};
}

#[cfg(test)]
mod tests {
    use core::cell::Cell;

    /// Mock weight meter for testing the macro.
    struct MockWeightMeter {
        limit: u64,
        used: u64,
    }

    impl MockWeightMeter {
        fn with_limit(limit: u64) -> Self {
            Self { limit, used: 0 }
        }
        fn can_consume(&self, weight: u64) -> bool {
            self.used.saturating_add(weight) <= self.limit
        }
        fn consume(&mut self, weight: u64) {
            self.used = self.used.saturating_add(weight);
        }
        fn consumed(&self) -> u64 {
            self.used
        }
    }

    /// Helper: the macro's early return yields u64, so it must be in a fn returning u64.
    fn run_with_meter(mut meter: MockWeightMeter) -> u64 {
        WeightMeterWrapper!(meter, 10u64, {
            // body executes when we can consume
        });
        WeightMeterWrapper!(meter, 20u64, {
            // body executes
        });
        meter.consumed()
    }

    #[test]
    fn test_weight_meter_wrapper_consumes_weight() {
        let meter = MockWeightMeter::with_limit(100);
        let consumed = run_with_meter(meter);
        assert_eq!(consumed, 30, "should consume 10 + 20 = 30");
    }

    #[test]
    fn test_weight_meter_wrapper_returns_early_when_over_limit() {
        let meter = MockWeightMeter::with_limit(15);
        let consumed = run_with_meter(meter);
        // First block consumes 10, second would need 20 but only 5 remain -> early return
        assert_eq!(
            consumed, 10,
            "should return after first consume when second would exceed limit"
        );
    }

    #[test]
    fn test_weight_meter_wrapper_body_executes() {
        fn helper() -> u64 {
            let executed = Cell::new(false);
            let mut meter = MockWeightMeter::with_limit(100);
            WeightMeterWrapper!(meter, 10u64, {
                executed.set(true);
            });
            assert!(
                executed.get(),
                "body should execute when weight is available"
            );
            meter.consumed()
        }
        assert_eq!(helper(), 10);
    }

    #[test]
    fn test_weight_meter_wrapper_body_does_not_execute_when_over_limit() {
        let executed = Cell::new(false);
        let mut meter = MockWeightMeter::with_limit(5);
        fn run(executed: &Cell<bool>, meter: &mut MockWeightMeter) -> u64 {
            WeightMeterWrapper!(meter, 10u64, {
                executed.set(true);
            });
            meter.consumed()
        }
        let consumed = run(&executed, &mut meter);
        assert!(
            !executed.get(),
            "body should not execute when weight exceeds limit"
        );
        assert_eq!(consumed, 0);
    }
}
