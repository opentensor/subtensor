use super::*;
pub mod evm;
pub mod identity;
pub mod misc;
pub mod rate_limiting;
#[cfg(feature = "try-runtime")]
pub mod try_state;
pub mod voting_power;

#[cfg(test)]
mod tests {

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
        WeightMeterWrapper!(meter, 10u64);
        WeightMeterWrapper!(meter, 20u64);
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
            let mut meter = MockWeightMeter::with_limit(100);
            WeightMeterWrapper!(meter, 10u64);
            meter.consumed()
        }
        assert_eq!(helper(), 10);
    }

    #[test]
    fn test_weight_meter_wrapper_body_does_not_execute_when_over_limit() {
        let mut meter = MockWeightMeter::with_limit(5);
        fn run(meter: &mut MockWeightMeter) -> u64 {
            WeightMeterWrapper!(meter, 10u64);
            meter.consumed()
        }
        let consumed = run(&mut meter);
        assert_eq!(consumed, 0);
    }
}
