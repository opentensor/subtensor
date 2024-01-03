use
{
    super::
    {
        *
    },
    crate::
    {
        system::
        {
            ensure_root,
            ensure_signed_or_root
        }
    },
    sp_std::
    {
        vec,
        vec::
        {
            Vec
        }
    },
    frame_support::
    {
        IterableStorageDoubleMap,
        pallet_prelude::
        {
            DispatchResult,
            DispatchError
        }
    },
    sp_core::
    {
        U256
    },
    substrate_fixed::
    {
        types::
        {
            I32F32,
            I64F64
        }
    },
    crate::
    {
        math::
        {
            *
        }
    }
};

include!("subnet.rs");
include!("consensus.rs");
include!("token.rs");
include!("rate.rs");

impl<T: Config> Pallet<T> 
{
    pub fn ensure_subnet_owner_or_root(o: T::RuntimeOrigin, netuid: u16) -> Result<(), DispatchError> 
    {
        let coldkey = ensure_signed_or_root(o);
        match coldkey 
        {
            Ok(Some(who)) if SubnetOwner::<T>::get(netuid) == who => 
            {
                return Ok(());
            }
        
            Ok(Some(_)) => 
            {
                return Err(DispatchError::BadOrigin.into());
            },

            Ok(None) =>
            {
                return Ok(());
            }

            Err(x) =>
            {
                return Err(x.into());
            }
        }
    }

    // ========================
    // ==== Global Getters ====
    // ========================
    pub fn get_total_issuance() -> u64 
    {
        TotalIssuance::<T>::get()
    }

    pub fn get_block_emission() -> u64 
    {
        BlockEmission::<T>::get()
    }

    pub fn get_current_block_as_u64() -> u64 
    {
        return TryInto::try_into(<frame_system::Pallet<T>>::block_number())
            .ok()
            .expect("blockchain will not exceed 2^64 blocks; QED.");
    }

    // util funcs
    pub fn u64_to_balance(input: u64) -> Option<<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance> 
    {
        return input.try_into().ok();
    }

    // Returns the coldkey owning this hotkey. This function should only be called for active accounts.
    //
    pub fn get_owning_coldkey_for_hotkey(hotkey: &T::AccountId) -> T::AccountId 
    {
        return Owner::<T>::get(hotkey);
    }

    // Returns true if the hotkey account has been created.
    //
    pub fn hotkey_account_exists(hotkey: &T::AccountId) -> bool 
    {
        return Owner::<T>::contains_key(hotkey);
    }

    // Return true if the passed coldkey owns the hotkey.
    //
    pub fn coldkey_owns_hotkey(coldkey: &T::AccountId, hotkey: &T::AccountId) -> bool 
    {
        if Self::hotkey_account_exists(hotkey) 
        {
            return Owner::<T>::get(hotkey) == *coldkey;
        } 
        else 
        {
            return false;
        }
    }

    // Returns true if the passed hotkey allow delegative staking.
    //
    pub fn hotkey_is_delegate(hotkey: &T::AccountId) -> bool 
    {
        return Delegates::<T>::contains_key(hotkey);
    }

    // Sets the hotkey as a delegate with take.
    //
    pub fn delegate_hotkey(hotkey: &T::AccountId, take: u16) 
    {
        Delegates::<T>::insert(hotkey, take);
    }
}
