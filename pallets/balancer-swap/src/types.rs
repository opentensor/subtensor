use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::pallet_prelude::*;
use scale_info::TypeInfo;
use sp_runtime::Perbill;
use substrate_fixed::types::U96F32;
use subtensor_macros::freeze_struct;
use subtensor_runtime_common::{AlphaCurrency, Currency, TaoCurrency};

/// Represents which token in the pool
#[derive(Debug, Clone, Copy, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub enum TokenType {
    /// TAO token
    Tao,
    /// Alpha token
    Alpha,
}

/// Balancer-style weighted pool for TAO/Alpha swaps
///
/// Uses the weighted constant product formula:
/// V = (balance_tao ^ weight_tao) * (balance_alpha ^ weight_alpha)
/// Where V remains constant during swaps
#[freeze_struct("a1b2c3d4e5f6g7h8")]
#[derive(Debug, Clone, Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, Eq)]
pub struct Pool {
    /// Current TAO balance in the pool
    pub tao_balance: TaoCurrency,
    /// Current Alpha balance in the pool
    pub alpha_balance: AlphaCurrency,
    /// Weight for TAO (e.g., 50% = Perbill::from_percent(50))
    pub tao_weight: Perbill,
    /// Weight for Alpha (e.g., 50% = Perbill::from_percent(50))
    pub alpha_weight: Perbill,
    /// Total LP shares issued for this pool
    pub total_shares: u128,
    /// Swap fee as percentage (e.g., 0.3% = Perbill::from_rational(3u32, 1000u32))
    pub swap_fee: Perbill,
}

impl Default for Pool {
    fn default() -> Self {
        Self {
            tao_balance: TaoCurrency::ZERO,
            alpha_balance: AlphaCurrency::ZERO,
            tao_weight: Perbill::from_percent(50),
            alpha_weight: Perbill::from_percent(50),
            total_shares: 0,
            swap_fee: Perbill::from_rational(3u32, 1000u32), // 0.3%
        }
    }
}

impl Pool {
    /// Creates a new pool with given parameters
    pub fn new(
        tao_balance: TaoCurrency,
        alpha_balance: AlphaCurrency,
        tao_weight: Perbill,
        alpha_weight: Perbill,
        swap_fee: Perbill,
    ) -> Self {
        Self {
            tao_balance,
            alpha_balance,
            tao_weight,
            alpha_weight,
            total_shares: 0,
            swap_fee,
        }
    }

    /// Returns true if pool has been initialized with liquidity
    pub fn is_initialized(&self) -> bool {
        !self.tao_balance.is_zero() && !self.alpha_balance.is_zero()
    }

    /// Calculates the spot price of Alpha in terms of TAO
    ///
    /// Formula: spot_price = (balance_tao / weight_tao) / (balance_alpha / weight_alpha)
    pub fn spot_price(&self) -> U96F32 {
        if self.alpha_balance.is_zero() {
            return U96F32::from_num(0);
        }

        let tao_value = U96F32::from_num(self.tao_balance.to_u64())
            / U96F32::from_num(self.tao_weight);
        let alpha_value = U96F32::from_num(self.alpha_balance.to_u64())
            / U96F32::from_num(self.alpha_weight);

        tao_value / alpha_value
    }

    /// Returns the balance for a given token type
    pub fn balance(&self, token: TokenType) -> u64 {
        match token {
            TokenType::Tao => self.tao_balance.to_u64(),
            TokenType::Alpha => self.alpha_balance.to_u64(),
        }
    }

    /// Returns the weight for a given token type
    pub fn weight(&self, token: TokenType) -> Perbill {
        match token {
            TokenType::Tao => self.tao_weight,
            TokenType::Alpha => self.alpha_weight,
        }
    }

    /// Sets the balance for a given token type
    pub fn set_balance(&mut self, token: TokenType, amount: u64) {
        match token {
            TokenType::Tao => self.tao_balance = amount.into(),
            TokenType::Alpha => self.alpha_balance = amount.into(),
        }
    }

    /// Returns the other token type
    pub fn other_token(token: TokenType) -> TokenType {
        match token {
            TokenType::Tao => TokenType::Alpha,
            TokenType::Alpha => TokenType::Tao,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool_default() {
        let pool = Pool::default();
        assert_eq!(pool.tao_balance, TaoCurrency::ZERO);
        assert_eq!(pool.alpha_balance, AlphaCurrency::ZERO);
        assert_eq!(pool.tao_weight, Perbill::from_percent(50));
        assert_eq!(pool.alpha_weight, Perbill::from_percent(50));
        assert_eq!(pool.total_shares, 0);
    }

    #[test]
    fn test_pool_is_initialized() {
        let mut pool = Pool::default();
        assert!(!pool.is_initialized());

        pool.tao_balance = 1000.into();
        assert!(!pool.is_initialized());

        pool.alpha_balance = 1000.into();
        assert!(pool.is_initialized());
    }

    #[test]
    fn test_spot_price_50_50() {
        let pool = Pool {
            tao_balance: 1000.into(),
            alpha_balance: 2000.into(),
            tao_weight: Perbill::from_percent(50),
            alpha_weight: Perbill::from_percent(50),
            ..Default::default()
        };

        let price = pool.spot_price();
        // Price should be 1000/2000 = 0.5 TAO per Alpha
        assert_eq!(price, U96F32::from_num(0.5));
    }

    #[test]
    fn test_spot_price_80_20() {
        let pool = Pool {
            tao_balance: 8000.into(),
            alpha_balance: 2000.into(),
            tao_weight: Perbill::from_percent(80),
            alpha_weight: Perbill::from_percent(20),
            ..Default::default()
        };

        let price = pool.spot_price();
        // Price = (8000/0.8) / (2000/0.2) = 10000 / 10000 = 1.0
        assert_eq!(price, U96F32::from_num(1.0));
    }

    #[test]
    fn test_token_type_other() {
        assert_eq!(Pool::other_token(TokenType::Tao), TokenType::Alpha);
        assert_eq!(Pool::other_token(TokenType::Alpha), TokenType::Tao);
    }
}



