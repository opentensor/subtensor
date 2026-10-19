//! Dispatch and hook weights. Reference measurements supply the runtime implementation.
use frame_support::weights::Weight;

pub trait WeightInfo {
    fn open() -> Weight;
    fn open_exact() -> Weight;
    fn open_long_and_sell() -> Weight;
    fn close() -> Weight;
    fn set_enabled() -> Weight;
    fn set_lift_params() -> Weight;
    fn set_min_price_impact() -> Weight;
    fn collect() -> Weight;
    fn update_reference() -> Weight;
    fn settle() -> Weight;
}

/// The mock runtime has no execution budget. Production must provide measured weights.
impl WeightInfo for () {
    fn open() -> Weight {
        Weight::zero()
    }
    fn open_exact() -> Weight {
        Weight::zero()
    }
    fn open_long_and_sell() -> Weight {
        Weight::zero()
    }
    fn close() -> Weight {
        Weight::zero()
    }
    fn set_enabled() -> Weight {
        Weight::zero()
    }
    fn set_lift_params() -> Weight {
        Weight::zero()
    }
    fn set_min_price_impact() -> Weight {
        Weight::zero()
    }
    fn collect() -> Weight {
        Weight::zero()
    }
    fn update_reference() -> Weight {
        Weight::zero()
    }
    fn settle() -> Weight {
        Weight::zero()
    }
}
