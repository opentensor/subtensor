#[allow(dead_code)]
#[macro_export]
macro_rules! assert_i64f64_approx_eq {
    ($left:expr, $right:expr $(,)?) => {{
        const PRECISION: u64 = 10000;
        let left = I64F64::from_num($left);
        let right = I64F64::from_num($right);
        let prec = I64F64::from_num(PRECISION);

        // TODO: Consider using Arithmetic rounding
        let l_rounded = (prec * left).round() / prec;
        let r_rounded = (prec * right).round() / prec;

        assert_eq!(l_rounded, r_rounded);
    }};
}

#[allow(dead_code)]
#[macro_export]
macro_rules! assert_i32f32_approx_eq {
    ($left:expr, $right:expr $(,)?) => {{
        const PRECISION: u64 = 10000;
        let left = I32F32::from_num($left);
        let right = I32F32::from_num($right);
        let prec = I32F32::from_num(PRECISION);

        let l_rounded = (prec * left).round() / prec;
        let r_rounded = (prec * right).round() / prec;

        assert_eq!(l_rounded, r_rounded);
    }};
}

#[allow(dead_code)]
#[macro_export]
macro_rules! assert_approx_eq {
    ($left:expr, $right:expr $(,)?) => {{
        const PRECISION: f64 = 100.;
        let left = $left;
        let right = $right;

        let l_rounded = (PRECISION * left).round() / PRECISION;
        let r_rounded = (PRECISION * right).round() / PRECISION;

        assert_eq!(l_rounded, r_rounded);
    }};
}

#[allow(dead_code)]
#[macro_export]
macro_rules! assert_substake_eq {
    ($coldkey:expr, $hotkey:expr, $netuid:expr, $amount:expr $(,)?) => {{
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey($coldkey, $hotkey, $netuid),
            $amount
        );
    }};
}

#[allow(dead_code)]
#[macro_export]
macro_rules! assert_substake_approx_eq {
    ($coldkey:expr, $hotkey:expr, $netuid:expr, $amount:expr $(,)?) => {{
        let subst =
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey($coldkey, $hotkey, $netuid)
                as f64;
        assert_approx_eq!(subst / 1_000_000_000f64, $amount);
    }};
}
