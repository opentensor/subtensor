use super::*;

impl<T: Config> Pallet<T> {
  
    pub fn is_valid_ip_type(ip_type: u8) -> bool {
        let allowed_values = [4, 6];
        allowed_values.contains(&ip_type)
    }

    // @todo (Parallax 2-1-2021) : Implement exclusion of private IP ranges
    pub fn is_valid_ip_address(ip_type: u8, addr: u128) -> bool {
        if !Self::is_valid_ip_type(ip_type) {
            return false;
        }
        if addr == 0 {
            return false;
        }
        if ip_type == 4 {
            if addr == 0 {
                return false;
            }
            if addr >= u32::MAX as u128 {
                return false;
            }
            if addr == 0x7f000001 {
                return false;
            } // Localhost
        }
        if ip_type == 6 {
            if addr == 0x0 {
                return false;
            }
            if addr == u128::MAX {
                return false;
            }
            if addr == 1 {
                return false;
            } // IPv6 localhost
        }
        true
    }


}
