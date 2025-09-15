use substrate_fixed::types::U64F64;
use subtensor_runtime_common::{AlphaCurrency, Currency, TaoCurrency};

pub trait Order: Clone {
    type PaidIn: Currency;
    type PaidOut: Currency;

    fn with_amount(amount: Self::PaidIn) -> Self;
    fn amount(&self) -> Self::PaidIn;
    fn is_beyond_price_limit(&self, alpha_sqrt_price: U64F64, limit_sqrt_price: U64F64) -> bool;
}

#[derive(Clone)]
pub struct AlphaForTao {
    amount: TaoCurrency,
}

impl Order for AlphaForTao {
    type PaidIn = TaoCurrency;
    type PaidOut = AlphaCurrency;

    fn with_amount(amount: TaoCurrency) -> Self {
        Self { amount }
    }

    fn amount(&self) -> TaoCurrency {
        self.amount
    }

    fn is_beyond_price_limit(&self, alpha_sqrt_price: U64F64, limit_sqrt_price: U64F64) -> bool {
        alpha_sqrt_price < limit_sqrt_price
    }
}

#[derive(Clone)]
pub struct TaoForAlpha {
    amount: AlphaCurrency,
}

impl Order for TaoForAlpha {
    type PaidIn = AlphaCurrency;
    type PaidOut = TaoCurrency;

    fn with_amount(amount: AlphaCurrency) -> Self {
        Self { amount }
    }

    fn amount(&self) -> AlphaCurrency {
        self.amount
    }

    fn is_beyond_price_limit(&self, alpha_sqrt_price: U64F64, limit_sqrt_price: U64F64) -> bool {
        alpha_sqrt_price > limit_sqrt_price
    }
}
