/// Balancer weighted pool math
///
/// This module implements the core mathematical formulas for Balancer-style weighted pools.
/// Based on the Balancer V2 whitepaper and implementation.
///
/// Key Formula: Weighted Constant Product
/// V = ∏ (balance_i ^ weight_i) = constant
///
/// For a two-token pool (TAO/Alpha):
/// V = (balance_tao ^ weight_tao) * (balance_alpha ^ weight_alpha)

use safe_math::SafeArith;
use sp_runtime::Perbill;
use substrate_fixed::types::U64F64;

/// Calculates amount out given amount in for a swap
///
/// Formula:
/// amount_out = balance_out * (1 - (balance_in / (balance_in + amount_in * (1 - fee))) ^ (weight_in / weight_out))
///
/// # Arguments
/// * `balance_in` - Current balance of token being sold
/// * `weight_in` - Weight of token being sold
/// * `balance_out` - Current balance of token being bought
/// * `weight_out` - Weight of token being bought
/// * `amount_in` - Amount of token being sold
/// * `swap_fee` - Swap fee as Perbill
///
/// # Returns
/// Amount of token out (before fees)
pub fn calc_out_given_in(
    balance_in: u64,
    weight_in: Perbill,
    balance_out: u64,
    weight_out: Perbill,
    amount_in: u64,
    swap_fee: Perbill,
) -> u64 {
    if amount_in == 0 || balance_in == 0 || balance_out == 0 {
        return 0;
    }

    // Apply fee: adjusted_amount_in = amount_in * (1 - swap_fee)
    let fee_complement = Perbill::one().saturating_sub(swap_fee);
    let adjusted_amount_in = fee_complement.mul_floor(amount_in as u128) as u64;

    // Convert to U64F64 for precise calculations
    let balance_in_f = U64F64::from_num(balance_in);
    let balance_out_f = U64F64::from_num(balance_out);
    let adjusted_amount_in_f = U64F64::from_num(adjusted_amount_in);

    // Calculate: balance_in / (balance_in + adjusted_amount_in)
    let base = balance_in_f.safe_div(
        balance_in_f.saturating_add(adjusted_amount_in_f)
    );

    // Calculate exponent: weight_in / weight_out
    let weight_in_f = U64F64::from_num(weight_in);
    let weight_out_f = U64F64::from_num(weight_out);
    let exponent = weight_in_f.safe_div(weight_out_f);

    // Calculate: base ^ exponent
    // Using natural log: x^y = e^(y * ln(x))
    let ln_base = base.checked_ln().unwrap_or(U64F64::from_num(0));
    let power_result = (exponent.saturating_mul(ln_base))
        .checked_exp()
        .unwrap_or(U64F64::from_num(1));

    // Calculate: 1 - power_result
    let one = U64F64::from_num(1);
    let factor = one.saturating_sub(power_result);

    // Calculate: balance_out * factor
    let amount_out_f = balance_out_f.saturating_mul(factor);

    amount_out_f.saturating_to_num::<u64>()
}

/// Calculates amount in given amount out for a swap
///
/// Formula:
/// amount_in = balance_in * ((balance_out / (balance_out - amount_out)) ^ (weight_out / weight_in) - 1) / (1 - fee)
///
/// # Arguments
/// * `balance_in` - Current balance of token being bought with
/// * `weight_in` - Weight of token being bought with
/// * `balance_out` - Current balance of token being bought
/// * `weight_out` - Weight of token being bought
/// * `amount_out` - Desired amount of token out
/// * `swap_fee` - Swap fee as Perbill
///
/// # Returns
/// Amount of token in required (before fees)
pub fn calc_in_given_out(
    balance_in: u64,
    weight_in: Perbill,
    balance_out: u64,
    weight_out: Perbill,
    amount_out: u64,
    swap_fee: Perbill,
) -> u64 {
    if amount_out == 0 || balance_in == 0 || balance_out == 0 || amount_out >= balance_out {
        return 0;
    }

    let balance_in_f = U64F64::from_num(balance_in);
    let balance_out_f = U64F64::from_num(balance_out);
    let amount_out_f = U64F64::from_num(amount_out);

    // Calculate: balance_out / (balance_out - amount_out)
    let base = balance_out_f.safe_div(
        balance_out_f.saturating_sub(amount_out_f)
    );

    // Calculate exponent: weight_out / weight_in
    let weight_in_f = U64F64::from_num(weight_in);
    let weight_out_f = U64F64::from_num(weight_out);
    let exponent = weight_out_f.safe_div(weight_in_f);

    // Calculate: base ^ exponent
    let ln_base = base.checked_ln().unwrap_or(U64F64::from_num(0));
    let power_result = (exponent.saturating_mul(ln_base))
        .checked_exp()
        .unwrap_or(U64F64::from_num(1));

    // Calculate: power_result - 1
    let one = U64F64::from_num(1);
    let factor = power_result.saturating_sub(one);

    // Calculate: balance_in * factor
    let amount_in_before_fee_f = balance_in_f.saturating_mul(factor);

    // Adjust for fee: amount_in = amount_in_before_fee / (1 - fee)
    let fee_complement = Perbill::one().saturating_sub(swap_fee);
    let fee_complement_f = U64F64::from_num(fee_complement);
    
    let amount_in_f = amount_in_before_fee_f.safe_div(fee_complement_f);

    amount_in_f.saturating_to_num::<u64>()
}

/// Calculates spot price with fee
///
/// Formula:
/// spot_price = (balance_in / weight_in) / (balance_out / weight_out) * (1 / (1 - fee))
///
/// # Arguments
/// * `balance_in` - Balance of token being sold
/// * `weight_in` - Weight of token being sold
/// * `balance_out` - Balance of token being bought
/// * `weight_out` - Weight of token being bought
/// * `swap_fee` - Swap fee as Perbill
pub fn calc_spot_price(
    balance_in: u64,
    weight_in: Perbill,
    balance_out: u64,
    weight_out: Perbill,
    swap_fee: Perbill,
) -> U64F64 {
    if balance_out == 0 {
        return U64F64::from_num(0);
    }

    let balance_in_f = U64F64::from_num(balance_in);
    let balance_out_f = U64F64::from_num(balance_out);
    let weight_in_f = U64F64::from_num(weight_in);
    let weight_out_f = U64F64::from_num(weight_out);

    // Calculate: (balance_in / weight_in) / (balance_out / weight_out)
    let numer = balance_in_f.safe_div(weight_in_f);
    let denom = balance_out_f.safe_div(weight_out_f);
    let spot_price = numer.safe_div(denom);

    // Apply fee: spot_price / (1 - fee)
    let fee_complement = Perbill::one().saturating_sub(swap_fee);
    let fee_complement_f = U64F64::from_num(fee_complement);

    spot_price.safe_div(fee_complement_f)
}

/// Calculates LP shares to mint when adding single-sided liquidity
///
/// Formula:
/// shares_out = total_shares * ((balance_in + amount_in) / balance_in) ^ weight_in - total_shares)
///
/// # Arguments
/// * `balance_in` - Current balance of token being added
/// * `weight_in` - Weight of token being added
/// * `total_shares` - Current total LP shares
/// * `amount_in` - Amount of token being added
///
/// # Returns
/// Number of LP shares to mint
pub fn calc_shares_for_single_token_in(
    balance_in: u64,
    weight_in: Perbill,
    total_shares: u128,
    amount_in: u64,
) -> u128 {
    if amount_in == 0 || total_shares == 0 || balance_in == 0 {
        return 0;
    }

    let balance_in_f = U64F64::from_num(balance_in);
    let amount_in_f = U64F64::from_num(amount_in);
    let total_shares_f = U64F64::from_num(total_shares);
    let weight_in_f = U64F64::from_num(weight_in);

    // Calculate: (balance_in + amount_in) / balance_in
    let base = balance_in_f.saturating_add(amount_in_f).safe_div(balance_in_f);

    // Calculate: base ^ weight_in
    let ln_base = base.checked_ln().unwrap_or(U64F64::from_num(0));
    let power_result = (weight_in_f.saturating_mul(ln_base))
        .checked_exp()
        .unwrap_or(U64F64::from_num(1));

    // Calculate: total_shares * (power_result - 1)
    let one = U64F64::from_num(1);
    let multiplier = power_result.saturating_sub(one);
    let shares_out_f = total_shares_f.saturating_mul(multiplier);

    shares_out_f.saturating_to_num::<u128>()
}

/// Calculates token amount out when burning LP shares (single-sided exit)
///
/// Formula:
/// amount_out = balance_out * (1 - (1 - shares_in / total_shares) ^ (1 / weight_out))
///
/// # Arguments
/// * `balance_out` - Current balance of token being removed
/// * `weight_out` - Weight of token being removed
/// * `total_shares` - Current total LP shares
/// * `shares_in` - Number of LP shares being burned
///
/// # Returns
/// Amount of token to return
pub fn calc_token_out_for_shares(
    balance_out: u64,
    weight_out: Perbill,
    total_shares: u128,
    shares_in: u128,
) -> u64 {
    if shares_in == 0 || total_shares == 0 || shares_in > total_shares {
        return 0;
    }

    let balance_out_f = U64F64::from_num(balance_out);
    let total_shares_f = U64F64::from_num(total_shares);
    let shares_in_f = U64F64::from_num(shares_in);
    let weight_out_f = U64F64::from_num(weight_out);

    // Calculate: 1 - (shares_in / total_shares)
    let share_ratio = shares_in_f.safe_div(total_shares_f);
    let one = U64F64::from_num(1);
    let base = one.saturating_sub(share_ratio);

    // Calculate: base ^ (1 / weight_out)
    let exponent = one.safe_div(weight_out_f);
    let ln_base = base.checked_ln().unwrap_or(U64F64::from_num(0));
    let power_result = (exponent.saturating_mul(ln_base))
        .checked_exp()
        .unwrap_or(U64F64::from_num(1));

    // Calculate: balance_out * (1 - power_result)
    let factor = one.saturating_sub(power_result);
    let amount_out_f = balance_out_f.saturating_mul(factor);

    amount_out_f.saturating_to_num::<u64>()
}

/// Calculates proportional shares when adding balanced liquidity
///
/// Formula:
/// shares_out = total_shares * (amount_in / balance_in)
///
/// This is used when adding liquidity in the exact pool ratio
pub fn calc_shares_proportional(
    balance: u64,
    total_shares: u128,
    amount_in: u64,
) -> u128 {
    if balance == 0 || total_shares == 0 {
        return 0;
    }

    let amount_in_u128 = amount_in as u128;
    let balance_u128 = balance as u128;

    // shares_out = total_shares * (amount_in / balance)
    total_shares.saturating_mul(amount_in_u128).saturating_div(balance_u128)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calc_out_given_in_50_50() {
        // 50/50 pool with 1000 TAO and 1000 Alpha
        let balance_in = 1000;
        let weight_in = Perbill::from_percent(50);
        let balance_out = 1000;
        let weight_out = Perbill::from_percent(50);
        let amount_in = 100;
        let swap_fee = Perbill::from_rational(3u32, 1000u32); // 0.3%

        let amount_out = calc_out_given_in(
            balance_in,
            weight_in,
            balance_out,
            weight_out,
            amount_in,
            swap_fee,
        );

        // With 100 in, should get approximately 90 out (slightly less due to price impact + fee)
        assert!(amount_out > 80 && amount_out < 95);
    }

    #[test]
    fn test_calc_spot_price() {
        let balance_in = 1000;
        let weight_in = Perbill::from_percent(50);
        let balance_out = 1000;
        let weight_out = Perbill::from_percent(50);
        let swap_fee = Perbill::from_rational(3u32, 1000u32);

        let price = calc_spot_price(
            balance_in,
            weight_in,
            balance_out,
            weight_out,
            swap_fee,
        );

        // For 50/50 pool with equal balances, price should be close to 1.0 (adjusted for fee)
        let expected = U64F64::from_num(1.0);
        let diff = if price > expected {
            price - expected
        } else {
            expected - price
        };
        assert!(diff < U64F64::from_num(0.01)); // Within 1% tolerance
    }

    #[test]
    fn test_calc_shares_proportional() {
        let balance = 1000;
        let total_shares = 10000;
        let amount_in = 100;

        let shares = calc_shares_proportional(balance, total_shares, amount_in);
        
        // Adding 10% of liquidity should give 10% of shares
        assert_eq!(shares, 1000);
    }

    #[test]
    fn test_no_output_with_zero_input() {
        let amount_out = calc_out_given_in(
            1000,
            Perbill::from_percent(50),
            1000,
            Perbill::from_percent(50),
            0, // Zero input
            Perbill::zero(),
        );
        assert_eq!(amount_out, 0);
    }
}



