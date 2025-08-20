use core::marker::PhantomData;

use subtensor_runtime_common::Currency;

pub struct OrderType<PaidIn, PaidOut>
where
    PaidIn: Currency,
    PaidOut: Currency,
{
    amount: PaidIn,
    _paid_in: PhantomData<PaidIn>,
    _paid_out: PhantomData<PaidOut>,
}

pub trait Order<PaidIn: Currency, PaidOut: Currency> {}
