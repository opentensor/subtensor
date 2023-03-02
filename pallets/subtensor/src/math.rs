use frame_support::sp_std::vec;
use frame_support::inherent::Vec;
use substrate_fixed::transcendental::exp;
use substrate_fixed::types::{I32F32, I64F64};

#[allow(dead_code)]
pub fn fixed_to_u16( x: I32F32 ) -> u16 { x.to_num::<u16>() }

#[allow(dead_code)]
pub fn fixed_to_u64( x: I32F32 ) -> u64 { x.to_num::<u64>() }

#[allow(dead_code)]
pub fn fixed64_to_u64( x: I64F64 ) -> u64 { x.to_num::<u64>() }

#[allow(dead_code)]
pub fn fixed64_to_fixed32( x: I64F64 ) -> I32F32 { I32F32::from_num( x ) }

#[allow(dead_code)]
pub fn u16_to_fixed( x: u16 ) -> I32F32 { I32F32::from_num( x ) }

#[allow(dead_code)]
pub fn u16_proportion_to_fixed( x: u16 ) -> I32F32 { I32F32::from_num( x ) / I32F32::from_num( u16::MAX ) }

#[allow(dead_code)]
pub fn fixed_proportion_to_u16( x: I32F32 ) -> u16 { fixed_to_u16( x * I32F32::from_num( u16::MAX )) }

#[allow(dead_code)]
pub fn vec_fixed64_to_fixed32( vec: Vec<I64F64> ) -> Vec<I32F32> { vec.into_iter().map(|e| fixed64_to_fixed32(e) ).collect() }

#[allow(dead_code)]
pub fn vec_u16_proportions_to_fixed( vec: Vec<u16> ) -> Vec<I32F32> { vec.into_iter().map(|e| u16_proportion_to_fixed(e) ).collect() }

#[allow(dead_code)]
pub fn vec_fixed_proportions_to_u16( vec: Vec<I32F32> ) -> Vec<u16> { vec.into_iter().map(|e| fixed_proportion_to_u16(e) ).collect() }

#[allow(dead_code)]
pub fn sum( x: &Vec<I32F32> ) -> I32F32 { x.iter().sum() }

/// Return true when vector sum is zero.
#[allow(dead_code)]
pub fn is_zero( vector: &Vec<I32F32> ) -> bool {
    let vector_sum: I32F32 = sum( &vector );
    vector_sum == I32F32::from_num( 0 )
}

/// Exp safe function with I32F32 output of I32F32 input.
#[allow(dead_code)]
pub fn exp_safe(input: I32F32) -> I32F32 {
    let min_input: I32F32 = I32F32::from_num(-20); // <= 1/exp(-20) = 485 165 195,4097903
    let max_input: I32F32 = I32F32::from_num(20); // <= exp(20) = 485 165 195,4097903
    let mut safe_input: I32F32 = input;
    if input < min_input {
        safe_input = min_input;
    }
    else if max_input < input {
        safe_input = max_input;
    }
    let output: I32F32;
    match exp(safe_input) {
        Ok(val) => {
            output = val;
        },
        Err(_err) => {
            if safe_input <= 0 {
                output = I32F32::from_num(0);
            }
            else {
                output = I32F32::max_value();
            }
        }
    }
    output
}

/// Sigmoid safe function with I32F32 output of I32F32 input with offset kappa and (recommended) scaling 0 < rho <= 40.
#[allow(dead_code)]
pub fn sigmoid_safe(input: I32F32, rho: I32F32, kappa: I32F32) -> I32F32 {
    let one: I32F32 = I32F32::from_num(1);
    let offset: I32F32 = input.saturating_sub(kappa); // (input - kappa)
    let neg_rho: I32F32 = rho.saturating_mul(-one); // -rho
    let exp_input: I32F32 = neg_rho.saturating_mul(offset); // -rho*(input-kappa)
    let exp_output: I32F32 = exp_safe(exp_input); // exp(-rho*(input-kappa))
    let denominator: I32F32 = exp_output.saturating_add(one); // 1 + exp(-rho*(input-kappa))
    let sigmoid_output: I32F32 = one.saturating_div(denominator); // 1 / (1 + exp(-rho*(input-kappa)))
    sigmoid_output
}

/// Returns a bool vector where an item is true if the vector item is in topk values.
#[allow(dead_code)]
pub fn is_topk( vector: &Vec<I32F32>, k: usize ) -> Vec<bool> {
    let n: usize = vector.len();
    let mut result: Vec<bool> = vec![ true; n ];
    if n < k { return result; }
    let mut idxs: Vec<usize> = (0..n).collect();
    idxs.sort_by_key( | &idx | &vector[ idx ] ); // ascending stable sort
    for &idx in &idxs[0..(n-k)] {
        result[ idx ] = false;
    }
    result
}

/// Returns a normalized (sum to 1 except 0) copy of the input vector.
#[allow(dead_code)]
pub fn normalize( x: &Vec<I32F32> ) -> Vec<I32F32> {
    let x_sum: I32F32 = sum( x );
    if x_sum != I32F32::from_num( 0.0 as f32 ) {
        return x.iter().map( |xi| xi / x_sum ).collect();
    } else {
        return x.clone();
    }
}

/// Normalizes (sum to 1 except 0) the input vector directly in-place.
#[allow(dead_code)]
pub fn inplace_normalize( x: &mut Vec<I32F32> ) {
    let x_sum: I32F32 = x.iter().sum();
    if x_sum == I32F32::from_num( 0.0 as f32 ){ return }
    for i in 0..x.len() {
        x[i] = x[i]/x_sum;
    }
}

/// Normalizes (sum to 1 except 0) the I64F64 input vector directly in-place.
#[allow(dead_code)]
pub fn inplace_normalize_64( x: &mut Vec<I64F64> ) {
    let x_sum: I64F64 = x.iter().sum();
    if x_sum == I64F64::from_num( 0 ){ return }
    for i in 0..x.len() {
        x[i] = x[i]/x_sum;
    }
}

/// Normalizes (sum to 1 except 0) each row (dim=0) of a matrix in-place.
#[allow(dead_code)]
pub fn inplace_row_normalize( x: &mut Vec<Vec<I32F32>> ) {
    for i in 0..x.len() {
        let row_sum: I32F32 = x[i].iter().sum();
        if row_sum > I32F32::from_num( 0.0 as f32 ) {
            x[i].iter_mut().for_each(|x_ij: &mut I32F32| *x_ij /= row_sum);
        }
    }
}

/// Normalizes (sum to 1 except 0) each row (dim=0) of a sparse matrix in-place.
#[allow(dead_code)]
pub fn inplace_row_normalize_sparse( sparse_matrix: &mut Vec<Vec<(u16, I32F32)>> ) {
    for sparse_row in sparse_matrix.iter_mut() {
        let row_sum: I32F32 = sparse_row.iter().map( | (_j, value) | *value ).sum();
        if row_sum > I32F32::from_num( 0.0 ) {
            sparse_row.iter_mut().for_each( | (_j, value) | *value /= row_sum );
        }
    }
}

/// Sum across each row (dim=0) of a matrix.
#[allow(dead_code)]
pub fn row_sum( x: &Vec<Vec<I32F32>> ) -> Vec<I32F32> {
    if x.len() == 0 { return vec![]; }
    if x[0].len() == 0 { return vec![]; }
    let rows = x.len();
    let mut result: Vec<I32F32> = vec![ I32F32::from_num( 0 ); rows ];
    for i in 0..x.len() {
        for j in 0..x[i].len() {
            result[i] += x[i][j];
        }
    }
    result
}

/// Sum across each row (dim=0) of a sparse matrix.
#[allow(dead_code)]
pub fn row_sum_sparse( sparse_matrix: &Vec<Vec<(u16, I32F32)>> ) -> Vec<I32F32> {
    let rows = sparse_matrix.len();
    let mut result: Vec<I32F32> = vec![ I32F32::from_num( 0 ); rows ];
    for (i, sparse_row) in sparse_matrix.iter().enumerate() {
        for (_j, value) in sparse_row.iter() {
            result[i] += value;
        }
    }
    result
}

/// Sum across each column (dim=1) of a matrix.
#[allow(dead_code)]
pub fn col_sum( x: &Vec<Vec<I32F32>> ) -> Vec<I32F32> {
    if x.len() == 0 { return vec![]; }
    if x[0].len() == 0 { return vec![]; }
    let cols = x[0].len();
    let mut result: Vec<I32F32> = vec![ I32F32::from_num( 0 ); cols ];
    for i in 0..x.len() {
        assert_eq!( x[i].len(), cols );
        for j in 0..cols {
            result[j] += x[i][j];
        }
    }
    result
}

/// Sum across each column (dim=1) of a sparse matrix.
#[allow(dead_code)]
pub fn col_sum_sparse( sparse_matrix: &Vec<Vec<(u16, I32F32)>>, columns: u16 ) -> Vec<I32F32> {
    let mut result: Vec<I32F32> = vec![ I32F32::from_num( 0 ); columns as usize ];
    for sparse_row in sparse_matrix.iter() {
        for (j, value) in sparse_row.iter() {
            result[*j as usize] += value;
        }
    }
    result
}

/// Normalizes (sum to 1 except 0) each column (dim=1) of a sparse matrix in-place.
#[allow(dead_code)]
pub fn inplace_col_normalize_sparse( sparse_matrix: &mut Vec<Vec<(u16, I32F32)>>, columns: u16 ) {
    let mut col_sum: Vec<I32F32> = vec![ I32F32::from_num( 0.0 ); columns as usize]; // assume square matrix, rows=cols
    for sparse_row in sparse_matrix.iter() {
        for (j, value) in sparse_row.iter() {
            col_sum[*j as usize] += value;
        }
    }
    for sparse_row in sparse_matrix.iter_mut() {
        for (j, value) in sparse_row.iter_mut() {
            if col_sum[*j as usize] == I32F32::from_num( 0.0 as f32 ) { continue }
            *value /= col_sum[*j as usize];
        }
    }
}

/// Normalizes (sum to 1 except 0) each column (dim=1) of a matrix in-place.
#[allow(dead_code)]
pub fn inplace_col_normalize( x: &mut Vec<Vec<I32F32>> ) {
    if x.len() == 0 { return }
    if x[0].len() == 0 { return }
    let cols = x[0].len();
    let mut col_sum: Vec<I32F32> = vec![ I32F32::from_num( 0.0 ); cols ];
    for i in 0..x.len() {
        assert_eq!( x[i].len(), cols );
        for j in 0..cols {
            col_sum[j] += x[i][j];
        }
    }
    for j in 0..cols {
        if col_sum[j] == I32F32::from_num( 0.0 as f32 ) { continue }
        for i in 0..x.len() {
            x[i][j] /= col_sum[j];
        }
    }
}

/// Apply mask to vector, mask=true will mask out, i.e. set to 0.
#[allow(dead_code)]
pub fn inplace_mask_vector( mask: &Vec<bool>, vector: &mut Vec<I32F32> ) {
    if mask.len() == 0 { return }
    assert_eq!( mask.len(), vector.len() );
    let zero: I32F32 = I32F32::from_num( 0.0 );
    for i in 0..mask.len() {
        if mask[i] {
            vector[i] = zero;
        }
    }
}

/// Apply mask to matrix, mask=true will mask out, i.e. set to 0.
#[allow(dead_code)]
pub fn inplace_mask_matrix( mask: &Vec<Vec<bool>>, matrix: &mut Vec<Vec<I32F32>> ) {
    if mask.len() == 0 { return }
    if mask[0].len() == 0 { return }
    assert_eq!( mask.len(), matrix.len() );
    let zero: I32F32 = I32F32::from_num( 0.0 );
    for i in 0..mask.len() {
        for j in 0..mask[i].len() {
            if mask[i][j] {
                matrix[i][j] = zero;
            }
        }
    }
}

/// Apply row mask to matrix, mask=true will mask out, i.e. set to 0.
#[allow(dead_code)]
pub fn inplace_mask_rows( mask: &Vec<bool>, matrix: &mut Vec<Vec<I32F32>> ) {
    let rows = matrix.len();
    if rows == 0 { return }
    let cols = matrix[0].len();
    assert_eq!( mask.len(), rows );
    let zero: I32F32 = I32F32::from_num( 0 );
    for i in 0..rows {
        if mask[i] {
            matrix[i] = vec![zero; cols];
        }
    }
}

/// Mask out the diagonal of the input matrix in-place.
#[allow(dead_code)]
pub fn inplace_mask_diag( matrix: &mut Vec<Vec<I32F32>> ) {
    if matrix.len() == 0 { return }
    if matrix[0].len() == 0 { return }
    assert_eq!( matrix.len(), matrix[0].len() );
    let zero: I32F32 = I32F32::from_num( 0.0 );
    for i in 0..matrix.len() {
        matrix[i][i] = zero;
    }
}

/// Return a new sparse matrix that replaces masked rows with an empty vector placeholder.
#[allow(dead_code)]
pub fn mask_rows_sparse( mask: &Vec<bool>, sparse_matrix: &Vec<Vec<(u16, I32F32)>> ) -> Vec<Vec<(u16, I32F32)>> {
    let n: usize = sparse_matrix.len();
    assert_eq!( n, mask.len() );
    let mut result: Vec<Vec<(u16, I32F32)>> = vec![ vec![]; n];
    for (i, sparse_row) in sparse_matrix.iter().enumerate() {
        if !mask[i] {
            result[i] = sparse_row.clone();
        }
    }
    result
}

/// Return a new sparse matrix with a masked out diagonal of input sparse matrix.
#[allow(dead_code)]
pub fn mask_diag_sparse( sparse_matrix: &Vec<Vec<(u16, I32F32)>> ) -> Vec<Vec<(u16, I32F32)>> {
    let n: usize = sparse_matrix.len();
    let mut result: Vec<Vec<(u16, I32F32)>> = vec![ vec![]; n];
    for (i, sparse_row) in sparse_matrix.iter().enumerate() {
        for (j, value) in sparse_row.iter() {
            if i != (*j as usize) {
                result[i].push( (*j, *value) );
            }
        }
    }
    result
}

/// Remove cells from sparse matrix where the mask function of two vectors is true.
#[allow(dead_code)]
pub fn vec_mask_sparse_matrix( sparse_matrix: &Vec<Vec<(u16, I32F32)>>, first_vector: &Vec<u64>, second_vector: &Vec<u64>, mask_fn: &dyn Fn(u64, u64) -> bool) -> Vec<Vec<(u16, I32F32)>> {
    let n: usize = sparse_matrix.len();
    let mut result: Vec<Vec<(u16, I32F32)>> = vec![ vec![]; n];
    for (i, sparse_row) in sparse_matrix.iter().enumerate() {
        for (j, value) in sparse_row.iter() {
            if !mask_fn(first_vector[i], second_vector[*j as usize]) {
                result[i].push( (*j, *value) );
            }
        }
    }
    result
}

/// Row-wise matrix-vector hadamard product.
#[allow(dead_code)]
pub fn row_hadamard( matrix: &Vec<Vec<I32F32>>, vector: &Vec<I32F32> ) -> Vec<Vec<I32F32>> {
    if matrix.len() == 0 { return vec![ vec![] ] }
    if matrix[0].len() == 0 { return vec![ vec![] ] }
    let mut result: Vec<Vec<I32F32>> = vec![ vec![ I32F32::from_num( 0.0 ); matrix[0].len() ]; matrix.len() ];
    for i in 0..matrix.len() {
        for j in 0..matrix[i].len() {
            result[i][j] = vector[i] * matrix[i][j];
        }
    }
    result
}

/// Row-wise sparse matrix-vector hadamard product.
#[allow(dead_code)]
pub fn row_hadamard_sparse( sparse_matrix: &Vec<Vec<(u16, I32F32)>>, vector: &Vec<I32F32> ) -> Vec<Vec<(u16, I32F32)>> {
    let mut result: Vec<Vec<(u16, I32F32)>> = sparse_matrix.clone();
    for (i, sparse_row) in result.iter_mut().enumerate() {
        for (_j, value) in sparse_row.iter_mut() {
            *value *= vector[i];
        }
    }
    result
}

/// Row-wise matrix-vector product, column-wise sum: result_j = SUM(i) vector_i * matrix_ij.
#[allow(dead_code)]
pub fn matmul( matrix: &Vec<Vec<I32F32>>, vector: &Vec<I32F32> ) -> Vec<I32F32> {
    if matrix.len() == 0 { return vec![] }
    if matrix[0].len() == 0 { return vec![] }
    assert!( matrix.len() == vector.len() );
    let mut result: Vec<I32F32> = vec![ I32F32::from_num( 0.0 ); matrix[0].len() ];
    for i in 0..matrix.len() {
        for j in 0..matrix[i].len() {
            // Compute ranks: r_j = SUM(i) w_ij * s_i
            // Compute trust scores: t_j = SUM(i) w_ij * s_i
            // result_j = SUM(i) vector_i * matrix_ij
            result[j] += vector[i] * matrix[i][j];
        }
    }
    result
}

/// Column-wise matrix-vector product, row-wise sum: result_i = SUM(j) vector_j * matrix_ij.
#[allow(dead_code)]
pub fn matmul_transpose( matrix: &Vec<Vec<I32F32>>, vector: &Vec<I32F32> ) -> Vec<I32F32> {
    if matrix.len() == 0 { return vec![] }
    if matrix[0].len() == 0 { return vec![] }
    assert!( matrix[0].len() == vector.len() );
    let mut result: Vec<I32F32> = vec![ I32F32::from_num( 0.0 ); matrix.len() ];
    for i in 0..matrix.len() {
        for j in 0..matrix[i].len() {
            // Compute dividends: d_j = SUM(i) b_ji * inc_i
            // result_j = SUM(i) vector_i * matrix_ji
            // result_i = SUM(j) vector_j * matrix_ij
            result[i] += vector[j] * matrix[i][j];
        }
    }
    result
}

/// Row-wise sparse_matrix-vector product, column-wise sum: result_j = SUM(i) vector_i * matrix_ij.
#[allow(dead_code)]
pub fn matmul_sparse( sparse_matrix: &Vec<Vec<(u16, I32F32)>>, vector: &Vec<I32F32>, columns: u16 ) -> Vec<I32F32> {
    let mut result: Vec<I32F32> = vec![ I32F32::from_num( 0.0 ); columns as usize ];
    for (i, sparse_row) in sparse_matrix.iter().enumerate() {
        for (j, value) in sparse_row.iter() {
            // Compute ranks: r_j = SUM(i) w_ij * s_i
            // Compute trust scores: t_j = SUM(i) w_ij * s_i
            // result_j = SUM(i) vector_i * matrix_ij
            result[*j as usize] += vector[i] * value;
        }
    }
    result
}

/// Column-wise sparse_matrix-vector product, row-wise sum: result_i = SUM(j) vector_j * matrix_ij.
#[allow(dead_code)]
pub fn matmul_transpose_sparse( sparse_matrix: &Vec<Vec<(u16, I32F32)>>, vector: &Vec<I32F32> ) -> Vec<I32F32> {
    let mut result: Vec<I32F32> = vec![ I32F32::from_num( 0.0 ); sparse_matrix.len() ];
    for (i, sparse_row) in sparse_matrix.iter().enumerate() {
        for (j, value) in sparse_row.iter() {
            // Compute dividends: d_j = SUM(i) b_ji * inc_i
            // result_j = SUM(i) vector_i * matrix_ji
            // result_i = SUM(j) vector_j * matrix_ij
            result[i] += vector[*j as usize] * value;
        }
    }
    result
}

/// Set inplace matrix values above column threshold to threshold value.
#[allow(dead_code)]
pub fn inplace_col_clip( x: &mut Vec<Vec<I32F32>>, col_threshold: &Vec<I32F32> ) {
    for i in 0..x.len() {
        for j in 0..x[i].len() {
            if x[i][j] > col_threshold[j] {
                x[i][j] = col_threshold[j];
            }
        }
    }
}

/// Return sparse matrix with values above column threshold set to threshold value.
#[allow(dead_code)]
pub fn col_clip_sparse( sparse_matrix: &Vec<Vec<(u16, I32F32)>>, col_threshold: &Vec<I32F32> ) -> Vec<Vec<(u16, I32F32)>> {
    let mut result: Vec<Vec<(u16, I32F32)>> = vec![ vec![]; sparse_matrix.len() ];
    for (i, sparse_row) in sparse_matrix.iter().enumerate() {
        for (j, value) in sparse_row.iter() {
            if col_threshold[*j as usize] < *value {
                if 0 < col_threshold[*j as usize] {
                    result[i].push( (*j, col_threshold[*j as usize]) );
                }
            }
            else {
                result[i].push( (*j, *value) );
            }
        }
    }
    result
}

/// Set matrix values below threshold to lower, and equal-above to upper.
#[allow(dead_code)]
pub fn clip( x: &Vec<Vec<I32F32>>, threshold: I32F32, upper: I32F32, lower: I32F32) -> Vec<Vec<I32F32>> {
    // Check Nill length. 
    if x.len() == 0 {
        return vec![ vec![ ] ];
    }
    let mut result: Vec<Vec<I32F32>> = vec![ vec![ lower; x[0].len() ]; x.len() ]; 
    for i in 0..x.len() {
        for j in 0..x[i].len() {
            if x [ i ][ j ] >= threshold {
                result[ i ][ j ] = upper;
            }
        }
    }
    result
}

/// Set inplace matrix values below threshold to lower, and equal-above to upper.
#[allow(dead_code)]
pub fn inplace_clip( x: &mut Vec<Vec<I32F32>>, threshold: I32F32, upper: I32F32, lower: I32F32 ) {
    for i in 0..x.len() {
        for j in 0..x[i].len() {
            if x [ i ][ j ] >= threshold {
                x[ i ][ j ] = upper;
            } else {
                x[ i ][ j ] = lower;
            }
        }
    }
}

/// Set sparse matrix values below threshold to lower, and equal-above to upper.
/// Does not add missing elements (0 value assumed) when lower!=0.
#[allow(dead_code)]
pub fn clip_sparse( sparse_matrix: &Vec<Vec<(u16, I32F32)>>, threshold: I32F32, upper: I32F32, lower: I32F32) -> Vec<Vec<(u16, I32F32)>> {
    let mut result: Vec<Vec<(u16, I32F32)>> = vec![ vec![]; sparse_matrix.len() ];
    for (i, sparse_row) in sparse_matrix.iter().enumerate() {
        for (j, value) in sparse_row.iter() {
            if *value < threshold {
                result[i].push( (*j, lower) );
            }
            else {
                result[i].push( (*j, upper) );
            }
        }
    }
    result
}

/// Element-wise product of two matrices.
#[allow(dead_code)]
pub fn hadamard( mat1: &Vec<Vec<I32F32>>, mat2: &Vec<Vec<I32F32>> ) -> Vec<Vec<I32F32>> {
    assert!( mat1.len() == mat2.len() );
    if mat1.len() == 0 { return vec![vec![];1] }
    if mat1[0].len() == 0 { return vec![vec![];1] }
    let mut result: Vec<Vec<I32F32>> = vec![ vec![ I32F32::from_num( 0 ); mat1[0].len() ]; mat1.len() ];
    for i in 0..mat1.len() {
        assert!(mat1[i].len() == mat2[i].len());
        for j in 0..mat1[i].len() {
            result[i][j] = mat1[i][j] * mat2[i][j];
        }
    }
    result
}

/// Element-wise product of two sparse matrices.
#[allow(dead_code)]
pub fn hadamard_sparse( mat1: &Vec<Vec<(u16, I32F32)>>, mat2: &Vec<Vec<(u16, I32F32)>>, columns: u16 ) -> Vec<Vec<(u16, I32F32)>> {
    assert!(mat1.len() == mat2.len());
    let rows = mat1.len();
    let zero: I32F32 = I32F32::from_num( 0 );
    let mut result: Vec<Vec<(u16, I32F32)>> = vec![ vec![]; rows ];
    for i in 0..rows {
        let mut row1: Vec<I32F32> = vec![ zero; columns as usize ];
        for (j, value) in mat1[i].iter() {
            row1[*j as usize] += value;
        }
        let mut row2: Vec<I32F32> = vec![ zero; columns as usize ];
        for (j, value) in mat2[i].iter() {
            row2[*j as usize] += value;
        }
        for j in 0..columns as usize {
            let prod: I32F32 = row1[j] * row2[j];
            if zero < prod {
                result[i].push( (j as u16, prod) )
            }
        }
    }
    result
}

/// Return matrix exponential moving average: `alpha * a_ij + one_minus_alpha * b_ij`.
/// `alpha` is the EMA coefficient, how much to add of the new observation, typically small, 
/// higher alpha discounts older observations faster.
#[allow(dead_code)]
pub fn mat_ema( new: &Vec<Vec<I32F32>>, old: &Vec<Vec<I32F32>>, alpha: I32F32 ) -> Vec<Vec<I32F32>> {
    if new.len() == 0 { return vec![vec![];1] }
    if new[0].len() == 0 { return vec![vec![];1] }
    let one_minus_alpha:I32F32 = I32F32::from_num( 1.0 ) - alpha;
    let mut result: Vec<Vec<I32F32>> = vec![ vec![ I32F32::from_num( 0.0 ); new[0].len() ]; new.len() ]; 
    assert!(new.len() == old.len());
    for i in 0..new.len() {
        assert!(new[i].len() == old[i].len());
        for j in 0..new[i].len() {
            result[i][j] = alpha * new[i][j] + one_minus_alpha * old[i][j] 
        }
    }
    result
}

/// Return sparse matrix exponential moving average: `alpha * a_ij + one_minus_alpha * b_ij`.
/// `alpha` is the EMA coefficient, how much to add of the new observation, typically small, 
/// higher alpha discounts older observations faster.
#[allow(dead_code)]
pub fn mat_ema_sparse( new: &Vec<Vec<(u16, I32F32)>>, old: &Vec<Vec<(u16, I32F32)>>, alpha: I32F32 ) -> Vec<Vec<(u16, I32F32)>> {
    assert!(new.len() == old.len());
    let n = new.len(); // assume square matrix, rows=cols
    let zero: I32F32 = I32F32::from_num( 0.0 );
    let one_minus_alpha:I32F32 = I32F32::from_num( 1.0 ) - alpha;
    let mut result: Vec<Vec<(u16, I32F32)>> = vec![ vec![]; n ];
    for i in 0..new.len() {
        let mut row: Vec<I32F32> = vec![ zero; n];
        for (j, value) in new[i].iter() {
            row[*j as usize] += alpha * value;
        }
        for (j, value) in old[i].iter() {
            row[*j as usize] += one_minus_alpha * value;
        }
        for (j, value) in row.iter().enumerate() {
            if *value > zero {
                result[i].push( (j as u16, *value) )
            }
        }
    }
    result
}

/// Return sparse matrix only with elements >= threshold of an input sparse matrix.
#[allow(dead_code)]
pub fn sparse_threshold( w: &Vec<Vec<(u16, I32F32)>>, threshold: I32F32 ) -> Vec<Vec<(u16, I32F32)>> {
    let mut sparse_threshold_result: Vec<Vec<(u16, I32F32)>> = vec![ vec![]; w.len() ]; 
    for ( uid_i, weights_i ) in w.iter().enumerate() {
        for (uid_j, weight_ij) in weights_i.iter() { 
            if *weight_ij >= threshold {
                sparse_threshold_result [ uid_i as usize ].push( ( *uid_j, *weight_ij ));
            }
        }
    }
    sparse_threshold_result
}

#[cfg(test)]
mod tests {
    use crate::math::*;
    use substrate_fixed::transcendental::exp;
    use substrate_fixed::types::{I32F32, I64F64, I96F32, I110F18};

    fn assert_float_compare(a: I32F32, b: I32F32, epsilon: I32F32 ) {
        assert!( I32F32::abs( a - b ) <= epsilon, "a({:?}) != b({:?})", a, b);
    }

    fn assert_float_compare_64(a: I64F64, b: I64F64, epsilon: I64F64 ) {
        assert!( I64F64::abs( a - b ) <= epsilon, "a({:?}) != b({:?})", a, b);
    }
    
    fn assert_vec_compare(va: &Vec<I32F32>, vb: &Vec<I32F32>, epsilon: I32F32) {
        assert!(va.len() == vb.len());
        for i in 0..va.len(){
            assert_float_compare(va[i], vb[i], epsilon);
        }  
    }
    
    fn assert_vec_compare_64(va: &Vec<I64F64>, vb: &Vec<I64F64>, epsilon: I64F64) {
        assert!(va.len() == vb.len());
        for i in 0..va.len(){
            assert_float_compare_64(va[i], vb[i], epsilon);
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
    
    fn assert_sparse_mat_compare(ma: &Vec<Vec<(u16, I32F32)>>, mb: &Vec<Vec<(u16, I32F32)>>, epsilon: I32F32) {
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
        vector.iter().map( | x | I32F32::from_num( *x ) ).collect()
    }

    #[test]
    fn test_math_fixed_overflow() {
        let max_32: I32F32 = I32F32::max_value();
        let max_u64: u64 = u64::MAX;
        let _prod_96: I96F32 = I96F32::from_num(max_32) * I96F32::from_num(max_u64);
        // let one: I96F32 = I96F32::from_num(1);
        // let prod_96: I96F32 = (I96F32::from_num(max_32) + one) * I96F32::from_num(max_u64); // overflows
        let _prod_110: I110F18 = I110F18::from_num(max_32) * I110F18::from_num(max_u64);
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
        assert_eq!(min32_frac,    I64F64::from_num(0.00000000023283066664));
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
        let output: Vec<u16> = vector.iter().map( |e: &I32F32| e.to_num::<u16>() ).collect();
        assert_eq!(output, target);
        let output: Vec<u16> = vector.iter().map( |e: &I32F32| (*e).to_num::<u16>() ).collect();
        assert_eq!(output, target);
        let val: I32F32 = I32F32::max_value();
        let res: u64 = val.to_num::<u64>();
        let vector: Vec<I32F32> = vec![val; 1000];
        let target: Vec<u64> = vec![res; 1000];
        let output: Vec<u64> = vector.iter().map( |e: &I32F32| e.to_num::<u64>() ).collect();
        assert_eq!(output, target);
        let output: Vec<u64> = vector.iter().map( |e: &I32F32| (*e).to_num::<u64>() ).collect();
        assert_eq!(output, target);
        let val: I32F32 = I32F32::from_num(0);
        let res: u64 = val.to_num::<u64>();
        let vector: Vec<I32F32> = vec![val; 1000];
        let target: Vec<u64> = vec![res; 1000];
        let output: Vec<u64> = vector.iter().map( |e: &I32F32| e.to_num::<u64>() ).collect();
        assert_eq!(output, target);
        let output: Vec<u64> = vector.iter().map( |e: &I32F32| (*e).to_num::<u64>() ).collect();
        assert_eq!(output, target);
        let val: I96F32 = I96F32::from_num(u64::MAX);
        let res: u64 = val.to_num::<u64>();
        assert_eq!(res, u64::MAX);
        let vector: Vec<I96F32> = vec![val; 1000];
        let target: Vec<u64> = vec![u64::MAX; 1000];
        let output: Vec<u64> = vector.iter().map( |e: &I96F32| e.to_num::<u64>() ).collect();
        assert_eq!(output, target);
        let output: Vec<u64> = vector.iter().map( |e: &I96F32| (*e).to_num::<u64>() ).collect();
        assert_eq!(output, target);
    }
    
    #[test]
    fn test_math_vec_to_fixed() {
        let vector: Vec<f32> = vec![ 0., 1., 2., 3.];
        let target: Vec<I32F32> = vec![I32F32::from_num(0.), I32F32::from_num(1.), I32F32::from_num(2.), I32F32::from_num(3.)];
        let result = vec_to_fixed(&vector);
        assert_vec_compare(&result, &target, I32F32::from_num(0));
    }

    /// Reshape vector to matrix with specified number of rows, cast to I32F32.
    fn vec_to_mat_fixed(vector: &Vec<f32>, rows: usize, transpose: bool) -> Vec<Vec<I32F32>> {
        assert!( vector.len() % rows == 0, "Vector of len {:?} cannot reshape to {rows} rows.", vector.len());
        let cols: usize = vector.len() / rows;
        let mut mat: Vec<Vec<I32F32>> = vec![];
        if transpose {
            for col in 0..cols as usize {
                let mut vals: Vec<I32F32> = vec![];
                for row in 0..rows as usize {
                    vals.push( I32F32::from_num( vector[row * cols + col]) );
                }
                mat.push(vals);
            }
        }
        else {
            for row in 0..rows as usize {
                mat.push( vector[row * cols .. (row + 1) * cols].iter().map( | v | I32F32::from_num( *v ) ).collect() );
            }
        }
        mat
    }

    #[test]
    fn test_math_vec_to_mat_fixed() {
        let vector: Vec<f32> = vec![ 0., 1., 2.,
                                    0., 10., 100.];
        let target: Vec<Vec<I32F32>> = vec![vec![I32F32::from_num(0.), I32F32::from_num(1.), I32F32::from_num(2.)], 
                                            vec![I32F32::from_num(0.), I32F32::from_num(10.), I32F32::from_num(100.)] ];
        let mat = vec_to_mat_fixed(&vector, 2, false);
        assert_mat_compare(&mat, &target, I32F32::from_num(0));
    }

    /// Reshape vector to sparse matrix with specified number of input rows, cast f32 to I32F32.
    fn vec_to_sparse_mat_fixed(vector: &Vec<f32>, rows: usize, transpose: bool) -> Vec<Vec<(u16, I32F32)>> {
        assert!( vector.len() % rows == 0, "Vector of len {:?} cannot reshape to {rows} rows.", vector.len());
        let cols: usize = vector.len() / rows;
        let mut mat: Vec<Vec<(u16, I32F32)>> = vec![];
        if transpose {
            for col in 0..cols as usize {
                let mut row_vec: Vec<(u16, I32F32)> = vec![];
                for row in 0..rows as usize {
                    if vector[row * cols + col] > 0. {
                        row_vec.push( (row as u16, I32F32::from_num(vector[row * cols + col])) );
                    }
                }
                mat.push(row_vec);
            }
        }
        else {
            for row in 0..rows as usize {
                let mut row_vec: Vec<(u16, I32F32)> = vec![];
                for col in 0..cols as usize {
                    if vector[row * cols + col] > 0. {
                        row_vec.push( (col as u16, I32F32::from_num(vector[row * cols + col])) );
                    }
                }
                mat.push(row_vec);
            }
        }
        mat
    }

    #[test]
    fn test_math_vec_to_sparse_mat_fixed() {
        let vector: Vec<f32> = vec![ 0., 1., 2.,
                                    0., 10., 100.];
        let target: Vec<Vec<(u16, I32F32)>> = vec![ vec![(1 as u16, I32F32::from_num(1.)), (2 as u16, I32F32::from_num(2.))], 
                                                    vec![(1 as u16, I32F32::from_num(10.)), (2 as u16, I32F32::from_num(100.))] ];
        let mat = vec_to_sparse_mat_fixed(&vector, 2, false);
        assert_sparse_mat_compare(&mat, &target, I32F32::from_num(0));
        let vector: Vec<f32> = vec![ 0.,
                                     0.];
        let target: Vec<Vec<(u16, I32F32)>> = vec![ vec![], 
                                                    vec![] ];
        let mat = vec_to_sparse_mat_fixed(&vector, 2, false);
        assert_sparse_mat_compare(&mat, &target, I32F32::from_num(0));
        let vector: Vec<f32> = vec![ 0., 1., 2.,
                                    0., 10., 100.];
        let target: Vec<Vec<(u16, I32F32)>> = vec![ vec![],
                                                    vec![(0 as u16, I32F32::from_num(1.)), (1 as u16, I32F32::from_num(10.))], 
                                                    vec![(0 as u16, I32F32::from_num(2.)), (1 as u16, I32F32::from_num(100.))] ];
        let mat = vec_to_sparse_mat_fixed(&vector, 2, true);
        assert_sparse_mat_compare(&mat, &target, I32F32::from_num(0));
        let vector: Vec<f32> = vec![ 0.,
                                     0.];
        let target: Vec<Vec<(u16, I32F32)>> = vec![ vec![] ];
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
        let trust: Vec<I32F32> = vec![I32F32::min_value(), I32F32::from_num(0), I32F32::from_num(0.4), I32F32::from_num(0.5), I32F32::from_num(0.6), I32F32::from_num(1), I32F32::max_value()];
        let consensus: Vec<I32F32> = trust.iter().map(|t: &I32F32| sigmoid_safe(*t, I32F32::max_value(), I32F32::max_value())).collect();
        let target: Vec<I32F32> = vec_to_fixed(&vec![0.0000000019, 0.0000000019, 0.0000000019, 0.0000000019, 0.0000000019, 0.0000000019, 0.5]);
        assert_eq!(&consensus, &target);
        let consensus: Vec<I32F32> = trust.iter().map(|t: &I32F32| sigmoid_safe(*t, I32F32::min_value(), I32F32::min_value())).collect();
        let target: Vec<I32F32> = vec_to_fixed(&vec![0.5, 0.0000000019, 0.0000000019, 0.0000000019, 0.0000000019, 0.0000000019, 0.0000000019]);
        assert_eq!(&consensus, &target);
        let consensus: Vec<I32F32> = trust.iter().map(|t: &I32F32| sigmoid_safe(*t, I32F32::from_num(30), I32F32::from_num(0.5))).collect();
        let target: Vec<f64> = vec![0.0000000019, 0.0000003057, 0.0474258729, 0.5, 0.952574127, 0.9999996943, 0.9999999981];
        let target: Vec<I32F32> = target.iter().map(|c: &f64| I32F32::from_num(*c)).collect();
        assert_eq!(&consensus, &target);
        let trust: Vec<I32F32> = vec_to_fixed(&vec![0., 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.]);
        let consensus: Vec<I32F32> = trust.iter().map(|t: &I32F32| sigmoid_safe(*t, I32F32::from_num(40), I32F32::from_num(0.5))).collect();
        let target: Vec<f64> = vec![0.0000000019, 0.0000001125, 0.0000061442, 0.0003353502, 0.017986214, 0.5, 0.9820138067, 0.9996646498, 0.9999938558, 0.9999998875, 0.9999999981];
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
        let target: Vec<bool> = vec![false, false, false, false, false, false, false, false, false, false];
        assert_eq!(&result, &target);
        let result = is_topk(&vector, 5);
        let target: Vec<bool> = vec![false, false, false, false, false, true, true, true, true, true];
        assert_eq!(&result, &target);
        let result = is_topk(&vector, 10);
        let target: Vec<bool> = vec![true, true, true, true, true, true, true, true, true, true];
        assert_eq!(&result, &target);
        let result = is_topk(&vector, 100);
        assert_eq!(&result, &target);
        let vector: Vec<I32F32> = vec_to_fixed(&vec![9., 8., 7., 6., 5., 4., 3., 2., 1., 0.]);
        let result = is_topk(&vector, 5);
        let target: Vec<bool> = vec![true, true, true, true, true, false, false, false, false, false];
        assert_eq!(&result, &target);
        let vector: Vec<I32F32> = vec_to_fixed(&vec![9., 0., 8., 1., 7., 2., 6., 3., 5., 4.]);
        let result = is_topk(&vector, 5);
        let target: Vec<bool> = vec![true, false, true, false, true, false, true, false, true, false];
        assert_eq!(&result, &target);
        let vector: Vec<I32F32> = vec_to_fixed(&vec![0.9, 0., 0.8, 0.1, 0.7, 0.2, 0.6, 0.3, 0.5, 0.4]);
        let result = is_topk(&vector, 5);
        let target: Vec<bool> = vec![true, false, true, false, true, false, true, false, true, false];
        assert_eq!(&result, &target);
        let vector: Vec<I32F32> = vec_to_fixed(&vec![0., 1., 2., 3., 4., 5., 5., 5., 5., 6.]);
        let result = is_topk(&vector, 5);
        let target: Vec<bool> = vec![false, false, false, false, false, true, true, true, true, true];
        assert_eq!(&result, &target);
    }

    #[test]
    fn test_math_sum() {
        assert!( sum(&vec![]) == I32F32::from_num(0));
        assert!( sum(&vec![ I32F32::from_num(1.0),  I32F32::from_num(10.0),  I32F32::from_num(30.0)]) == I32F32::from_num(41));
        assert!( sum(&vec![ I32F32::from_num(-1.0),  I32F32::from_num(10.0),  I32F32::from_num(30.0)]) == I32F32::from_num(39));
    }

    #[test]
    fn test_math_normalize() {
        let epsilon: I32F32 = I32F32::from_num(0.0001);
        let x: Vec<I32F32> = vec![]; 
        let y: Vec<I32F32> = normalize(&x);
        assert_vec_compare( &x, &y, epsilon);
        let x: Vec<I32F32> = vec![ I32F32::from_num(1.0),  I32F32::from_num(10.0),  I32F32::from_num(30.0)]; 
        let y: Vec<I32F32> = normalize(&x);
        assert_vec_compare( &y, &vec![ I32F32::from_num(0.0243902437),  I32F32::from_num(0.243902439),  I32F32::from_num(0.7317073171)], epsilon );
        assert_float_compare( sum( &y ), I32F32::from_num(1.0), epsilon);
        let x: Vec<I32F32> = vec![ I32F32::from_num(-1.0),  I32F32::from_num(10.0),  I32F32::from_num(30.0)]; 
        let y: Vec<I32F32> = normalize(&x);
        assert_vec_compare( &y, &vec![ I32F32::from_num(-0.0256410255),  I32F32::from_num(0.2564102563),  I32F32::from_num(0.769230769)], epsilon );
        assert_float_compare( sum( &y ), I32F32::from_num(1.0), epsilon );
    }

    #[test]
    fn test_math_inplace_normalize() {
        let epsilon: I32F32 = I32F32::from_num(0.0001);
        let mut x1: Vec<I32F32> = vec![ I32F32::from_num(1.0),  I32F32::from_num(10.0),  I32F32::from_num(30.0)]; 
        inplace_normalize(&mut x1);
        assert_vec_compare( &x1, &vec![ I32F32::from_num(0.0243902437),  I32F32::from_num(0.243902439),  I32F32::from_num(0.7317073171)], epsilon );
        let mut x2: Vec<I32F32> = vec![ I32F32::from_num(-1.0),  I32F32::from_num(10.0),  I32F32::from_num(30.0)]; 
        inplace_normalize(&mut x2);
        assert_vec_compare( &x2, &vec![ I32F32::from_num(-0.0256410255),  I32F32::from_num(0.2564102563),  I32F32::from_num(0.769230769)], epsilon );
    }

    #[test]
    fn test_math_inplace_normalize_64() {
        let epsilon: I64F64 = I64F64::from_num(0.0001);
        let mut x1: Vec<I64F64> = vec![ I64F64::from_num(1.0),  I64F64::from_num(10.0),  I64F64::from_num(30.0)]; 
        inplace_normalize_64(&mut x1);
        assert_vec_compare_64( &x1, &vec![ I64F64::from_num(0.0243902437),  I64F64::from_num(0.243902439),  I64F64::from_num(0.7317073171)], epsilon );
        let mut x2: Vec<I64F64> = vec![ I64F64::from_num(-1.0),  I64F64::from_num(10.0),  I64F64::from_num(30.0)]; 
        inplace_normalize_64(&mut x2);
        assert_vec_compare_64( &x2, &vec![ I64F64::from_num(-0.0256410255),  I64F64::from_num(0.2564102563),  I64F64::from_num(0.769230769)], epsilon );
    }

    #[test]
    fn test_math_inplace_row_normalize() {
        let epsilon: I32F32 = I32F32::from_num(0.0001);
        let vector:Vec<f32> = vec![ 0., 1., 2., 3., 4., 
                                    0., 10., 100., 1000., 10000., 
                                    0., 0., 0., 0., 0., 
                                    1., 1., 1., 1., 1.];
        let mut mat = vec_to_mat_fixed(&vector, 4, false);
        inplace_row_normalize(&mut mat);
        let target:Vec<f32> = vec![ 0., 0.1, 0.2, 0.3, 0.4, 
                                    0., 0.0009, 0.009, 0.09, 0.9, 
                                    0., 0., 0., 0., 0., 
                                    0.2, 0.2, 0.2, 0.2, 0.2 ];
        assert_mat_compare(&mat, &vec_to_mat_fixed(&target, 4, false), epsilon);
    }

    #[test]
    fn test_math_inplace_row_normalize_sparse() {
        let epsilon: I32F32 = I32F32::from_num(0.0001);
        let vector:Vec<f32> = vec![ 0., 1., 0., 2., 0., 3., 4., 
                                    0., 1., 0., 2., 0., 3., 0., 
                                    1., 0., 0., 2., 0., 3., 4., 
                                    0., 10., 0., 100., 1000., 0., 10000., 
                                    0., 0., 0., 0., 0., 0., 0., 
                                    1., 1., 1., 1., 1., 1., 1.];
        let mut mat = vec_to_sparse_mat_fixed(&vector, 6, false);
        inplace_row_normalize_sparse(&mut mat);
        let target:Vec<f32> = vec![ 0., 0.1, 0., 0.2, 0., 0.3, 0.4, 
                                    0., 0.166666, 0., 0.333333, 0., 0.5, 0., 
                                    0.1, 0., 0., 0.2, 0., 0.3, 0.4, 
                                    0., 0.0009, 0., 0.009, 0.09, 0., 0.9, 
                                    0., 0., 0., 0., 0., 0., 0., 
                                    0.142857, 0.142857, 0.142857, 0.142857, 0.142857, 0.142857, 0.142857];
        assert_sparse_mat_compare(&mat, &vec_to_sparse_mat_fixed(&target, 6, false), epsilon);
        let vector:Vec<f32> = vec![ 0., 0., 0., 0.,
                                    0., 0., 0., 0., 
                                    0., 0., 0., 0.];
        let target:Vec<f32> = vec![ 0., 0., 0., 0.,
                                    0., 0., 0., 0.,
                                    0., 0., 0., 0.];
        let mut mat = vec_to_sparse_mat_fixed(&vector, 3, false);
        inplace_row_normalize_sparse(&mut mat);
        assert_sparse_mat_compare(&mat, &vec_to_sparse_mat_fixed(&target, 3, false), I32F32::from_num( 0 ));
    }

    #[test]
    fn test_math_inplace_col_normalize() {
        let epsilon: I32F32 = I32F32::from_num(0.0001);
        let vector:Vec<f32> = vec![ 0., 1., 2., 3., 4., 
                                    0., 10., 100., 1000., 10000., 
                                    0., 0., 0., 0., 0., 
                                    1., 1., 1., 1., 1.];
        let mut mat = vec_to_mat_fixed(&vector, 4, true);
        inplace_col_normalize(&mut mat);
        let target:Vec<f32> = vec![ 0., 0.1, 0.2, 0.3, 0.4, 
                                    0., 0.0009, 0.009, 0.09, 0.9, 
                                    0., 0., 0., 0., 0., 
                                    0.2, 0.2, 0.2, 0.2, 0.2 ];
        assert_mat_compare(&mat, &vec_to_mat_fixed(&target, 4, true), epsilon);
    }

    #[test]
    fn test_math_inplace_col_normalize_sparse() {
        let epsilon: I32F32 = I32F32::from_num(0.0001);
        let vector:Vec<f32> = vec![ 0., 1., 0., 2., 0., 3., 4., 
                                    0., 1., 0., 2., 0., 3., 0., 
                                    1., 0., 0., 2., 0., 3., 4., 
                                    0., 10., 0., 100., 1000., 0., 10000., 
                                    0., 0., 0., 0., 0., 0., 0., 
                                    1., 1., 1., 1., 1., 1., 1.];
        let mut mat = vec_to_sparse_mat_fixed(&vector, 6, true);
        inplace_col_normalize_sparse(&mut mat, 6);
        let target:Vec<f32> = vec![ 0., 0.1, 0., 0.2, 0., 0.3, 0.4, 
                                    0., 0.166666, 0., 0.333333, 0., 0.5, 0., 
                                    0.1, 0., 0., 0.2, 0., 0.3, 0.4, 
                                    0., 0.0009, 0., 0.009, 0.09, 0., 0.9, 
                                    0., 0., 0., 0., 0., 0., 0., 
                                    0.142857, 0.142857, 0.142857, 0.142857, 0.142857, 0.142857, 0.142857];
        assert_sparse_mat_compare(&mat, &vec_to_sparse_mat_fixed(&target, 6, true), epsilon);
        let vector:Vec<f32> = vec![ 0., 0., 0., 0.,
                                    0., 0., 0., 0., 
                                    0., 0., 0., 0.];
        let target:Vec<f32> = vec![ 0., 0., 0., 0.,
                                    0., 0., 0., 0.,
                                    0., 0., 0., 0.];
        let mut mat = vec_to_sparse_mat_fixed(&vector, 3, false);
        inplace_col_normalize_sparse(&mut mat, 6);
        assert_sparse_mat_compare(&mat, &vec_to_sparse_mat_fixed(&target, 3, false), I32F32::from_num( 0 ));
        let mut mat: Vec<Vec<(u16, I32F32)>> = vec![];
        let target: Vec<Vec<(u16, I32F32)>> = vec![];
        inplace_col_normalize_sparse(&mut mat, 0);
        assert_sparse_mat_compare(&mat, &target, epsilon);
    }

    #[test]
    fn test_math_inplace_mask_vector() {
        let mask:Vec<bool> = vec![ false, false, false ];
        let mut vector:Vec<I32F32> = vec_to_fixed(&vec![ 0., 1., 2.]);
        let target:Vec<I32F32> = vec_to_fixed(&vec![ 0., 1., 2.]);
        inplace_mask_vector(&mask, &mut vector);
        assert_vec_compare(&vector, &target, I32F32::from_num( 0 ));
        let mask:Vec<bool> = vec![ false, true, false ];
        let mut vector:Vec<I32F32> = vec_to_fixed(&vec![ 0., 1., 2.]);
        let target:Vec<I32F32> = vec_to_fixed(&vec![ 0., 0., 2.]);
        inplace_mask_vector(&mask, &mut vector);
        assert_vec_compare(&vector, &target, I32F32::from_num( 0 ));
        let mask:Vec<bool> = vec![ true, true, true ];
        let mut vector:Vec<I32F32> = vec_to_fixed(&vec![ 0., 1., 2.]);
        let target:Vec<I32F32> = vec_to_fixed(&vec![ 0., 0., 0.]);
        inplace_mask_vector(&mask, &mut vector);
        assert_vec_compare(&vector, &target, I32F32::from_num( 0 ));
    }

    #[test]
    fn test_math_inplace_mask_matrix() {
        let mask:Vec<Vec<bool>> = vec![ vec![false, false, false], 
                                        vec![false, false, false], 
                                        vec![false, false, false]];
        let vector:Vec<f32> = vec![ 0., 1., 2., 
                                    3., 4., 5., 
                                    6., 7., 8.];
        let mut mat = vec_to_mat_fixed(&vector, 3, false);
        inplace_mask_matrix(&mask, &mut mat);
        assert_mat_compare(&mat, &vec_to_mat_fixed(&vector, 3, false), I32F32::from_num( 0 ));
        let mask:Vec<Vec<bool>> = vec![ vec![true, false, false], 
                                        vec![false, true, false], 
                                        vec![false, false, true]];
        let target:Vec<f32> = vec![ 0., 1., 2., 
                                    3., 0., 5., 
                                    6., 7., 0.];
        let mut mat = vec_to_mat_fixed(&vector, 3, false);
        inplace_mask_matrix(&mask, &mut mat);
        assert_mat_compare(&mat, &vec_to_mat_fixed(&target, 3, false), I32F32::from_num( 0 ));
        let mask:Vec<Vec<bool>> = vec![ vec![true, true, true], 
                                        vec![true, true, true], 
                                        vec![true, true, true]];
        let target:Vec<f32> = vec![ 0., 0., 0., 
                                    0., 0., 0., 
                                    0., 0., 0.];
        let mut mat = vec_to_mat_fixed(&vector, 3, false);
        inplace_mask_matrix(&mask, &mut mat);
        assert_mat_compare(&mat, &vec_to_mat_fixed(&target, 3, false), I32F32::from_num( 0 ));
    }

    #[test]
    fn test_math_inplace_mask_rows() {
        let input:Vec<f32> = vec![  1., 2., 3., 
                                    4., 5., 6., 
                                    7., 8., 9.];
        let mask:Vec<bool> = vec![false, false, false];
        let target:Vec<f32> = vec![ 1., 2., 3., 
                                    4., 5., 6., 
                                    7., 8., 9.];
        let mut mat = vec_to_mat_fixed(&input, 3, false);
        inplace_mask_rows(&mask, &mut mat);
        assert_mat_compare(&mat, &vec_to_mat_fixed(&target, 3, false), I32F32::from_num( 0 ));
        let mask:Vec<bool> = vec![true, true, true];
        let target:Vec<f32> = vec![ 0., 0., 0., 
                                    0., 0., 0., 
                                    0., 0., 0.];
        let mut mat = vec_to_mat_fixed(&input, 3, false);
        inplace_mask_rows(&mask, &mut mat);
        assert_mat_compare(&mat, &vec_to_mat_fixed(&target, 3, false), I32F32::from_num( 0 ));
        let mask:Vec<bool> = vec![true, false, true];
        let target:Vec<f32> = vec![ 0., 0., 0., 
                                    4., 5., 6., 
                                    0., 0., 0.];
        let mut mat = vec_to_mat_fixed(&input, 3, false);
        inplace_mask_rows(&mask, &mut mat);
        assert_mat_compare(&mat, &vec_to_mat_fixed(&target, 3, false), I32F32::from_num( 0 ));
        let input:Vec<f32> = vec![  0., 0., 0., 
                                    0., 0., 0., 
                                    0., 0., 0.];
        let mut mat = vec_to_mat_fixed(&input, 3, false);
        let mask:Vec<bool> = vec![false, false, false];
        let target:Vec<f32> = vec![ 0., 0., 0., 
                                    0., 0., 0., 
                                    0., 0., 0.];
        inplace_mask_rows(&mask, &mut mat);
        assert_mat_compare(&mat, &vec_to_mat_fixed(&target, 3, false), I32F32::from_num( 0 ));
    }

    #[test]
    fn test_math_inplace_mask_diag() {
        let vector:Vec<f32> = vec![ 1., 2., 3., 
                                    4., 5., 6., 
                                    7., 8., 9.];
        let target:Vec<f32> = vec![ 0., 2., 3., 
                                    4., 0., 6., 
                                    7., 8., 0.];
        let mut mat = vec_to_mat_fixed(&vector, 3, false);
        inplace_mask_diag(&mut mat);
        assert_mat_compare(&mat, &vec_to_mat_fixed(&target, 3, false), I32F32::from_num( 0 ));
    }

    #[test]
    fn test_math_mask_rows_sparse() {
        let input:Vec<f32> = vec![  1., 2., 3., 
                                    4., 5., 6., 
                                    7., 8., 9.];
        let mat = vec_to_sparse_mat_fixed(&input, 3, false);
        let mask:Vec<bool> = vec![false, false, false];
        let target:Vec<f32> = vec![ 1., 2., 3., 
                                    4., 5., 6., 
                                    7., 8., 9.];
        let result = mask_rows_sparse(&mask, &mat);
        assert_sparse_mat_compare(&result, &vec_to_sparse_mat_fixed(&target, 3, false), I32F32::from_num( 0 ));
        let mask:Vec<bool> = vec![true, true, true];
        let target:Vec<f32> = vec![ 0., 0., 0., 
                                    0., 0., 0., 
                                    0., 0., 0.];
        let result = mask_rows_sparse(&mask, &mat);
        assert_sparse_mat_compare(&result, &vec_to_sparse_mat_fixed(&target, 3, false), I32F32::from_num( 0 ));
        let mask:Vec<bool> = vec![true, false, true];
        let target:Vec<f32> = vec![ 0., 0., 0., 
                                    4., 5., 6., 
                                    0., 0., 0.];
        let result = mask_rows_sparse(&mask, &mat);
        assert_sparse_mat_compare(&result, &vec_to_sparse_mat_fixed(&target, 3, false), I32F32::from_num( 0 ));
        let input:Vec<f32> = vec![  0., 0., 0., 
                                    0., 0., 0., 
                                    0., 0., 0.];
        let mat = vec_to_sparse_mat_fixed(&input, 3, false);
        let mask:Vec<bool> = vec![false, false, false];
        let target:Vec<f32> = vec![ 0., 0., 0., 
                                    0., 0., 0., 
                                    0., 0., 0.];
        let result = mask_rows_sparse(&mask, &mat);
        assert_sparse_mat_compare(&result, &vec_to_sparse_mat_fixed(&target, 3, false), I32F32::from_num( 0 ));
    }

    #[test]
    fn test_math_mask_diag_sparse() {
        let vector:Vec<f32> = vec![ 1., 2., 3., 
                                    4., 5., 6., 
                                    7., 8., 9.];
        let target:Vec<f32> = vec![ 0., 2., 3., 
                                    4., 0., 6., 
                                    7., 8., 0.];
        let mat = vec_to_sparse_mat_fixed(&vector, 3, false);
        let result = mask_diag_sparse(&mat);
        assert_sparse_mat_compare(&result, &vec_to_sparse_mat_fixed(&target, 3, false), I32F32::from_num( 0 ));
        let vector:Vec<f32> = vec![ 1., 0., 0., 
                                    0., 5., 0., 
                                    0., 0., 9.];
        let target:Vec<f32> = vec![ 0., 0., 0., 
                                    0., 0., 0., 
                                    0., 0., 0.];
        let mat = vec_to_sparse_mat_fixed(&vector, 3, false);
        let result = mask_diag_sparse(&mat);
        assert_sparse_mat_compare(&result, &vec_to_sparse_mat_fixed(&target, 3, false), I32F32::from_num( 0 ));
        let vector:Vec<f32> = vec![ 0., 0., 0., 
                                    0., 0., 0., 
                                    0., 0., 0.];
        let target:Vec<f32> = vec![ 0., 0., 0., 
                                    0., 0., 0., 
                                    0., 0., 0.];
        let mat = vec_to_sparse_mat_fixed(&vector, 3, false);
        let result = mask_diag_sparse(&mat);
        assert_sparse_mat_compare(&result, &vec_to_sparse_mat_fixed(&target, 3, false), I32F32::from_num( 0 ));
    }

    #[test]
    fn test_math_vec_mask_sparse_matrix() {
        let vector:Vec<f32> = vec![ 1., 2., 3., 
                                    4., 5., 6., 
                                    7., 8., 9.];
        let target:Vec<f32> = vec![ 0., 2., 3., 
                                    4., 0., 6., 
                                    7., 8., 0.];
        let mat = vec_to_sparse_mat_fixed(&vector, 3, false);
        let first_vector: Vec<u64> = vec![ 1, 2, 3 ];
        let second_vector: Vec<u64> = vec![ 1, 2, 3 ];
        let result = vec_mask_sparse_matrix(&mat, &first_vector, &second_vector, &|a, b| a == b);
        assert_sparse_mat_compare(&result, &vec_to_sparse_mat_fixed(&target, 3, false), I32F32::from_num( 0 ));
        let target:Vec<f32> = vec![ 1., 0., 0., 
                                    4., 5., 0., 
                                    7., 8., 9.];
        let mat = vec_to_sparse_mat_fixed(&vector, 3, false);
        let first_vector: Vec<u64> = vec![ 1, 2, 3 ];
        let second_vector: Vec<u64> = vec![ 1, 2, 3 ];
        let result = vec_mask_sparse_matrix(&mat, &first_vector, &second_vector, &|a, b| a < b);
        assert_sparse_mat_compare(&result, &vec_to_sparse_mat_fixed(&target, 3, false), I32F32::from_num( 0 ));
        let vector:Vec<f32> = vec![ 0., 0., 0., 
                                    0., 0., 0., 
                                    0., 0., 0.];
        let target:Vec<f32> = vec![ 0., 0., 0., 
                                    0., 0., 0., 
                                    0., 0., 0.];
        let mat = vec_to_sparse_mat_fixed(&vector, 3, false);
        let first_vector: Vec<u64> = vec![ 1, 2, 3 ];
        let second_vector: Vec<u64> = vec![ 1, 2, 3 ];
        let result = vec_mask_sparse_matrix(&mat, &first_vector, &second_vector, &|a, b| a == b);
        assert_sparse_mat_compare(&result, &vec_to_sparse_mat_fixed(&target, 3, false), I32F32::from_num( 0 ));
    }

    #[test]
    fn test_math_row_hadamard() {
        let vector:Vec<I32F32> = vec_to_fixed( &vec![ 1., 2., 3., 4.] );
        let matrix:Vec<f32> = vec![ 1., 2., 3., 
                                    4., 5., 6., 
                                    7., 8., 9.,
                                    10., 11., 12.];
        let matrix = vec_to_mat_fixed(&matrix, 4, false);
        let result = row_hadamard(&matrix, &vector);
        let target:Vec<f32> = vec![ 1., 2., 3., 
                                    8., 10., 12., 
                                    21., 24., 27.,
                                    40., 44., 48.];
        let target = vec_to_mat_fixed(&target, 4, false);
        assert_mat_compare(&result, &target, I32F32::from_num( 0 ));
    }

    #[test]
    fn test_math_row_hadamard_sparse() {
        let vector:Vec<I32F32> = vec_to_fixed( &vec![ 1., 2., 3., 4.] );
        let matrix:Vec<f32> = vec![ 1., 2., 3., 
                                    4., 5., 6., 
                                    7., 8., 9.,
                                    10., 11., 12.];
        let matrix = vec_to_sparse_mat_fixed(&matrix, 4, false);
        let result = row_hadamard_sparse(&matrix, &vector);
        let target:Vec<f32> = vec![ 1., 2., 3., 
                                    8., 10., 12., 
                                    21., 24., 27.,
                                    40., 44., 48.];
        let target = vec_to_sparse_mat_fixed(&target, 4, false);
        assert_sparse_mat_compare(&result, &target, I32F32::from_num( 0 ));
        let matrix:Vec<f32> = vec![ 0., 2., 3., 
                                    4., 0., 6., 
                                    7., 8., 0.,
                                    10., 11., 12.];
        let matrix = vec_to_sparse_mat_fixed(&matrix, 4, false);
        let result = row_hadamard_sparse(&matrix, &vector);
        let target:Vec<f32> = vec![ 0., 2., 3., 
                                    8., 0., 12., 
                                    21., 24., 0.,
                                    40., 44., 48.];
        let target = vec_to_sparse_mat_fixed(&target, 4, false);
        assert_sparse_mat_compare(&result, &target, I32F32::from_num( 0 ));
        let matrix:Vec<f32> = vec![ 0., 0., 0., 
                                    0., 0., 0., 
                                    0., 0., 0.,
                                    0., 0., 0.];
        let matrix = vec_to_sparse_mat_fixed(&matrix, 4, false);
        let result = row_hadamard_sparse(&matrix, &vector);
        let target:Vec<f32> = vec![ 0., 0., 0., 
                                    0., 0., 0., 
                                    0., 0., 0.,
                                    0., 0., 0.];
        let target = vec_to_sparse_mat_fixed(&target, 4, false);
        assert_sparse_mat_compare(&result, &target, I32F32::from_num( 0 ));
    }

    #[test]
    fn test_math_row_sum() {
        let matrix:Vec<f32> = vec![ 1., 2., 3., 
                                    4., 5., 6., 
                                    7., 8., 9.,
                                    10., 11., 12.];
        let matrix = vec_to_mat_fixed(&matrix, 4, false);
        let result = row_sum( &matrix );
        let target: Vec<I32F32> = vec_to_fixed( &vec![ 6., 15., 24., 33. ] );
        assert_vec_compare(&result, &target, I32F32::from_num( 0 ));
    }
    
    #[test]
    fn test_math_row_sum_sparse() {
        let matrix:Vec<f32> = vec![ 1., 2., 3., 
                                    4., 5., 6., 
                                    7., 8., 9.,
                                    10., 11., 12.];
        let matrix = vec_to_sparse_mat_fixed(&matrix, 4, false);
        let result = row_sum_sparse(&matrix);
        let target: Vec<I32F32> = vec_to_fixed( &vec![ 6., 15., 24., 33. ] );
        assert_vec_compare(&result, &target, I32F32::from_num( 0 ));
        let matrix:Vec<f32> = vec![ 0., 2., 3., 
                                    4., 0., 6., 
                                    7., 8., 0.,
                                    10., 11., 12.];
        let matrix = vec_to_sparse_mat_fixed(&matrix, 4, false);
        let result = row_sum_sparse(&matrix);
        let target: Vec<I32F32> = vec_to_fixed( &vec![ 5., 10., 15., 33. ] );
        assert_vec_compare(&result, &target, I32F32::from_num( 0 ));
        let matrix:Vec<f32> = vec![ 1., 2., 3., 
                                    0., 0., 0., 
                                    7., 8., 9.,
                                    10., 11., 12.];
        let matrix = vec_to_sparse_mat_fixed(&matrix, 4, false);
        let result = row_sum_sparse(&matrix);
        let target: Vec<I32F32> = vec_to_fixed( &vec![ 6., 0., 24., 33. ] );
        assert_vec_compare(&result, &target, I32F32::from_num( 0 ));
        let matrix:Vec<f32> = vec![ 0., 0., 0., 
                                    0., 0., 0., 
                                    0., 0., 0.,
                                    0., 0., 0.];
        let matrix = vec_to_sparse_mat_fixed(&matrix, 4, false);
        let result = row_sum_sparse(&matrix);
        let target: Vec<I32F32> = vec_to_fixed( &vec![ 0., 0., 0., 0. ] );
        assert_vec_compare(&result, &target, I32F32::from_num( 0 ));
    }

    #[test]
    fn test_math_col_sum() {
        let matrix:Vec<f32> = vec![ 1., 2., 3., 
                                    4., 5., 6., 
                                    7., 8., 9.,
                                    10., 11., 12.];
        let matrix = vec_to_mat_fixed(&matrix, 4, false);
        let result = col_sum( &matrix );
        let target: Vec<I32F32> = vec_to_fixed( &vec![ 22., 26., 30. ] );
        assert_vec_compare(&result, &target, I32F32::from_num( 0 ));
    }
    
    #[test]
    fn test_math_col_sum_sparse() {
        let matrix:Vec<f32> = vec![ 1., 2., 3., 
                                    4., 5., 6., 
                                    7., 8., 9.,
                                    10., 11., 12.];
        let matrix = vec_to_sparse_mat_fixed(&matrix, 4, false);
        let result = col_sum_sparse(&matrix, 3);
        let target: Vec<I32F32> = vec_to_fixed( &vec![ 22., 26., 30. ] );
        assert_vec_compare(&result, &target, I32F32::from_num( 0 ));
        let matrix:Vec<f32> = vec![ 0., 2., 3., 
                                    4., 0., 6., 
                                    7., 8., 0.,
                                    10., 11., 12.];
        let matrix = vec_to_sparse_mat_fixed(&matrix, 4, false);
        let result = col_sum_sparse(&matrix, 3);
        let target: Vec<I32F32> = vec_to_fixed( &vec![ 21., 21., 21. ] );
        assert_vec_compare(&result, &target, I32F32::from_num( 0 ));
        let matrix:Vec<f32> = vec![ 1., 0., 3., 
                                    4., 0., 6., 
                                    7., 0., 9.,
                                    10., 0., 12.];
        let matrix = vec_to_sparse_mat_fixed(&matrix, 4, false);
        let result = col_sum_sparse(&matrix, 3);
        let target: Vec<I32F32> = vec_to_fixed( &vec![ 22., 0., 30. ] );
        assert_vec_compare(&result, &target, I32F32::from_num( 0 ));
        let matrix:Vec<f32> = vec![ 0., 0., 0., 
                                    0., 0., 0., 
                                    0., 0., 0.,
                                    0., 0., 0.];
        let matrix = vec_to_sparse_mat_fixed(&matrix, 4, false);
        let result = col_sum_sparse(&matrix,  3);
        let target: Vec<I32F32> = vec_to_fixed( &vec![ 0., 0., 0. ] );
        assert_vec_compare(&result, &target, I32F32::from_num( 0 ));
    }


    #[test]
    fn test_math_matmul() {
        let vector:Vec<I32F32> = vec_to_fixed( &vec![ 1., 2., 3., 4.] );
        let matrix:Vec<f32> = vec![ 1., 2., 3., 
                                    4., 5., 6., 
                                    7., 8., 9.,
                                    10., 11., 12.];
        let matrix = vec_to_mat_fixed(&matrix, 4, false);
        let result = matmul(&matrix, &vector);
        let target: Vec<I32F32> = vec_to_fixed( &vec![ 70., 80., 90. ] );
        assert_vec_compare(&result, &target, I32F32::from_num( 0 ));
    }

    #[test]
    fn test_math_matmul_transpose() {
        let vector:Vec<I32F32> = vec_to_fixed( &vec![ 1., 2., 3.] );
        let matrix:Vec<f32> = vec![ 1., 2., 3., 
                                    4., 5., 6., 
                                    7., 8., 9.,
                                    10., 11., 12.];
        let matrix = vec_to_mat_fixed(&matrix, 4, false);
        let result = matmul_transpose(&matrix, &vector);
        let target: Vec<I32F32> = vec_to_fixed( &vec![ 14., 32., 50., 68. ] );
        assert_vec_compare(&result, &target, I32F32::from_num( 0 ));
    }

    #[test]
    fn test_math_sparse_matmul() {
        let vector:Vec<I32F32> = vec_to_fixed( &vec![ 1., 2., 3., 4.] );
        let matrix:Vec<f32> = vec![ 1., 2., 3., 
                                    4., 5., 6., 
                                    7., 8., 9.,
                                    10., 11., 12.];
        let matrix = vec_to_sparse_mat_fixed(&matrix, 4, false);
        let result = matmul_sparse(&matrix, &vector, 3);
        let target: Vec<I32F32> = vec_to_fixed( &vec![ 70., 80., 90. ] );
        assert_vec_compare(&result, &target, I32F32::from_num( 0 ));
        let matrix:Vec<f32> = vec![ 0., 2., 3., 
                                    4., 0., 6., 
                                    7., 8., 0.,
                                    10., 11., 12.];
        let matrix = vec_to_sparse_mat_fixed(&matrix, 4, false);
        let result = matmul_sparse(&matrix, &vector, 3);
        let target: Vec<I32F32> = vec_to_fixed( &vec![ 69., 70., 63. ] );
        assert_vec_compare(&result, &target, I32F32::from_num( 0 ));
        let matrix:Vec<f32> = vec![ 0., 0., 0., 
                                    0., 0., 0., 
                                    0., 0., 0.,
                                    0., 0., 0.];
        let matrix = vec_to_sparse_mat_fixed(&matrix, 4, false);
        let result = matmul_sparse(&matrix, &vector, 3);
        let target: Vec<I32F32> = vec_to_fixed( &vec![ 0., 0., 0. ] );
        assert_vec_compare(&result, &target, I32F32::from_num( 0 ));
    }

    #[test]
    fn test_math_sparse_matmul_transpose() {
        let vector:Vec<I32F32> = vec_to_fixed( &vec![ 1., 2., 3.] );
        let matrix:Vec<f32> = vec![ 1., 2., 3., 
                                    4., 5., 6., 
                                    7., 8., 9.,
                                    10., 11., 12.];
        let matrix = vec_to_sparse_mat_fixed(&matrix, 4, false);
        let result = matmul_transpose_sparse(&matrix, &vector);
        let target: Vec<I32F32> = vec_to_fixed( &vec![ 14., 32., 50., 68. ] );
        assert_vec_compare(&result, &target, I32F32::from_num( 0 ));
        let matrix:Vec<f32> = vec![ 0., 2., 3., 
                                    4., 0., 6., 
                                    7., 8., 0.,
                                    10., 11., 12.];
        let matrix = vec_to_sparse_mat_fixed(&matrix, 4, false);
        let result = matmul_transpose_sparse(&matrix, &vector);
        let target: Vec<I32F32> = vec_to_fixed( &vec![ 13., 22., 23., 68. ] );
        assert_vec_compare(&result, &target, I32F32::from_num( 0 ));
        let matrix:Vec<f32> = vec![ 0., 0., 0., 
                                    0., 0., 0., 
                                    0., 0., 0.,
                                    0., 0., 0.];
        let matrix = vec_to_sparse_mat_fixed(&matrix, 4, false);
        let result = matmul_transpose_sparse(&matrix, &vector);
        let target: Vec<I32F32> = vec_to_fixed( &vec![ 0., 0., 0., 0. ] );
        assert_vec_compare(&result, &target, I32F32::from_num( 0 ));
    }

    #[test]
    fn test_math_inplace_col_clip() {
        let vector:Vec<I32F32> = vec_to_fixed( &vec![ 0., 5., 12.] );
        let matrix:Vec<f32> = vec![ 0., 2., 3., 
                                    4., 5., 6., 
                                    7., 8., 9.,
                                    10., 11., 12.];
        let mut matrix = vec_to_mat_fixed(&matrix, 4, false);
        let target:Vec<f32> = vec![ 0., 2., 3., 
                                    0., 5., 6., 
                                    0., 5., 9.,
                                    0., 5., 12.];
        let target = vec_to_mat_fixed(&target, 4, false);
        inplace_col_clip(&mut matrix, &vector);
        assert_mat_compare(&matrix, &target, I32F32::from_num( 0 ));
    }

    #[test]
    fn test_math_col_clip_sparse() {
        let vector:Vec<I32F32> = vec_to_fixed( &vec![ 0., 5., 12.] );
        let matrix:Vec<f32> = vec![ 0., 2., 3., 
                                    4., 5., 6., 
                                    7., 8., 9.,
                                    10., 11., 12.];
        let matrix = vec_to_sparse_mat_fixed(&matrix, 4, false);
        let target:Vec<f32> = vec![ 0., 2., 3., 
                                    0., 5., 6., 
                                    0., 5., 9.,
                                    0., 5., 12.];
        let target = vec_to_sparse_mat_fixed(&target, 4, false);
        let result = col_clip_sparse(&matrix, &vector);
        assert_sparse_mat_compare(&result, &target, I32F32::from_num( 0 ));
        let matrix:Vec<f32> = vec![ 0., 2., 3., 
                                    4., 5., 6., 
                                    0., 0., 0.,
                                    10., 11., 12.];
        let matrix = vec_to_sparse_mat_fixed(&matrix, 4, false);
        let target:Vec<f32> = vec![ 0., 2., 3., 
                                    0., 5., 6., 
                                    0., 0., 0.,
                                    0., 5., 12.];
        let target = vec_to_sparse_mat_fixed(&target, 4, false);
        let result = col_clip_sparse(&matrix, &vector);
        assert_sparse_mat_compare(&result, &target, I32F32::from_num( 0 ));
        let matrix:Vec<f32> = vec![ 0., 0., 0., 
                                    0., 0., 0., 
                                    0., 0., 0.,
                                    0., 0., 0.];
        let matrix = vec_to_sparse_mat_fixed(&matrix, 4, false);
        let target:Vec<f32> = vec![ 0., 0., 0., 
                                    0., 0., 0., 
                                    0., 0., 0.,
                                    0., 0., 0.];
        let target = vec_to_sparse_mat_fixed(&target, 4, false);
        let result = col_clip_sparse(&matrix, &vector);
        assert_sparse_mat_compare(&result, &target, I32F32::from_num( 0 ));
    }

    #[test]
    fn test_math_clip_sparse() {
        let matrix:Vec<f32> = vec![ 0., 2., 3., 
                                    4., 5., 6., 
                                    7., 8., 9.,
                                    10., 11., 12.];
        let matrix = vec_to_sparse_mat_fixed(&matrix, 4, false);
        let target:Vec<f32> = vec![ 0., 1., 1., 
                                    1., 1., 1., 
                                    1., 100., 100.,
                                    100., 100., 100.];
        let target = vec_to_sparse_mat_fixed(&target, 4, false);
        let result = clip_sparse(&matrix, I32F32::from_num(8), I32F32::from_num(100), I32F32::from_num(1));
        assert_sparse_mat_compare(&result, &target, I32F32::from_num( 0 ));
    }

    #[test]
    fn test_math_clip() {
        let matrix:Vec<f32> = vec![ 0., 2., 3., 
                                    4., 5., 6., 
                                    7., 8., 9.,
                                    10., 11., 12.];
        let matrix = vec_to_mat_fixed(&matrix, 4, false);
        let target:Vec<f32> = vec![ 1., 1., 1., 
                                    1., 1., 1., 
                                    1., 100., 100.,
                                    100., 100., 100.];
        let target = vec_to_mat_fixed(&target, 4, false);
        let result = clip(&matrix, I32F32::from_num(8), I32F32::from_num(100), I32F32::from_num(1));
        assert_mat_compare(&result, &target, I32F32::from_num( 0 ));
    }

    #[test]
    fn test_math_inplace_clip() {
        let matrix:Vec<f32> = vec![ 0., 2., 3., 
                                    4., 5., 6., 
                                    7., 8., 9.,
                                    10., 11., 12.];
        let mut matrix = vec_to_mat_fixed(&matrix, 4, false);
        let target:Vec<f32> = vec![ 1., 1., 1., 
                                    1., 1., 1., 
                                    1., 100., 100.,
                                    100., 100., 100.];
        let target = vec_to_mat_fixed(&target, 4, false);
        inplace_clip(&mut matrix, I32F32::from_num(8), I32F32::from_num(100), I32F32::from_num(1));
        assert_mat_compare(&matrix, &target, I32F32::from_num( 0 ));
    }

    #[test]
    fn test_math_hadamard() {
        let mat2: Vec<f32> = vec![  1., 2., 3., 
                                    4., 5., 6., 
                                    7., 8., 9.,
                                    10., 11., 12.];
        let mat1: Vec<f32> = vec![  10., 20., 30., 
                                    40., 50., 60., 
                                    70., 80., 90.,
                                    100., 110., 120.];
        let target: Vec<f32> = vec![10., 40., 90., 
                                    160., 250., 360., 
                                    490., 640., 810.,
                                    1000., 1210., 1440.];
        let mat2 = vec_to_mat_fixed(&mat2, 4, false);
        let mat1 = vec_to_mat_fixed(&mat1, 4, false);
        let target = vec_to_mat_fixed(&target, 4, false);
        let result = hadamard(&mat1, &mat2);
        assert_mat_compare(&result, &target, I32F32::from_num( 0.000001 ));
        let mat2: Vec<f32> = vec![  0., 0., 0., 
                                    0., 0., 0., 
                                    0., 0., 0.,
                                    0., 0., 0.];
        let mat1: Vec<f32> = vec![  0., 0., 0., 
                                    0., 0., 0., 
                                    0., 0., 0.,
                                    0., 0., 0.];
        let target: Vec<f32> = vec![0., 0., 0., 
                                    0., 0., 0., 
                                    0., 0., 0.,
                                    0., 0., 0.];
        let mat2 = vec_to_mat_fixed(&mat2, 4, false);
        let mat1 = vec_to_mat_fixed(&mat1, 4, false);
        let target = vec_to_mat_fixed(&target, 4, false);
        let result = hadamard(&mat1, &mat2);
        assert_mat_compare(&result, &target, I32F32::from_num( 0.000001 ));
        let mat2: Vec<f32> = vec![  1., 0., 0., 
                                    0., 2., 0., 
                                    0., 0., 3.,
                                    0., 0., 0.];
        let mat1: Vec<f32> = vec![  0., 0., 4., 
                                    0., 5., 0., 
                                    6., 0., 0.,
                                    0., 0., 0.];
        let target: Vec<f32> = vec![0., 0., 0., 
                                    0., 10., 0., 
                                    0., 0., 0.,
                                    0., 0., 0.];
        let mat2 = vec_to_mat_fixed(&mat2, 4, false);
        let mat1 = vec_to_mat_fixed(&mat1, 4, false);
        let target = vec_to_mat_fixed(&target, 4, false);
        let result = hadamard(&mat1, &mat2);
        assert_mat_compare(&result, &target, I32F32::from_num( 0.000001 ));
    }

    #[test]
    fn test_math_hadamard_sparse() {
        let mat2: Vec<f32> = vec![  1., 2., 3., 
                                    4., 5., 6., 
                                    7., 8., 9.,
                                    10., 11., 12.];
        let mat1: Vec<f32> = vec![  10., 20., 30., 
                                    40., 50., 60., 
                                    70., 80., 90.,
                                    100., 110., 120.];
        let target: Vec<f32> = vec![10., 40., 90., 
                                    160., 250., 360., 
                                    490., 640., 810.,
                                    1000., 1210., 1440.];
        let mat2 = vec_to_sparse_mat_fixed(&mat2, 4, false);
        let mat1 = vec_to_sparse_mat_fixed(&mat1, 4, false);
        let target = vec_to_sparse_mat_fixed(&target, 4, false);
        let result = hadamard_sparse(&mat1, &mat2, 3);
        assert_sparse_mat_compare(&result, &target, I32F32::from_num( 0.000001 ));
        let mat2: Vec<f32> = vec![  0., 0., 0., 
                                    0., 0., 0., 
                                    0., 0., 0.,
                                    0., 0., 0.];
        let mat1: Vec<f32> = vec![  0., 0., 0., 
                                    0., 0., 0., 
                                    0., 0., 0.,
                                    0., 0., 0.];
        let target: Vec<f32> = vec![0., 0., 0., 
                                    0., 0., 0., 
                                    0., 0., 0.,
                                    0., 0., 0.];
        let mat2 = vec_to_sparse_mat_fixed(&mat2, 4, false);
        let mat1 = vec_to_sparse_mat_fixed(&mat1, 4, false);
        let target = vec_to_sparse_mat_fixed(&target, 4, false);
        let result = hadamard_sparse(&mat1, &mat2, 3);
        assert_sparse_mat_compare(&result, &target, I32F32::from_num( 0.000001 ));
        let mat2: Vec<f32> = vec![  1., 0., 0., 
                                    0., 2., 0., 
                                    0., 0., 3.,
                                    0., 0., 0.];
        let mat1: Vec<f32> = vec![  0., 0., 4., 
                                    0., 5., 0., 
                                    6., 0., 0.,
                                    0., 0., 0.];
        let target: Vec<f32> = vec![0., 0., 0., 
                                    0., 10., 0., 
                                    0., 0., 0.,
                                    0., 0., 0.];
        let mat2 = vec_to_sparse_mat_fixed(&mat2, 4, false);
        let mat1 = vec_to_sparse_mat_fixed(&mat1, 4, false);
        let target = vec_to_sparse_mat_fixed(&target, 4, false);
        let result = hadamard_sparse(&mat1, &mat2, 3);
        assert_sparse_mat_compare(&result, &target, I32F32::from_num( 0.000001 ));
    }
    
    #[test]
    fn test_math_mat_ema() {
        let old: Vec<f32> = vec![   1., 2., 3., 
                                    4., 5., 6., 
                                    7., 8., 9.,
                                    10., 11., 12.];
        let new: Vec<f32> = vec![   10., 20., 30., 
                                    40., 50., 60., 
                                    70., 80., 90.,
                                    100., 110., 120.];
        let target: Vec<f32> = vec![1.9, 3.8, 5.7, 
                                    7.6, 9.5, 11.4, 
                                    13.3, 15.2, 17.1,
                                    19., 20.9, 22.8];
        let old = vec_to_mat_fixed(&old, 4, false);
        let new = vec_to_mat_fixed(&new, 4, false);
        let target = vec_to_mat_fixed(&target, 4, false);
        let result = mat_ema(&new, &old, I32F32::from_num(0.1));
        assert_mat_compare(&result, &target, I32F32::from_num( 0.000001 ));
        let old: Vec<f32> = vec![   1., 2., 3., 
                                    4., 5., 6., 
                                    7., 8., 9.,
                                    10., 11., 12.];
        let new: Vec<f32> = vec![   10., 20., 30., 
                                    40., 50., 60., 
                                    70., 80., 90.,
                                    100., 110., 120.];
        let target: Vec<f32> = vec![1., 2., 3., 
                                    4., 5., 6., 
                                    7., 8., 9.,
                                    10., 11., 12.];
        let old = vec_to_mat_fixed(&old, 4, false);
        let new = vec_to_mat_fixed(&new, 4, false);
        let target = vec_to_mat_fixed(&target, 4, false);
        let result = mat_ema(&new, &old, I32F32::from_num(0));
        assert_mat_compare(&result, &target, I32F32::from_num( 0.000001 ));
        let old: Vec<f32> = vec![   1., 2., 3., 
                                    4., 5., 6., 
                                    7., 8., 9.,
                                    10., 11., 12.];
        let new: Vec<f32> = vec![   10., 20., 30., 
                                    40., 50., 60., 
                                    70., 80., 90.,
                                    100., 110., 120.];
        let target: Vec<f32> = vec![10., 20., 30., 
                                    40., 50., 60., 
                                    70., 80., 90.,
                                    100., 110., 120.];
        let old = vec_to_mat_fixed(&old, 4, false);
        let new = vec_to_mat_fixed(&new, 4, false);
        let target = vec_to_mat_fixed(&target, 4, false);
        let result = mat_ema(&new, &old, I32F32::from_num(1));
        assert_mat_compare(&result, &target, I32F32::from_num( 0.000001 ));
    }

    #[test]
    fn test_math_sparse_mat_ema() {
        let old: Vec<f32> = vec![   1., 2., 3., 
                                    4., 5., 6., 
                                    7., 8., 9.,
                                    10., 11., 12.];
        let new: Vec<f32> = vec![   10., 20., 30., 
                                    40., 50., 60., 
                                    70., 80., 90.,
                                    100., 110., 120.];
        let target: Vec<f32> = vec![1.9, 3.8, 5.7, 
                                    7.6, 9.5, 11.4, 
                                    13.3, 15.2, 17.1,
                                    19., 20.9, 22.8];
        let old = vec_to_sparse_mat_fixed(&old, 4, false);
        let new = vec_to_sparse_mat_fixed(&new, 4, false);
        let target = vec_to_sparse_mat_fixed(&target, 4, false);
        let result = mat_ema_sparse(&new, &old, I32F32::from_num(0.1));
        assert_sparse_mat_compare(&result, &target, I32F32::from_num( 0.000001 ));
        let old: Vec<f32> = vec![   0., 2., 3., 
                                    4., 0., 6., 
                                    7., 8., 0.,
                                    10., 11., 12.];
        let new: Vec<f32> = vec![   10., 20., 0., 
                                    40., 0., 60., 
                                    0., 80., 90.,
                                    100., 110., 120.];
        let target: Vec<f32> = vec![1., 3.8, 2.7, 
                                    7.6, 0., 11.4, 
                                    6.3, 15.2, 9.,
                                    19., 20.9, 22.8];
        let old = vec_to_sparse_mat_fixed(&old, 4, false);
        let new = vec_to_sparse_mat_fixed(&new, 4, false);
        let target = vec_to_sparse_mat_fixed(&target, 4, false);
        let result = mat_ema_sparse(&new, &old, I32F32::from_num(0.1));
        assert_sparse_mat_compare(&result, &target, I32F32::from_num( 0.000001 ));
        let old: Vec<f32> = vec![   0., 0., 0., 
                                    0., 0., 0., 
                                    0., 0., 0.,
                                    0., 0., 0.];
        let new: Vec<f32> = vec![   10., 20., 0., 
                                    40., 0., 60., 
                                    0., 80., 90.,
                                    100., 110., 120.];
        let target: Vec<f32> = vec![1., 2., 0., 
                                    4., 0., 6., 
                                    0., 8., 9.,
                                    10., 11., 12.];
        let old = vec_to_sparse_mat_fixed(&old, 4, false);
        let new = vec_to_sparse_mat_fixed(&new, 4, false);
        let target = vec_to_sparse_mat_fixed(&target, 4, false);
        let result = mat_ema_sparse(&new, &old, I32F32::from_num(0.1));
        assert_sparse_mat_compare(&result, &target, I32F32::from_num( 0.000001 ));
        let old: Vec<f32> = vec![   0., 0., 0., 
                                    0., 0., 0., 
                                    0., 0., 0.,
                                    0., 0., 0.];
        let new: Vec<f32> = vec![   0., 0., 0., 
                                    0., 0., 0., 
                                    0., 0., 0.,
                                    0., 0., 0.];
        let target: Vec<f32> = vec![0., 0., 0., 
                                    0., 0., 0., 
                                    0., 0., 0.,
                                    0., 0., 0.];
        let old = vec_to_sparse_mat_fixed(&old, 4, false);
        let new = vec_to_sparse_mat_fixed(&new, 4, false);
        let target = vec_to_sparse_mat_fixed(&target, 4, false);
        let result = mat_ema_sparse(&new, &old, I32F32::from_num(0.1));
        assert_sparse_mat_compare(&result, &target, I32F32::from_num( 0.000001 ));
        let old: Vec<f32> = vec![   1., 0., 0., 
                                    0., 0., 0., 
                                    0., 0., 0.,
                                    0., 0., 0.];
        let new: Vec<f32> = vec![   0., 0., 0., 
                                    0., 2., 0., 
                                    0., 0., 0.,
                                    0., 0., 0.];
        let target: Vec<f32> = vec![0.9, 0., 0., 
                                    0., 0.2, 0., 
                                    0., 0., 0.,
                                    0., 0., 0.];
        let old = vec_to_sparse_mat_fixed(&old, 4, false);
        let new = vec_to_sparse_mat_fixed(&new, 4, false);
        let target = vec_to_sparse_mat_fixed(&target, 4, false);
        let result = mat_ema_sparse(&new, &old, I32F32::from_num(0.1));
        assert_sparse_mat_compare(&result, &target, I32F32::from_num( 0.000001 ));
    }

    #[test]
    fn test_math_matmul2() {
        let epsilon: I32F32 = I32F32::from_num(0.0001);
        let w: Vec<Vec<I32F32>> = vec![ vec![ I32F32::from_num(1.0);3 ]; 3 ]; 
        assert_vec_compare( &matmul( &w, &vec![ I32F32::from_num(1.0); 3] ), &vec![ I32F32::from_num(3),  I32F32::from_num(3),  I32F32::from_num(3)], epsilon );
        assert_vec_compare( &matmul( &w, &vec![ I32F32::from_num(2.0); 3] ), &vec![ I32F32::from_num(6),  I32F32::from_num(6),  I32F32::from_num(6)], epsilon );
        assert_vec_compare( &matmul( &w, &vec![ I32F32::from_num(3.0); 3] ), &vec![ I32F32::from_num(9),  I32F32::from_num(9),  I32F32::from_num(9)], epsilon );
        assert_vec_compare( &matmul( &w, &vec![ I32F32::from_num(-1.0); 3] ), &vec![ I32F32::from_num(-3),  I32F32::from_num(-3),  I32F32::from_num(-3)], epsilon );
        let w: Vec<Vec<I32F32>> = vec![ vec![ I32F32::from_num(-1.0);3 ]; 3 ]; 
        assert_vec_compare( &matmul( &w, &vec![ I32F32::from_num(1.0); 3] ), &vec![ I32F32::from_num(-3),  I32F32::from_num(-3),  I32F32::from_num(-3)], epsilon );
        assert_vec_compare( &matmul( &w, &vec![ I32F32::from_num(2.0); 3] ), &vec![ I32F32::from_num(-6),  I32F32::from_num(-6),  I32F32::from_num(-6)], epsilon );
        assert_vec_compare( &matmul( &w, &vec![ I32F32::from_num(3.0); 3] ), &vec![ I32F32::from_num(-9),  I32F32::from_num(-9),  I32F32::from_num(-9)], epsilon );
        assert_vec_compare( &matmul( &w, &vec![ I32F32::from_num(-1.0); 3] ), &vec![ I32F32::from_num(3),  I32F32::from_num(3),  I32F32::from_num(3)], epsilon );
        let w: Vec<Vec<I32F32>> = vec![ vec![ I32F32::from_num(1.0);3 ], vec![ I32F32::from_num(2.0); 3], vec![ I32F32::from_num(3.0);3 ] ]; 
        assert_vec_compare( &matmul( &w, &vec![ I32F32::from_num(0.0); 3] ), &vec![ I32F32::from_num(0.0),  I32F32::from_num(0.0),  I32F32::from_num(0.0)], epsilon );
        assert_vec_compare( &matmul( &w, &vec![ I32F32::from_num(2.0); 3] ), &vec![ I32F32::from_num(12),  I32F32::from_num(12),  I32F32::from_num(12)], epsilon );
        let w: Vec<Vec<I32F32>> = vec![ vec![ I32F32::from_num(1), I32F32::from_num(2), I32F32::from_num(3) ]; 3 ]; 
        assert_vec_compare( &matmul( &w, &vec![ I32F32::from_num(0.0); 3] ), &vec![ I32F32::from_num(0.0),  I32F32::from_num(0.0),  I32F32::from_num(0.0)], epsilon );
        assert_vec_compare( &matmul( &w, &vec![ I32F32::from_num(2.0); 3] ), &vec![ I32F32::from_num(6),  I32F32::from_num(12),  I32F32::from_num(18)], epsilon );
    }

}