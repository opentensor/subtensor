
impl<T: Config> Pallet<T> 
{
    pub fn burn_tokens(amount: u64) 
    {
        TotalIssuance::<T>::put(TotalIssuance::<T>::get().saturating_sub(amount));
    }
    
    pub fn get_default_take() -> u16 
    {
        return DefaultTake::<T>::get();
    }
    
    pub fn set_default_take(default_take: u16) 
    {
        DefaultTake::<T>::put(default_take);

        Self::deposit_event(Event::DefaultTakeSet(default_take));
    }

    pub fn set_subnet_locked_balance(netuid: u16, amount: u64) 
    {
        SubnetLocked::<T>::insert(netuid, amount);
    }

    pub fn get_subnet_locked_balance(netuid: u16) -> u64 
    {
        return SubnetLocked::<T>::get(netuid);
    }
}