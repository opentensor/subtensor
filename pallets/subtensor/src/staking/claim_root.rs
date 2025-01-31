use super::*;
use frame_support::weights::Weight;
use sp_core::Get;

impl<T: Config> Pallet<T> {
    pub fn do_root_claim(coldkey: T::AccountId) -> Weight {
        let mut weight = Weight::default();

        let hotkeys = StakingHotkeys::<T>::get(&coldkey);
        weight.saturating_accrue(T::DbWeight::get().reads(1));

        hotkeys.iter().for_each(|hotkey| {
            weight.saturating_accrue(T::DbWeight::get().reads(1));
            weight.saturating_accrue(Self::root_claim_all(hotkey, &coldkey));
        });

        weight.into()
    }
}
