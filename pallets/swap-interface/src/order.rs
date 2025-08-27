use core::marker::PhantomData;

use substrate_fixed::types::U64F64;
use subtensor_runtime_common::{AlphaCurrency, Currency, TaoCurrency};

pub trait Order<PaidIn: Currency, PaidOut: Currency>: Clone {
    fn amount(&self) -> PaidIn;
    fn is_beyond_price_limit(&self, alpha_sqrt_price: U64F64, limit_sqrt_price: U64F64) -> bool;
}

#[derive(Clone)]
pub struct BasicOrder<PaidIn, PaidOut>
where
    PaidIn: Currency,
    PaidOut: Currency,
{
    amount: PaidIn,
    _paid_in: PhantomData<PaidIn>,
    _paid_out: PhantomData<PaidOut>,
}

impl Order<TaoCurrency, AlphaCurrency> for BasicOrder<TaoCurrency, AlphaCurrency> {
    fn amount(&self) -> TaoCurrency {
        self.amount
    }

    fn is_beyond_price_limit(&self, alpha_sqrt_price: U64F64, limit_sqrt_price: U64F64) -> bool {
        alpha_sqrt_price < limit_sqrt_price
    }
}

impl Order<AlphaCurrency, TaoCurrency> for BasicOrder<AlphaCurrency, TaoCurrency> {
    fn amount(&self) -> AlphaCurrency {
        self.amount
    }

    fn is_beyond_price_limit(&self, alpha_sqrt_price: U64F64, limit_sqrt_price: U64F64) -> bool {
        alpha_sqrt_price > limit_sqrt_price
    }
}
