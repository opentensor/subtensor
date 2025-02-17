use super::*;

impl<T: Config> Pallet<T> {
    pub fn do_add_tao_liquidity(
        origin: T::RuntimeOrigin,
        netuid: u16,
        tao: u64,
    ) -> dispatch::DispatchResult {
        // Check that the transaction is signed by the caller and retrieve the T::AccountId coldkey information.
        let coldkey = ensure_signed(origin)?;

        // Ensure the coldkey has tao + ED balance 
        // TBD

        Self::util_add_tao_liquidity(
            &coldkey,
            netuid,
            tao
        );

        Ok(())
    }

    pub fn do_add_alpha_liquidity(
        origin: T::RuntimeOrigin,
        hotkey: T::AccountId,
        netuid: u16,
        alpha: u64,
    ) -> dispatch::DispatchResult {
        // Check that the transaction is signed by the caller and retrieve the T::AccountId coldkey information.
        let coldkey = ensure_signed(origin)?;

        // Ensure the coldkey has alpha stake at hotkey
        // TBD

        Self::util_add_alpha_liquidity(
            &coldkey,
            &hotkey,
            netuid,
            alpha
        );

        Ok(())
    }

    pub fn do_remove_tao_liquidity(
        origin: T::RuntimeOrigin,
        netuid: u16,
        tao: u64,
    ) -> dispatch::DispatchResult {
        // Check that the transaction is signed by the caller and retrieve the T::AccountId coldkey information.
        let coldkey = ensure_signed(origin)?;

        // Ensure the coldkey has tao liquidity to remove
        // TBD

        Self::util_remove_tao_liquidity(
            &coldkey,
            netuid,
            tao
        );

        Ok(())
    }

    pub fn do_remove_alpha_liquidity(
        origin: T::RuntimeOrigin,
        hotkey: T::AccountId,
        netuid: u16,
        alpha: u64,
    ) -> dispatch::DispatchResult {
        // Check that the transaction is signed by the caller and retrieve the T::AccountId coldkey information.
        let coldkey = ensure_signed(origin)?;

        // Ensure coldkey owns the hotkey

        // Ensure the coldkey has alpha liquidity to remove 
        // TBD

        Self::util_remove_alpha_liquidity(
            &coldkey,
            &hotkey,
            netuid,
            alpha
        );

        Ok(())
    }


}