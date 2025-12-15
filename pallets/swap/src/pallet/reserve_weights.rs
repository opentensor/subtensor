use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::pallet_prelude::*;
use safe_math::SafeDiv;
use subtensor_macros::freeze_struct;

#[freeze_struct("6382cc997c2e2049")]
#[derive(Clone, Encode, Decode, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct ReserveWeight {
    quote: u64,
}

// Lower imit of weights is 0.01
const MIN_WEIGHT: u64 = 184467440737095516;

#[derive(Debug)]
pub enum ReserveWeightError {
    InvalidValue,
}

impl Default for ReserveWeight {
    fn default() -> Self {
        Self {
            quote: u64::MAX.safe_div(2_u64)
        }
    }
}

impl ReserveWeight {
    pub fn new(quote: u64) -> Result<Self, ReserveWeightError> {
        if Self::check_constraints(quote) {
            Ok(ReserveWeight { quote })
        } else {
            Err(ReserveWeightError::InvalidValue)
        }
    }

    fn check_constraints(quote: u64) -> bool {
        let base = u64::MAX.saturating_sub(quote);
        (base >= MIN_WEIGHT) && (quote >= MIN_WEIGHT)
    }

    pub fn get_quote_weight(&self) -> u64 {
        self.quote
    }

    pub fn get_base_weight(&self) -> u64 {
        u64::MAX.saturating_sub(self.quote)
    }

    pub fn set_quote_weight(&self, new_value: u64) -> Result<(), ReserveWeightError> {
        if Self::check_constraints(new_value) {
            Ok(())
        } else {
            Err(ReserveWeightError::InvalidValue)
        }
    }
}
