use super::*;
use frame_support::weights::Weight;
use log;
use scale_info::prelude::string::String;
use crate::{Config, HasMigrationRun, SubnetLocked, TaoCurrency};
use subtensor_runtime_common::NetUid;

pub fn migrate_restore_subnet_locked<T: Config>() -> Weight {
    // Track whether we've already run this migration
    let migration_name = b"migrate_restore_subnet_locked".to_vec();
    let mut weight = T::DbWeight::get().reads(1);

    if HasMigrationRun::<T>::get(&migration_name) {
        log::info!(
            target: "runtime",
            "Migration '{}' already run - skipping.",
            String::from_utf8_lossy(&migration_name)
        );
        return weight;
    }

    // (netuid, locked_rao) pairs taken from the historical snapshot (block #4_828_623).
    const SUBNET_LOCKED: &[(u16, u64)] = &[
        (  2,  976_893_069_056),
        (  3,2_569_362_397_490),
        (  4,1_928_551_593_932),
        (  5,1_712_540_082_588),
        (  6,1_495_929_556_770),
        (  7,1_011_702_451_936),
        (  8,  337_484_391_024),
        (  9,  381_240_180_320),
        ( 10,1_253_515_128_353),
        ( 11,1_453_924_672_132),
        ( 12,  100_000_000_000),
        ( 13,  100_000_000_000),
        ( 14,1_489_714_521_808),
        ( 15,1_784_089_225_496),
        ( 16,  889_176_219_484),
        ( 17,1_266_310_122_772),
        ( 18,  222_355_058_433),
        ( 19,  100_000_000_000),
        ( 20,  100_000_000_000),
        ( 21,  885_096_322_978),
        ( 22,  100_000_000_000),
        ( 23,  100_000_000_000),
        ( 24,5_146_073_854_481),
        ( 25,1_782_920_948_214),
        ( 26,  153_583_865_248),
        ( 27,  201_344_183_084),
        ( 28,  901_455_879_445),
        ( 29,  175_000_001_600),
        ( 30,1_419_730_660_074),
        ( 31,  319_410_100_502),
        ( 32,2_016_397_028_246),
        ( 33,1_626_477_274_174),
        ( 34,1_455_297_496_345),
        ( 35,1_191_275_979_639),
        ( 36,1_097_008_574_216),
        ( 37,  864_664_455_362),
        ( 38,1_001_936_494_076),
        ( 39,1_366_096_404_884),
        ( 40,  100_000_000_000),
        ( 41,  535_937_523_200),
        ( 42,1_215_698_423_344),
        ( 43,1_641_308_676_800),
        ( 44,1_514_636_189_434),
        ( 45,1_605_608_381_438),
        ( 46,1_095_943_027_350),
        ( 47,1_499_235_469_986),
        ( 48,1_308_073_720_362),
        ( 49,1_222_672_092_068),
        ( 50,2_628_355_421_561),
        ( 51,1_520_860_720_561),
        ( 52,1_794_457_248_725),
        ( 53,1_721_472_811_492),
        ( 54,2_048_900_691_868),
        ( 55,1_278_597_446_119),
        ( 56,2_016_045_544_480),
        ( 57,1_920_563_399_676),
        ( 58,2_246_525_691_504),
        ( 59,1_776_159_384_888),
        ( 60,2_173_138_865_414),
        ( 61,1_435_634_867_728),
        ( 62,2_061_282_563_888),
        ( 63,3_008_967_320_998),
        ( 64,2_099_236_359_026),
    ];

    let mut inserted: u32 = 0;
    let mut total_rao: u128 = 0;

    // ── 1) Re-insert the historical values ────────────────────────────────
    for &(netuid_u16, amount_rao_u64) in SUBNET_LOCKED.iter() {
        let key: NetUid = NetUid::from(netuid_u16);
        let amount: TaoCurrency = TaoCurrency::from(amount_rao_u64);

        SubnetLocked::<T>::insert(key, amount);

        inserted = inserted.saturating_add(1);
        total_rao = total_rao.saturating_add(amount_rao_u64 as u128);

        weight = weight.saturating_add(T::DbWeight::get().writes(1));
    }

    // ── 2) Mark migration done ────────────────────────────────────────────
    HasMigrationRun::<T>::insert(&migration_name, true);
    weight = weight.saturating_add(T::DbWeight::get().writes(1));

    log::info!(
        target: "runtime",
        "Migration '{}' completed - inserted {} SubnetLocked entries; total≈{} RAO.",
        String::from_utf8_lossy(&migration_name),
        inserted,
        total_rao
    );

    weight
}
