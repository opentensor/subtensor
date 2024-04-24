#[macro_export]
macro_rules! assert_i64f64_approx_eq {
    ($left:expr, $right:expr $(,)?) => {{
        const PRECISION: u64 = 10000;
        let left = $left;
        let right = I64F64::from_num($right);
        let prec = I64F64::from_num(PRECISION);

        let l_rounded = (prec * left).round() / prec;
        let r_rounded = (prec * right).round() / prec;

        assert_eq!(l_rounded, r_rounded);
    }};
}

macro_rules! assert_i32f32_approx_eq {
    ($left:expr, $right:expr $(,)?) => {{
        const PRECISION: u64 = 10000;
        let left = $left;
        let right = I32F32::from_num($right);
        let prec = I32F32::from_num(PRECISION);

        let l_rounded = (prec * left).round() / prec;
        let r_rounded = (prec * right).round() / prec;

        assert_eq!(l_rounded, r_rounded);
    }};
}
