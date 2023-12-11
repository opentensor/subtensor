// Normalizes (sum to 1 except 0) each row (dim=0) of a matrix in-place.
#[allow(dead_code)]
pub fn inplace_row_normalize(x: &mut Vec<Vec<I32F32>>) 
{
    for i in 0..x.len() 
    {
        let row_sum: I32F32 = x[i].iter().sum();
        if row_sum > I32F32::from_num(0.0 as f32) 
        {
            x[i].iter_mut()
                .for_each(|x_ij: &mut I32F32| 
                    *x_ij /= row_sum
                );
        }
    }
}

// Normalizes (sum to 1 except 0) each row (dim=0) of a sparse matrix in-place.
#[allow(dead_code)]
pub fn inplace_row_normalize_sparse(sparse_matrix: &mut Vec<Vec<(u16, I32F32)>>) 
{
    for sparse_row in sparse_matrix.iter_mut() 
    {
        let row_sum: I32F32 = sparse_row.iter().map(|(_j, value)| *value).sum();
        if row_sum > I32F32::from_num(0.0) 
        {
            sparse_row
                .iter_mut()
                .for_each(|(_j, value)| 
                    *value /= row_sum
                );
        }
    }
}

// Sum across each row (dim=0) of a matrix.
#[allow(dead_code)]
pub fn row_sum(x: &Vec<Vec<I32F32>>) -> Vec<I32F32> 
{
    if x.len() == 0 
    {
        return vec![];
    }

    if x[0].len() == 0 
    {
        return vec![];
    }

    let rows:       usize       = x.len();
    let mut result: Vec<I32F32> = vec![I32F32::from_num(0); rows];

    for i in 0..x.len() 
    {
        for j in 0..x[i].len() 
        {
            result[i] += x[i][j];
        }
    }

    return result;
}

// Sum across each row (dim=0) of a sparse matrix.
#[allow(dead_code)]
pub fn row_sum_sparse(sparse_matrix: &Vec<Vec<(u16, I32F32)>>) -> Vec<I32F32> 
{
    let rows:       usize       = sparse_matrix.len();
    let mut result: Vec<I32F32> = vec![I32F32::from_num(0); rows];

    for (i, sparse_row) in sparse_matrix.iter().enumerate() 
    {
        for (_j, value) in sparse_row.iter() 
        {
            result[i] += value;
        }
    }

    return result;
}

// Sum across each column (dim=1) of a matrix.
#[allow(dead_code)]
pub fn col_sum(x: &Vec<Vec<I32F32>>) -> Vec<I32F32> 
{
    if x.len() == 0 
    {
        return vec![];
    }

    if x[0].len() == 0 
    {
        return vec![];
    }

    let cols:       usize       = x[0].len();
    let mut result: Vec<I32F32> = vec![I32F32::from_num(0); cols];

    for i in 0..x.len() 
    {
        assert_eq!(x[i].len(), cols);
        for j in 0..cols 
        {
            result[j] += x[i][j];
        }
    }

    return result;
}

// Sum across each column (dim=1) of a sparse matrix.
#[allow(dead_code)]
pub fn col_sum_sparse(sparse_matrix: &Vec<Vec<(u16, I32F32)>>, columns: u16) -> Vec<I32F32> 
{
    let mut result: Vec<I32F32> = vec![I32F32::from_num(0); columns as usize];
    for sparse_row in sparse_matrix.iter() 
    {
        for (j, value) in sparse_row.iter() 
        {
            result[*j as usize] += value;
        }
    }

    return result;
}

// Normalizes (sum to 1 except 0) each column (dim=1) of a sparse matrix in-place.
#[allow(dead_code)]
pub fn inplace_col_normalize_sparse(sparse_matrix: &mut Vec<Vec<(u16, I32F32)>>, columns: u16) 
{
    let mut col_sum: Vec<I32F32> = vec![I32F32::from_num(0.0); columns as usize]; // assume square matrix, rows=cols
    for sparse_row in sparse_matrix.iter() 
    {
        for (j, value) in sparse_row.iter() 
        {
            col_sum[*j as usize] += value;
        }
    }

    for sparse_row in sparse_matrix.iter_mut() 
    {
        for (j, value) in sparse_row.iter_mut() 
        {
            if col_sum[*j as usize] == I32F32::from_num(0.0 as f32) 
            {
                continue;
            }

            *value /= col_sum[*j as usize];
        }
    }
}

// Normalizes (sum to 1 except 0) each column (dim=1) of a matrix in-place.
#[allow(dead_code)]
pub fn inplace_col_normalize(x: &mut Vec<Vec<I32F32>>) 
{
    if x.len() == 0 
    {
        return;
    }

    if x[0].len() == 0 
    {
        return;
    }

    let cols:           usize       = x[0].len();
    let mut col_sum:    Vec<I32F32> = vec![I32F32::from_num(0.0); cols];

    for i in 0..x.len() 
    {
        assert_eq!(x[i].len(), cols);
        for j in 0..cols 
        {
            col_sum[j] += x[i][j];
        }
    }

    for j in 0..cols 
    {
        if col_sum[j] == I32F32::from_num(0.0 as f32) 
        {
            continue;
        }

        for i in 0..x.len() 
        {
            x[i][j] /= col_sum[j];
        }
    }
}

// Max-upscale each column (dim=1) of a sparse matrix in-place.
#[allow(dead_code)]
pub fn inplace_col_max_upscale_sparse(sparse_matrix: &mut Vec<Vec<(u16, I32F32)>>, columns: u16) 
{
    let mut col_max: Vec<I32F32> = vec![I32F32::from_num(0.0); columns as usize]; // assume square matrix, rows=cols
    for sparse_row in sparse_matrix.iter() 
    {
        for (j, value) in sparse_row.iter() 
        {
            if col_max[*j as usize] < *value 
            {
                col_max[*j as usize] = *value;
            }
        }
    }

    for sparse_row in sparse_matrix.iter_mut() 
    {
        for (j, value) in sparse_row.iter_mut() 
        {
            if col_max[*j as usize] == I32F32::from_num(0.0 as f32) 
            {
                continue;
            }

            *value /= col_max[*j as usize];
        }
    }
}

// Max-upscale each column (dim=1) of a matrix in-place.
#[allow(dead_code)]
pub fn inplace_col_max_upscale(x: &mut Vec<Vec<I32F32>>) 
{
    if x.len() == 0 
    {
        return;
    }

    if x[0].len() == 0 
    {
        return;
    }

    let cols:           usize       = x[0].len();
    let mut col_max:    Vec<I32F32> = vec![I32F32::from_num(0.0); cols];

    for i in 0..x.len() 
    {
        assert_eq!(x[i].len(), cols);
        for j in 0..cols 
        {
            if col_max[j] < x[i][j] 
            {
                col_max[j] = x[i][j];
            }
        }
    }

    for j in 0..cols 
    {
        if col_max[j] == I32F32::from_num(0.0 as f32) 
        {
            continue;
        }

        for i in 0..x.len() 
        {
            x[i][j] /= col_max[j];
        }
    }
}

// Apply mask to vector, mask=true will mask out, i.e. set to 0.
#[allow(dead_code)]
pub fn inplace_mask_vector(mask: &Vec<bool>, vector: &mut Vec<I32F32>) 
{
    if mask.len() == 0 
    {
        return;
    }

    assert_eq!(mask.len(), vector.len());

    let zero: I32F32 = I32F32::from_num(0.0);
    for i in 0..mask.len() 
    {
        if mask[i] 
        {
            vector[i] = zero;
        }
    }
}

// Apply mask to matrix, mask=true will mask out, i.e. set to 0.
#[allow(dead_code)]
pub fn inplace_mask_matrix(mask: &Vec<Vec<bool>>, matrix: &mut Vec<Vec<I32F32>>) 
{
    if mask.len() == 0 
    {
        return;
    }

    if mask[0].len() == 0 
    {
        return;
    }

    assert_eq!(mask.len(), matrix.len());

    let zero: I32F32 = I32F32::from_num(0.0);
    for i in 0..mask.len() 
    {
        for j in 0..mask[i].len() 
        {
            if mask[i][j] 
            {
                matrix[i][j] = zero;
            }
        }
    }
}

// Apply row mask to matrix, mask=true will mask out, i.e. set to 0.
#[allow(dead_code)]
pub fn inplace_mask_rows(mask: &Vec<bool>, matrix: &mut Vec<Vec<I32F32>>) 
{
    let rows: usize = matrix.len();
    if rows == 0 
    {
        return;
    }

    let cols: usize = matrix[0].len();
    assert_eq!(mask.len(), rows);

    let zero: I32F32 = I32F32::from_num(0);
    for i in 0..rows 
    {
        if mask[i] 
        {
            matrix[i] = vec![zero; cols];
        }
    }
}

// Mask out the diagonal of the input matrix in-place.
#[allow(dead_code)]
pub fn inplace_mask_diag(matrix: &mut Vec<Vec<I32F32>>) 
{
    if matrix.len() == 0 
    {
        return;
    }

    if matrix[0].len() == 0 
    {
        return;
    }

    assert_eq!(matrix.len(), matrix[0].len());

    let zero: I32F32 = I32F32::from_num(0.0);
    for i in 0..matrix.len() 
    {
        matrix[i][i] = zero;
    }
}

// Return a new sparse matrix that replaces masked rows with an empty vector placeholder.
#[allow(dead_code)]
pub fn mask_rows_sparse(mask: &Vec<bool>, sparse_matrix: &Vec<Vec<(u16, I32F32)>>) -> Vec<Vec<(u16, I32F32)>> 
{
    let n:          usize                   = sparse_matrix.len();
    assert_eq!(n, mask.len());
    let mut result: Vec<Vec<(u16, I32F32)>> = vec![vec![]; n];
    for (i, sparse_row) in sparse_matrix.iter().enumerate() 
    {
        if !mask[i] 
        {
            result[i] = sparse_row.clone();
        }
    }

    return result;
}

// Return a new sparse matrix with a masked out diagonal of input sparse matrix.
#[allow(dead_code)]
pub fn mask_diag_sparse(sparse_matrix: &Vec<Vec<(u16, I32F32)>>) -> Vec<Vec<(u16, I32F32)>> 
{
    let n:          usize                   = sparse_matrix.len();
    let mut result: Vec<Vec<(u16, I32F32)>> = vec![vec![]; n];
    for (i, sparse_row) in sparse_matrix.iter().enumerate() 
    {
        for (j, value) in sparse_row.iter() 
        {
            if i != (*j as usize) 
            {
                result[i].push((*j, *value));
            }
        }
    }

    return result;
}

// Remove cells from sparse matrix where the mask function of two vectors is true.
#[allow(dead_code)]
pub fn vec_mask_sparse_matrix(sparse_matrix: &Vec<Vec<(u16, I32F32)>>, first_vector: &Vec<u64>, second_vector: &Vec<u64>, mask_fn: &dyn Fn(u64, u64) -> bool) 
    -> Vec<Vec<(u16, I32F32)>> 
{
    let n:          usize                   = sparse_matrix.len();
    let mut result: Vec<Vec<(u16, I32F32)>> = vec![vec![]; n];
    for (i, sparse_row) in sparse_matrix.iter().enumerate() 
    {
        for (j, value) in sparse_row.iter() 
        {
            if !mask_fn(first_vector[i], second_vector[*j as usize]) 
            {
                result[i].push((*j, *value));
            }
        }
    }

    return result;
}

// Row-wise matrix-vector hadamard product.
#[allow(dead_code)]
pub fn row_hadamard(matrix: &Vec<Vec<I32F32>>, vector: &Vec<I32F32>) -> Vec<Vec<I32F32>> 
{
    if matrix.len() == 0 
    {
        return vec![vec![]];
    }

    if matrix[0].len() == 0 
    {
        return vec![vec![]];
    }

    let mut result: Vec<Vec<I32F32>> = vec![vec![I32F32::from_num(0.0); matrix[0].len()]; matrix.len()];
    for i in 0..matrix.len() 
    {
        for j in 0..matrix[i].len()
        {
            result[i][j] = vector[i] * matrix[i][j];
        }
    }

    return result;
}

// Row-wise sparse matrix-vector hadamard product.
#[allow(dead_code)]
pub fn row_hadamard_sparse(sparse_matrix: &Vec<Vec<(u16, I32F32)>>, vector: &Vec<I32F32>) -> Vec<Vec<(u16, I32F32)>> 
{
    let mut result: Vec<Vec<(u16, I32F32)>> = sparse_matrix.clone();
    for (i, sparse_row) in result.iter_mut().enumerate() 
    {
        for (_j, value) in sparse_row.iter_mut() 
        {
            *value *= vector[i];
        }
    }

    return result;
}

// Row-wise matrix-vector product, column-wise sum: result_j = SUM(i) vector_i * matrix_ij.
#[allow(dead_code)]
pub fn matmul(matrix: &Vec<Vec<I32F32>>, vector: &Vec<I32F32>) -> Vec<I32F32> 
{
    if matrix.len() == 0 
    {
        return vec![];
    }

    if matrix[0].len() == 0 
    {
        return vec![];
    }

    assert!(matrix.len() == vector.len());
    let mut result: Vec<I32F32> = vec![I32F32::from_num(0.0); matrix[0].len()];

    for i in 0..matrix.len() 
    {
        for j in 0..matrix[i].len() 
        {
            // Compute ranks: r_j = SUM(i) w_ij * s_i
            // Compute trust scores: t_j = SUM(i) w_ij * s_i
            // result_j = SUM(i) vector_i * matrix_ij
            result[j] += vector[i] * matrix[i][j];
        }
    }
    
    return result;
}

// Row-wise matrix-vector product, column-wise sum: result_j = SUM(i) vector_i * matrix_ij.
#[allow(dead_code)]
pub fn matmul_64(matrix: &Vec<Vec<I64F64>>, vector: &Vec<I64F64>) -> Vec<I64F64> 
{
    if matrix.len() == 0 
    {
        return vec![];
    }

    if matrix[0].len() == 0 
    {
        return vec![];
    }

    assert!(matrix.len() == vector.len());
    let mut result: Vec<I64F64> = vec![I64F64::from_num(0.0); matrix[0].len()];

    for i in 0..matrix.len() 
    {
        for j in 0..matrix[i].len() 
        {
            // Compute ranks: r_j = SUM(i) w_ij * s_i
            // Compute trust scores: t_j = SUM(i) w_ij * s_i
            // result_j = SUM(i) vector_i * matrix_ij
            result[j] += vector[i] * matrix[i][j];
        }
    }

    return result;
}

// Column-wise matrix-vector product, row-wise sum: result_i = SUM(j) vector_j * matrix_ij.
#[allow(dead_code)]
pub fn matmul_transpose(matrix: &Vec<Vec<I32F32>>, vector: &Vec<I32F32>) -> Vec<I32F32> 
{
    if matrix.len() == 0 
    {
        return vec![];
    }

    if matrix[0].len() == 0 
    {
        return vec![];
    }

    assert!(matrix[0].len() == vector.len());
    let mut result: Vec<I32F32> = vec![I32F32::from_num(0.0); matrix.len()];

    for i in 0..matrix.len()
    {
        for j in 0..matrix[i].len() 
        {
            // Compute dividends: d_j = SUM(i) b_ji * inc_i
            // result_j = SUM(i) vector_i * matrix_ji
            // result_i = SUM(j) vector_j * matrix_ij
            result[i] += vector[j] * matrix[i][j];
        }
    }

    return result;
}

// Row-wise sparse_matrix-vector product, column-wise sum: result_j = SUM(i) vector_i * matrix_ij.
#[allow(dead_code)]
pub fn matmul_sparse(sparse_matrix: &Vec<Vec<(u16, I32F32)>>, vector: &Vec<I32F32>, columns: u16) -> Vec<I32F32> 
{
    let mut result: Vec<I32F32> = vec![I32F32::from_num(0.0); columns as usize];
    for (i, sparse_row) in sparse_matrix.iter().enumerate() 
    {
        for (j, value) in sparse_row.iter() 
        {
            // Compute ranks: r_j = SUM(i) w_ij * s_i
            // Compute trust scores: t_j = SUM(i) w_ij * s_i
            // result_j = SUM(i) vector_i * matrix_ij
            result[*j as usize] += vector[i] * value;
        }
    }

    return result;
}

// Column-wise sparse_matrix-vector product, row-wise sum: result_i = SUM(j) vector_j * matrix_ij.
#[allow(dead_code)]
pub fn matmul_transpose_sparse(sparse_matrix: &Vec<Vec<(u16, I32F32)>>, vector: &Vec<I32F32>) -> Vec<I32F32> 
{
    let mut result: Vec<I32F32> = vec![I32F32::from_num(0.0); sparse_matrix.len()];
    for (i, sparse_row) in sparse_matrix.iter().enumerate() 
    {
        for (j, value) in sparse_row.iter() 
        {
            // Compute dividends: d_j = SUM(i) b_ji * inc_i
            // result_j = SUM(i) vector_i * matrix_ji
            // result_i = SUM(j) vector_j * matrix_ij
            result[i] += vector[*j as usize] * value;
        }
    }

    return result;
}

// Set inplace matrix values above column threshold to threshold value.
#[allow(dead_code)]
pub fn inplace_col_clip(x: &mut Vec<Vec<I32F32>>, col_threshold: &Vec<I32F32>) 
{
    for i in 0..x.len() 
    {
        for j in 0..x[i].len() 
        {
            if x[i][j] > col_threshold[j] 
            {
                x[i][j] = col_threshold[j];
            }
        }
    }
}

// Return sparse matrix with values above column threshold set to threshold value.
#[allow(dead_code)]
pub fn col_clip_sparse(sparse_matrix: &Vec<Vec<(u16, I32F32)>>, col_threshold: &Vec<I32F32>) -> Vec<Vec<(u16, I32F32)>> 
{
    let mut result: Vec<Vec<(u16, I32F32)>> = vec![vec![]; sparse_matrix.len()];
    for (i, sparse_row) in sparse_matrix.iter().enumerate() 
    {
        for (j, value) in sparse_row.iter() 
        {
            if col_threshold[*j as usize] < *value 
            {
                if 0 < col_threshold[*j as usize] 
                {
                    result[i].push((*j, col_threshold[*j as usize]));
                }
            } 
            else
            {
                result[i].push((*j, *value));
            }
        }
    }

    return result;
}

// Set matrix values below threshold to lower, and equal-above to upper.
#[allow(dead_code)]
pub fn clip(x: &Vec<Vec<I32F32>>, threshold: I32F32, upper: I32F32, lower: I32F32) -> Vec<Vec<I32F32>> 
{
    // Check Nill length.
    if x.len() == 0 
    {
        return vec![vec![]];
    }

    let mut result: Vec<Vec<I32F32>> = vec![vec![lower; x[0].len()]; x.len()];
    for i in 0..x.len() 
    {
        for j in 0..x[i].len() 
        {
            if x[i][j] >= threshold 
            {
                result[i][j] = upper;
            }
        }
    }

    return result;
}

// Set inplace matrix values below threshold to lower, and equal-above to upper.
#[allow(dead_code)]
pub fn inplace_clip(x: &mut Vec<Vec<I32F32>>, threshold: I32F32, upper: I32F32, lower: I32F32) 
{
    for i in 0..x.len() 
    {
        for j in 0..x[i].len() 
        {
            if x[i][j] >= threshold 
            {
                x[i][j] = upper;
            } 
            else 
            {
                x[i][j] = lower;
            }
        }
    }
}

// Set sparse matrix values below threshold to lower, and equal-above to upper.
// Does not add missing elements (0 value assumed) when lower!=0.
#[allow(dead_code)]
pub fn clip_sparse(sparse_matrix: &Vec<Vec<(u16, I32F32)>>, threshold: I32F32, upper: I32F32, lower: I32F32) -> Vec<Vec<(u16, I32F32)>> 
{
    let mut result: Vec<Vec<(u16, I32F32)>> = vec![vec![]; sparse_matrix.len()];
    for (i, sparse_row) in sparse_matrix.iter().enumerate() 
    {
        for (j, value) in sparse_row.iter() 
        {
            if *value < threshold 
            {
                result[i].push((*j, lower));
            } 
            else 
            {
                result[i].push((*j, upper));
            }
        }
    }

    return result;
}

// Element-wise product of two matrices.
#[allow(dead_code)]
pub fn hadamard(mat1: &Vec<Vec<I32F32>>, mat2: &Vec<Vec<I32F32>>) -> Vec<Vec<I32F32>> 
{
    assert!(mat1.len() == mat2.len());
    
    if mat1.len() == 0 
    {
        return vec![vec![]; 1];
    }

    if mat1[0].len() == 0 
    {
        return vec![vec![]; 1];
    }

    let mut result: Vec<Vec<I32F32>> = vec![vec![I32F32::from_num(0); mat1[0].len()]; mat1.len()];
    for i in 0..mat1.len() 
    {
        assert!(mat1[i].len() == mat2[i].len());
        for j in 0..mat1[i].len() 
        {
            result[i][j] = mat1[i][j] * mat2[i][j];
        }
    }

    return result;
}

// Element-wise product of two sparse matrices.
#[allow(dead_code)]
pub fn hadamard_sparse(mat1: &Vec<Vec<(u16, I32F32)>>, mat2: &Vec<Vec<(u16, I32F32)>>, columns: u16) -> Vec<Vec<(u16, I32F32)>> 
{
    assert!(mat1.len() == mat2.len());

    let rows:       usize                   = mat1.len();
    let zero:       I32F32                  = I32F32::from_num(0);
    let mut result: Vec<Vec<(u16, I32F32)>> = vec![vec![]; rows];

    for i in 0..rows 
    {
        let mut row1: Vec<I32F32> = vec![zero; columns as usize];
        for (j, value) in mat1[i].iter() 
        {
            row1[*j as usize] += value;
        }

        let mut row2: Vec<I32F32> = vec![zero; columns as usize];
        for (j, value) in mat2[i].iter() 
        {
            row2[*j as usize] += value;
        }

        for j in 0..columns as usize 
        {
            let prod: I32F32 = row1[j] * row2[j];
            if zero < prod 
            {
                result[i].push((j as u16, prod))
            }
        }
    }

    return result;
}

// Return matrix exponential moving average: `alpha * a_ij + one_minus_alpha * b_ij`.
// `alpha` is the EMA coefficient, how much to add of the new observation, typically small,
// higher alpha discounts older observations faster.
#[allow(dead_code)]
pub fn mat_ema(new: &Vec<Vec<I32F32>>, old: &Vec<Vec<I32F32>>, alpha: I32F32) -> Vec<Vec<I32F32>> 
{
    if new.len() == 0 
    {
        return vec![vec![]; 1];
    }

    if new[0].len() == 0 
    {
        return vec![vec![]; 1];
    }

    let one_minus_alpha:    I32F32              = I32F32::from_num(1.0) - alpha;
    let mut result:         Vec<Vec<I32F32>>    = vec![vec![I32F32::from_num(0.0); new[0].len()]; new.len()];
    assert!(new.len() == old.len());
    for i in 0..new.len() 
    {
        assert!(new[i].len() == old[i].len());
        for j in 0..new[i].len() 
        {
            result[i][j] = alpha * new[i][j] + one_minus_alpha * old[i][j];
        }
    }

    return result;
}

// Return sparse matrix exponential moving average: `alpha * a_ij + one_minus_alpha * b_ij`.
// `alpha` is the EMA coefficient, how much to add of the new observation, typically small,
// higher alpha discounts older observations faster.
#[allow(dead_code)]
pub fn mat_ema_sparse(new: &Vec<Vec<(u16, I32F32)>>, old: &Vec<Vec<(u16, I32F32)>>, alpha: I32F32) -> Vec<Vec<(u16, I32F32)>> 
{
    assert!(new.len() == old.len());

    let n:                  usize                   = new.len(); // assume square matrix, rows=cols
    let zero:               I32F32                  = I32F32::from_num(0.0);
    let one_minus_alpha:    I32F32                  = I32F32::from_num(1.0) - alpha;
    let mut result:         Vec<Vec<(u16, I32F32)>> = vec![vec![]; n];

    for i in 0..new.len() 
    {
        let mut row: Vec<I32F32> = vec![zero; n];
        for (j, value) in new[i].iter() 
        {
            row[*j as usize] += alpha * value;
        }

        for (j, value) in old[i].iter() 
        {
            row[*j as usize] += one_minus_alpha * value;
        }

        for (j, value) in row.iter().enumerate() 
        {
            if *value > zero 
            {
                result[i].push((j as u16, *value))
            }
        }
    }

    return result;
}

// Return sparse matrix only with elements >= threshold of an input sparse matrix.
#[allow(dead_code)]
pub fn sparse_threshold(w: &Vec<Vec<(u16, I32F32)>>, threshold: I32F32) -> Vec<Vec<(u16, I32F32)>> 
{
    let mut sparse_threshold_result: Vec<Vec<(u16, I32F32)>> = vec![vec![]; w.len()];
    for (uid_i, weights_i) in w.iter().enumerate() 
    {
        for (uid_j, weight_ij) in weights_i.iter() 
        {
            if *weight_ij >= threshold 
            {
                sparse_threshold_result[uid_i as usize].push((*uid_j, *weight_ij));
            }
        }
    }

    return sparse_threshold_result;
}