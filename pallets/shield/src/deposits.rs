use super::*;

impl<T: Config> Pallet<T> {
    pub fn ibe_submission_deposit() -> BalanceOf<T> {
        T::SubmissionDeposit::get()
    }

    pub(crate) fn reserve_ibe_submission_deposit(
        index: u32,
        who: &T::AccountId,
        amount: BalanceOf<T>,
    ) -> DispatchResult {
        if amount.is_zero() {
            return Ok(());
        }

        T::Currency::reserve(who, amount)?;
        PendingIbeSubmissionDeposits::<T>::insert(index, (who.clone(), amount));
        Self::deposit_event(Event::IbeSubmissionDepositReserved {
            index,
            who: who.clone(),
            amount,
        });
        Ok(())
    }

    pub fn refund_ibe_submission_deposit(index: u32) {
        let Some((who, amount)) = PendingIbeSubmissionDeposits::<T>::take(index) else {
            return;
        };
        if amount.is_zero() {
            return;
        }
        let _ = T::Currency::unreserve(&who, amount);
        Self::deposit_event(Event::IbeSubmissionDepositRefunded { index, who, amount });
    }

    pub fn forfeit_ibe_submission_deposit(index: u32) {
        let Some((who, amount)) = PendingIbeSubmissionDeposits::<T>::take(index) else {
            return;
        };
        if amount.is_zero() {
            return;
        }
        let (_slashed, unslashed) = T::Currency::slash_reserved(&who, amount);
        if !unslashed.is_zero() {
            let _ = T::Currency::unreserve(&who, unslashed);
        }
        Self::deposit_event(Event::IbeSubmissionDepositForfeited { index, who, amount });
    }
}
