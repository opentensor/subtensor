use
{
    super::
    {
        *
    },
};

include!("axon.rs");
include!("prometheus.rs");

impl<T: Config> Pallet<T> 
{
    /********************************
     --==[[  Helper functions   ]]==--
    *********************************/
    pub fn is_valid_ip_type(ip_type: u8) -> bool 
    {
        return ip_type == 4 || ip_type == 6;
    }

    // @todo (Parallax 2-1-2021) : Implement exclusion of private IP ranges
    pub fn is_valid_ip_address(ip_type: u8, addr: u128) -> bool 
    {
        match ip_type
        {
            4 => match addr
            {
                0 | 0x7f000001 => { return false; },
                _ => { if addr >= u32::MAX as u128 { return false; } else { return true; } }
            }
            6 => match addr
            {
                0 | u128::MAX | 1 => { return false; },
                _ => { return true; }
            },
            _ => { return false; }
        }
    }
}