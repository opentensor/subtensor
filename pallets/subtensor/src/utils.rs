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
        pallet_prelude::
        {
            DispatchResult,
            DispatchError
        }
    },
    sp_core::
    {
        U256
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
}
