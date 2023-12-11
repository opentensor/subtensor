// included by math.rs

#[allow(dead_code)]
pub fn vec_fixed32_to_u64(vec: Vec<I32F32>) -> Vec<u64> 
{
    return vec.into_iter()
        .map(|e| 
            fixed_to_u64(e)
        ).collect();
}

#[allow(dead_code)]
pub fn vec_fixed64_to_fixed32(vec: Vec<I64F64>) -> Vec<I32F32> 
{
    return vec.into_iter()
        .map(|e| 
            fixed64_to_fixed32(e)
        ).collect();
}

#[allow(dead_code)]
pub fn vec_fixed32_to_fixed64(vec: Vec<I32F32>) -> Vec<I64F64> 
{
    return vec.into_iter()
        .map(|e| 
            fixed32_to_fixed64(e)
        ).collect();
}

#[allow(dead_code)]
pub fn vec_fixed64_to_u64(vec: Vec<I64F64>) -> Vec<u64> 
{
    return vec.into_iter()
        .map(|e| 
            fixed64_to_u64(e)
        ).collect();
}

#[allow(dead_code)]
pub fn vec_u16_proportions_to_fixed(vec: Vec<u16>) -> Vec<I32F32> {
    return vec.into_iter()
        .map(|e| 
            u16_proportion_to_fixed(e)
        ).collect();
}

#[allow(dead_code)]
pub fn vec_fixed_proportions_to_u16(vec: Vec<I32F32>) -> Vec<u16>
{
    return vec.into_iter()
        .map(|e| 
            fixed_proportion_to_u16(e)
        ).collect();
}

#[allow(dead_code)]
// Max-upscale vector and convert to u16 so max_value = u16::MAX. Assumes non-negative normalized input.
pub fn vec_max_upscale_to_u16(vec: &Vec<I32F32>) -> Vec<u16> 
{
    let u16_max:    I32F32          = I32F32::from_num(u16::MAX);
    let threshold:  I32F32          = I32F32::from_num(32768);
    match vec.iter().max()
    {
        Some(val) => 
        {
            if *val == I32F32::from_num(0) 
            {
                return vec
                    .iter()
                    .map(|e: &I32F32| 
                        (e * u16_max).to_num::<u16>()
                    ).collect();
            }
            else if *val > threshold 
            {
                return vec
                    .iter()
                    .map(|e: &I32F32| 
                        (e * (u16_max / *val)).round().to_num::<u16>()
                    ).collect();
            }

            return vec
                .iter()
                .map(|e: &I32F32| 
                    ((e * u16_max) / *val).round().to_num::<u16>()
                ).collect();
        }
        None => 
        {
            let sum: I32F32 = vec.iter().sum();
            return vec
                .iter()
                .map(|e: &I32F32| 
                    ((e * u16_max) / sum).to_num::<u16>()
                ).collect();
        }
    }
}

#[allow(dead_code)]
// Max-upscale u16 vector and convert to u16 so max_value = u16::MAX. Assumes u16 vector input.
pub fn vec_u16_max_upscale_to_u16(vec: &Vec<u16>) -> Vec<u16> 
{
    let vec_fixed: Vec<I32F32> = vec.iter()
        .map(|e: &u16| 
            I32F32::from_num(*e)
        ).collect();

    return vec_max_upscale_to_u16(&vec_fixed);
}

#[allow(dead_code)]
// Checks if u16 vector, when normalized, has a max value not greater than a u16 ratio max_limit.
pub fn check_vec_max_limited(vec: &Vec<u16>, max_limit: u16) -> bool 
{
    let max_limit_fixed:    I32F32          = I32F32::from_num(max_limit) / I32F32::from_num(u16::MAX);
    let mut vec_fixed:      Vec<I32F32>     = vec.iter().map(|e: &u16| I32F32::from_num(*e)).collect();

    inplace_normalize(&mut vec_fixed);
    let max_value:          Option<&I32F32> = vec_fixed.iter().max();

    match max_value 
    {
        Some(val) => 
        {
            return *val <= max_limit_fixed;
        }
        None => 
        {
            return true;
        }
    }
}

#[allow(dead_code)]
pub fn sum(x: &Vec<I32F32>) -> I32F32 
{
    return x.iter().sum();
}

#[allow(dead_code)]
// Sums a Vector of type that has CheckedAdd trait.
// Returns None if overflow occurs during sum using T::checked_add.
// Returns Some(T::default()) if input vector is empty.
pub fn checked_sum<T>(x: &Vec<T>) -> Option<T>
where
    T: Copy + Default + CheckedAdd,
{
    if x.len() == 0 
    {
        return Some(T::default());
    }

    let mut sum: T = x[0];
    for i in x[1..].iter() 
    {
        match sum.checked_add(i) 
        {
            Some(val) => 
            {
                sum = val;
            },

            None =>
            {
                return None;
            }
        }
    }

    return Some(sum);
}

// Return true when vector sum is zero.
#[allow(dead_code)]
pub fn is_zero(vector: &Vec<I32F32>) -> bool 
{
    return sum(&vector) == I32F32::from_num(0);
}

// Exp safe function with I32F32 output of I32F32 input.
#[allow(dead_code)]
pub fn exp_safe(input: I32F32) -> I32F32 
{
    let min_input:      I32F32 = I32F32::from_num(-20); // <= 1/exp(-20) = 485 165 195,4097903
    let max_input:      I32F32 = I32F32::from_num(20); // <= exp(20) = 485 165 195,4097903
    let mut safe_input: I32F32 = input;

    if input < min_input 
    {
        safe_input = min_input;
    } 
    else if max_input < input 
    {
        safe_input = max_input;
    }

    let output: I32F32;
    match exp(safe_input) 
    {
        Ok(val) => 
        {
            output = val;
        }
        Err(_err) => 
        {
            if safe_input <= 0 
            {
                output = I32F32::from_num(0);
            } 
            else 
            {
                output = I32F32::max_value();
            }
        }
    }

    return output;
}

// Sigmoid safe function with I32F32 output of I32F32 input with offset kappa and (recommended) scaling 0 < rho <= 40.
#[allow(dead_code)]
pub fn sigmoid_safe(input: I32F32, rho: I32F32, kappa: I32F32) -> I32F32 
{
    let one:            I32F32 = I32F32::from_num(1);
    let offset:         I32F32 = input.saturating_sub(kappa); // (input - kappa)
    let neg_rho:        I32F32 = rho.saturating_mul(-one); // -rho
    let exp_input:      I32F32 = neg_rho.saturating_mul(offset); // -rho*(input-kappa)
    let exp_output:     I32F32 = exp_safe(exp_input); // exp(-rho*(input-kappa))
    let denominator:    I32F32 = exp_output.saturating_add(one); // 1 + exp(-rho*(input-kappa))
    let sigmoid_output: I32F32 = one.saturating_div(denominator); // 1 / (1 + exp(-rho*(input-kappa)))
    
    return sigmoid_output;
}

// Returns a bool vector where an item is true if the vector item is in topk values.
#[allow(dead_code)]
pub fn is_topk(vector: &Vec<I32F32>, k: usize) -> Vec<bool> 
{
    let n:          usize       = vector.len();
    let mut result: Vec<bool>   = vec![true; n];
    if n < k 
    {
        return result;
    }

    let mut idxs: Vec<usize> = (0..n).collect();
    idxs.sort_by_key(|&idx| &vector[idx]); // ascending stable sort

    for &idx in &idxs[0..(n - k)] 
    {
        result[idx] = false;
    }

    return result;
}

// Returns a normalized (sum to 1 except 0) copy of the input vector.
#[allow(dead_code)]
pub fn normalize(x: &Vec<I32F32>) -> Vec<I32F32> 
{
    let x_sum: I32F32 = sum(x);
    if x_sum != I32F32::from_num(0.0 as f32) 
    {
        return x.iter()
                .map(|xi| 
                    xi / x_sum
                ).collect();
    } 
    else 
    {
        return x.clone();
    }
}

// Normalizes (sum to 1 except 0) the input vector directly in-place.
#[allow(dead_code)]
pub fn inplace_normalize(x: &mut Vec<I32F32>) 
{
    let x_sum: I32F32 = x.iter().sum();
    if x_sum == I32F32::from_num(0.0 as f32) 
    {
        return;
    }

    for i in 0..x.len() 
    {
        x[i] = x[i] / x_sum;
    }
}

// Normalizes (sum to 1 except 0) the input vector directly in-place, using the sum arg.
#[allow(dead_code)]
pub fn inplace_normalize_using_sum(x: &mut Vec<I32F32>, x_sum: I32F32) 
{
    if x_sum == I32F32::from_num(0.0 as f32) 
    {
        return;
    }

    for i in 0..x.len() 
    {
        x[i] = x[i] / x_sum;
    }
}

// Normalizes (sum to 1 except 0) the I64F64 input vector directly in-place.
#[allow(dead_code)]
pub fn inplace_normalize_64(x: &mut Vec<I64F64>) 
{
    let x_sum: I64F64 = x.iter().sum();
    if x_sum == I64F64::from_num(0) 
    {
        return;
    }

    for i in 0..x.len() 
    {
        x[i] = x[i] / x_sum;
    }
}

/// Returns x / y for input vectors x and y, if y == 0 return 0.
#[allow(dead_code)]
pub fn vecdiv(x: &Vec<I32F32>, y: &Vec<I32F32>) -> Vec<I32F32> 
{
    assert_eq!(x.len(), y.len());

    let n:          usize       = x.len();
    let mut result: Vec<I32F32> = vec![I32F32::from_num(0); n];

    for i in 0..n 
    {
        if y[i] != 0 
        {
            result[i] = x[i] / y[i];
        }
    }

    return result;
}