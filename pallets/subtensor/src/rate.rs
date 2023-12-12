impl<T: Config> Pallet<T> 
{ 
    pub fn set_last_tx_block(key: &T::AccountId, block: u64) 
    {
        return LastTxBlock::<T>::insert(key, block);
    }

    pub fn get_last_tx_block(key: &T::AccountId) -> u64 
    {
        return LastTxBlock::<T>::get(key);
    }

    pub fn exceeds_tx_rate_limit(prev_tx_block: u64, current_block: u64) -> bool 
    {
        let rate_limit: u64 = Self::get_tx_rate_limit();
        if rate_limit == 0 || prev_tx_block == 0 
        {
            return false;
        }

        return current_block - prev_tx_block <= rate_limit;
    }

    pub fn get_tx_rate_limit() -> u64 
    {
        return TxRateLimit::<T>::get();
    }

    pub fn set_tx_rate_limit(tx_rate_limit: u64) 
    {
        TxRateLimit::<T>::put(tx_rate_limit);
        
        Self::deposit_event(Event::TxRateLimitSet(tx_rate_limit));
    }

    pub fn get_serving_rate_limit(netuid: u16) -> u64 
    {
        return ServingRateLimit::<T>::get(netuid);
    }

    pub fn set_serving_rate_limit(netuid: u16, serving_rate_limit: u64) 
    {
        ServingRateLimit::<T>::insert(netuid, serving_rate_limit);

        Self::deposit_event(Event::ServingRateLimitSet(netuid, serving_rate_limit));
    }

    pub fn get_network_rate_limit() -> u64 
    {
        return NetworkRateLimit::<T>::get()
    }

    pub fn set_network_rate_limit(limit: u64) 
    {
        NetworkRateLimit::<T>::set(limit);

        Self::deposit_event(Event::NetworkRateLimitSet(limit));
    }
}