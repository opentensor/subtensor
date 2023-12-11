use
{
    super::
    {
        *
    },
    frame_support::
    {
        pallet_prelude::
        {
            Decode, 
            Encode
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
    codec::
    {
        Compact
    },
    sp_core::
    {
        hexdisplay::
        {
            AsBytesRef
        }
    }
};

#[derive(Decode, Encode, PartialEq, Eq, Clone, Debug)]
pub struct StakeInfo<T: Config> 
{
    hotkey:     T::AccountId,
    coldkey:    T::AccountId,
    stake:      Compact<u64>,
}

impl<T: Config> Pallet<T> 
{
    fn _get_stake_info_for_coldkeys(coldkeys: Vec<T::AccountId>) -> Vec<(T::AccountId, Vec<StakeInfo<T>>)> 
    {
        if coldkeys.len() == 0 
        {
            return Vec::new(); // No coldkeys to check
        }

        let mut stake_info: Vec<(T::AccountId, Vec<StakeInfo<T>>)> = Vec::new();
        for coldkey_ in coldkeys 
        {
            let mut stake_info_for_coldkey: Vec<StakeInfo<T>> = Vec::new();

            for (hotkey, coldkey, stake) in <Stake<T>>::iter() 
            {
                if coldkey == coldkey_ 
                {
                    stake_info_for_coldkey.push(StakeInfo 
                    {
                        hotkey,
                        coldkey,
                        stake: stake.into(),
                    });
                }
            }

            stake_info.push((coldkey_, stake_info_for_coldkey));
        }

        return stake_info;
    }

    pub fn get_stake_info_for_coldkeys(coldkey_account_vecs: Vec<Vec<u8>>) -> Vec<(T::AccountId, Vec<StakeInfo<T>>)> 
    {
        let mut coldkeys: Vec<T::AccountId> = Vec::new();
        for coldkey_account_vec in coldkey_account_vecs 
        {
            if coldkey_account_vec.len() != 32 
            {
                continue; // Invalid coldkey
            }

            coldkeys.push(T::AccountId::decode(&mut coldkey_account_vec.as_bytes_ref()).unwrap());
        }

        if coldkeys.len() == 0 
        {
            return Vec::new(); // Invalid coldkey
        }

        return Self::_get_stake_info_for_coldkeys(coldkeys);
    }

    pub fn get_stake_info_for_coldkey(coldkey_account_vec: Vec<u8>) -> Vec<StakeInfo<T>> 
    {
        if coldkey_account_vec.len() != 32 
        {
            return Vec::new(); // Invalid coldkey
        }

        let coldkey: AccountIdOf<T> = T::AccountId::decode(&mut coldkey_account_vec.as_bytes_ref()).unwrap();
        let stake_info = Self::_get_stake_info_for_coldkeys(vec![coldkey]);
        if stake_info.len() == 0 
        {
            return Vec::new(); // Invalid coldkey
        } 
        else 
        {
            return stake_info.get(0).unwrap().1.clone();
        }
    }
}
