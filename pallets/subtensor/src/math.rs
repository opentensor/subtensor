use
{
    sp_std::
    {
        vec,
        vec::
        {
            Vec
        }
    },
    sp_runtime::
    {
        traits::
        {
            CheckedAdd
        }
    },
    substrate_fixed::
    {
        transcendental::
        {
            exp
        },
        types::
        {
            I32F32,
            I64F64
        }
    }
};

#[allow(dead_code)]
pub fn fixed(val: f32) -> I32F32 
{
    return I32F32::from_num(val);
}

#[allow(dead_code)]
pub fn fixed_to_u16(x: I32F32) -> u16
{
    return x.to_num::<u16>();
}

#[allow(dead_code)]
pub fn fixed_to_u64(x: I32F32) -> u64 
{
    return x.to_num::<u64>();
}

#[allow(dead_code)]
pub fn fixed64_to_u64(x: I64F64) -> u64 
{
    return x.to_num::<u64>();
}

#[allow(dead_code)]
pub fn fixed64_to_fixed32(x: I64F64) -> I32F32 
{
    return I32F32::from_num(x);
}

#[allow(dead_code)]
pub fn fixed32_to_fixed64(x: I32F32) -> I64F64 
{
    return I64F64::from_num(x);
}

#[allow(dead_code)]
pub fn u16_to_fixed(x: u16) -> I32F32 
{
    return I32F32::from_num(x);
}

#[allow(dead_code)]
pub fn u16_proportion_to_fixed(x: u16) -> I32F32 
{
    return I32F32::from_num(x) / I32F32::from_num(u16::MAX);
}

#[allow(dead_code)]
pub fn fixed_proportion_to_u16(x: I32F32) -> u16 
{
    return fixed_to_u16(x * I32F32::from_num(u16::MAX));
}

include!("math_vec.rs");
include!("math_mat.rs");

// Stake-weighted median score finding algorithm, based on a mid pivot binary search.
// Normally a random pivot is used, but to ensure full determinism the mid point is chosen instead.
// Assumes relatively random score order for efficiency, typically less than O(nlogn) complexity.
//
// # Args:
// 	* 'stake': ( &Vec<I32F32> ):
//         - stake, assumed to be normalized.
//
// 	* 'score': ( &Vec<I32F32> ):
//         - score for which median is sought, 0 <= score <= 1
//
// 	* 'partition_idx' ( &Vec<usize> ):
// 		- indices as input partition
//
// 	* 'minority' ( I32F32 ):
// 		- minority_ratio = 1 - majority_ratio
//
// 	* 'partition_lo' ( I32F32 ):
// 		- lower edge of stake for partition, where partition is a segment [lo, hi] inside stake integral [0, 1].
//
// 	* 'partition_hi' ( I32F32 ):
// 		- higher edge of stake for partition, where partition is a segment [lo, hi] inside stake integral [0, 1].
//
// # Returns:
//     * 'median': ( I32F32 ):
//         - median via random pivot binary search.
//
#[allow(dead_code)]
pub fn weighted_median(stake: &Vec<I32F32>, score: &Vec<I32F32>, partition_idx: &Vec<usize>, minority: I32F32, partition_lo: I32F32, partition_hi: I32F32,) 
    -> I32F32 
{
    let n = partition_idx.len();
    if n == 0 
    {
        return I32F32::from_num(0);
    }

    if n == 1 
    {
        return score[partition_idx[0]];
    }

    assert!(stake.len() == score.len());

    let mid_idx:        usize       = n / 2;
    let pivot:          I32F32      = score[partition_idx[mid_idx]];
    let mut lo_stake:   I32F32      = I32F32::from_num(0);
    let mut hi_stake:   I32F32      = I32F32::from_num(0);
    let mut lower:      Vec<usize>  = vec![];
    let mut upper:      Vec<usize>  = vec![];

    for &idx in partition_idx.iter() 
    {
        if score[idx] == pivot 
        {
            continue;
        }

        if score[idx] < pivot 
        {
            lo_stake += stake[idx];
            lower.push(idx);
        } 
        else 
        {
            hi_stake += stake[idx];
            upper.push(idx);
        }
    }

    if (partition_lo + lo_stake <= minority) && (minority < partition_hi - hi_stake) 
    {
        return pivot;
    }
    else if (minority < partition_lo + lo_stake) && (lower.len() > 0) 
    {
        return weighted_median(
            stake,
            score,
            &lower,
            minority,
            partition_lo,
            partition_lo + lo_stake,
        );
    } 
    else if (partition_hi - hi_stake <= minority) && (upper.len() > 0) 
    {
        return weighted_median(
            stake,
            score,
            &upper,
            minority,
            partition_hi - hi_stake,
            partition_hi,
        );
    }

    return pivot;
}

/// Column-wise weighted median, e.g. stake-weighted median scores per server (column) over all validators (rows).
#[allow(dead_code)]
pub fn weighted_median_col(stake: &Vec<I32F32>, score: &Vec<Vec<I32F32>>, majority: I32F32) -> Vec<I32F32> 
{
    let rows:       usize       = stake.len();
    let columns:    usize       = score[0].len();
    let zero:       I32F32      = I32F32::from_num(0);
    let mut median: Vec<I32F32> = vec![zero; columns];
    for c in 0..columns 
    {
        let mut use_stake: Vec<I32F32> = vec![];
        let mut use_score: Vec<I32F32> = vec![];
        for r in 0..rows 
        {
            assert_eq!(columns, score[r].len());
            if stake[r] > zero 
            {
                use_stake.push(stake[r]);
                use_score.push(score[r][c]);
            }
        }
        if use_stake.len() > 0 
        {
            inplace_normalize(&mut use_stake);

            let stake_sum:  I32F32 = use_stake.iter().sum();
            let minority:   I32F32 = stake_sum - majority;

            median[c] = weighted_median(
                &use_stake,
                &use_score,
                &(0..use_stake.len()).collect(),
                minority,
                zero,
                stake_sum,
            );
        }
    }

    return median;
}

/// Column-wise weighted median, e.g. stake-weighted median scores per server (column) over all validators (rows).
#[allow(dead_code)]
pub fn weighted_median_col_sparse(stake: &Vec<I32F32>, score: &Vec<Vec<(u16, I32F32)>>, columns: u16, majority: I32F32) -> Vec<I32F32> 
{
    let rows:           usize               = stake.len();
    let zero:           I32F32              = I32F32::from_num(0);
    let mut use_stake:  Vec<I32F32>         = stake.iter().copied().filter(|&s| s > zero).collect();
    inplace_normalize(&mut use_stake);
    let stake_sum:      I32F32              = use_stake.iter().sum();
    let stake_idx:      Vec<usize>          = (0..use_stake.len()).collect();
    let minority:       I32F32              = stake_sum - majority;
    let mut use_score:  Vec<Vec<I32F32>>    = vec![vec![zero; use_stake.len()]; columns as usize];
    let mut median:     Vec<I32F32>         = vec![zero; columns as usize];
    let mut k:          usize               = 0;

    for r in 0..rows 
    {
        if stake[r] <= zero
        {
            continue;
        }

        for (c, val) in score[r].iter() 
        {
            use_score[*c as usize][k] = *val;
        }

        k += 1;
    }

    for c in 0..columns as usize 
    {
        median[c] = weighted_median(
            &use_stake,
            &use_score[c],
            &stake_idx,
            minority,
            zero,
            stake_sum,
        );
    }

    return median;
}

#[cfg(test)]
mod tests {
    use crate::math::*;
    use rand::{seq::SliceRandom, thread_rng, Rng};
    use substrate_fixed::transcendental::exp;
    use substrate_fixed::types::{I110F18, I32F32, I64F64, I96F32};

    fn assert_float_compare(a: I32F32, b: I32F32, epsilon: I32F32) {
        assert!(I32F32::abs(a - b) <= epsilon, "a({:?}) != b({:?})", a, b);
    }

    fn assert_float_compare_64(a: I64F64, b: I64F64, epsilon: I64F64) {
        assert!(I64F64::abs(a - b) <= epsilon, "a({:?}) != b({:?})", a, b);
    }

    fn assert_vec_compare(va: &Vec<I32F32>, vb: &Vec<I32F32>, epsilon: I32F32) {
        assert!(va.len() == vb.len());
        for i in 0..va.len() {
            assert_float_compare(va[i], vb[i], epsilon);
        }
    }

    fn assert_vec_compare_64(va: &Vec<I64F64>, vb: &Vec<I64F64>, epsilon: I64F64) {
        assert!(va.len() == vb.len());
        for i in 0..va.len() {
            assert_float_compare_64(va[i], vb[i], epsilon);
        }
    }

    fn assert_vec_compare_u16(va: &Vec<u16>, vb: &Vec<u16>) {
        assert!(va.len() == vb.len());
        for i in 0..va.len() {
            assert_eq!(va[i], vb[i]);
        }
    }

    fn assert_mat_compare(ma: &Vec<Vec<I32F32>>, mb: &Vec<Vec<I32F32>>, epsilon: I32F32) {
        assert!(ma.len() == mb.len());
        for row in 0..ma.len() {
            assert!(ma[row].len() == mb[row].len());
            for col in 0..ma[row].len() {
                assert_float_compare(ma[row][col], mb[row][col], epsilon)
            }
        }
    }

    fn assert_sparse_mat_compare(
        ma: &Vec<Vec<(u16, I32F32)>>,
        mb: &Vec<Vec<(u16, I32F32)>>,
        epsilon: I32F32,
    ) {
        assert!(ma.len() == mb.len());
        for row in 0..ma.len() {
            assert!(ma[row].len() == mb[row].len());
            for j in 0..ma[row].len() {
                assert!(ma[row][j].0 == mb[row][j].0); // u16
                assert_float_compare(ma[row][j].1, mb[row][j].1, epsilon) // I32F32
            }
        }
    }

    fn vec_to_fixed(vector: &Vec<f32>) -> Vec<I32F32> {
        vector.iter().map(|x| I32F32::from_num(*x)).collect()
    }

    #[test]
    fn test_vec_max_upscale_to_u16() {
        let vector: Vec<I32F32> = vec_to_fixed(&vec![]);
        let target: Vec<u16> = vec![];
        let result: Vec<u16> = vec_max_upscale_to_u16(&vector);
        assert_vec_compare_u16(&result, &target);
        let vector: Vec<I32F32> = vec_to_fixed(&vec![0.]);
        let target: Vec<u16> = vec![0];
        let result: Vec<u16> = vec_max_upscale_to_u16(&vector);
        assert_vec_compare_u16(&result, &target);
        let vector: Vec<I32F32> = vec_to_fixed(&vec![0., 0.]);
        let target: Vec<u16> = vec![0, 0];
        let result: Vec<u16> = vec_max_upscale_to_u16(&vector);
        assert_vec_compare_u16(&result, &target);
        let vector: Vec<I32F32> = vec_to_fixed(&vec![0., 1.]);
        let target: Vec<u16> = vec![0, 65535];
        let result: Vec<u16> = vec_max_upscale_to_u16(&vector);
        assert_vec_compare_u16(&result, &target);
        let vector: Vec<I32F32> = vec_to_fixed(&vec![0., 0.000000001]);
        let target: Vec<u16> = vec![0, 65535];
        let result: Vec<u16> = vec_max_upscale_to_u16(&vector);
        assert_vec_compare_u16(&result, &target);
        let vector: Vec<I32F32> = vec_to_fixed(&vec![0., 0.000016, 1.]);
        let target: Vec<u16> = vec![0, 1, 65535];
        let result: Vec<u16> = vec_max_upscale_to_u16(&vector);
        assert_vec_compare_u16(&result, &target);
        let vector: Vec<I32F32> = vec_to_fixed(&vec![0.000000001, 0.000000001]);
        let target: Vec<u16> = vec![65535, 65535];
        let result: Vec<u16> = vec_max_upscale_to_u16(&vector);
        assert_vec_compare_u16(&result, &target);
        let vector: Vec<I32F32> = vec_to_fixed(&vec![
            0.000001, 0.000006, 0.000007, 0.0001, 0.001, 0.01, 0.1, 0.2, 0.3, 0.4,
        ]);
        let target: Vec<u16> = vec![0, 1, 1, 16, 164, 1638, 16384, 32768, 49151, 65535];
        let result: Vec<u16> = vec_max_upscale_to_u16(&vector);
        assert_vec_compare_u16(&result, &target);
        let vector: Vec<I32F32> = vec![I32F32::from_num(16384)];
        let target: Vec<u16> = vec![65535];
        let result: Vec<u16> = vec_max_upscale_to_u16(&vector);
        assert_vec_compare_u16(&result, &target);
        let vector: Vec<I32F32> = vec![I32F32::from_num(32768)];
        let target: Vec<u16> = vec![65535];
        let result: Vec<u16> = vec_max_upscale_to_u16(&vector);
        assert_vec_compare_u16(&result, &target);
        let vector: Vec<I32F32> = vec![I32F32::from_num(32769)];
        let target: Vec<u16> = vec![65535];
        let result: Vec<u16> = vec_max_upscale_to_u16(&vector);
        assert_vec_compare_u16(&result, &target);
        let vector: Vec<I32F32> = vec![I32F32::from_num(65535)];
        let target: Vec<u16> = vec![65535];
        let result: Vec<u16> = vec_max_upscale_to_u16(&vector);
        assert_vec_compare_u16(&result, &target);
        let vector: Vec<I32F32> = vec![I32F32::max_value()];
        let target: Vec<u16> = vec![65535];
        let result: Vec<u16> = vec_max_upscale_to_u16(&vector);
        assert_vec_compare_u16(&result, &target);
        let vector: Vec<I32F32> = vec_to_fixed(&vec![0., 1., 65535.]);
        let target: Vec<u16> = vec![0, 1, 65535];
        let result: Vec<u16> = vec_max_upscale_to_u16(&vector);
        assert_vec_compare_u16(&result, &target);
        let vector: Vec<I32F32> = vec_to_fixed(&vec![0., 0.5, 1., 1.5, 2., 32768.]);
        let target: Vec<u16> = vec![0, 1, 2, 3, 4, 65535];
        let result: Vec<u16> = vec_max_upscale_to_u16(&vector);
        assert_vec_compare_u16(&result, &target);
        let vector: Vec<I32F32> = vec_to_fixed(&vec![0., 0.5, 1., 1.5, 2., 32768., 32769.]);
        let target: Vec<u16> = vec![0, 1, 2, 3, 4, 65533, 65535];
        let result: Vec<u16> = vec_max_upscale_to_u16(&vector);
        assert_vec_compare_u16(&result, &target);
        let vector: Vec<I32F32> = vec![
            I32F32::from_num(0),
            I32F32::from_num(1),
            I32F32::from_num(32768),
            I32F32::from_num(32769),
            I32F32::max_value(),
        ];
        let target: Vec<u16> = vec![0, 0, 1, 1, 65535];
        let result: Vec<u16> = vec_max_upscale_to_u16(&vector);
        assert_vec_compare_u16(&result, &target);
    }

    #[test]
    fn test_vec_u16_max_upscale_to_u16() {
        let vector: Vec<u16> = vec![];
        let result: Vec<u16> = vec_u16_max_upscale_to_u16(&vector);
        assert_vec_compare_u16(&result, &vector);
        let vector: Vec<u16> = vec![0];
        let result: Vec<u16> = vec_u16_max_upscale_to_u16(&vector);
        assert_vec_compare_u16(&result, &vector);
        let vector: Vec<u16> = vec![0, 0];
        let result: Vec<u16> = vec_u16_max_upscale_to_u16(&vector);
        assert_vec_compare_u16(&result, &vector);
        let vector: Vec<u16> = vec![1];
        let target: Vec<u16> = vec![65535];
        let result: Vec<u16> = vec_u16_max_upscale_to_u16(&vector);
        assert_vec_compare_u16(&result, &target);
        let vector: Vec<u16> = vec![0, 1];
        let target: Vec<u16> = vec![0, 65535];
        let result: Vec<u16> = vec_u16_max_upscale_to_u16(&vector);
        assert_vec_compare_u16(&result, &target);
        let vector: Vec<u16> = vec![65534];
        let target: Vec<u16> = vec![65535];
        let result: Vec<u16> = vec_u16_max_upscale_to_u16(&vector);
        assert_vec_compare_u16(&result, &target);
        let vector: Vec<u16> = vec![65535];
        let target: Vec<u16> = vec![65535];
        let result: Vec<u16> = vec_u16_max_upscale_to_u16(&vector);
        assert_vec_compare_u16(&result, &target);
        let vector: Vec<u16> = vec![65535, 65535];
        let target: Vec<u16> = vec![65535, 65535];
        let result: Vec<u16> = vec_u16_max_upscale_to_u16(&vector);
        assert_vec_compare_u16(&result, &target);
        let vector: Vec<u16> = vec![0, 1, 65534];
        let target: Vec<u16> = vec![0, 1, 65535];
        let result: Vec<u16> = vec_u16_max_upscale_to_u16(&vector);
        assert_vec_compare_u16(&result, &target);
        let vector: Vec<u16> = vec![0, 1, 2, 3, 4, 65533, 65535];
        let result: Vec<u16> = vec_u16_max_upscale_to_u16(&vector);
        assert_vec_compare_u16(&result, &vector);
    }

    #[test]
    fn test_check_vec_max_limited() {
        let vector: Vec<u16> = vec![];
        let max_limit: u16 = 0;
        assert!(check_vec_max_limited(&vector, max_limit));
        let vector: Vec<u16> = vec![];
        let max_limit: u16 = u16::MAX;
        assert!(check_vec_max_limited(&vector, max_limit));
        let vector: Vec<u16> = vec![u16::MAX];
        let max_limit: u16 = u16::MAX;
        assert!(check_vec_max_limited(&vector, max_limit));
        let vector: Vec<u16> = vec![u16::MAX];
        let max_limit: u16 = u16::MAX - 1;
        assert!(!check_vec_max_limited(&vector, max_limit));
        let vector: Vec<u16> = vec![u16::MAX];
        let max_limit: u16 = 0;
        assert!(!check_vec_max_limited(&vector, max_limit));
        let vector: Vec<u16> = vec![0];
        let max_limit: u16 = u16::MAX;
        assert!(check_vec_max_limited(&vector, max_limit));
        let vector: Vec<u16> = vec![0, u16::MAX];
        let max_limit: u16 = u16::MAX;
        assert!(check_vec_max_limited(&vector, max_limit));
        let vector: Vec<u16> = vec![0, u16::MAX, u16::MAX];
        let max_limit: u16 = u16::MAX / 2;
        assert!(!check_vec_max_limited(&vector, max_limit));
        let vector: Vec<u16> = vec![0, u16::MAX, u16::MAX];
        let max_limit: u16 = u16::MAX / 2 + 1;
        assert!(check_vec_max_limited(&vector, max_limit));
        let vector: Vec<u16> = vec![0, u16::MAX, u16::MAX, u16::MAX];
        let max_limit: u16 = u16::MAX / 3 - 1;
        assert!(!check_vec_max_limited(&vector, max_limit));
        let vector: Vec<u16> = vec![0, u16::MAX, u16::MAX, u16::MAX];
        let max_limit: u16 = u16::MAX / 3;
        assert!(check_vec_max_limited(&vector, max_limit));
    }

    #[test]
    fn test_math_fixed_overflow() {
        let max_32: I32F32 = I32F32::max_value();
        let max_u64: u64 = u64::MAX;
        let _prod_96: I96F32 = I96F32::from_num(max_32) * I96F32::from_num(max_u64);
        // let one: I96F32 = I96F32::from_num(1);
        // let prod_96: I96F32 = (I96F32::from_num(max_32) + one) * I96F32::from_num(max_u64); // overflows
        let _prod_110: I110F18 = I110F18::from_num(max_32) * I110F18::from_num(max_u64);

        let bonds_moving_average_val: u64 = 900_000 as u64;
        let bonds_moving_average: I64F64 =
            I64F64::from_num(bonds_moving_average_val) / I64F64::from_num(1_000_000);
        let alpha: I32F32 = I32F32::from_num(1) - I32F32::from_num(bonds_moving_average);
        assert_eq!(I32F32::from_num(0.1), alpha);

        let bonds_moving_average: I64F64 = I64F64::from_num(max_32) / I64F64::from_num(max_32);
        let alpha: I32F32 = I32F32::from_num(1) - I32F32::from_num(bonds_moving_average);
        assert_eq!(I32F32::from_num(0), alpha);
    }

    #[test]
    fn test_math_u64_normalization() {
        let min: u64 = 1;
        let min32: u64 = 4_889_444; // 21_000_000_000_000_000 / 4_294_967_296
        let mid: u64 = 10_500_000_000_000_000;
        let max: u64 = 21_000_000_000_000_000;
        let min_64: I64F64 = I64F64::from_num(min);
        let min32_64: I64F64 = I64F64::from_num(min32);
        let mid_64: I64F64 = I64F64::from_num(mid);
        let max_64: I64F64 = I64F64::from_num(max);
        let max_sum: I64F64 = I64F64::from_num(max);
        let min_frac: I64F64 = min_64 / max_sum;
        assert_eq!(min_frac, I64F64::from_num(0.0000000000000000476));
        let min_frac_32: I32F32 = I32F32::from_num(min_frac);
        assert_eq!(min_frac_32, I32F32::from_num(0));
        let min32_frac: I64F64 = min32_64 / max_sum;
        assert_eq!(min32_frac, I64F64::from_num(0.00000000023283066664));
        let min32_frac_32: I32F32 = I32F32::from_num(min32_frac);
        assert_eq!(min32_frac_32, I32F32::from_num(0.0000000002));
        let half: I64F64 = mid_64 / max_sum;
        assert_eq!(half, I64F64::from_num(0.5));
        let half_32: I32F32 = I32F32::from_num(half);
        assert_eq!(half_32, I32F32::from_num(0.5));
        let one: I64F64 = max_64 / max_sum;
        assert_eq!(one, I64F64::from_num(1));
        let one_32: I32F32 = I32F32::from_num(one);
        assert_eq!(one_32, I32F32::from_num(1));
    }

    #[test]
    fn test_math_to_num() {
        let val: I32F32 = I32F32::from_num(u16::MAX);
        let res: u16 = val.to_num::<u16>();
        assert_eq!(res, u16::MAX);
        let vector: Vec<I32F32> = vec![val; 1000];
        let target: Vec<u16> = vec![u16::MAX; 1000];
        let output: Vec<u16> = vector.iter().map(|e: &I32F32| e.to_num::<u16>()).collect();
        assert_eq!(output, target);
        let output: Vec<u16> = vector
            .iter()
            .map(|e: &I32F32| (*e).to_num::<u16>())
            .collect();
        assert_eq!(output, target);
        let val: I32F32 = I32F32::max_value();
        let res: u64 = val.to_num::<u64>();
        let vector: Vec<I32F32> = vec![val; 1000];
        let target: Vec<u64> = vec![res; 1000];
        let output: Vec<u64> = vector.iter().map(|e: &I32F32| e.to_num::<u64>()).collect();
        assert_eq!(output, target);
        let output: Vec<u64> = vector
            .iter()
            .map(|e: &I32F32| (*e).to_num::<u64>())
            .collect();
        assert_eq!(output, target);
        let val: I32F32 = I32F32::from_num(0);
        let res: u64 = val.to_num::<u64>();
        let vector: Vec<I32F32> = vec![val; 1000];
        let target: Vec<u64> = vec![res; 1000];
        let output: Vec<u64> = vector.iter().map(|e: &I32F32| e.to_num::<u64>()).collect();
        assert_eq!(output, target);
        let output: Vec<u64> = vector
            .iter()
            .map(|e: &I32F32| (*e).to_num::<u64>())
            .collect();
        assert_eq!(output, target);
        let val: I96F32 = I96F32::from_num(u64::MAX);
        let res: u64 = val.to_num::<u64>();
        assert_eq!(res, u64::MAX);
        let vector: Vec<I96F32> = vec![val; 1000];
        let target: Vec<u64> = vec![u64::MAX; 1000];
        let output: Vec<u64> = vector.iter().map(|e: &I96F32| e.to_num::<u64>()).collect();
        assert_eq!(output, target);
        let output: Vec<u64> = vector
            .iter()
            .map(|e: &I96F32| (*e).to_num::<u64>())
            .collect();
        assert_eq!(output, target);
    }

    #[test]
    fn test_math_vec_to_fixed() {
        let vector: Vec<f32> = vec![0., 1., 2., 3.];
        let target: Vec<I32F32> = vec![
            I32F32::from_num(0.),
            I32F32::from_num(1.),
            I32F32::from_num(2.),
            I32F32::from_num(3.),
        ];
        let result = vec_to_fixed(&vector);
        assert_vec_compare(&result, &target, I32F32::from_num(0));
    }

    // Reshape vector to matrix with specified number of rows, cast to I32F32.
    fn vec_to_mat_fixed(vector: &Vec<f32>, rows: usize, transpose: bool) -> Vec<Vec<I32F32>> {
        assert!(
            vector.len() % rows == 0,
            "Vector of len {:?} cannot reshape to {rows} rows.",
            vector.len()
        );
        let cols: usize = vector.len() / rows;
        let mut mat: Vec<Vec<I32F32>> = vec![];
        if transpose {
            for col in 0..cols as usize {
                let mut vals: Vec<I32F32> = vec![];
                for row in 0..rows as usize {
                    vals.push(I32F32::from_num(vector[row * cols + col]));
                }
                mat.push(vals);
            }
        } else {
            for row in 0..rows as usize {
                mat.push(
                    vector[row * cols..(row + 1) * cols]
                        .iter()
                        .map(|v| I32F32::from_num(*v))
                        .collect(),
                );
            }
        }
        mat
    }

    #[test]
    fn test_math_vec_to_mat_fixed() {
        let vector: Vec<f32> = vec![0., 1., 2., 0., 10., 100.];
        let target: Vec<Vec<I32F32>> = vec![
            vec![
                I32F32::from_num(0.),
                I32F32::from_num(1.),
                I32F32::from_num(2.),
            ],
            vec![
                I32F32::from_num(0.),
                I32F32::from_num(10.),
                I32F32::from_num(100.),
            ],
        ];
        let mat = vec_to_mat_fixed(&vector, 2, false);
        assert_mat_compare(&mat, &target, I32F32::from_num(0));
    }

    // Reshape vector to sparse matrix with specified number of input rows, cast f32 to I32F32.
    fn vec_to_sparse_mat_fixed(
        vector: &Vec<f32>,
        rows: usize,
        transpose: bool,
    ) -> Vec<Vec<(u16, I32F32)>> {
        assert!(
            vector.len() % rows == 0,
            "Vector of len {:?} cannot reshape to {rows} rows.",
            vector.len()
        );
        let cols: usize = vector.len() / rows;
        let mut mat: Vec<Vec<(u16, I32F32)>> = vec![];
        if transpose {
            for col in 0..cols as usize {
                let mut row_vec: Vec<(u16, I32F32)> = vec![];
                for row in 0..rows as usize {
                    if vector[row * cols + col] > 0. {
                        row_vec.push((row as u16, I32F32::from_num(vector[row * cols + col])));
                    }
                }
                mat.push(row_vec);
            }
        } else {
            for row in 0..rows as usize {
                let mut row_vec: Vec<(u16, I32F32)> = vec![];
                for col in 0..cols as usize {
                    if vector[row * cols + col] > 0. {
                        row_vec.push((col as u16, I32F32::from_num(vector[row * cols + col])));
                    }
                }
                mat.push(row_vec);
            }
        }
        mat
    }

    #[test]
    fn test_math_vec_to_sparse_mat_fixed() {
        let vector: Vec<f32> = vec![0., 1., 2., 0., 10., 100.];
        let target: Vec<Vec<(u16, I32F32)>> = vec![
            vec![
                (1 as u16, I32F32::from_num(1.)),
                (2 as u16, I32F32::from_num(2.)),
            ],
            vec![
                (1 as u16, I32F32::from_num(10.)),
                (2 as u16, I32F32::from_num(100.)),
            ],
        ];
        let mat = vec_to_sparse_mat_fixed(&vector, 2, false);
        assert_sparse_mat_compare(&mat, &target, I32F32::from_num(0));
        let vector: Vec<f32> = vec![0., 0.];
        let target: Vec<Vec<(u16, I32F32)>> = vec![vec![], vec![]];
        let mat = vec_to_sparse_mat_fixed(&vector, 2, false);
        assert_sparse_mat_compare(&mat, &target, I32F32::from_num(0));
        let vector: Vec<f32> = vec![0., 1., 2., 0., 10., 100.];
        let target: Vec<Vec<(u16, I32F32)>> = vec![
            vec![],
            vec![
                (0 as u16, I32F32::from_num(1.)),
                (1 as u16, I32F32::from_num(10.)),
            ],
            vec![
                (0 as u16, I32F32::from_num(2.)),
                (1 as u16, I32F32::from_num(100.)),
            ],
        ];
        let mat = vec_to_sparse_mat_fixed(&vector, 2, true);
        assert_sparse_mat_compare(&mat, &target, I32F32::from_num(0));
        let vector: Vec<f32> = vec![0., 0.];
        let target: Vec<Vec<(u16, I32F32)>> = vec![vec![]];
        let mat = vec_to_sparse_mat_fixed(&vector, 2, true);
        assert_sparse_mat_compare(&mat, &target, I32F32::from_num(0));
    }

    #[test]
    fn test_math_exp_safe() {
        let zero: I32F32 = I32F32::from_num(0);
        let one: I32F32 = I32F32::from_num(1);
        let target: I32F32 = exp(zero).unwrap();
        assert_eq!(exp_safe(zero), target);
        let target: I32F32 = exp(one).unwrap();
        assert_eq!(exp_safe(one), target);
        let min_input: I32F32 = I32F32::from_num(-20); // <= 1/exp(-20) = 485 165 195,4097903
        let max_input: I32F32 = I32F32::from_num(20); // <= exp(20) = 485 165 195,4097903
        let target: I32F32 = exp(min_input).unwrap();
        assert_eq!(exp_safe(min_input), target);
        assert_eq!(exp_safe(min_input - one), target);
        assert_eq!(exp_safe(I32F32::min_value()), target);
        let target: I32F32 = exp(max_input).unwrap();
        assert_eq!(exp_safe(max_input), target);
        assert_eq!(exp_safe(max_input + one), target);
        assert_eq!(exp_safe(I32F32::max_value()), target);
    }

    #[test]
    fn test_math_sigmoid_safe() {
        let trust: Vec<I32F32> = vec![
            I32F32::min_value(),
            I32F32::from_num(0),
            I32F32::from_num(0.4),
            I32F32::from_num(0.5),
            I32F32::from_num(0.6),
            I32F32::from_num(1),
            I32F32::max_value(),
        ];
        let consensus: Vec<I32F32> = trust
            .iter()
            .map(|t: &I32F32| sigmoid_safe(*t, I32F32::max_value(), I32F32::max_value()))
            .collect();
        let target: Vec<I32F32> = vec_to_fixed(&vec![
            0.0000000019,
            0.0000000019,
            0.0000000019,
            0.0000000019,
            0.0000000019,
            0.0000000019,
            0.5,
        ]);
        assert_eq!(&consensus, &target);
        let consensus: Vec<I32F32> = trust
            .iter()
            .map(|t: &I32F32| sigmoid_safe(*t, I32F32::min_value(), I32F32::min_value()))
            .collect();
        let target: Vec<I32F32> = vec_to_fixed(&vec![
            0.5,
            0.0000000019,
            0.0000000019,
            0.0000000019,
            0.0000000019,
            0.0000000019,
            0.0000000019,
        ]);
        assert_eq!(&consensus, &target);
        let consensus: Vec<I32F32> = trust
            .iter()
            .map(|t: &I32F32| sigmoid_safe(*t, I32F32::from_num(30), I32F32::from_num(0.5)))
            .collect();
        let target: Vec<f64> = vec![
            0.0000000019,
            0.0000003057,
            0.0474258729,
            0.5,
            0.952574127,
            0.9999996943,
            0.9999999981,
        ];
        let target: Vec<I32F32> = target.iter().map(|c: &f64| I32F32::from_num(*c)).collect();
        assert_eq!(&consensus, &target);
        let trust: Vec<I32F32> =
            vec_to_fixed(&vec![0., 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.]);
        let consensus: Vec<I32F32> = trust
            .iter()
            .map(|t: &I32F32| sigmoid_safe(*t, I32F32::from_num(40), I32F32::from_num(0.5)))
            .collect();
        let target: Vec<f64> = vec![
            0.0000000019,
            0.0000001125,
            0.0000061442,
            0.0003353502,
            0.017986214,
            0.5,
            0.9820138067,
            0.9996646498,
            0.9999938558,
            0.9999998875,
            0.9999999981,
        ];
        let target: Vec<I32F32> = target.iter().map(|c: &f64| I32F32::from_num(*c)).collect();
        assert_eq!(&consensus, &target);
    }

    #[test]
    fn test_math_is_topk() {
        let vector: Vec<I32F32> = vec_to_fixed(&vec![]);
        let result = is_topk(&vector, 5);
        let target: Vec<bool> = vec![];
        assert_eq!(&result, &target);
        let vector: Vec<I32F32> = vec_to_fixed(&vec![0., 1., 2., 3., 4., 5., 6., 7., 8., 9.]);
        let result = is_topk(&vector, 0);
        let target: Vec<bool> = vec![
            false, false, false, false, false, false, false, false, false, false,
        ];
        assert_eq!(&result, &target);
        let result = is_topk(&vector, 5);
        let target: Vec<bool> = vec![
            false, false, false, false, false, true, true, true, true, true,
        ];
        assert_eq!(&result, &target);
        let result = is_topk(&vector, 10);
        let target: Vec<bool> = vec![true, true, true, true, true, true, true, true, true, true];
        assert_eq!(&result, &target);
        let result = is_topk(&vector, 100);
        assert_eq!(&result, &target);
        let vector: Vec<I32F32> = vec_to_fixed(&vec![9., 8., 7., 6., 5., 4., 3., 2., 1., 0.]);
        let result = is_topk(&vector, 5);
        let target: Vec<bool> = vec![
            true, true, true, true, true, false, false, false, false, false,
        ];
        assert_eq!(&result, &target);
        let vector: Vec<I32F32> = vec_to_fixed(&vec![9., 0., 8., 1., 7., 2., 6., 3., 5., 4.]);
        let result = is_topk(&vector, 5);
        let target: Vec<bool> = vec![
            true, false, true, false, true, false, true, false, true, false,
        ];
        assert_eq!(&result, &target);
        let vector: Vec<I32F32> =
            vec_to_fixed(&vec![0.9, 0., 0.8, 0.1, 0.7, 0.2, 0.6, 0.3, 0.5, 0.4]);
        let result = is_topk(&vector, 5);
        let target: Vec<bool> = vec![
            true, false, true, false, true, false, true, false, true, false,
        ];
        assert_eq!(&result, &target);
        let vector: Vec<I32F32> = vec_to_fixed(&vec![0., 1., 2., 3., 4., 5., 5., 5., 5., 6.]);
        let result = is_topk(&vector, 5);
        let target: Vec<bool> = vec![
            false, false, false, false, false, true, true, true, true, true,
        ];
        assert_eq!(&result, &target);
    }

    #[test]
    fn test_math_sum() {
        assert!(sum(&vec![]) == I32F32::from_num(0));
        assert!(
            sum(&vec![
                I32F32::from_num(1.0),
                I32F32::from_num(10.0),
                I32F32::from_num(30.0)
            ]) == I32F32::from_num(41)
        );
        assert!(
            sum(&vec![
                I32F32::from_num(-1.0),
                I32F32::from_num(10.0),
                I32F32::from_num(30.0)
            ]) == I32F32::from_num(39)
        );
    }

    #[test]
    fn test_math_normalize() {
        let epsilon: I32F32 = I32F32::from_num(0.0001);
        let x: Vec<I32F32> = vec![];
        let y: Vec<I32F32> = normalize(&x);
        assert_vec_compare(&x, &y, epsilon);
        let x: Vec<I32F32> = vec![
            I32F32::from_num(1.0),
            I32F32::from_num(10.0),
            I32F32::from_num(30.0),
        ];
        let y: Vec<I32F32> = normalize(&x);
        assert_vec_compare(
            &y,
            &vec![
                I32F32::from_num(0.0243902437),
                I32F32::from_num(0.243902439),
                I32F32::from_num(0.7317073171),
            ],
            epsilon,
        );
        assert_float_compare(sum(&y), I32F32::from_num(1.0), epsilon);
        let x: Vec<I32F32> = vec![
            I32F32::from_num(-1.0),
            I32F32::from_num(10.0),
            I32F32::from_num(30.0),
        ];
        let y: Vec<I32F32> = normalize(&x);
        assert_vec_compare(
            &y,
            &vec![
                I32F32::from_num(-0.0256410255),
                I32F32::from_num(0.2564102563),
                I32F32::from_num(0.769230769),
            ],
            epsilon,
        );
        assert_float_compare(sum(&y), I32F32::from_num(1.0), epsilon);
    }

    #[test]
    fn test_math_inplace_normalize() {
        let epsilon: I32F32 = I32F32::from_num(0.0001);
        let mut x1: Vec<I32F32> = vec![
            I32F32::from_num(1.0),
            I32F32::from_num(10.0),
            I32F32::from_num(30.0),
        ];
        inplace_normalize(&mut x1);
        assert_vec_compare(
            &x1,
            &vec![
                I32F32::from_num(0.0243902437),
                I32F32::from_num(0.243902439),
                I32F32::from_num(0.7317073171),
            ],
            epsilon,
        );
        let mut x2: Vec<I32F32> = vec![
            I32F32::from_num(-1.0),
            I32F32::from_num(10.0),
            I32F32::from_num(30.0),
        ];
        inplace_normalize(&mut x2);
        assert_vec_compare(
            &x2,
            &vec![
                I32F32::from_num(-0.0256410255),
                I32F32::from_num(0.2564102563),
                I32F32::from_num(0.769230769),
            ],
            epsilon,
        );
    }

    #[test]
    fn test_math_inplace_normalize_64() {
        let epsilon: I64F64 = I64F64::from_num(0.0001);
        let mut x1: Vec<I64F64> = vec![
            I64F64::from_num(1.0),
            I64F64::from_num(10.0),
            I64F64::from_num(30.0),
        ];
        inplace_normalize_64(&mut x1);
        assert_vec_compare_64(
            &x1,
            &vec![
                I64F64::from_num(0.0243902437),
                I64F64::from_num(0.243902439),
                I64F64::from_num(0.7317073171),
            ],
            epsilon,
        );
        let mut x2: Vec<I64F64> = vec![
            I64F64::from_num(-1.0),
            I64F64::from_num(10.0),
            I64F64::from_num(30.0),
        ];
        inplace_normalize_64(&mut x2);
        assert_vec_compare_64(
            &x2,
            &vec![
                I64F64::from_num(-0.0256410255),
                I64F64::from_num(0.2564102563),
                I64F64::from_num(0.769230769),
            ],
            epsilon,
        );
    }

    #[test]
    fn test_math_vecdiv() {
        let x: Vec<I32F32> = vec_to_fixed(&vec![]);
        let y: Vec<I32F32> = vec_to_fixed(&vec![]);
        let result: Vec<I32F32> = vec_to_fixed(&vec![]);
        assert_eq!(result, vecdiv(&x, &y));

        let x: Vec<I32F32> = vec_to_fixed(&vec![0., 1., 0., 1.]);
        let y: Vec<I32F32> = vec_to_fixed(&vec![0., 1., 1., 0.]);
        let result: Vec<I32F32> = vec_to_fixed(&vec![0., 1., 0., 0.]);
        assert_eq!(result, vecdiv(&x, &y));

        let x: Vec<I32F32> = vec_to_fixed(&vec![1., 1., 10.]);
        let y: Vec<I32F32> = vec_to_fixed(&vec![2., 3., 2.]);
        let result: Vec<I32F32> = vec![fixed(1.) / fixed(2.), fixed(1.) / fixed(3.), fixed(5.)];
        assert_eq!(result, vecdiv(&x, &y));
    }

    #[test]
    fn test_math_inplace_row_normalize() {
        let epsilon: I32F32 = I32F32::from_num(0.0001);
        let vector: Vec<f32> = vec![
            0., 1., 2., 3., 4., 0., 10., 100., 1000., 10000., 0., 0., 0., 0., 0., 1., 1., 1., 1.,
            1.,
        ];
        let mut mat = vec_to_mat_fixed(&vector, 4, false);
        inplace_row_normalize(&mut mat);
        let target: Vec<f32> = vec![
            0., 0.1, 0.2, 0.3, 0.4, 0., 0.0009, 0.009, 0.09, 0.9, 0., 0., 0., 0., 0., 0.2, 0.2,
            0.2, 0.2, 0.2,
        ];
        assert_mat_compare(&mat, &vec_to_mat_fixed(&target, 4, false), epsilon);
    }

    #[test]
    fn test_math_inplace_row_normalize_sparse() {
        let epsilon: I32F32 = I32F32::from_num(0.0001);
        let vector: Vec<f32> = vec![
            0., 1., 0., 2., 0., 3., 4., 0., 1., 0., 2., 0., 3., 0., 1., 0., 0., 2., 0., 3., 4., 0.,
            10., 0., 100., 1000., 0., 10000., 0., 0., 0., 0., 0., 0., 0., 1., 1., 1., 1., 1., 1.,
            1.,
        ];
        let mut mat = vec_to_sparse_mat_fixed(&vector, 6, false);
        inplace_row_normalize_sparse(&mut mat);
        let target: Vec<f32> = vec![
            0., 0.1, 0., 0.2, 0., 0.3, 0.4, 0., 0.166666, 0., 0.333333, 0., 0.5, 0., 0.1, 0., 0.,
            0.2, 0., 0.3, 0.4, 0., 0.0009, 0., 0.009, 0.09, 0., 0.9, 0., 0., 0., 0., 0., 0., 0.,
            0.142857, 0.142857, 0.142857, 0.142857, 0.142857, 0.142857, 0.142857,
        ];
        assert_sparse_mat_compare(&mat, &vec_to_sparse_mat_fixed(&target, 6, false), epsilon);
        let vector: Vec<f32> = vec![0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0.];
        let target: Vec<f32> = vec![0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0.];
        let mut mat = vec_to_sparse_mat_fixed(&vector, 3, false);
        inplace_row_normalize_sparse(&mut mat);
        assert_sparse_mat_compare(
            &mat,
            &vec_to_sparse_mat_fixed(&target, 3, false),
            I32F32::from_num(0),
        );
    }

    #[test]
    fn test_math_inplace_col_normalize() {
        let epsilon: I32F32 = I32F32::from_num(0.0001);
        let vector: Vec<f32> = vec![
            0., 1., 2., 3., 4., 0., 10., 100., 1000., 10000., 0., 0., 0., 0., 0., 1., 1., 1., 1.,
            1.,
        ];
        let mut mat = vec_to_mat_fixed(&vector, 4, true);
        inplace_col_normalize(&mut mat);
        let target: Vec<f32> = vec![
            0., 0.1, 0.2, 0.3, 0.4, 0., 0.0009, 0.009, 0.09, 0.9, 0., 0., 0., 0., 0., 0.2, 0.2,
            0.2, 0.2, 0.2,
        ];
        assert_mat_compare(&mat, &vec_to_mat_fixed(&target, 4, true), epsilon);
    }

    #[test]
    fn test_math_inplace_col_normalize_sparse() {
        let epsilon: I32F32 = I32F32::from_num(0.0001);
        let vector: Vec<f32> = vec![
            0., 1., 0., 2., 0., 3., 4., 0., 1., 0., 2., 0., 3., 0., 1., 0., 0., 2., 0., 3., 4., 0.,
            10., 0., 100., 1000., 0., 10000., 0., 0., 0., 0., 0., 0., 0., 1., 1., 1., 1., 1., 1.,
            1.,
        ];
        let mut mat = vec_to_sparse_mat_fixed(&vector, 6, true);
        inplace_col_normalize_sparse(&mut mat, 6);
        let target: Vec<f32> = vec![
            0., 0.1, 0., 0.2, 0., 0.3, 0.4, 0., 0.166666, 0., 0.333333, 0., 0.5, 0., 0.1, 0., 0.,
            0.2, 0., 0.3, 0.4, 0., 0.0009, 0., 0.009, 0.09, 0., 0.9, 0., 0., 0., 0., 0., 0., 0.,
            0.142857, 0.142857, 0.142857, 0.142857, 0.142857, 0.142857, 0.142857,
        ];
        assert_sparse_mat_compare(&mat, &vec_to_sparse_mat_fixed(&target, 6, true), epsilon);
        let vector: Vec<f32> = vec![0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0.];
        let target: Vec<f32> = vec![0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0.];
        let mut mat = vec_to_sparse_mat_fixed(&vector, 3, false);
        inplace_col_normalize_sparse(&mut mat, 6);
        assert_sparse_mat_compare(
            &mat,
            &vec_to_sparse_mat_fixed(&target, 3, false),
            I32F32::from_num(0),
        );
        let mut mat: Vec<Vec<(u16, I32F32)>> = vec![];
        let target: Vec<Vec<(u16, I32F32)>> = vec![];
        inplace_col_normalize_sparse(&mut mat, 0);
        assert_sparse_mat_compare(&mat, &target, epsilon);
    }

    #[test]
    fn test_math_inplace_col_max_upscale() {
        let mut mat: Vec<Vec<I32F32>> = vec![vec![]];
        let target: Vec<Vec<I32F32>> = vec![vec![]];
        inplace_col_max_upscale(&mut mat);
        assert_eq!(&mat, &target);
        let mut mat: Vec<Vec<I32F32>> = vec![vec![I32F32::from_num(0)]];
        let target: Vec<Vec<I32F32>> = vec![vec![I32F32::from_num(0)]];
        inplace_col_max_upscale(&mut mat);
        assert_eq!(&mat, &target);
        let epsilon: I32F32 = I32F32::from_num(0.0001);
        let vector: Vec<f32> = vec![
            0., 1., 2., 3., 4., 0., 10., 100., 1000., 10000., 0., 0., 0., 0., 0., 1., 1., 1., 1.,
            1.,
        ];
        let mut mat: Vec<Vec<I32F32>> = vec_to_mat_fixed(&vector, 4, true);
        inplace_col_max_upscale(&mut mat);
        let target: Vec<f32> = vec![
            0., 0.25, 0.5, 0.75, 1., 0., 0.001, 0.01, 0.1, 1., 0., 0., 0., 0., 0., 1., 1., 1., 1.,
            1.,
        ];
        assert_mat_compare(&mat, &vec_to_mat_fixed(&target, 4, true), epsilon);
    }

    #[test]
    fn test_math_inplace_col_max_upscale_sparse() {
        let mut mat: Vec<Vec<(u16, I32F32)>> = vec![vec![]];
        let target: Vec<Vec<(u16, I32F32)>> = vec![vec![]];
        inplace_col_max_upscale_sparse(&mut mat, 0);
        assert_eq!(&mat, &target);
        let mut mat: Vec<Vec<(u16, I32F32)>> = vec![vec![(0, I32F32::from_num(0))]];
        let target: Vec<Vec<(u16, I32F32)>> = vec![vec![(0, I32F32::from_num(0))]];
        inplace_col_max_upscale_sparse(&mut mat, 1);
        assert_eq!(&mat, &target);
        let epsilon: I32F32 = I32F32::from_num(0.0001);
        let vector: Vec<f32> = vec![
            0., 1., 0., 2., 0., 3., 4., 0., 1., 0., 2., 0., 3., 0., 1., 0., 0., 2., 0., 3., 4., 0.,
            10., 0., 100., 1000., 0., 10000., 0., 0., 0., 0., 0., 0., 0., 1., 1., 1., 1., 1., 1.,
            1.,
        ];
        let mut mat = vec_to_sparse_mat_fixed(&vector, 6, true);
        inplace_col_max_upscale_sparse(&mut mat, 6);
        let target: Vec<f32> = vec![
            0., 0.25, 0., 0.5, 0., 0.75, 1., 0., 0.333333, 0., 0.666666, 0., 1., 0., 0.25, 0., 0.,
            0.5, 0., 0.75, 1., 0., 0.001, 0., 0.01, 0.1, 0., 1., 0., 0., 0., 0., 0., 0., 0., 1.,
            1., 1., 1., 1., 1., 1.,
        ];
        assert_sparse_mat_compare(&mat, &vec_to_sparse_mat_fixed(&target, 6, true), epsilon);
        let vector: Vec<f32> = vec![0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0.];
        let target: Vec<f32> = vec![0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0.];
        let mut mat = vec_to_sparse_mat_fixed(&vector, 3, false);
        inplace_col_max_upscale_sparse(&mut mat, 6);
        assert_sparse_mat_compare(
            &mat,
            &vec_to_sparse_mat_fixed(&target, 3, false),
            I32F32::from_num(0),
        );
        let mut mat: Vec<Vec<(u16, I32F32)>> = vec![];
        let target: Vec<Vec<(u16, I32F32)>> = vec![];
        inplace_col_max_upscale_sparse(&mut mat, 0);
        assert_sparse_mat_compare(&mat, &target, epsilon);
    }

    #[test]
    fn test_math_inplace_mask_vector() {
        let mask: Vec<bool> = vec![false, false, false];
        let mut vector: Vec<I32F32> = vec_to_fixed(&vec![0., 1., 2.]);
        let target: Vec<I32F32> = vec_to_fixed(&vec![0., 1., 2.]);
        inplace_mask_vector(&mask, &mut vector);
        assert_vec_compare(&vector, &target, I32F32::from_num(0));
        let mask: Vec<bool> = vec![false, true, false];
        let mut vector: Vec<I32F32> = vec_to_fixed(&vec![0., 1., 2.]);
        let target: Vec<I32F32> = vec_to_fixed(&vec![0., 0., 2.]);
        inplace_mask_vector(&mask, &mut vector);
        assert_vec_compare(&vector, &target, I32F32::from_num(0));
        let mask: Vec<bool> = vec![true, true, true];
        let mut vector: Vec<I32F32> = vec_to_fixed(&vec![0., 1., 2.]);
        let target: Vec<I32F32> = vec_to_fixed(&vec![0., 0., 0.]);
        inplace_mask_vector(&mask, &mut vector);
        assert_vec_compare(&vector, &target, I32F32::from_num(0));
    }

    #[test]
    fn test_math_inplace_mask_matrix() {
        let mask: Vec<Vec<bool>> = vec![
            vec![false, false, false],
            vec![false, false, false],
            vec![false, false, false],
        ];
        let vector: Vec<f32> = vec![0., 1., 2., 3., 4., 5., 6., 7., 8.];
        let mut mat = vec_to_mat_fixed(&vector, 3, false);
        inplace_mask_matrix(&mask, &mut mat);
        assert_mat_compare(
            &mat,
            &vec_to_mat_fixed(&vector, 3, false),
            I32F32::from_num(0),
        );
        let mask: Vec<Vec<bool>> = vec![
            vec![true, false, false],
            vec![false, true, false],
            vec![false, false, true],
        ];
        let target: Vec<f32> = vec![0., 1., 2., 3., 0., 5., 6., 7., 0.];
        let mut mat = vec_to_mat_fixed(&vector, 3, false);
        inplace_mask_matrix(&mask, &mut mat);
        assert_mat_compare(
            &mat,
            &vec_to_mat_fixed(&target, 3, false),
            I32F32::from_num(0),
        );
        let mask: Vec<Vec<bool>> = vec![
            vec![true, true, true],
            vec![true, true, true],
            vec![true, true, true],
        ];
        let target: Vec<f32> = vec![0., 0., 0., 0., 0., 0., 0., 0., 0.];
        let mut mat = vec_to_mat_fixed(&vector, 3, false);
        inplace_mask_matrix(&mask, &mut mat);
        assert_mat_compare(
            &mat,
            &vec_to_mat_fixed(&target, 3, false),
            I32F32::from_num(0),
        );
    }

    #[test]
    fn test_math_inplace_mask_rows() {
        let input: Vec<f32> = vec![1., 2., 3., 4., 5., 6., 7., 8., 9.];
        let mask: Vec<bool> = vec![false, false, false];
        let target: Vec<f32> = vec![1., 2., 3., 4., 5., 6., 7., 8., 9.];
        let mut mat = vec_to_mat_fixed(&input, 3, false);
        inplace_mask_rows(&mask, &mut mat);
        assert_mat_compare(
            &mat,
            &vec_to_mat_fixed(&target, 3, false),
            I32F32::from_num(0),
        );
        let mask: Vec<bool> = vec![true, true, true];
        let target: Vec<f32> = vec![0., 0., 0., 0., 0., 0., 0., 0., 0.];
        let mut mat = vec_to_mat_fixed(&input, 3, false);
        inplace_mask_rows(&mask, &mut mat);
        assert_mat_compare(
            &mat,
            &vec_to_mat_fixed(&target, 3, false),
            I32F32::from_num(0),
        );
        let mask: Vec<bool> = vec![true, false, true];
        let target: Vec<f32> = vec![0., 0., 0., 4., 5., 6., 0., 0., 0.];
        let mut mat = vec_to_mat_fixed(&input, 3, false);
        inplace_mask_rows(&mask, &mut mat);
        assert_mat_compare(
            &mat,
            &vec_to_mat_fixed(&target, 3, false),
            I32F32::from_num(0),
        );
        let input: Vec<f32> = vec![0., 0., 0., 0., 0., 0., 0., 0., 0.];
        let mut mat = vec_to_mat_fixed(&input, 3, false);
        let mask: Vec<bool> = vec![false, false, false];
        let target: Vec<f32> = vec![0., 0., 0., 0., 0., 0., 0., 0., 0.];
        inplace_mask_rows(&mask, &mut mat);
        assert_mat_compare(
            &mat,
            &vec_to_mat_fixed(&target, 3, false),
            I32F32::from_num(0),
        );
    }

    #[test]
    fn test_math_inplace_mask_diag() {
        let vector: Vec<f32> = vec![1., 2., 3., 4., 5., 6., 7., 8., 9.];
        let target: Vec<f32> = vec![0., 2., 3., 4., 0., 6., 7., 8., 0.];
        let mut mat = vec_to_mat_fixed(&vector, 3, false);
        inplace_mask_diag(&mut mat);
        assert_mat_compare(
            &mat,
            &vec_to_mat_fixed(&target, 3, false),
            I32F32::from_num(0),
        );
    }

    #[test]
    fn test_math_mask_rows_sparse() {
        let input: Vec<f32> = vec![1., 2., 3., 4., 5., 6., 7., 8., 9.];
        let mat = vec_to_sparse_mat_fixed(&input, 3, false);
        let mask: Vec<bool> = vec![false, false, false];
        let target: Vec<f32> = vec![1., 2., 3., 4., 5., 6., 7., 8., 9.];
        let result = mask_rows_sparse(&mask, &mat);
        assert_sparse_mat_compare(
            &result,
            &vec_to_sparse_mat_fixed(&target, 3, false),
            I32F32::from_num(0),
        );
        let mask: Vec<bool> = vec![true, true, true];
        let target: Vec<f32> = vec![0., 0., 0., 0., 0., 0., 0., 0., 0.];
        let result = mask_rows_sparse(&mask, &mat);
        assert_sparse_mat_compare(
            &result,
            &vec_to_sparse_mat_fixed(&target, 3, false),
            I32F32::from_num(0),
        );
        let mask: Vec<bool> = vec![true, false, true];
        let target: Vec<f32> = vec![0., 0., 0., 4., 5., 6., 0., 0., 0.];
        let result = mask_rows_sparse(&mask, &mat);
        assert_sparse_mat_compare(
            &result,
            &vec_to_sparse_mat_fixed(&target, 3, false),
            I32F32::from_num(0),
        );
        let input: Vec<f32> = vec![0., 0., 0., 0., 0., 0., 0., 0., 0.];
        let mat = vec_to_sparse_mat_fixed(&input, 3, false);
        let mask: Vec<bool> = vec![false, false, false];
        let target: Vec<f32> = vec![0., 0., 0., 0., 0., 0., 0., 0., 0.];
        let result = mask_rows_sparse(&mask, &mat);
        assert_sparse_mat_compare(
            &result,
            &vec_to_sparse_mat_fixed(&target, 3, false),
            I32F32::from_num(0),
        );
    }

    #[test]
    fn test_math_mask_diag_sparse() {
        let vector: Vec<f32> = vec![1., 2., 3., 4., 5., 6., 7., 8., 9.];
        let target: Vec<f32> = vec![0., 2., 3., 4., 0., 6., 7., 8., 0.];
        let mat = vec_to_sparse_mat_fixed(&vector, 3, false);
        let result = mask_diag_sparse(&mat);
        assert_sparse_mat_compare(
            &result,
            &vec_to_sparse_mat_fixed(&target, 3, false),
            I32F32::from_num(0),
        );
        let vector: Vec<f32> = vec![1., 0., 0., 0., 5., 0., 0., 0., 9.];
        let target: Vec<f32> = vec![0., 0., 0., 0., 0., 0., 0., 0., 0.];
        let mat = vec_to_sparse_mat_fixed(&vector, 3, false);
        let result = mask_diag_sparse(&mat);
        assert_sparse_mat_compare(
            &result,
            &vec_to_sparse_mat_fixed(&target, 3, false),
            I32F32::from_num(0),
        );
        let vector: Vec<f32> = vec![0., 0., 0., 0., 0., 0., 0., 0., 0.];
        let target: Vec<f32> = vec![0., 0., 0., 0., 0., 0., 0., 0., 0.];
        let mat = vec_to_sparse_mat_fixed(&vector, 3, false);
        let result = mask_diag_sparse(&mat);
        assert_sparse_mat_compare(
            &result,
            &vec_to_sparse_mat_fixed(&target, 3, false),
            I32F32::from_num(0),
        );
    }

    #[test]
    fn test_math_vec_mask_sparse_matrix() {
        let vector: Vec<f32> = vec![1., 2., 3., 4., 5., 6., 7., 8., 9.];
        let target: Vec<f32> = vec![0., 2., 3., 4., 0., 6., 7., 8., 0.];
        let mat = vec_to_sparse_mat_fixed(&vector, 3, false);
        let first_vector: Vec<u64> = vec![1, 2, 3];
        let second_vector: Vec<u64> = vec![1, 2, 3];
        let result = vec_mask_sparse_matrix(&mat, &first_vector, &second_vector, &|a, b| a == b);
        assert_sparse_mat_compare(
            &result,
            &vec_to_sparse_mat_fixed(&target, 3, false),
            I32F32::from_num(0),
        );
        let target: Vec<f32> = vec![1., 0., 0., 4., 5., 0., 7., 8., 9.];
        let mat = vec_to_sparse_mat_fixed(&vector, 3, false);
        let first_vector: Vec<u64> = vec![1, 2, 3];
        let second_vector: Vec<u64> = vec![1, 2, 3];
        let result = vec_mask_sparse_matrix(&mat, &first_vector, &second_vector, &|a, b| a < b);
        assert_sparse_mat_compare(
            &result,
            &vec_to_sparse_mat_fixed(&target, 3, false),
            I32F32::from_num(0),
        );
        let vector: Vec<f32> = vec![0., 0., 0., 0., 0., 0., 0., 0., 0.];
        let target: Vec<f32> = vec![0., 0., 0., 0., 0., 0., 0., 0., 0.];
        let mat = vec_to_sparse_mat_fixed(&vector, 3, false);
        let first_vector: Vec<u64> = vec![1, 2, 3];
        let second_vector: Vec<u64> = vec![1, 2, 3];
        let result = vec_mask_sparse_matrix(&mat, &first_vector, &second_vector, &|a, b| a == b);
        assert_sparse_mat_compare(
            &result,
            &vec_to_sparse_mat_fixed(&target, 3, false),
            I32F32::from_num(0),
        );
    }

    #[test]
    fn test_math_row_hadamard() {
        let vector: Vec<I32F32> = vec_to_fixed(&vec![1., 2., 3., 4.]);
        let matrix: Vec<f32> = vec![1., 2., 3., 4., 5., 6., 7., 8., 9., 10., 11., 12.];
        let matrix = vec_to_mat_fixed(&matrix, 4, false);
        let result = row_hadamard(&matrix, &vector);
        let target: Vec<f32> = vec![1., 2., 3., 8., 10., 12., 21., 24., 27., 40., 44., 48.];
        let target = vec_to_mat_fixed(&target, 4, false);
        assert_mat_compare(&result, &target, I32F32::from_num(0));
    }

    #[test]
    fn test_math_row_hadamard_sparse() {
        let vector: Vec<I32F32> = vec_to_fixed(&vec![1., 2., 3., 4.]);
        let matrix: Vec<f32> = vec![1., 2., 3., 4., 5., 6., 7., 8., 9., 10., 11., 12.];
        let matrix = vec_to_sparse_mat_fixed(&matrix, 4, false);
        let result = row_hadamard_sparse(&matrix, &vector);
        let target: Vec<f32> = vec![1., 2., 3., 8., 10., 12., 21., 24., 27., 40., 44., 48.];
        let target = vec_to_sparse_mat_fixed(&target, 4, false);
        assert_sparse_mat_compare(&result, &target, I32F32::from_num(0));
        let matrix: Vec<f32> = vec![0., 2., 3., 4., 0., 6., 7., 8., 0., 10., 11., 12.];
        let matrix = vec_to_sparse_mat_fixed(&matrix, 4, false);
        let result = row_hadamard_sparse(&matrix, &vector);
        let target: Vec<f32> = vec![0., 2., 3., 8., 0., 12., 21., 24., 0., 40., 44., 48.];
        let target = vec_to_sparse_mat_fixed(&target, 4, false);
        assert_sparse_mat_compare(&result, &target, I32F32::from_num(0));
        let matrix: Vec<f32> = vec![0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0.];
        let matrix = vec_to_sparse_mat_fixed(&matrix, 4, false);
        let result = row_hadamard_sparse(&matrix, &vector);
        let target: Vec<f32> = vec![0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0.];
        let target = vec_to_sparse_mat_fixed(&target, 4, false);
        assert_sparse_mat_compare(&result, &target, I32F32::from_num(0));
    }

    #[test]
    fn test_math_row_sum() {
        let matrix: Vec<f32> = vec![1., 2., 3., 4., 5., 6., 7., 8., 9., 10., 11., 12.];
        let matrix = vec_to_mat_fixed(&matrix, 4, false);
        let result = row_sum(&matrix);
        let target: Vec<I32F32> = vec_to_fixed(&vec![6., 15., 24., 33.]);
        assert_vec_compare(&result, &target, I32F32::from_num(0));
    }

    #[test]
    fn test_math_row_sum_sparse() {
        let matrix: Vec<f32> = vec![1., 2., 3., 4., 5., 6., 7., 8., 9., 10., 11., 12.];
        let matrix = vec_to_sparse_mat_fixed(&matrix, 4, false);
        let result = row_sum_sparse(&matrix);
        let target: Vec<I32F32> = vec_to_fixed(&vec![6., 15., 24., 33.]);
        assert_vec_compare(&result, &target, I32F32::from_num(0));
        let matrix: Vec<f32> = vec![0., 2., 3., 4., 0., 6., 7., 8., 0., 10., 11., 12.];
        let matrix = vec_to_sparse_mat_fixed(&matrix, 4, false);
        let result = row_sum_sparse(&matrix);
        let target: Vec<I32F32> = vec_to_fixed(&vec![5., 10., 15., 33.]);
        assert_vec_compare(&result, &target, I32F32::from_num(0));
        let matrix: Vec<f32> = vec![1., 2., 3., 0., 0., 0., 7., 8., 9., 10., 11., 12.];
        let matrix = vec_to_sparse_mat_fixed(&matrix, 4, false);
        let result = row_sum_sparse(&matrix);
        let target: Vec<I32F32> = vec_to_fixed(&vec![6., 0., 24., 33.]);
        assert_vec_compare(&result, &target, I32F32::from_num(0));
        let matrix: Vec<f32> = vec![0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0.];
        let matrix = vec_to_sparse_mat_fixed(&matrix, 4, false);
        let result = row_sum_sparse(&matrix);
        let target: Vec<I32F32> = vec_to_fixed(&vec![0., 0., 0., 0.]);
        assert_vec_compare(&result, &target, I32F32::from_num(0));
    }

    #[test]
    fn test_math_col_sum() {
        let matrix: Vec<f32> = vec![1., 2., 3., 4., 5., 6., 7., 8., 9., 10., 11., 12.];
        let matrix = vec_to_mat_fixed(&matrix, 4, false);
        let result = col_sum(&matrix);
        let target: Vec<I32F32> = vec_to_fixed(&vec![22., 26., 30.]);
        assert_vec_compare(&result, &target, I32F32::from_num(0));
    }

    #[test]
    fn test_math_col_sum_sparse() {
        let matrix: Vec<f32> = vec![1., 2., 3., 4., 5., 6., 7., 8., 9., 10., 11., 12.];
        let matrix = vec_to_sparse_mat_fixed(&matrix, 4, false);
        let result = col_sum_sparse(&matrix, 3);
        let target: Vec<I32F32> = vec_to_fixed(&vec![22., 26., 30.]);
        assert_vec_compare(&result, &target, I32F32::from_num(0));
        let matrix: Vec<f32> = vec![0., 2., 3., 4., 0., 6., 7., 8., 0., 10., 11., 12.];
        let matrix = vec_to_sparse_mat_fixed(&matrix, 4, false);
        let result = col_sum_sparse(&matrix, 3);
        let target: Vec<I32F32> = vec_to_fixed(&vec![21., 21., 21.]);
        assert_vec_compare(&result, &target, I32F32::from_num(0));
        let matrix: Vec<f32> = vec![1., 0., 3., 4., 0., 6., 7., 0., 9., 10., 0., 12.];
        let matrix = vec_to_sparse_mat_fixed(&matrix, 4, false);
        let result = col_sum_sparse(&matrix, 3);
        let target: Vec<I32F32> = vec_to_fixed(&vec![22., 0., 30.]);
        assert_vec_compare(&result, &target, I32F32::from_num(0));
        let matrix: Vec<f32> = vec![0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0.];
        let matrix = vec_to_sparse_mat_fixed(&matrix, 4, false);
        let result = col_sum_sparse(&matrix, 3);
        let target: Vec<I32F32> = vec_to_fixed(&vec![0., 0., 0.]);
        assert_vec_compare(&result, &target, I32F32::from_num(0));
    }

    #[test]
    fn test_math_matmul() {
        let vector: Vec<I32F32> = vec_to_fixed(&vec![1., 2., 3., 4.]);
        let matrix: Vec<f32> = vec![1., 2., 3., 4., 5., 6., 7., 8., 9., 10., 11., 12.];
        let matrix = vec_to_mat_fixed(&matrix, 4, false);
        let result = matmul(&matrix, &vector);
        let target: Vec<I32F32> = vec_to_fixed(&vec![70., 80., 90.]);
        assert_vec_compare(&result, &target, I32F32::from_num(0));
    }

    #[test]
    fn test_math_matmul_transpose() {
        let vector: Vec<I32F32> = vec_to_fixed(&vec![1., 2., 3.]);
        let matrix: Vec<f32> = vec![1., 2., 3., 4., 5., 6., 7., 8., 9., 10., 11., 12.];
        let matrix = vec_to_mat_fixed(&matrix, 4, false);
        let result = matmul_transpose(&matrix, &vector);
        let target: Vec<I32F32> = vec_to_fixed(&vec![14., 32., 50., 68.]);
        assert_vec_compare(&result, &target, I32F32::from_num(0));
    }

    #[test]
    fn test_math_sparse_matmul() {
        let vector: Vec<I32F32> = vec_to_fixed(&vec![1., 2., 3., 4.]);
        let matrix: Vec<f32> = vec![1., 2., 3., 4., 5., 6., 7., 8., 9., 10., 11., 12.];
        let matrix = vec_to_sparse_mat_fixed(&matrix, 4, false);
        let result = matmul_sparse(&matrix, &vector, 3);
        let target: Vec<I32F32> = vec_to_fixed(&vec![70., 80., 90.]);
        assert_vec_compare(&result, &target, I32F32::from_num(0));
        let matrix: Vec<f32> = vec![0., 2., 3., 4., 0., 6., 7., 8., 0., 10., 11., 12.];
        let matrix = vec_to_sparse_mat_fixed(&matrix, 4, false);
        let result = matmul_sparse(&matrix, &vector, 3);
        let target: Vec<I32F32> = vec_to_fixed(&vec![69., 70., 63.]);
        assert_vec_compare(&result, &target, I32F32::from_num(0));
        let matrix: Vec<f32> = vec![0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0.];
        let matrix = vec_to_sparse_mat_fixed(&matrix, 4, false);
        let result = matmul_sparse(&matrix, &vector, 3);
        let target: Vec<I32F32> = vec_to_fixed(&vec![0., 0., 0.]);
        assert_vec_compare(&result, &target, I32F32::from_num(0));
    }

    #[test]
    fn test_math_sparse_matmul_transpose() {
        let vector: Vec<I32F32> = vec_to_fixed(&vec![1., 2., 3.]);
        let matrix: Vec<f32> = vec![1., 2., 3., 4., 5., 6., 7., 8., 9., 10., 11., 12.];
        let matrix = vec_to_sparse_mat_fixed(&matrix, 4, false);
        let result = matmul_transpose_sparse(&matrix, &vector);
        let target: Vec<I32F32> = vec_to_fixed(&vec![14., 32., 50., 68.]);
        assert_vec_compare(&result, &target, I32F32::from_num(0));
        let matrix: Vec<f32> = vec![0., 2., 3., 4., 0., 6., 7., 8., 0., 10., 11., 12.];
        let matrix = vec_to_sparse_mat_fixed(&matrix, 4, false);
        let result = matmul_transpose_sparse(&matrix, &vector);
        let target: Vec<I32F32> = vec_to_fixed(&vec![13., 22., 23., 68.]);
        assert_vec_compare(&result, &target, I32F32::from_num(0));
        let matrix: Vec<f32> = vec![0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0.];
        let matrix = vec_to_sparse_mat_fixed(&matrix, 4, false);
        let result = matmul_transpose_sparse(&matrix, &vector);
        let target: Vec<I32F32> = vec_to_fixed(&vec![0., 0., 0., 0.]);
        assert_vec_compare(&result, &target, I32F32::from_num(0));
    }

    #[test]
    fn test_math_inplace_col_clip() {
        let vector: Vec<I32F32> = vec_to_fixed(&vec![0., 5., 12.]);
        let matrix: Vec<f32> = vec![0., 2., 3., 4., 5., 6., 7., 8., 9., 10., 11., 12.];
        let mut matrix = vec_to_mat_fixed(&matrix, 4, false);
        let target: Vec<f32> = vec![0., 2., 3., 0., 5., 6., 0., 5., 9., 0., 5., 12.];
        let target = vec_to_mat_fixed(&target, 4, false);
        inplace_col_clip(&mut matrix, &vector);
        assert_mat_compare(&matrix, &target, I32F32::from_num(0));
    }

    #[test]
    fn test_math_col_clip_sparse() {
        let vector: Vec<I32F32> = vec_to_fixed(&vec![0., 5., 12.]);
        let matrix: Vec<f32> = vec![0., 2., 3., 4., 5., 6., 7., 8., 9., 10., 11., 12.];
        let matrix = vec_to_sparse_mat_fixed(&matrix, 4, false);
        let target: Vec<f32> = vec![0., 2., 3., 0., 5., 6., 0., 5., 9., 0., 5., 12.];
        let target = vec_to_sparse_mat_fixed(&target, 4, false);
        let result = col_clip_sparse(&matrix, &vector);
        assert_sparse_mat_compare(&result, &target, I32F32::from_num(0));
        let matrix: Vec<f32> = vec![0., 2., 3., 4., 5., 6., 0., 0., 0., 10., 11., 12.];
        let matrix = vec_to_sparse_mat_fixed(&matrix, 4, false);
        let target: Vec<f32> = vec![0., 2., 3., 0., 5., 6., 0., 0., 0., 0., 5., 12.];
        let target = vec_to_sparse_mat_fixed(&target, 4, false);
        let result = col_clip_sparse(&matrix, &vector);
        assert_sparse_mat_compare(&result, &target, I32F32::from_num(0));
        let matrix: Vec<f32> = vec![0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0.];
        let matrix = vec_to_sparse_mat_fixed(&matrix, 4, false);
        let target: Vec<f32> = vec![0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0.];
        let target = vec_to_sparse_mat_fixed(&target, 4, false);
        let result = col_clip_sparse(&matrix, &vector);
        assert_sparse_mat_compare(&result, &target, I32F32::from_num(0));
    }

    #[test]
    fn test_math_clip_sparse() {
        let matrix: Vec<f32> = vec![0., 2., 3., 4., 5., 6., 7., 8., 9., 10., 11., 12.];
        let matrix = vec_to_sparse_mat_fixed(&matrix, 4, false);
        let target: Vec<f32> = vec![0., 1., 1., 1., 1., 1., 1., 100., 100., 100., 100., 100.];
        let target = vec_to_sparse_mat_fixed(&target, 4, false);
        let result = clip_sparse(
            &matrix,
            I32F32::from_num(8),
            I32F32::from_num(100),
            I32F32::from_num(1),
        );
        assert_sparse_mat_compare(&result, &target, I32F32::from_num(0));
    }

    #[test]
    fn test_math_clip() {
        let matrix: Vec<f32> = vec![0., 2., 3., 4., 5., 6., 7., 8., 9., 10., 11., 12.];
        let matrix = vec_to_mat_fixed(&matrix, 4, false);
        let target: Vec<f32> = vec![1., 1., 1., 1., 1., 1., 1., 100., 100., 100., 100., 100.];
        let target = vec_to_mat_fixed(&target, 4, false);
        let result = clip(
            &matrix,
            I32F32::from_num(8),
            I32F32::from_num(100),
            I32F32::from_num(1),
        );
        assert_mat_compare(&result, &target, I32F32::from_num(0));
    }

    #[test]
    fn test_math_inplace_clip() {
        let matrix: Vec<f32> = vec![0., 2., 3., 4., 5., 6., 7., 8., 9., 10., 11., 12.];
        let mut matrix = vec_to_mat_fixed(&matrix, 4, false);
        let target: Vec<f32> = vec![1., 1., 1., 1., 1., 1., 1., 100., 100., 100., 100., 100.];
        let target = vec_to_mat_fixed(&target, 4, false);
        inplace_clip(
            &mut matrix,
            I32F32::from_num(8),
            I32F32::from_num(100),
            I32F32::from_num(1),
        );
        assert_mat_compare(&matrix, &target, I32F32::from_num(0));
    }

    #[test]
    fn test_math_weighted_median() {
        let mut rng = thread_rng();
        let zero: I32F32 = fixed(0.);
        let one: I32F32 = fixed(1.);
        for _ in 0..100 {
            let stake: Vec<I32F32> = vec_to_fixed(&vec![]);
            let score: Vec<I32F32> = vec_to_fixed(&vec![]);
            let majority: I32F32 = fixed(0.51);
            assert_eq!(
                zero,
                weighted_median(
                    &stake,
                    &score,
                    &(0..stake.len()).collect(),
                    one - majority,
                    zero,
                    stake.iter().sum()
                )
            );

            let stake: Vec<I32F32> = normalize(&vec_to_fixed(&vec![0.51]));
            let score: Vec<I32F32> = vec_to_fixed(&vec![1.]);
            let majority: I32F32 = fixed(0.51);
            assert_eq!(
                one,
                weighted_median(
                    &stake,
                    &score,
                    &(0..stake.len()).collect(),
                    one - majority,
                    zero,
                    stake.iter().sum()
                )
            );

            let stake: Vec<I32F32> = vec_to_fixed(&vec![0.49, 0.51]);
            let score: Vec<I32F32> = vec_to_fixed(&vec![0.5, 1.]);
            let majority: I32F32 = fixed(0.51);
            assert_eq!(
                one,
                weighted_median(
                    &stake,
                    &score,
                    &(0..stake.len()).collect(),
                    one - majority,
                    zero,
                    stake.iter().sum()
                )
            );

            let stake: Vec<I32F32> = vec_to_fixed(&vec![0.51, 0.49]);
            let score: Vec<I32F32> = vec_to_fixed(&vec![0.5, 1.]);
            let majority: I32F32 = fixed(0.51);
            assert_eq!(
                fixed(0.5),
                weighted_median(
                    &stake,
                    &score,
                    &(0..stake.len()).collect(),
                    one - majority,
                    zero,
                    stake.iter().sum()
                )
            );

            let stake: Vec<I32F32> = vec_to_fixed(&vec![0.49, 0., 0.51]);
            let score: Vec<I32F32> = vec_to_fixed(&vec![0.5, 0.7, 1.]);
            let majority: I32F32 = fixed(0.51);
            assert_eq!(
                one,
                weighted_median(
                    &stake,
                    &score,
                    &(0..stake.len()).collect(),
                    one - majority,
                    zero,
                    stake.iter().sum()
                )
            );

            let stake: Vec<I32F32> = vec_to_fixed(&vec![0.49, 0.01, 0.5]);
            let score: Vec<I32F32> = vec_to_fixed(&vec![0.5, 0.7, 1.]);
            let majority: I32F32 = fixed(0.51);
            assert_eq!(
                fixed(0.7),
                weighted_median(
                    &stake,
                    &score,
                    &(0..stake.len()).collect(),
                    one - majority,
                    zero,
                    stake.iter().sum()
                )
            );

            let stake: Vec<I32F32> = vec_to_fixed(&vec![0.49, 0.51, 0.0]);
            let score: Vec<I32F32> = vec_to_fixed(&vec![0.5, 0.7, 1.]);
            let majority: I32F32 = fixed(0.51);
            assert_eq!(
                fixed(0.7),
                weighted_median(
                    &stake,
                    &score,
                    &(0..stake.len()).collect(),
                    one - majority,
                    zero,
                    stake.iter().sum()
                )
            );

            let stake: Vec<I32F32> = vec_to_fixed(&vec![0.0, 0.49, 0.51]);
            let score: Vec<I32F32> = vec_to_fixed(&vec![0.5, 0.7, 1.]);
            let majority: I32F32 = fixed(0.51);
            assert_eq!(
                one,
                weighted_median(
                    &stake,
                    &score,
                    &(0..stake.len()).collect(),
                    one - majority,
                    zero,
                    stake.iter().sum()
                )
            );

            let stake: Vec<I32F32> = vec_to_fixed(&vec![0.0, 0.49, 0.0, 0.51]);
            let score: Vec<I32F32> = vec_to_fixed(&vec![0.5, 0.5, 1., 1.]);
            let majority: I32F32 = fixed(0.51);
            assert_eq!(
                one,
                weighted_median(
                    &stake,
                    &score,
                    &(0..stake.len()).collect(),
                    one - majority,
                    zero,
                    stake.iter().sum()
                )
            );

            let stake: Vec<I32F32> = vec_to_fixed(&vec![0.0, 0.49, 0.0, 0.51, 0.0]);
            let score: Vec<I32F32> = vec_to_fixed(&vec![0.5, 0.5, 1., 1., 0.5]);
            let majority: I32F32 = fixed(0.51);
            assert_eq!(
                one,
                weighted_median(
                    &stake,
                    &score,
                    &(0..stake.len()).collect(),
                    one - majority,
                    zero,
                    stake.iter().sum()
                )
            );

            let stake: Vec<I32F32> = vec_to_fixed(&vec![0.2, 0.2, 0.2, 0.2, 0.2]);
            let score: Vec<I32F32> = vec_to_fixed(&vec![0.8, 0.2, 1., 0.6, 0.4]);
            let majority: I32F32 = fixed(0.51);
            assert_eq!(
                fixed(0.6),
                weighted_median(
                    &stake,
                    &score,
                    &(0..stake.len()).collect(),
                    one - majority,
                    zero,
                    stake.iter().sum()
                )
            );

            let stake: Vec<I32F32> =
                vec_to_fixed(&vec![0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1]);
            let score: Vec<I32F32> =
                vec_to_fixed(&vec![0.8, 0.8, 0.2, 0.2, 1.0, 1.0, 0.6, 0.6, 0.4, 0.4]);
            let majority: I32F32 = fixed(0.51);
            assert_eq!(
                fixed(0.6),
                weighted_median(
                    &stake,
                    &score,
                    &(0..stake.len()).collect(),
                    one - majority,
                    zero,
                    stake.iter().sum()
                )
            );

            let n: usize = 100;
            for majority in vec_to_fixed(&vec![
                0.,
                0.0000001,
                0.25,
                0.48999999999999,
                0.49,
                0.49000000000001,
                0.5,
                0.509999999999,
                0.51,
                0.5100000000001,
                0.9999999,
                1.,
            ]) {
                for allow_equal in vec![false, true] {
                    let mut stake: Vec<I32F32> = vec![];
                    let mut score: Vec<I32F32> = vec![];
                    let mut last_score: I32F32 = zero;
                    for i in 0..n {
                        if allow_equal {
                            match rng.gen_range(0..2) {
                                1 => stake.push(one),
                                _ => stake.push(zero),
                            }
                            match rng.gen_range(0..2) {
                                1 => last_score += one,
                                _ => (),
                            }
                            score.push(last_score);
                        } else {
                            stake.push(one);
                            score.push(I32F32::from_num(i));
                        }
                    }
                    inplace_normalize(&mut stake);
                    let total_stake: I32F32 = stake.iter().sum();
                    let mut minority: I32F32 = total_stake - majority;
                    if minority < zero {
                        minority = zero;
                    }
                    let mut medians: Vec<I32F32> = vec![];
                    let mut median_stake: I32F32 = zero;
                    let mut median_set = false;
                    let mut stake_sum: I32F32 = zero;
                    for i in 0..n {
                        stake_sum += stake[i];
                        if !median_set && stake_sum >= minority {
                            median_stake = stake_sum;
                            median_set = true;
                        }
                        if median_set {
                            if median_stake < stake_sum {
                                if median_stake == minority && !medians.contains(&score[i]) {
                                    medians.push(score[i]);
                                }
                                break;
                            }
                            if !medians.contains(&score[i]) {
                                medians.push(score[i]);
                            }
                        }
                    }
                    if medians.len() == 0 {
                        medians.push(zero);
                    }
                    let stake_idx: Vec<usize> = (0..stake.len()).collect();
                    let result: I32F32 =
                        weighted_median(&stake, &score, &stake_idx, minority, zero, total_stake);
                    assert!(medians.contains(&result));
                    for _ in 0..10 {
                        let mut permuted_uids: Vec<usize> = (0..n).collect();
                        permuted_uids.shuffle(&mut thread_rng());
                        stake = permuted_uids.iter().map(|&i| stake[i]).collect();
                        score = permuted_uids.iter().map(|&i| score[i]).collect();
                        let result: I32F32 = weighted_median(
                            &stake,
                            &score,
                            &stake_idx,
                            minority,
                            zero,
                            total_stake,
                        );
                        assert!(medians.contains(&result));
                    }
                }
            }
        }
    }

    #[test]
    fn test_math_weighted_median_col() {
        let stake: Vec<I32F32> = vec_to_fixed(&vec![]);
        let weights: Vec<Vec<I32F32>> = vec![vec![]];
        let median: Vec<I32F32> = vec_to_fixed(&vec![]);
        assert_eq!(median, weighted_median_col(&stake, &weights, fixed(0.5)));

        let stake: Vec<I32F32> = vec_to_fixed(&vec![0., 0.]);
        let weights: Vec<f32> = vec![0., 0., 0., 0.];
        let weights: Vec<Vec<I32F32>> = vec_to_mat_fixed(&weights, 2, false);
        let median: Vec<I32F32> = vec_to_fixed(&vec![0., 0.]);
        assert_eq!(median, weighted_median_col(&stake, &weights, fixed(0.5)));

        let stake: Vec<I32F32> = vec_to_fixed(&vec![0., 0.75, 0.25, 0.]);
        let weights: Vec<f32> = vec![0., 0.1, 0., 0., 0.2, 0.4, 0., 0.3, 0.1, 0., 0.4, 0.5];
        let weights: Vec<Vec<I32F32>> = vec_to_mat_fixed(&weights, 4, false);
        let median: Vec<I32F32> = vec_to_fixed(&vec![0., 0.3, 0.4]);
        assert_eq!(median, weighted_median_col(&stake, &weights, fixed(0.24)));
        let median: Vec<I32F32> = vec_to_fixed(&vec![0., 0.2, 0.4]);
        assert_eq!(median, weighted_median_col(&stake, &weights, fixed(0.26)));
        let median: Vec<I32F32> = vec_to_fixed(&vec![0., 0.2, 0.1]);
        assert_eq!(median, weighted_median_col(&stake, &weights, fixed(0.76)));

        let stake: Vec<I32F32> = vec_to_fixed(&vec![0., 0.3, 0.2, 0.5]);
        let weights: Vec<f32> = vec![0., 0.1, 0., 0., 0.2, 0.4, 0., 0.3, 0.1, 0., 0., 0.5];
        let weights: Vec<Vec<I32F32>> = vec_to_mat_fixed(&weights, 4, false);
        let median: Vec<I32F32> = vec_to_fixed(&vec![0., 0., 0.4]);
        assert_eq!(median, weighted_median_col(&stake, &weights, fixed(0.51)));
    }

    #[test]
    fn test_math_weighted_median_col_sparse() {
        let stake: Vec<I32F32> = vec_to_fixed(&vec![]);
        let weights: Vec<Vec<(u16, I32F32)>> = vec![vec![]];
        let median: Vec<I32F32> = vec_to_fixed(&vec![]);
        assert_eq!(
            median,
            weighted_median_col_sparse(&stake, &weights, 0, fixed(0.5))
        );

        let stake: Vec<I32F32> = vec_to_fixed(&vec![0., 0.]);
        let weights: Vec<f32> = vec![0., 0., 0., 0.];
        let weights: Vec<Vec<(u16, I32F32)>> = vec_to_sparse_mat_fixed(&weights, 2, false);
        let median: Vec<I32F32> = vec_to_fixed(&vec![0., 0.]);
        assert_eq!(
            median,
            weighted_median_col_sparse(&stake, &weights, 2, fixed(0.5))
        );

        let stake: Vec<I32F32> = vec_to_fixed(&vec![0., 0.75, 0.25, 0.]);
        let weights: Vec<f32> = vec![0., 0.1, 0., 0., 0.2, 0.4, 0., 0.3, 0.1, 0., 0.4, 0.5];
        let weights: Vec<Vec<(u16, I32F32)>> = vec_to_sparse_mat_fixed(&weights, 4, false);
        let median: Vec<I32F32> = vec_to_fixed(&vec![0., 0.3, 0.4]);
        assert_eq!(
            median,
            weighted_median_col_sparse(&stake, &weights, 3, fixed(0.24))
        );
        let median: Vec<I32F32> = vec_to_fixed(&vec![0., 0.2, 0.4]);
        assert_eq!(
            median,
            weighted_median_col_sparse(&stake, &weights, 3, fixed(0.26))
        );
        let median: Vec<I32F32> = vec_to_fixed(&vec![0., 0.2, 0.1]);
        assert_eq!(
            median,
            weighted_median_col_sparse(&stake, &weights, 3, fixed(0.76))
        );

        let stake: Vec<I32F32> = vec_to_fixed(&vec![0., 0.3, 0.2, 0.5]);
        let weights: Vec<f32> = vec![0., 0.1, 0., 0., 0.2, 0.4, 0., 0.3, 0.1, 0., 0., 0.5];
        let weights: Vec<Vec<(u16, I32F32)>> = vec_to_sparse_mat_fixed(&weights, 4, false);
        let median: Vec<I32F32> = vec_to_fixed(&vec![0., 0., 0.4]);
        assert_eq!(
            median,
            weighted_median_col_sparse(&stake, &weights, 3, fixed(0.51))
        );
    }

    #[test]
    fn test_math_hadamard() {
        let mat2: Vec<f32> = vec![1., 2., 3., 4., 5., 6., 7., 8., 9., 10., 11., 12.];
        let mat1: Vec<f32> = vec![
            10., 20., 30., 40., 50., 60., 70., 80., 90., 100., 110., 120.,
        ];
        let target: Vec<f32> = vec![
            10., 40., 90., 160., 250., 360., 490., 640., 810., 1000., 1210., 1440.,
        ];
        let mat2 = vec_to_mat_fixed(&mat2, 4, false);
        let mat1 = vec_to_mat_fixed(&mat1, 4, false);
        let target = vec_to_mat_fixed(&target, 4, false);
        let result = hadamard(&mat1, &mat2);
        assert_mat_compare(&result, &target, I32F32::from_num(0.000001));
        let mat2: Vec<f32> = vec![0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0.];
        let mat1: Vec<f32> = vec![0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0.];
        let target: Vec<f32> = vec![0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0.];
        let mat2 = vec_to_mat_fixed(&mat2, 4, false);
        let mat1 = vec_to_mat_fixed(&mat1, 4, false);
        let target = vec_to_mat_fixed(&target, 4, false);
        let result = hadamard(&mat1, &mat2);
        assert_mat_compare(&result, &target, I32F32::from_num(0.000001));
        let mat2: Vec<f32> = vec![1., 0., 0., 0., 2., 0., 0., 0., 3., 0., 0., 0.];
        let mat1: Vec<f32> = vec![0., 0., 4., 0., 5., 0., 6., 0., 0., 0., 0., 0.];
        let target: Vec<f32> = vec![0., 0., 0., 0., 10., 0., 0., 0., 0., 0., 0., 0.];
        let mat2 = vec_to_mat_fixed(&mat2, 4, false);
        let mat1 = vec_to_mat_fixed(&mat1, 4, false);
        let target = vec_to_mat_fixed(&target, 4, false);
        let result = hadamard(&mat1, &mat2);
        assert_mat_compare(&result, &target, I32F32::from_num(0.000001));
    }

    #[test]
    fn test_math_hadamard_sparse() {
        let mat2: Vec<f32> = vec![1., 2., 3., 4., 5., 6., 7., 8., 9., 10., 11., 12.];
        let mat1: Vec<f32> = vec![
            10., 20., 30., 40., 50., 60., 70., 80., 90., 100., 110., 120.,
        ];
        let target: Vec<f32> = vec![
            10., 40., 90., 160., 250., 360., 490., 640., 810., 1000., 1210., 1440.,
        ];
        let mat2 = vec_to_sparse_mat_fixed(&mat2, 4, false);
        let mat1 = vec_to_sparse_mat_fixed(&mat1, 4, false);
        let target = vec_to_sparse_mat_fixed(&target, 4, false);
        let result = hadamard_sparse(&mat1, &mat2, 3);
        assert_sparse_mat_compare(&result, &target, I32F32::from_num(0.000001));
        let mat2: Vec<f32> = vec![0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0.];
        let mat1: Vec<f32> = vec![0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0.];
        let target: Vec<f32> = vec![0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0.];
        let mat2 = vec_to_sparse_mat_fixed(&mat2, 4, false);
        let mat1 = vec_to_sparse_mat_fixed(&mat1, 4, false);
        let target = vec_to_sparse_mat_fixed(&target, 4, false);
        let result = hadamard_sparse(&mat1, &mat2, 3);
        assert_sparse_mat_compare(&result, &target, I32F32::from_num(0.000001));
        let mat2: Vec<f32> = vec![1., 0., 0., 0., 2., 0., 0., 0., 3., 0., 0., 0.];
        let mat1: Vec<f32> = vec![0., 0., 4., 0., 5., 0., 6., 0., 0., 0., 0., 0.];
        let target: Vec<f32> = vec![0., 0., 0., 0., 10., 0., 0., 0., 0., 0., 0., 0.];
        let mat2 = vec_to_sparse_mat_fixed(&mat2, 4, false);
        let mat1 = vec_to_sparse_mat_fixed(&mat1, 4, false);
        let target = vec_to_sparse_mat_fixed(&target, 4, false);
        let result = hadamard_sparse(&mat1, &mat2, 3);
        assert_sparse_mat_compare(&result, &target, I32F32::from_num(0.000001));
    }

    #[test]
    fn test_math_mat_ema() {
        let old: Vec<f32> = vec![1., 2., 3., 4., 5., 6., 7., 8., 9., 10., 11., 12.];
        let new: Vec<f32> = vec![
            10., 20., 30., 40., 50., 60., 70., 80., 90., 100., 110., 120.,
        ];
        let target: Vec<f32> = vec![
            1.9, 3.8, 5.7, 7.6, 9.5, 11.4, 13.3, 15.2, 17.1, 19., 20.9, 22.8,
        ];
        let old = vec_to_mat_fixed(&old, 4, false);
        let new = vec_to_mat_fixed(&new, 4, false);
        let target = vec_to_mat_fixed(&target, 4, false);
        let result = mat_ema(&new, &old, I32F32::from_num(0.1));
        assert_mat_compare(&result, &target, I32F32::from_num(0.000001));
        let old: Vec<f32> = vec![1., 2., 3., 4., 5., 6., 7., 8., 9., 10., 11., 12.];
        let new: Vec<f32> = vec![
            10., 20., 30., 40., 50., 60., 70., 80., 90., 100., 110., 120.,
        ];
        let target: Vec<f32> = vec![1., 2., 3., 4., 5., 6., 7., 8., 9., 10., 11., 12.];
        let old = vec_to_mat_fixed(&old, 4, false);
        let new = vec_to_mat_fixed(&new, 4, false);
        let target = vec_to_mat_fixed(&target, 4, false);
        let result = mat_ema(&new, &old, I32F32::from_num(0));
        assert_mat_compare(&result, &target, I32F32::from_num(0.000001));
        let old: Vec<f32> = vec![1., 2., 3., 4., 5., 6., 7., 8., 9., 10., 11., 12.];
        let new: Vec<f32> = vec![
            10., 20., 30., 40., 50., 60., 70., 80., 90., 100., 110., 120.,
        ];
        let target: Vec<f32> = vec![
            10., 20., 30., 40., 50., 60., 70., 80., 90., 100., 110., 120.,
        ];
        let old = vec_to_mat_fixed(&old, 4, false);
        let new = vec_to_mat_fixed(&new, 4, false);
        let target = vec_to_mat_fixed(&target, 4, false);
        let result = mat_ema(&new, &old, I32F32::from_num(1));
        assert_mat_compare(&result, &target, I32F32::from_num(0.000001));
    }

    #[test]
    fn test_math_sparse_mat_ema() {
        let old: Vec<f32> = vec![1., 2., 3., 4., 5., 6., 7., 8., 9., 10., 11., 12.];
        let new: Vec<f32> = vec![
            10., 20., 30., 40., 50., 60., 70., 80., 90., 100., 110., 120.,
        ];
        let target: Vec<f32> = vec![
            1.9, 3.8, 5.7, 7.6, 9.5, 11.4, 13.3, 15.2, 17.1, 19., 20.9, 22.8,
        ];
        let old = vec_to_sparse_mat_fixed(&old, 4, false);
        let new = vec_to_sparse_mat_fixed(&new, 4, false);
        let target = vec_to_sparse_mat_fixed(&target, 4, false);
        let result = mat_ema_sparse(&new, &old, I32F32::from_num(0.1));
        assert_sparse_mat_compare(&result, &target, I32F32::from_num(0.000001));
        let old: Vec<f32> = vec![0., 2., 3., 4., 0., 6., 7., 8., 0., 10., 11., 12.];
        let new: Vec<f32> = vec![10., 20., 0., 40., 0., 60., 0., 80., 90., 100., 110., 120.];
        let target: Vec<f32> = vec![1., 3.8, 2.7, 7.6, 0., 11.4, 6.3, 15.2, 9., 19., 20.9, 22.8];
        let old = vec_to_sparse_mat_fixed(&old, 4, false);
        let new = vec_to_sparse_mat_fixed(&new, 4, false);
        let target = vec_to_sparse_mat_fixed(&target, 4, false);
        let result = mat_ema_sparse(&new, &old, I32F32::from_num(0.1));
        assert_sparse_mat_compare(&result, &target, I32F32::from_num(0.000001));
        let old: Vec<f32> = vec![0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0.];
        let new: Vec<f32> = vec![10., 20., 0., 40., 0., 60., 0., 80., 90., 100., 110., 120.];
        let target: Vec<f32> = vec![1., 2., 0., 4., 0., 6., 0., 8., 9., 10., 11., 12.];
        let old = vec_to_sparse_mat_fixed(&old, 4, false);
        let new = vec_to_sparse_mat_fixed(&new, 4, false);
        let target = vec_to_sparse_mat_fixed(&target, 4, false);
        let result = mat_ema_sparse(&new, &old, I32F32::from_num(0.1));
        assert_sparse_mat_compare(&result, &target, I32F32::from_num(0.000001));
        let old: Vec<f32> = vec![0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0.];
        let new: Vec<f32> = vec![0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0.];
        let target: Vec<f32> = vec![0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0.];
        let old = vec_to_sparse_mat_fixed(&old, 4, false);
        let new = vec_to_sparse_mat_fixed(&new, 4, false);
        let target = vec_to_sparse_mat_fixed(&target, 4, false);
        let result = mat_ema_sparse(&new, &old, I32F32::from_num(0.1));
        assert_sparse_mat_compare(&result, &target, I32F32::from_num(0.000001));
        let old: Vec<f32> = vec![1., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0.];
        let new: Vec<f32> = vec![0., 0., 0., 0., 2., 0., 0., 0., 0., 0., 0., 0.];
        let target: Vec<f32> = vec![0.9, 0., 0., 0., 0.2, 0., 0., 0., 0., 0., 0., 0.];
        let old = vec_to_sparse_mat_fixed(&old, 4, false);
        let new = vec_to_sparse_mat_fixed(&new, 4, false);
        let target = vec_to_sparse_mat_fixed(&target, 4, false);
        let result = mat_ema_sparse(&new, &old, I32F32::from_num(0.1));
        assert_sparse_mat_compare(&result, &target, I32F32::from_num(0.000001));
    }

    #[test]
    fn test_math_matmul2() {
        let epsilon: I32F32 = I32F32::from_num(0.0001);
        let w: Vec<Vec<I32F32>> = vec![vec![I32F32::from_num(1.0); 3]; 3];
        assert_vec_compare(
            &matmul(&w, &vec![I32F32::from_num(1.0); 3]),
            &vec![
                I32F32::from_num(3),
                I32F32::from_num(3),
                I32F32::from_num(3),
            ],
            epsilon,
        );
        assert_vec_compare(
            &matmul(&w, &vec![I32F32::from_num(2.0); 3]),
            &vec![
                I32F32::from_num(6),
                I32F32::from_num(6),
                I32F32::from_num(6),
            ],
            epsilon,
        );
        assert_vec_compare(
            &matmul(&w, &vec![I32F32::from_num(3.0); 3]),
            &vec![
                I32F32::from_num(9),
                I32F32::from_num(9),
                I32F32::from_num(9),
            ],
            epsilon,
        );
        assert_vec_compare(
            &matmul(&w, &vec![I32F32::from_num(-1.0); 3]),
            &vec![
                I32F32::from_num(-3),
                I32F32::from_num(-3),
                I32F32::from_num(-3),
            ],
            epsilon,
        );
        let w: Vec<Vec<I32F32>> = vec![vec![I32F32::from_num(-1.0); 3]; 3];
        assert_vec_compare(
            &matmul(&w, &vec![I32F32::from_num(1.0); 3]),
            &vec![
                I32F32::from_num(-3),
                I32F32::from_num(-3),
                I32F32::from_num(-3),
            ],
            epsilon,
        );
        assert_vec_compare(
            &matmul(&w, &vec![I32F32::from_num(2.0); 3]),
            &vec![
                I32F32::from_num(-6),
                I32F32::from_num(-6),
                I32F32::from_num(-6),
            ],
            epsilon,
        );
        assert_vec_compare(
            &matmul(&w, &vec![I32F32::from_num(3.0); 3]),
            &vec![
                I32F32::from_num(-9),
                I32F32::from_num(-9),
                I32F32::from_num(-9),
            ],
            epsilon,
        );
        assert_vec_compare(
            &matmul(&w, &vec![I32F32::from_num(-1.0); 3]),
            &vec![
                I32F32::from_num(3),
                I32F32::from_num(3),
                I32F32::from_num(3),
            ],
            epsilon,
        );
        let w: Vec<Vec<I32F32>> = vec![
            vec![I32F32::from_num(1.0); 3],
            vec![I32F32::from_num(2.0); 3],
            vec![I32F32::from_num(3.0); 3],
        ];
        assert_vec_compare(
            &matmul(&w, &vec![I32F32::from_num(0.0); 3]),
            &vec![
                I32F32::from_num(0.0),
                I32F32::from_num(0.0),
                I32F32::from_num(0.0),
            ],
            epsilon,
        );
        assert_vec_compare(
            &matmul(&w, &vec![I32F32::from_num(2.0); 3]),
            &vec![
                I32F32::from_num(12),
                I32F32::from_num(12),
                I32F32::from_num(12),
            ],
            epsilon,
        );
        let w: Vec<Vec<I32F32>> = vec![
            vec![
                I32F32::from_num(1),
                I32F32::from_num(2),
                I32F32::from_num(3)
            ];
            3
        ];
        assert_vec_compare(
            &matmul(&w, &vec![I32F32::from_num(0.0); 3]),
            &vec![
                I32F32::from_num(0.0),
                I32F32::from_num(0.0),
                I32F32::from_num(0.0),
            ],
            epsilon,
        );
        assert_vec_compare(
            &matmul(&w, &vec![I32F32::from_num(2.0); 3]),
            &vec![
                I32F32::from_num(6),
                I32F32::from_num(12),
                I32F32::from_num(18),
            ],
            epsilon,
        );
    }

    #[test]
    fn test_math_fixed_to_u16() {
        let expected = u16::MIN;
        assert_eq!(fixed_to_u16(I32F32::from_num(expected)), expected);

        let expected = u16::MAX / 2;
        assert_eq!(fixed_to_u16(I32F32::from_num(expected)), expected);

        let expected = u16::MAX;
        assert_eq!(fixed_to_u16(I32F32::from_num(expected)), expected);
    }

    #[test]
    #[should_panic(expected = "overflow")]
    fn test_math_fixed_to_u16_panics() {
        let bad_input = I32F32::from_num(u32::MAX);
        fixed_to_u16(bad_input);

        let bad_input = I32F32::from_num(-1);
        fixed_to_u16(bad_input);
    }

    // TODO: Investigate why `I32F32` and not `I64F64`
    #[test]
    fn test_math_fixed_to_u64() {
        let expected = u64::MIN;
        assert_eq!(fixed_to_u64(I32F32::from_num(expected)), expected);

        // let expected = u64::MAX / 2;
        // assert_eq!(fixed_to_u64(I32F32::from_num(expected)), expected);

        // let expected = u64::MAX;
        // assert_eq!(fixed_to_u64(I32F32::from_num(expected)), expected);
    }

    #[test]
    #[should_panic(expected = "-1 overflows")]
    fn test_math_fixed_to_u64_panics() {
        let bad_input = I32F32::from_num(-1);
        fixed_to_u64(bad_input);
    }

    #[test]
    fn test_math_fixed64_to_u64() {
        let expected = u64::MIN;
        assert_eq!(fixed64_to_u64(I64F64::from_num(expected)), expected);

        let input = i64::MAX / 2;
        let expected = u64::try_from(input).unwrap();
        assert_eq!(fixed64_to_u64(I64F64::from_num(input)), expected);

        let input = i64::MAX;
        let expected = u64::try_from(input).unwrap();
        assert_eq!(fixed64_to_u64(I64F64::from_num(input)), expected);
    }

    #[test]
    #[should_panic(expected = "-1 overflows")]
    fn test_math_fixed64_to_u64_panics() {
        let bad_input = I64F64::from_num(-1);
        fixed64_to_u64(bad_input);
    }

    /* @TODO: find the _true_ max, and half, input values */
    #[test]
    fn test_math_fixed64_to_fixed32() {
        let input = u64::MIN;
        let expected = u32::try_from(input).unwrap();
        assert_eq!(fixed64_to_fixed32(I64F64::from_num(expected)), expected);

        let expected = u32::MAX / 2;
        let input = u64::try_from(expected).unwrap();
        assert_eq!(fixed64_to_fixed32(I64F64::from_num(input)), expected);
    }

    #[test]
    #[should_panic(expected = "overflow")]
    fn test_math_fixed64_to_fixed32_panics() {
        let bad_input = I64F64::from_num(u32::MAX);
        fixed64_to_fixed32(bad_input);
    }

    #[test]
    fn test_math_u16_to_fixed() {
        let input = u16::MIN;
        let expected = I32F32::from_num(input);
        assert_eq!(u16_to_fixed(input), expected);

        let input = u16::MAX / 2;
        let expected = I32F32::from_num(input);
        assert_eq!(u16_to_fixed(input), expected);

        let input = u16::MAX;
        let expected = I32F32::from_num(input);
        assert_eq!(u16_to_fixed(input), expected);
    }

    #[test]
    fn test_math_u16_proportion_to_fixed() {
        let input = u16::MIN;
        let expected = I32F32::from_num(input);
        assert_eq!(u16_proportion_to_fixed(input), expected);
    }

    #[test]
    fn test_fixed_proportion_to_u16() {
        let expected = u16::MIN;
        let input = I32F32::from_num(expected);
        assert_eq!(fixed_proportion_to_u16(input), expected);
    }

    #[test]
    #[should_panic(expected = "overflow")]
    fn test_fixed_proportion_to_u16_panics() {
        let expected = u16::MAX;
        let input = I32F32::from_num(expected);
        fixed_proportion_to_u16(input);
    }

    #[test]
    fn test_vec_fixed64_to_fixed32() {
        let input = vec![I64F64::from_num(i32::MIN)];
        let expected = vec![I32F32::from_num(i32::MIN)];
        assert_eq!(vec_fixed64_to_fixed32(input), expected);

        let input = vec![I64F64::from_num(i32::MAX)];
        let expected = vec![I32F32::from_num(i32::MAX)];
        assert_eq!(vec_fixed64_to_fixed32(input), expected);
    }

    #[test]
    #[should_panic(expected = "overflow")]
    fn test_vec_fixed64_to_fixed32_panics() {
        let bad_input = vec![I64F64::from_num(i64::MAX)];
        vec_fixed64_to_fixed32(bad_input);
    }

    #[test]
    #[allow(arithmetic_overflow)]
    fn test_checked_sum() {
        let overflowing_input = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, u64::MAX];
        // Expect None when overflow occurs
        assert_eq!(checked_sum(&overflowing_input), None);

        let normal_input = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        // Expect Some when no overflow occurs
        assert_eq!(checked_sum(&normal_input), Some(55));

        let empty_input: Vec<u16> = vec![];
        // Expect Some(u16::default()) when input is empty
        assert_eq!(checked_sum(&empty_input), Some(u16::default()));

        let single_input = vec![1];
        // Expect Some(...) when input is a single value
        assert_eq!(checked_sum(&single_input), Some(1));
    }
}
