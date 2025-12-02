// we get a compiler warning for this , even though  the trait is used in the
// quantile function.
use crate::alloc::borrow::ToOwned;
use safe_math::*;
use sp_runtime::traits::CheckedAdd;

use sp_std::vec;
use substrate_fixed::transcendental::{exp, ln};
use substrate_fixed::types::{I32F32, I64F64};

use sp_std::vec::Vec;

pub fn get_safe<T: Copy + Default>(slice: &[T], idx: usize) -> T {
    slice.get(idx).copied().unwrap_or_default()
}

pub fn fixed(val: f32) -> I32F32 {
    I32F32::saturating_from_num(val)
}

pub fn fixed_to_u16(x: I32F32) -> u16 {
    x.saturating_to_num::<u16>()
}

pub fn fixed_to_u64(x: I32F32) -> u64 {
    x.saturating_to_num::<u64>()
}

pub fn fixed64_to_u64(x: I64F64) -> u64 {
    x.saturating_to_num::<u64>()
}

pub fn fixed64_to_fixed32(x: I64F64) -> I32F32 {
    I32F32::saturating_from_num(x)
}

pub fn fixed32_to_fixed64(x: I32F32) -> I64F64 {
    I64F64::saturating_from_num(x)
}

pub fn u16_to_fixed(x: u16) -> I32F32 {
    I32F32::saturating_from_num(x)
}

pub fn u16_proportion_to_fixed(x: u16) -> I32F32 {
    I32F32::saturating_from_num(x).safe_div(I32F32::saturating_from_num(u16::MAX))
}

pub fn fixed_to_fixed_u16_proportion(x: I32F32) -> I32F32 {
    x.safe_div(I32F32::saturating_from_num(u16::MAX))
}

pub fn fixed_proportion_to_u16(x: I32F32) -> u16 {
    fixed_to_u16(x.saturating_mul(I32F32::saturating_from_num(u16::MAX)))
}

pub fn vec_fixed32_to_u64(vec: Vec<I32F32>) -> Vec<u64> {
    vec.into_iter().map(fixed_to_u64).collect()
}

pub fn vec_fixed64_to_fixed32(vec: Vec<I64F64>) -> Vec<I32F32> {
    vec.into_iter().map(fixed64_to_fixed32).collect()
}

pub fn vec_fixed32_to_fixed64(vec: Vec<I32F32>) -> Vec<I64F64> {
    vec.into_iter().map(fixed32_to_fixed64).collect()
}

pub fn vec_fixed64_to_u64(vec: Vec<I64F64>) -> Vec<u64> {
    vec.into_iter().map(fixed64_to_u64).collect()
}

pub fn vec_fixed_proportions_to_u16(vec: Vec<I32F32>) -> Vec<u16> {
    vec.into_iter().map(fixed_proportion_to_u16).collect()
}

// Max-upscale vector and convert to u16 so max_value = u16::MAX. Assumes non-negative normalized input.
pub fn vec_max_upscale_to_u16(vec: &[I32F32]) -> Vec<u16> {
    let u16_max: I32F32 = I32F32::saturating_from_num(u16::MAX);
    let threshold: I32F32 = I32F32::saturating_from_num(32768);
    let max_value: Option<&I32F32> = vec.iter().max();
    match max_value {
        Some(val) => {
            if *val == I32F32::saturating_from_num(0) {
                return vec
                    .iter()
                    .map(|e: &I32F32| e.saturating_mul(u16_max).saturating_to_num::<u16>())
                    .collect();
            }
            if *val > threshold {
                return vec
                    .iter()
                    .map(|e: &I32F32| {
                        e.saturating_mul(u16_max.safe_div(*val))
                            .round()
                            .saturating_to_num::<u16>()
                    })
                    .collect();
            }
            vec.iter()
                .map(|e: &I32F32| {
                    e.saturating_mul(u16_max)
                        .safe_div(*val)
                        .round()
                        .saturating_to_num::<u16>()
                })
                .collect()
        }
        None => {
            let sum: I32F32 = vec.iter().sum();
            vec.iter()
                .map(|e: &I32F32| {
                    e.saturating_mul(u16_max)
                        .safe_div(sum)
                        .saturating_to_num::<u16>()
                })
                .collect()
        }
    }
}

// Max-upscale u16 vector and convert to u16 so max_value = u16::MAX. Assumes u16 vector input.
pub fn vec_u16_max_upscale_to_u16(vec: &[u16]) -> Vec<u16> {
    let vec_fixed: Vec<I32F32> = vec
        .iter()
        .map(|e: &u16| I32F32::saturating_from_num(*e))
        .collect();
    vec_max_upscale_to_u16(&vec_fixed)
}

// Checks if u16 vector, when normalized, has a max value not greater than a u16 ratio max_limit.
pub fn check_vec_max_limited(vec: &[u16], max_limit: u16) -> bool {
    let max_limit_fixed: I32F32 =
        I32F32::saturating_from_num(max_limit).safe_div(I32F32::saturating_from_num(u16::MAX));
    let mut vec_fixed: Vec<I32F32> = vec
        .iter()
        .map(|e: &u16| I32F32::saturating_from_num(*e))
        .collect();
    inplace_normalize(&mut vec_fixed);
    let max_value: Option<&I32F32> = vec_fixed.iter().max();
    max_value.is_none_or(|v| *v <= max_limit_fixed)
}

pub fn sum(x: &[I32F32]) -> I32F32 {
    x.iter().sum()
}

// Sums a Vector of type that has CheckedAdd trait.
// Returns None if overflow occurs during sum using T::checked_add.
// Returns Some(T::default()) if input vector is empty.
pub fn checked_sum<T>(x: &[T]) -> Option<T>
where
    T: Copy + Default + CheckedAdd,
{
    let mut iter = x.iter();
    let Some(mut sum) = iter.next().copied() else {
        return Some(T::default());
    };
    for i in iter {
        sum = sum.checked_add(i)?;
    }
    Some(sum)
}

// Return true when vector sum is zero.
pub fn is_zero(vector: &[I32F32]) -> bool {
    let vector_sum: I32F32 = sum(vector);
    vector_sum == I32F32::saturating_from_num(0)
}

// Exp safe function with I32F32 output of I32F32 input.
pub fn exp_safe(input: I32F32) -> I32F32 {
    let min_input: I32F32 = I32F32::saturating_from_num(-20); // <= 1/exp(-20) = 485 165 195,4097903
    let max_input: I32F32 = I32F32::saturating_from_num(20); // <= exp(20) = 485 165 195,4097903
    let mut safe_input: I32F32 = input;
    if input < min_input {
        safe_input = min_input;
    } else if max_input < input {
        safe_input = max_input;
    }
    let output: I32F32;
    match exp(safe_input) {
        Ok(val) => {
            output = val;
        }
        Err(_err) => {
            if safe_input <= 0 {
                output = I32F32::saturating_from_num(0);
            } else {
                output = I32F32::max_value();
            }
        }
    }
    output
}

// Sigmoid safe function with I32F32 output of I32F32 input with offset kappa and (recommended) scaling 0 < rho <= 40.
pub fn sigmoid_safe(input: I32F32, rho: I32F32, kappa: I32F32) -> I32F32 {
    let one: I32F32 = I32F32::saturating_from_num(1);
    let offset: I32F32 = input.saturating_sub(kappa); // (input - kappa)
    let neg_rho: I32F32 = rho.saturating_mul(one.saturating_neg()); // -rho
    let exp_input: I32F32 = neg_rho.saturating_mul(offset); // -rho*(input-kappa)
    let exp_output: I32F32 = exp_safe(exp_input); // exp(-rho*(input-kappa))
    let denominator: I32F32 = exp_output.saturating_add(one); // 1 + exp(-rho*(input-kappa))
    let sigmoid_output: I32F32 = one.safe_div(denominator); // 1 / (1 + exp(-rho*(input-kappa)))
    sigmoid_output
}

// Returns a bool vector where an item is true if the vector item is in topk values.
pub fn is_topk(vector: &[I32F32], k: usize) -> Vec<bool> {
    let n: usize = vector.len();
    let mut result: Vec<bool> = vec![true; n];
    if n < k {
        return result;
    }
    let mut idxs: Vec<usize> = (0..n).collect();
    idxs.sort_by_key(|&idx| get_safe(vector, idx)); // ascending stable sort
    for &idx in idxs.iter().take(n.saturating_sub(k)) {
        if let Some(cell) = result.get_mut(idx) {
            *cell = false;
        }
    }
    result
}

// Returns a bool vector where an item is true if the vector item is in topk values and is non-zero.
pub fn is_topk_nonzero(vector: &[I32F32], k: usize) -> Vec<bool> {
    let n: usize = vector.len();
    let mut result: Vec<bool> = vector.iter().map(|&elem| elem != I32F32::from(0)).collect();
    if n < k {
        return result;
    }
    let mut idxs: Vec<usize> = (0..n).collect();
    idxs.sort_by_key(|&idx| get_safe(vector, idx)); // ascending stable sort
    for &idx in idxs.iter().take(n.saturating_sub(k)) {
        if let Some(cell) = result.get_mut(idx) {
            *cell = false;
        }
    }
    result
}

// Returns a normalized (sum to 1 except 0) copy of the input vector.
pub fn normalize(x: &[I32F32]) -> Vec<I32F32> {
    let x_sum: I32F32 = sum(x);
    if x_sum != I32F32::saturating_from_num(0.0_f32) {
        x.iter().map(|xi| xi.safe_div(x_sum)).collect()
    } else {
        x.to_vec()
    }
}

// Normalizes (sum to 1 except 0) the input vector directly in-place.
pub fn inplace_normalize(x: &mut [I32F32]) {
    let x_sum: I32F32 = x.iter().sum();
    if x_sum == I32F32::saturating_from_num(0.0_f32) {
        return;
    }
    x.iter_mut()
        .for_each(|value| *value = value.safe_div(x_sum));
}

// Normalizes (sum to 1 except 0) the input vector directly in-place, using the sum arg.
pub fn inplace_normalize_using_sum(x: &mut [I32F32], x_sum: I32F32) {
    if x_sum == I32F32::saturating_from_num(0.0_f32) {
        return;
    }
    x.iter_mut()
        .for_each(|value| *value = value.safe_div(x_sum));
}

// Normalizes (sum to 1 except 0) the I64F64 input vector directly in-place.
pub fn inplace_normalize_64(x: &mut [I64F64]) {
    let x_sum: I64F64 = x.iter().sum();
    if x_sum == I64F64::saturating_from_num(0) {
        return;
    }
    x.iter_mut()
        .for_each(|value| *value = value.safe_div(x_sum));
}

/// Normalizes (sum to 1 except 0) each row (dim=0) of a I64F64 matrix in-place.
pub fn inplace_row_normalize_64(x: &mut [Vec<I64F64>]) {
    for row in x {
        let row_sum: I64F64 = row.iter().sum();
        if row_sum > I64F64::saturating_from_num(0.0_f64) {
            row.iter_mut()
                .for_each(|x_ij: &mut I64F64| *x_ij = x_ij.safe_div(row_sum));
        }
    }
}

/// Returns x / y for input vectors x and y, if y == 0 return 0.
pub fn vecdiv(x: &[I32F32], y: &[I32F32]) -> Vec<I32F32> {
    if x.len() != y.len() {
        log::error!(
            "math error: vecdiv input lengths are not equal: {:?} != {:?}",
            x.len(),
            y.len()
        );
    }

    let zero = I32F32::saturating_from_num(0);

    let mut out = Vec::with_capacity(x.len());
    for (i, x_i) in x.iter().enumerate() {
        let y_i = y.get(i).copied().unwrap_or(zero);
        out.push(x_i.safe_div(y_i));
    }
    out
}

// Normalizes (sum to 1 except 0) each row (dim=0) of a matrix in-place.
pub fn inplace_row_normalize(x: &mut [Vec<I32F32>]) {
    for row in x {
        let row_sum: I32F32 = row.iter().sum();
        if row_sum > I32F32::saturating_from_num(0.0_f32) {
            row.iter_mut()
                .for_each(|x_ij: &mut I32F32| *x_ij = x_ij.safe_div(row_sum));
        }
    }
}

// Normalizes (sum to 1 except 0) each row (dim=0) of a sparse matrix in-place.
pub fn inplace_row_normalize_sparse(sparse_matrix: &mut [Vec<(u16, I32F32)>]) {
    for sparse_row in sparse_matrix.iter_mut() {
        let row_sum: I32F32 = sparse_row.iter().map(|(_j, value)| *value).sum();
        if row_sum > I32F32::saturating_from_num(0.0) {
            sparse_row
                .iter_mut()
                .for_each(|(_j, value)| *value = value.safe_div(row_sum));
        }
    }
}

// Sum across each row (dim=0) of a matrix.
pub fn row_sum(x: &[Vec<I32F32>]) -> Vec<I32F32> {
    if let Some(first_row) = x.first()
        && first_row.is_empty()
    {
        return vec![];
    }
    x.iter().map(|row| row.iter().sum()).collect()
}

// Sum across each row (dim=0) of a sparse matrix.
pub fn row_sum_sparse(sparse_matrix: &[Vec<(u16, I32F32)>]) -> Vec<I32F32> {
    sparse_matrix
        .iter()
        .map(|row| row.iter().map(|(_, value)| value).sum())
        .collect()
}

// Normalizes (sum to 1 except 0) each column (dim=1) of a sparse matrix in-place.
pub fn inplace_col_normalize_sparse(sparse_matrix: &mut [Vec<(u16, I32F32)>], columns: u16) {
    let zero = I32F32::saturating_from_num(0.0);
    let mut col_sum: Vec<I32F32> = vec![zero; columns as usize];

    // Pass 1: accumulate column sums.
    for sparse_row in sparse_matrix.iter() {
        for &(j, value) in sparse_row.iter() {
            if let Some(sum) = col_sum.get_mut(j as usize) {
                *sum = sum.saturating_add(value);
            }
        }
    }

    // Pass 2: normalize by column sums where non-zero.
    for sparse_row in sparse_matrix.iter_mut() {
        for (j, value) in sparse_row.iter_mut() {
            let denom = col_sum.get(*j as usize).copied().unwrap_or(zero);
            if denom != zero {
                *value = value.safe_div(denom);
            }
        }
    }
}

// Normalizes (sum to 1 except 0) each column (dim=1) of a matrix in-place.
// If a row is shorter/longer than the accumulator, pad with zeroes accordingly.
pub fn inplace_col_normalize(x: &mut [Vec<I32F32>]) {
    let zero = I32F32::saturating_from_num(0.0);

    // Build column sums; treat missing entries as zero, but don't modify rows.
    let mut col_sums: Vec<I32F32> = Vec::new();
    for row in x.iter() {
        if col_sums.len() < row.len() {
            col_sums.resize(row.len(), zero);
        }
        let mut sums_it = col_sums.iter_mut();
        for v in row.iter() {
            if let Some(sum) = sums_it.next() {
                *sum = sum.saturating_add(*v);
            } else {
                break;
            }
        }
    }

    if col_sums.is_empty() {
        return;
    }

    // Normalize only existing elements in each row.
    for row in x.iter_mut() {
        let mut sums_it = col_sums.iter();
        for m in row.iter_mut() {
            if let Some(sum) = sums_it.next() {
                if *sum != zero {
                    *m = m.safe_div(*sum);
                }
            } else {
                break;
            }
        }
    }
}

// Max-upscale each column (dim=1) of a sparse matrix in-place.
pub fn inplace_col_max_upscale_sparse(sparse_matrix: &mut [Vec<(u16, I32F32)>], columns: u16) {
    let zero = I32F32::saturating_from_num(0.0);
    let mut col_max: Vec<I32F32> = vec![zero; columns as usize];

    // Pass 1: compute per-column max
    for sparse_row in sparse_matrix.iter() {
        for (j, value) in sparse_row.iter() {
            if let Some(m) = col_max.get_mut(*j as usize)
                && *m < *value
            {
                *m = *value;
            }
        }
    }

    // Pass 2: divide each nonzero entry by its column max
    for sparse_row in sparse_matrix.iter_mut() {
        for (j, value) in sparse_row.iter_mut() {
            let m = col_max.get(*j as usize).copied().unwrap_or(zero);
            if m != zero {
                *value = value.safe_div(m);
            }
        }
    }
}

// Max-upscale each column (dim=1) of a matrix in-place.
pub fn inplace_col_max_upscale(x: &mut [Vec<I32F32>]) {
    let zero = I32F32::saturating_from_num(0.0);

    // Find the widest row to size the column-max buffer; don't modify rows.
    let max_cols = x.iter().map(|r| r.len()).max().unwrap_or(0);
    if max_cols == 0 {
        return;
    }

    // Pass 1: compute per-column maxima across existing entries only.
    let mut col_maxes = vec![zero; max_cols];
    for row in x.iter() {
        let mut max_it = col_maxes.iter_mut();
        for v in row.iter() {
            if let Some(m) = max_it.next() {
                if *m < *v {
                    *m = *v;
                }
            } else {
                break;
            }
        }
    }

    // Pass 2: divide each existing entry by its column max (if non-zero).
    for row in x.iter_mut() {
        let mut max_it = col_maxes.iter();
        for val in row.iter_mut() {
            if let Some(&m) = max_it.next() {
                if m != zero {
                    *val = val.safe_div(m);
                }
            } else {
                break;
            }
        }
    }
}

// Apply mask to vector, mask=true will mask out, i.e. set to 0.
pub fn inplace_mask_vector(mask: &[bool], vector: &mut [I32F32]) {
    if mask.len() != vector.len() {
        log::error!(
            "math error: inplace_mask_vector input lengths are not equal: {:?} != {:?}",
            mask.len(),
            vector.len()
        );
    }

    if mask.is_empty() {
        return;
    }
    let zero: I32F32 = I32F32::saturating_from_num(0.0);
    for (i, v) in vector.iter_mut().enumerate() {
        if *mask.get(i).unwrap_or(&true) {
            *v = zero;
        }
    }
}

// Apply mask to matrix, mask=true will mask out, i.e. set to 0.
pub fn inplace_mask_matrix(mask: &[Vec<bool>], matrix: &mut [Vec<I32F32>]) {
    if mask.len() != matrix.len() {
        log::error!(
            "math error: inplace_mask_matrix input sizes are not equal: {:?} != {:?}",
            mask.len(),
            matrix.len()
        );
    }
    let Some(first_row) = mask.first() else {
        return;
    };
    if first_row.is_empty() {
        return;
    }
    let zero: I32F32 = I32F32::saturating_from_num(0.0);
    for (r, row) in matrix.iter_mut().enumerate() {
        let mask_row_opt = mask.get(r);
        for (c, val) in row.iter_mut().enumerate() {
            let should_zero = mask_row_opt
                .and_then(|mr| mr.get(c))
                .copied()
                .unwrap_or(true);
            if should_zero {
                *val = zero;
            }
        }
    }
}

// Apply row mask to matrix, mask=true will mask out, i.e. set to 0.
pub fn inplace_mask_rows(mask: &[bool], matrix: &mut [Vec<I32F32>]) {
    if mask.len() != matrix.len() {
        log::error!(
            "math error: inplace_mask_rows input sizes are not equal: {:?} != {:?}",
            mask.len(),
            matrix.len()
        );
    }
    let Some(first_row) = matrix.first() else {
        return;
    };
    let cols = first_row.len();
    let zero: I32F32 = I32F32::saturating_from_num(0);
    for (r, row) in matrix.iter_mut().enumerate() {
        if mask.get(r).copied().unwrap_or(true) {
            *row = vec![zero; cols];
        }
    }
}

// Apply column mask to matrix, mask=true will mask out, i.e. set to 0.
// Assumes each column has the same length.
pub fn inplace_mask_cols(mask: &[bool], matrix: &mut [Vec<I32F32>]) {
    if mask.len() != matrix.len() {
        log::error!(
            "math error: inplace_mask_cols input sizes are not equal: {:?} != {:?}",
            mask.len(),
            matrix.len()
        );
    }
    if matrix.is_empty() {
        return;
    };
    let zero: I32F32 = I32F32::saturating_from_num(0);
    for row in matrix.iter_mut() {
        for (c, elem) in row.iter_mut().enumerate() {
            if mask.get(c).copied().unwrap_or(true) {
                *elem = zero;
            }
        }
    }
}

// Mask out the diagonal of the input matrix in-place.
pub fn inplace_mask_diag(matrix: &mut [Vec<I32F32>]) {
    let Some(first_row) = matrix.first() else {
        return;
    };
    if first_row.is_empty() {
        return;
    }
    // Weights that we use this function for are always a square matrix.
    // If something not square is passed to this function, it's safe to return
    // with no action. Log error if this happens.
    if matrix.len() != first_row.len() {
        log::error!(
            "math error: inplace_mask_diag: matrix.len {:?} != first_row.len {:?}",
            matrix.len(),
            first_row.len()
        );
        return;
    }

    let zero: I32F32 = I32F32::saturating_from_num(0.0);
    matrix.iter_mut().enumerate().for_each(|(idx, row)| {
        let Some(elem) = row.get_mut(idx) else {
            // Should not happen since matrix is square
            return;
        };
        *elem = zero;
    });
}

// Remove cells from sparse matrix where the mask function of a scalar and a vector is true.
pub fn scalar_vec_mask_sparse_matrix(
    sparse_matrix: &[Vec<(u16, I32F32)>],
    scalar: u64,
    vector: &[u64],
    mask_fn: &dyn Fn(u64, u64) -> bool,
) -> Vec<Vec<(u16, I32F32)>> {
    let mut result: Vec<Vec<(u16, I32F32)>> = Vec::with_capacity(sparse_matrix.len());

    for row in sparse_matrix.iter() {
        let mut out_row: Vec<(u16, I32F32)> = Vec::with_capacity(row.len());
        for &(j, value) in row.iter() {
            let vj = vector.get(j as usize).copied().unwrap_or(0);
            if !mask_fn(scalar, vj) {
                out_row.push((j, value));
            }
        }
        result.push(out_row);
    }

    result
}

// Mask out the diagonal of the input matrix in-place, except for the diagonal entry at except_index.
pub fn inplace_mask_diag_except_index(matrix: &mut [Vec<I32F32>], except_index: u16) {
    let Some(first_row) = matrix.first() else {
        return;
    };
    if first_row.is_empty() {
        return;
    }
    if matrix.len() != first_row.len() {
        log::error!(
            "math error: inplace_mask_diag input matrix is now square: {:?} != {:?}",
            matrix.len(),
            first_row.len()
        );
        return;
    }
    let diag_at_index = matrix
        .get(except_index as usize)
        .and_then(|row| row.get(except_index as usize))
        .cloned();

    inplace_mask_diag(matrix);

    matrix.get_mut(except_index as usize).map(|row| {
        row.get_mut(except_index as usize).map(|value| {
            if let Some(diag_at_index) = diag_at_index {
                *value = diag_at_index;
            }
        })
    });
}

// Return a new sparse matrix that replaces masked rows with an empty vector placeholder.
pub fn mask_rows_sparse(
    mask: &[bool],
    sparse_matrix: &[Vec<(u16, I32F32)>],
) -> Vec<Vec<(u16, I32F32)>> {
    let mut out = Vec::with_capacity(sparse_matrix.len());
    for (i, sparse_row) in sparse_matrix.iter().enumerate() {
        if mask.get(i).copied().unwrap_or(true) {
            out.push(Vec::new());
        } else {
            out.push(sparse_row.clone());
        }
    }
    out
}

// Return a new sparse matrix with a masked out diagonal of input sparse matrix.
pub fn mask_diag_sparse(sparse_matrix: &[Vec<(u16, I32F32)>]) -> Vec<Vec<(u16, I32F32)>> {
    sparse_matrix
        .iter()
        .enumerate()
        .map(|(i, sparse_row)| {
            sparse_row
                .iter()
                .filter(|(j, _)| i != (*j as usize))
                .copied()
                .collect()
        })
        .collect()
}

// Return a new sparse matrix with a masked out diagonal of input sparse matrix,
// except for the diagonal entry at except_index.
pub fn mask_diag_sparse_except_index(
    sparse_matrix: &[Vec<(u16, I32F32)>],
    except_index: u16,
) -> Vec<Vec<(u16, I32F32)>> {
    sparse_matrix
        .iter()
        .enumerate()
        .map(|(i, sparse_row)| {
            sparse_row
                .iter()
                .filter(|(j, _)| {
                    // Is not a diagonal OR is the diagonal at except_index
                    i != (*j as usize) || (i == except_index as usize && *j == except_index)
                })
                .copied()
                .collect()
        })
        .collect()
}

// Remove cells from sparse matrix where the mask function of two vectors is true.
pub fn vec_mask_sparse_matrix(
    sparse_matrix: &[Vec<(u16, I32F32)>],
    first_vector: &[u64],
    second_vector: &[u64],
    mask_fn: &dyn Fn(u64, u64) -> bool,
) -> Vec<Vec<(u16, I32F32)>> {
    let mut result: Vec<Vec<(u16, I32F32)>> = Vec::with_capacity(sparse_matrix.len());
    let mut fv_it = first_vector.iter();
    for row in sparse_matrix.iter() {
        let fv = fv_it.next().copied().unwrap_or(0);
        let mut out_row: Vec<(u16, I32F32)> = Vec::with_capacity(row.len());
        for &(j, val) in row.iter() {
            let sv = second_vector.get(j as usize).copied().unwrap_or(0);
            if !mask_fn(fv, sv) {
                out_row.push((j, val));
            }
        }
        result.push(out_row);
    }
    result
}

// Row-wise matrix-vector hadamard product.
pub fn row_hadamard(matrix: &[Vec<I32F32>], vector: &[I32F32]) -> Vec<Vec<I32F32>> {
    let Some(first_row) = matrix.first() else {
        return vec![vec![]];
    };
    if first_row.is_empty() {
        return vec![vec![]];
    }

    let mut out = Vec::with_capacity(matrix.len());
    let mut vec_it = vector.iter();

    for row in matrix.iter() {
        let Some(&scale) = vec_it.next() else { break };
        let mut new_row = Vec::with_capacity(row.len());
        for m_val in row.iter() {
            new_row.push(scale.saturating_mul(*m_val));
        }
        out.push(new_row);
    }

    out
}

// Row-wise sparse matrix-vector hadamard product.
pub fn row_hadamard_sparse(
    sparse_matrix: &[Vec<(u16, I32F32)>],
    vector: &[I32F32],
) -> Vec<Vec<(u16, I32F32)>> {
    let mut out = Vec::with_capacity(sparse_matrix.len());
    let mut vec_it = vector.iter();

    for sparse_row in sparse_matrix.iter() {
        let Some(&scale) = vec_it.next() else { break };
        let mut new_row = Vec::with_capacity(sparse_row.len());
        for &(j, val) in sparse_row.iter() {
            new_row.push((j, val.saturating_mul(scale)));
        }
        out.push(new_row);
    }

    out
}

// Row-wise matrix-vector product, column-wise sum: result_j = SUM(i) vector_i * matrix_ij.
pub fn matmul(matrix: &[Vec<I32F32>], vector: &[I32F32]) -> Vec<I32F32> {
    let Some(first_row) = matrix.first() else {
        return vec![];
    };
    let cols = first_row.len();
    if cols == 0 {
        return vec![];
    }
    if matrix.len() != vector.len() {
        log::error!(
            "math error: matmul input sizes are not equal: {:?} != {:?}",
            matrix.len(),
            vector.len()
        );
    }

    let zero = I32F32::saturating_from_num(0.0);
    let mut acc = vec![zero; cols];

    let mut vec_it = vector.iter();
    for row in matrix.iter() {
        // Use 0 if the vector ran out (rows beyond vector length contribute nothing).
        let scale = vec_it.next().copied().unwrap_or(zero);

        let mut acc_it = acc.iter_mut();
        for m_val in row.iter() {
            if let Some(a) = acc_it.next() {
                *a = a.saturating_add(scale.saturating_mul(*m_val));
            } else {
                // Ignore elements beyond the accumulator width (first row’s length).
                break;
            }
        }
    }

    acc
}

// Column-wise matrix-vector product, row-wise sum: result_i = SUM(j) vector_j * matrix_ij.
pub fn matmul_transpose(matrix: &[Vec<I32F32>], vector: &[I32F32]) -> Vec<I32F32> {
    let Some(first_row) = matrix.first() else {
        return vec![];
    };
    if first_row.is_empty() {
        return vec![];
    }
    if vector.len() != first_row.len() {
        log::error!(
            "math error: matmul_transpose matrix width doesn't match to vector height: {:?} != {:?}",
            first_row.len(),
            vector.len()
        );
    }

    let zero = I32F32::saturating_from_num(0.0);
    let mut out = Vec::with_capacity(matrix.len());

    for row in matrix.iter() {
        let mut sum = zero;
        let mut v_it = vector.iter();
        for m in row.iter() {
            if let Some(&v) = v_it.next() {
                sum = sum.saturating_add(m.saturating_mul(v));
            } else {
                break;
            }
        }
        out.push(sum);
    }

    out
}

// Row-wise sparse_matrix-vector product, column-wise sum: result_j = SUM(i) vector_i * matrix_ij.
pub fn matmul_sparse(
    sparse_matrix: &[Vec<(u16, I32F32)>],
    vector: &[I32F32],
    columns: u16,
) -> Vec<I32F32> {
    let zero = I32F32::saturating_from_num(0.0);
    let mut result = vec![zero; columns as usize];

    let mut vec_it = vector.iter();
    for row in sparse_matrix.iter() {
        let scale = vec_it.next().copied().unwrap_or(zero);
        for &(j, val) in row.iter() {
            if let Some(r) = result.get_mut(j as usize) {
                *r = r.saturating_add(scale.saturating_mul(val));
            }
        }
    }

    result
}

// Column-wise sparse_matrix-vector product, row-wise sum: result_i = SUM(j) vector_j * matrix_ij.
pub fn matmul_transpose_sparse(
    sparse_matrix: &[Vec<(u16, I32F32)>],
    vector: &[I32F32],
) -> Vec<I32F32> {
    let zero = I32F32::saturating_from_num(0.0);
    let mut result = vec![zero; sparse_matrix.len()];

    let mut out_it = result.iter_mut();
    for row in sparse_matrix.iter() {
        let Some(out_cell) = out_it.next() else { break };
        let mut acc = zero;
        for &(j, val) in row.iter() {
            let v = vector.get(j as usize).copied().unwrap_or(zero);
            acc = acc.saturating_add(v.saturating_mul(val));
        }
        *out_cell = acc;
    }

    result
}

// Set inplace matrix values above column threshold to threshold value.
pub fn inplace_col_clip(x: &mut [Vec<I32F32>], col_threshold: &[I32F32]) {
    for row in x.iter_mut() {
        let mut thr_it = col_threshold.iter();
        for value in row.iter_mut() {
            if let Some(th) = thr_it.next() {
                // Clip: value = min(value, threshold)
                *value = *th.min(&*value);
            } else {
                // No more thresholds; stop for this row.
                break;
            }
        }
    }
}

// Return sparse matrix with values above column threshold set to threshold value.
pub fn col_clip_sparse(
    sparse_matrix: &[Vec<(u16, I32F32)>],
    col_threshold: &[I32F32],
) -> Vec<Vec<(u16, I32F32)>> {
    let zero = I32F32::saturating_from_num(0.0);
    let mut result = Vec::with_capacity(sparse_matrix.len());

    for row in sparse_matrix.iter() {
        let mut out_row: Vec<(u16, I32F32)> = Vec::with_capacity(row.len());
        for &(j, val) in row.iter() {
            let th = col_threshold.get(j as usize).copied().unwrap_or(zero);
            if th < val {
                if th > zero {
                    // clip down to threshold, but drop if threshold <= 0
                    out_row.push((j, th));
                }
            } else {
                // keep original
                out_row.push((j, val));
            }
        }
        result.push(out_row);
    }

    result
}

// Stake-weighted median score finding algorithm, based on a mid pivot binary search.
// Normally a random pivot is used, but to ensure full determinism the mid point is chosen instead.
// Assumes relatively random score order for efficiency, typically less than O(nlogn) complexity.
//
// # Args:
// 	* 'stake': ( &[I32F32] ):
//         - stake, assumed to be normalized.
//
// 	* 'score': ( &[I32F32] ):
//         - score for which median is sought, 0 <= score <= 1
//
// 	* 'partition_idx' ( &[usize] ):
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
pub fn weighted_median(
    stake: &[I32F32],
    score: &[I32F32],
    partition_idx: &[usize],
    minority: I32F32,
    mut partition_lo: I32F32,
    mut partition_hi: I32F32,
) -> I32F32 {
    let zero = I32F32::saturating_from_num(0.0);
    if stake.len() != score.len() {
        log::error!(
            "math error: weighted_median stake and score have different lengths: {:?} != {:?}",
            stake.len(),
            score.len()
        );
        return zero;
    }
    let mut current_partition_index: Vec<usize> = partition_idx.to_vec();
    let mut iteration_counter: usize = 0;
    let iteration_limit = partition_idx.len();
    let mut lower: Vec<usize> = vec![];
    let mut upper: Vec<usize> = vec![];

    loop {
        let n = current_partition_index.len();
        if n == 0 {
            return zero;
        }
        if n == 1 {
            if let Some(&only_idx) = current_partition_index.first() {
                return get_safe::<I32F32>(score, only_idx);
            } else {
                return zero;
            }
        }
        let mid_idx: usize = n.safe_div(2);
        let pivot: I32F32 = get_safe::<I32F32>(
            score,
            current_partition_index.get(mid_idx).copied().unwrap_or(0),
        );
        let mut lo_stake: I32F32 = I32F32::saturating_from_num(0);
        let mut hi_stake: I32F32 = I32F32::saturating_from_num(0);

        for idx in current_partition_index.clone() {
            if get_safe::<I32F32>(score, idx) == pivot {
                continue;
            }
            if get_safe::<I32F32>(score, idx) < pivot {
                lo_stake = lo_stake.saturating_add(get_safe::<I32F32>(stake, idx));
                lower.push(idx);
            } else {
                hi_stake = hi_stake.saturating_add(get_safe::<I32F32>(stake, idx));
                upper.push(idx);
            }
        }
        if (minority < partition_lo.saturating_add(lo_stake)) && (!lower.is_empty()) {
            current_partition_index = lower.clone();
            partition_hi = partition_lo.saturating_add(lo_stake);
        } else if (partition_hi.saturating_sub(hi_stake) <= minority) && (!upper.is_empty()) {
            current_partition_index = upper.clone();
            partition_lo = partition_hi.saturating_sub(hi_stake);
        } else {
            return pivot;
        }

        lower.clear();
        upper.clear();

        // Safety limit: We should never need more than iteration_limit iterations.
        iteration_counter = iteration_counter.saturating_add(1);
        if iteration_counter > iteration_limit {
            break;
        }
    }
    zero
}

/// Column-wise weighted median, e.g. stake-weighted median scores per server (column) over all validators (rows).
pub fn weighted_median_col(
    stake: &[I32F32],
    score: &[Vec<I32F32>],
    majority: I32F32,
) -> Vec<I32F32> {
    let zero = I32F32::saturating_from_num(0.0);

    // Determine number of columns from the first row.
    let columns = score.first().map(|r| r.len()).unwrap_or(0);
    let mut median = vec![zero; columns];

    // Iterate columns into `median`.
    let mut c = 0usize;
    for med_cell in median.iter_mut() {
        let mut use_stake: Vec<I32F32> = Vec::new();
        let mut use_score: Vec<I32F32> = Vec::new();

        // Iterate rows aligned with `stake` length.
        let mut r = 0usize;
        while r < stake.len() {
            let st = get_safe::<I32F32>(stake, r);
            if st > zero {
                // Fetch row safely; if it's missing or has wrong width, push zeros to both.
                if let Some(row) = score.get(r) {
                    if row.len() == columns {
                        let val = row.get(c).copied().unwrap_or(zero);
                        use_stake.push(st);
                        use_score.push(val);
                    } else {
                        use_stake.push(zero);
                        use_score.push(zero);
                        log::error!(
                            "math error: weighted_median_col row.len() != columns: {:?} != {:?}",
                            row.len(),
                            columns
                        );
                    }
                } else {
                    // Missing row: insert zeroes.
                    use_stake.push(zero);
                    use_score.push(zero);
                }
            }
            r = r.saturating_add(1);
        }

        if !use_stake.is_empty() {
            inplace_normalize(&mut use_stake);
            let stake_sum: I32F32 = use_stake.iter().sum();
            let minority: I32F32 = stake_sum.saturating_sub(majority);

            let idxs: Vec<usize> = (0..use_stake.len()).collect();
            *med_cell = weighted_median(
                &use_stake,
                &use_score,
                idxs.as_slice(),
                minority,
                zero,
                stake_sum,
            );
        }

        c = c.saturating_add(1);
    }
    median
}

/// Column-wise weighted median, e.g. stake-weighted median scores per server (column) over all validators (rows).
pub fn weighted_median_col_sparse(
    stake: &[I32F32],
    score: &[Vec<(u16, I32F32)>],
    columns: u16,
    majority: I32F32,
) -> Vec<I32F32> {
    let zero = I32F32::saturating_from_num(0.0);

    // Keep only positive-stake rows; normalize them.
    let mut use_stake: Vec<I32F32> = stake.iter().copied().filter(|&s| s > zero).collect();
    inplace_normalize(&mut use_stake);

    let stake_sum: I32F32 = use_stake.iter().sum();
    let minority: I32F32 = stake_sum.saturating_sub(majority);
    let stake_idx: Vec<usize> = (0..use_stake.len()).collect();

    // use_score: columns x use_stake.len(), prefilled with zeros.
    let mut use_score: Vec<Vec<I32F32>> = (0..columns as usize)
        .map(|_| vec![zero; use_stake.len()])
        .collect();

    // Fill use_score by walking stake and score together, counting positives with k.
    let mut k: usize = 0;
    let mut stake_it = stake.iter();
    let mut score_it = score.iter();

    while let (Some(&s), Some(sparse_row)) = (stake_it.next(), score_it.next()) {
        if s > zero {
            for &(c, val) in sparse_row.iter() {
                if let Some(col_vec) = use_score.get_mut(c as usize)
                    && let Some(cell) = col_vec.get_mut(k)
                {
                    *cell = val;
                }
            }
            k = k.saturating_add(1);
        }
    }

    // Compute weighted median per column.
    let mut median: Vec<I32F32> = Vec::with_capacity(columns as usize);
    for col_vec in use_score.iter() {
        median.push(weighted_median(
            &use_stake,
            col_vec,
            stake_idx.as_slice(),
            minority,
            zero,
            stake_sum,
        ));
    }

    median
}

// Element-wise interpolation of two matrices: Result = A + ratio * (B - A).
// ratio has intended range [0, 1]
// ratio=0: Result = A
// ratio=1: Result = B
pub fn interpolate(mat1: &[Vec<I32F32>], mat2: &[Vec<I32F32>], ratio: I32F32) -> Vec<Vec<I32F32>> {
    if ratio == I32F32::saturating_from_num(0.0) {
        return mat1.to_owned();
    }
    if ratio == I32F32::saturating_from_num(1.0) {
        return mat2.to_owned();
    }
    if mat1.is_empty() || mat1.first().map(|r| r.is_empty()).unwrap_or(true) {
        return vec![vec![]];
    }
    if mat1.len() != mat2.len() {
        log::error!(
            "math error: interpolate mat1.len() != mat2.len(): {:?} != {:?}",
            mat1.len(),
            mat2.len()
        );
    }

    let zero = I32F32::saturating_from_num(0.0);
    let cols = mat1.first().map(|r| r.len()).unwrap_or(0);

    // Pre-size result to mat1's shape (row count = mat1.len(), col count = first row of mat1).
    let mut result: Vec<Vec<I32F32>> = {
        let mut out = Vec::with_capacity(mat1.len());
        for _ in mat1.iter() {
            out.push(vec![zero; cols]);
        }
        out
    };

    // Walk rows of mat1, mat2, and result in lockstep; stop when any iterator ends.
    let mut m2_it = mat2.iter();
    let mut out_it = result.iter_mut();

    for row1 in mat1.iter() {
        let (Some(row2), Some(out_row)) = (m2_it.next(), out_it.next()) else {
            log::error!("math error: interpolate: No more rows in mat2");
            break;
        };
        if row1.len() != row2.len() {
            log::error!(
                "math error: interpolate row1.len() != row2.len(): {:?} != {:?}",
                row1.len(),
                row2.len()
            );
        }

        // Walk elements of row1, row2, and out_row in lockstep; stop at the shortest.
        let mut r1_it = row1.iter();
        let mut r2_it = row2.iter();
        let mut out_cell_it = out_row.iter_mut();

        while let (Some(v1), Some(v2), Some(out_cell)) =
            (r1_it.next(), r2_it.next(), out_cell_it.next())
        {
            *out_cell = (*v1).saturating_add(ratio.saturating_mul((*v2).saturating_sub(*v1)));
        }
        // Any remaining cells in `out_row` (beyond min row length) stay as zero (pre-filled).
    }

    result
}

// Element-wise interpolation of two sparse matrices: Result = A + ratio * (B - A).
// ratio has intended range [0, 1]
// ratio=0: Result = A
// ratio=1: Result = B
pub fn interpolate_sparse(
    mat1: &[Vec<(u16, I32F32)>],
    mat2: &[Vec<(u16, I32F32)>],
    columns: u16,
    ratio: I32F32,
) -> Vec<Vec<(u16, I32F32)>> {
    if ratio == I32F32::saturating_from_num(0) {
        return mat1.to_owned();
    }
    if ratio == I32F32::saturating_from_num(1) {
        return mat2.to_owned();
    }
    if mat1.len() != mat2.len() {
        // In case if sizes mismatch, return clipped weights
        log::error!(
            "math error: interpolate_sparse: mat1.len() != mat2.len(): {:?} != {:?}",
            mat1.len(),
            mat2.len()
        );
        return mat2.to_owned();
    }
    let rows = mat1.len();
    let zero: I32F32 = I32F32::saturating_from_num(0);
    let mut result: Vec<Vec<(u16, I32F32)>> = vec![vec![]; rows];
    for i in 0..rows {
        let mut row1: Vec<I32F32> = vec![zero; columns as usize];
        if let Some(row) = mat1.get(i) {
            for (j, value) in row {
                if let Some(entry) = row1.get_mut(*j as usize) {
                    *entry = *value;
                }
            }
        }
        let mut row2: Vec<I32F32> = vec![zero; columns as usize];
        if let Some(row) = mat2.get(i) {
            for (j, value) in row {
                if let Some(entry) = row2.get_mut(*j as usize) {
                    *entry = *value;
                }
            }
        }
        for j in 0..columns as usize {
            let v1 = row1.get(j).unwrap_or(&zero);
            let v2 = row2.get(j).unwrap_or(&zero);
            let interp = v1.saturating_add(ratio.saturating_mul(v2.saturating_sub(*v1)));
            if zero < interp
                && let Some(res) = result.get_mut(i)
            {
                res.push((j as u16, interp));
            }
        }
    }
    result
}

// Element-wise product of two vectors.
pub fn vec_mul(a: &[I32F32], b: &[I32F32]) -> Vec<I32F32> {
    let mut out = Vec::with_capacity(core::cmp::min(a.len(), b.len()));
    let mut ai = a.iter();
    let mut bi = b.iter();

    while let (Some(x), Some(y)) = (ai.next(), bi.next()) {
        out.push(x.checked_mul(*y).unwrap_or_default());
    }

    out
}

// Element-wise product of matrix and vector
pub fn mat_vec_mul(matrix: &[Vec<I32F32>], vector: &[I32F32]) -> Vec<Vec<I32F32>> {
    let Some(first_row) = matrix.first() else {
        return vec![vec![]];
    };
    if first_row.is_empty() {
        return vec![vec![]];
    }

    let mut out = Vec::with_capacity(matrix.len());
    for row in matrix.iter() {
        out.push(vec_mul(row, vector));
    }
    out
}

// Element-wise product of matrix and vector
pub fn mat_vec_mul_sparse(
    matrix: &[Vec<(u16, I32F32)>],
    vector: &[I32F32],
) -> Vec<Vec<(u16, I32F32)>> {
    let mut result: Vec<Vec<(u16, I32F32)>> = vec![vec![]; matrix.len()];
    for (i, matrix_row) in matrix.iter().enumerate() {
        for (j, value) in matrix_row.iter() {
            if let Some(vector_value) = vector.get(*j as usize) {
                let new_value = value.saturating_mul(*vector_value);
                if new_value != I32F32::saturating_from_num(0.0)
                    && let Some(result_row) = result.get_mut(i)
                {
                    result_row.push((*j, new_value));
                }
            }
        }
    }
    result
}

/// Clamp the input value between high and low.
/// Note: assumes high > low
pub fn clamp_value(value: I32F32, low: I32F32, high: I32F32) -> I32F32 {
    // First, clamp the value to ensure it does not exceed the upper bound (high).
    // If the value is greater than 'high', it will be set to 'high'.
    // otherwise it remains unchanged.
    value
        .min(I32F32::from_num(high))
        // Next, clamp the value to ensure it does not go below the lower bound (_low).
        // If the value (after the first clamping) is less than 'low', it will be set to 'low'.
        // otherwise it remains unchanged.
        .max(I32F32::from_num(low))
}

// Return matrix exponential moving average: `alpha * a_ij + one_minus_alpha * b_ij`.
// `alpha` is the EMA coefficient, how much to add of the new observation, typically small,
// higher alpha discounts older observations faster.
pub fn mat_ema(new: &[Vec<I32F32>], old: &[Vec<I32F32>], alpha: I32F32) -> Vec<Vec<I32F32>> {
    let Some(first_row) = new.first() else {
        return vec![vec![]];
    };
    if first_row.is_empty() {
        return vec![vec![]; 1];
    }

    let one_minus_alpha = I32F32::saturating_from_num(1.0).saturating_sub(alpha);

    let mut out = Vec::with_capacity(new.len());
    let mut old_it = old.iter();

    for new_row in new.iter() {
        let Some(old_row) = old_it.next() else { break };

        let mut row_out = Vec::with_capacity(core::cmp::min(new_row.len(), old_row.len()));
        let mut n_it = new_row.iter();
        let mut o_it = old_row.iter();

        while let (Some(&n), Some(&o)) = (n_it.next(), o_it.next()) {
            row_out.push(
                alpha
                    .saturating_mul(n)
                    .saturating_add(one_minus_alpha.saturating_mul(o)),
            );
        }

        out.push(row_out);
    }

    out
}

// Return sparse matrix exponential moving average: `alpha * a_ij + one_minus_alpha * b_ij`.
// `alpha` is the EMA coefficient, how much to add of the new observation, typically small,
// higher alpha discounts older observations faster.
pub fn mat_ema_sparse(
    new: &[Vec<(u16, I32F32)>],
    old: &[Vec<(u16, I32F32)>],
    alpha: I32F32,
) -> Vec<Vec<(u16, I32F32)>> {
    if new.len() != old.len() {
        log::error!(
            "math error: mat_ema_sparse: new.len() == old.len(): {:?} != {:?}",
            new.len(),
            old.len()
        );
    }

    let zero = I32F32::saturating_from_num(0.0);
    let one_minus_alpha = I32F32::saturating_from_num(1.0).saturating_sub(alpha);

    let n = new.len(); // assume square (rows = cols)
    if n == 0 {
        return Vec::new();
    }

    let mut result: Vec<Vec<(u16, I32F32)>> = Vec::with_capacity(n);
    let mut old_it = old.iter();

    for new_row in new.iter() {
        let mut acc_row = vec![zero; n];

        // Add alpha * new
        for &(j, v) in new_row.iter() {
            if let Some(cell) = acc_row.get_mut(j as usize) {
                *cell = cell.saturating_add(alpha.saturating_mul(v));
            }
        }

        // Add (1 - alpha) * old
        if let Some(orow) = old_it.next() {
            for &(j, v) in orow.iter() {
                if let Some(cell) = acc_row.get_mut(j as usize) {
                    *cell = cell.saturating_add(one_minus_alpha.saturating_mul(v));
                }
            }
        }

        // Densified row -> sparse (keep positives)
        let mut out_row: Vec<(u16, I32F32)> = Vec::new();
        for (j, &val) in acc_row.iter().enumerate() {
            if val > zero {
                out_row.push((j as u16, val));
            }
        }

        result.push(out_row);
    }

    result
}

/// Calculates the exponential moving average (EMA) for a sparse matrix using dynamic alpha values.
pub fn mat_ema_alpha_sparse(
    new: &[Vec<(u16, I32F32)>],
    old: &[Vec<(u16, I32F32)>],
    alpha: &[Vec<I32F32>],
) -> Vec<Vec<(u16, I32F32)>> {
    // If shapes don't match, just return `new`
    if new.len() != old.len() || new.len() != alpha.len() {
        log::error!(
            "math error: mat_ema_alpha_sparse shapes don't match: {:?} vs. {:?} vs. {:?}",
            old.len(),
            new.len(),
            alpha.len()
        );
        return new.to_owned();
    }

    let zero = I32F32::saturating_from_num(0.0);
    let one = I32F32::saturating_from_num(1.0);

    let mut result: Vec<Vec<(u16, I32F32)>> = Vec::with_capacity(new.len());
    let mut old_it = old.iter();
    let mut alf_it = alpha.iter();

    for new_row in new.iter() {
        let Some(old_row) = old_it.next() else { break };
        let Some(alpha_row) = alf_it.next() else {
            break;
        };

        // Densified accumulator sized to alpha_row length (columns outside are ignored).
        let mut decayed_values = vec![zero; alpha_row.len()];

        // Apply (1 - alpha_j) * old_ij into accumulator.
        for &(j, old_val) in old_row.iter() {
            if let (Some(&a), Some(cell)) = (
                alpha_row.get(j as usize),
                decayed_values.get_mut(j as usize),
            ) {
                *cell = one.saturating_sub(a).saturating_mul(old_val);
            }
        }

        // Add alpha_j * new_ij, clamp to [0, 1], and emit sparse entries > 0.
        let mut out_row: Vec<(u16, I32F32)> = Vec::new();
        for &(j, new_val) in new_row.iter() {
            if let (Some(&a), Some(&decayed)) =
                (alpha_row.get(j as usize), decayed_values.get(j as usize))
            {
                let inc = a.saturating_mul(new_val).max(zero);
                let val = decayed.saturating_add(inc).min(one);
                if val > zero {
                    out_row.push((j, val));
                }
            }
        }

        result.push(out_row);
    }

    result
}

/// Calculates the exponential moving average (EMA) for a dense matrix using dynamic alpha values.
pub fn mat_ema_alpha(
    new: &[Vec<I32F32>], // Weights
    old: &[Vec<I32F32>], // Bonds
    alpha: &[Vec<I32F32>],
) -> Vec<Vec<I32F32>> {
    // Empty or degenerate input
    if new.is_empty() || new.first().map(|r| r.is_empty()).unwrap_or(true) {
        return vec![vec![]];
    }

    // If outer dimensions don't match, return bonds unchanged
    if new.len() != old.len() || new.len() != alpha.len() {
        log::error!(
            "math error: mat_ema_alpha shapes don't match: {:?} vs. {:?} vs. {:?}",
            old.len(),
            new.len(),
            alpha.len()
        );
        return old.to_owned();
    }

    // Ensure each corresponding row has matching length; otherwise return `new` unchanged.
    let mut old_it = old.iter();
    let mut alp_it = alpha.iter();
    for nrow in new.iter() {
        let (Some(orow), Some(arow)) = (old_it.next(), alp_it.next()) else {
            return new.to_owned();
        };
        if nrow.len() != orow.len() || nrow.len() != arow.len() {
            return new.to_owned();
        }
    }

    let zero = I32F32::saturating_from_num(0.0);
    let one = I32F32::saturating_from_num(1.0);

    // Compute EMA: result = (1 - α) * old + α * new, clamped to [0, 1].
    let mut out: Vec<Vec<I32F32>> = Vec::with_capacity(new.len());
    let mut old_it = old.iter();
    let mut alp_it = alpha.iter();

    for nrow in new.iter() {
        let (Some(orow), Some(arow)) = (old_it.next(), alp_it.next()) else {
            break;
        };

        let mut r: Vec<I32F32> = Vec::with_capacity(nrow.len());
        let mut n_it = nrow.iter();
        let mut o_it = orow.iter();
        let mut a_it = arow.iter();

        while let (Some(&n), Some(&o), Some(&a)) = (n_it.next(), o_it.next(), a_it.next()) {
            let one_minus_a = one.saturating_sub(a);
            let decayed = one_minus_a.saturating_mul(o);
            let inc = a.saturating_mul(n).max(zero);
            r.push(decayed.saturating_add(inc).min(one));
        }

        out.push(r);
    }

    out
}

/// Safe ln function, returns 0 if value is 0.
pub fn safe_ln(value: I32F32) -> I32F32 {
    ln(value).unwrap_or(I32F32::saturating_from_num(0.0))
}

/// Safe exp function, returns 0 if value is 0.
pub fn safe_exp(value: I32F32) -> I32F32 {
    exp(value).unwrap_or(I32F32::saturating_from_num(0.0))
}
