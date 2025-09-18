use core::marker::PhantomData;

use substrate_fixed::types::U64F64;
use subtensor_runtime_common::{AlphaCurrency, Currency, CurrencyReserve, TaoCurrency};

pub trait Order: Clone {
    type PaidIn: Currency;
    type PaidOut: Currency;
    type ReserveIn: CurrencyReserve<Self::PaidIn>;
    type ReserveOut: CurrencyReserve<Self::PaidOut>;

    fn with_amount(amount: impl Into<Self::PaidIn>) -> Self;
    fn amount(&self) -> Self::PaidIn;
    fn is_beyond_price_limit(&self, alpha_sqrt_price: U64F64, limit_sqrt_price: U64F64) -> bool;
}

#[derive(Clone)]
pub struct AlphaForTao<ReserveIn, ReserveOut>
where
    ReserveIn: CurrencyReserve<TaoCurrency>,
    ReserveOut: CurrencyReserve<AlphaCurrency>,
{
    amount: TaoCurrency,
    _phantom: PhantomData<(ReserveIn, ReserveOut)>,
}

impl<ReserveIn, ReserveOut> Order for AlphaForTao<ReserveIn, ReserveOut>
where
    ReserveIn: CurrencyReserve<TaoCurrency> + Clone,
    ReserveOut: CurrencyReserve<AlphaCurrency> + Clone,
{
    type PaidIn = TaoCurrency;
    type PaidOut = AlphaCurrency;
    type ReserveIn = ReserveIn;
    type ReserveOut = ReserveOut;

    fn with_amount(amount: impl Into<Self::PaidIn>) -> Self {
        Self {
            amount: amount.into(),
            _phantom: PhantomData,
        }
    }

    fn amount(&self) -> TaoCurrency {
        self.amount
    }

    fn is_beyond_price_limit(&self, alpha_sqrt_price: U64F64, limit_sqrt_price: U64F64) -> bool {
        alpha_sqrt_price < limit_sqrt_price
    }
}

#[derive(Clone)]
pub struct TaoForAlpha<ReserveIn, ReserveOut>
where
    ReserveIn: CurrencyReserve<AlphaCurrency>,
    ReserveOut: CurrencyReserve<TaoCurrency>,
{
    amount: AlphaCurrency,
    _phantom: PhantomData<(ReserveIn, ReserveOut)>,
}

impl<ReserveIn, ReserveOut> Order for TaoForAlpha<ReserveIn, ReserveOut>
where
    ReserveIn: CurrencyReserve<AlphaCurrency> + Clone,
    ReserveOut: CurrencyReserve<TaoCurrency> + Clone,
{
    type PaidIn = AlphaCurrency;
    type PaidOut = TaoCurrency;
    type ReserveIn = ReserveIn;
    type ReserveOut = ReserveOut;

    fn with_amount(amount: impl Into<Self::PaidIn>) -> Self {
        Self {
            amount: amount.into(),
            _phantom: PhantomData,
        }
    }

    fn amount(&self) -> AlphaCurrency {
        self.amount
    }

    fn is_beyond_price_limit(&self, alpha_sqrt_price: U64F64, limit_sqrt_price: U64F64) -> bool {
        alpha_sqrt_price > limit_sqrt_price
    }
}
