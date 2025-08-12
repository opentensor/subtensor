use sp_core::crypto::Ss58Codec;
use sp_core::ed25519;
use sp_core::sr25519;

pub struct KnownSs58 {
    pub sr25519: &'static str,
    pub ed25519: &'static str,
}

/// Grandpa keys (ed25519) use different crypto to Aura/Babe (sr25519).
///
/// It is sometimes required to perform this mapping, e.g. when initializing the pallet_session
/// `KeyOwner` storage during the Aura to Babe migration.
pub fn sr25519_to_ed25519(
    sr25519_pub: impl Into<sr25519::Public> + Ss58Codec,
) -> Option<ed25519::Public> {
    let res = known_ss58::ALL
        .iter()
        .find(|known| known.sr25519 == sr25519_pub.to_ss58check())
        .and_then(|known| ed25519::Public::from_ss58check(known.ed25519).ok());

    match res {
        Some(ed25519_pub) => {
            log::info!(
                "Successfully mapped SR25519 {} to ED25519 {}",
                sr25519_pub.to_ss58check(),
                ed25519_pub.to_ss58check()
            );
            Some(ed25519_pub)
        }
        None => {
            log::error!(
                "No valid SR25519 to ED25519 mapping exists for {}",
                sr25519_pub.to_ss58check()
            );
            None
        }
    }
}

pub mod known_ss58 {
    use super::*;

    pub const ALICE: KnownSs58 = KnownSs58 {
        sr25519: "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
        ed25519: "5FA9nQDVg267DEd8m1ZypXLBnvN7SFxYwV7ndqSYGiN9TTpu",
    };

    pub const TESTNET_VALI_1: KnownSs58 = KnownSs58 {
        sr25519: "5D5ABUyMsdmJdH7xrsz9vREq5eGXr5pXhHxix2dENQR62dEo",
        ed25519: "5H3qMjQjoeZxZ98jzDmoCwbz2sugd5fDN1wrr8Phf49zemKL",
    };

    pub const TESTNET_VALI_2: KnownSs58 = KnownSs58 {
        sr25519: "5GbRc5sNDdhcPAU9suV2g9P5zyK1hjAQ9JHeeadY1mb8kXoM",
        ed25519: "5GbkysfaCjK3cprKPhi3CUwaB5xWpBwcfrkzs6FmqHxej8HZ",
    };
    pub const TESTNET_VALI_3: KnownSs58 = KnownSs58 {
        sr25519: "5CoVWwBwXz2ndEChGcS46VfSTb3RMUZzZzAYdBKo263zDhEz",
        ed25519: "5HTLp4BvPp99iXtd8YTBZA1sMfzo8pd4mZzBJf7HYdCn2boU",
    };
    pub const TESTNET_VALI_4: KnownSs58 = KnownSs58 {
        sr25519: "5EekcbqupwbgWqF8hWGY4Pczsxp9sbarjDehqk7bdyLhDCwC",
        ed25519: "5GAemcU4Pzyfe8DwLwDFx3aWzyg3FuqYUCCw2h4sdDZhyFvE",
    };
    pub const TESTNET_VALI_5: KnownSs58 = KnownSs58 {
        sr25519: "5GgdEQyS5DZzUwKuyucEPEZLxFKGmasUFm1mqM3sx1MRC5RV",
        ed25519: "5EibpMomXmgekxcfs25SzFBpGWUsG9Lc8ALNjXN3TYH5Tube",
    };
    pub const TESTNET_VALI_6: KnownSs58 = KnownSs58 {
        sr25519: "5Ek5JLCGk2PuoT1fS23GXiWYUT98HVUBERFQBu5g57sNf44x",
        ed25519: "5Gyrc6b2mx1Af6zWJYHdx3gwgtXgZvD9YkcG9uTUPYry4V2a",
    };
    pub const FINNEY_VALI_1: KnownSs58 = KnownSs58 {
        sr25519: "5EJUcFbe74FDQwPsZDbRVpdDxVZQQxjoGZA9ayJqJTbcRrGf",
        ed25519: "5GRcfchgXZjkCfqgNvfjicjJw3vVGF4Ahqon2w8RfjXwyzy4",
    };
    pub const FINNEY_VALI_2: KnownSs58 = KnownSs58 {
        sr25519: "5H5oVSbQxDSw1TohAvLvp9CTAua6PN4yHme19UrG4c1ojS8J",
        ed25519: "5FAEYaHLZmLRX4XFs2SBHbLhkysbSPrcTp51w6sQNaYLa7Tu",
    };
    pub const FINNEY_VALI_3: KnownSs58 = KnownSs58 {
        sr25519: "5CfBazEwCAsmscGj1J9rhXess9ZXZ5qYcuZvFWii9sxT977v",
        ed25519: "5F6LgDAenzchE5tPmFHKGueYy1rj85oB2yxvm1xyKLVvk4gy",
    };
    pub const FINNEY_VALI_4: KnownSs58 = KnownSs58 {
        sr25519: "5HZDvVFWH3ifx1Sx8Uaaa7oiT6U4fAKrR3LKy9r1zFnptc1z",
        ed25519: "5GJY6A1X8KNvqHcf42Cpr5HZzG95FZVJkTHJvnHSBGgshEWn",
    };
    pub const FINNEY_VALI_5: KnownSs58 = KnownSs58 {
        sr25519: "5H3v2VfQmsAAgj63EDaB1ZWmruTHHkJ4kci5wkt6SwMi2VW1",
        ed25519: "5FXVk1gEsNweTB6AvS5jAWCivXQHTcyCWXs21wHvRU5UTZtb",
    };
    pub const FINNEY_VALI_6: KnownSs58 = KnownSs58 {
        sr25519: "5CPhKdvHmMqRmMUrpFnvLc6GUcduVwpNHsPPEhnYQ7QXjPdz",
        ed25519: "5GAzG6PhVvpeoZVkKupa2uZDrhwsUmk5fCHgwq95cN9s3Dvi",
    };
    pub const FINNEY_VALI_7: KnownSs58 = KnownSs58 {
        sr25519: "5DZTjVhqVjHyhXLhommE4jqY9w1hJEKNQWJ8p6QnUWghRYS1",
        ed25519: "5HmGN73kkcHaKNJrSPAxwiwAiiCkztDZ1AYi4gkpv6jaWaxi",
    };
    pub const FINNEY_VALI_8: KnownSs58 = KnownSs58 {
        sr25519: "5ETyBUhi3uVCzsk4gyTmtf41nheH7wALqQQxbUkmRPNqEMGS",
        ed25519: "5Cq63ca5KM5qScJYmQi7PvFPhJ6Cxr6yw6Xg9dLYoRYg33rN",
    };
    pub const FINNEY_VALI_9: KnownSs58 = KnownSs58 {
        sr25519: "5DUSt6KiZWxA3tsiFkv3xYSNuox6PCfhyvqqM9x7N5kuHV2S",
        ed25519: "5FF1kun4rb5B7C3tqh23XPVDDUJ3UchnaXxJeXu1i5n8KNHp",
    };
    pub const FINNEY_VALI_10: KnownSs58 = KnownSs58 {
        sr25519: "5GgsDz9yixsdHxFu52SN37f6TrUtU2RwmGJejbHVmN1ERXL4",
        ed25519: "5EZiep2gMyV2cz9x54TQDb1cuyFYYcwGRGZ7J19Ua4YSAWCZ",
    };
    pub const FINNEY_VALI_11: KnownSs58 = KnownSs58 {
        sr25519: "5HjhkCMa89QJbFULs8WPZBgVg8kMq5qdX1nx7CnQpZgoyKAN",
        ed25519: "5D5DL9sru2ep3AWoHvmEUbFLirVr7tJ6BxBWH5M8j3r9kUpe",
    };
    pub const FINNEY_VALI_12: KnownSs58 = KnownSs58 {
        sr25519: "5F257gHitacwDGvYm2Xm7dBE882auTU8wraG6w4T3r63wh9V",
        ed25519: "5CovRCaioWENKejfaeccDQY4vCF8kTGtZ5fwagSCeDGmiSyh",
    };
    pub const FINNEY_VALI_13: KnownSs58 = KnownSs58 {
        sr25519: "5CtGLbiHWs6XVgNi9nW7oqSP4D4JMot7yHYuFokidZzAP6ny",
        ed25519: "5DSxsR9aAiq33uSYXWt4zEibx6KT6xxtFGkT9S4GLaCavgDE",
    };
    pub const FINNEY_VALI_14: KnownSs58 = KnownSs58 {
        sr25519: "5DeVtxyiniPzoHo4iQiLhGfhED6RP3V73B5nGSYWr5Mgt82c",
        ed25519: "5HaWL2AvLZHwyPXofWFTEZ6jHVmUG8U9cFATggKZonN1xZjm",
    };
    pub const FINNEY_VALI_15: KnownSs58 = KnownSs58 {
        sr25519: "5GF4a6pQ8TQuPhdkKqugzrZSW7YnpQtB4ihouKGZsVMwoTn6",
        ed25519: "5DaEhFN8bWjvhDxavSWFBr962qoTAMB4b51QebdRZ75VA4h2",
    };
    pub const FINNEY_VALI_16: KnownSs58 = KnownSs58 {
        sr25519: "5DAC8Did2NgeVfZeNmEfZuU6t7UseJNf9J68XTvhLf5yCsBZ",
        ed25519: "5G27pyXx9ieSRCTuDoqPgTvpCynH6yhum9HiQQ1iMj3rAeaP",
    };
    pub const FINNEY_VALI_17: KnownSs58 = KnownSs58 {
        sr25519: "5FmxaYznqMqiorPHQgKoRQgEHN7ud4yKsJWr6FvXuS6FS6be",
        ed25519: "5Ch5XFMKETDiiPiuhUj9TumUtgsnVG1VzQRvBykP9bRdt4km",
    };
    pub const FINNEY_VALI_18: KnownSs58 = KnownSs58 {
        sr25519: "5GNAkfKYmFbVRAYm1tPr1yG6bHCapaY7WKRmzkEdendDXj1j",
        ed25519: "5EC6JjwnE11qaRnjKM85eevQFV1EoaKPPtcBRmTp1XsR7Kx3",
    };
    pub const FINNEY_VALI_19: KnownSs58 = KnownSs58 {
        sr25519: "5GYk3B38R9F2TEcWoqCLojqPwx6AA1TsD3EovoTgggyRdzki",
        ed25519: "5FjdhdAxujZVev6HYqQcTB6UBAKfKFKPoftgMLenoxbNWoe2",
    };
    pub const FINNEY_VALI_20: KnownSs58 = KnownSs58 {
        sr25519: "5D7fthS7zBDhwi2u2JYd74t7FpQuseDkUkTuaLZoenXNpXPK",
        ed25519: "5DhAKQ4MFg39mQAYzndzbznLGqSV4VMUJUyRXe8QPDqD5G1D",
    };
    pub const GREG_BAEDEKER_ALICE: KnownSs58 = KnownSs58 {
        sr25519: "5G6okTjVk3urYHR1MyJLXmF6AtSvZ9qiwzWhCatNBXV9JMJd",
        ed25519: "5DRcVKY6Ccs6MotaMdaDDX7zzjeQ3V4LvyRJHDz4wgJhcC1K",
    };
    pub const GREG_BAEDEKER_BOB: KnownSs58 = KnownSs58 {
        sr25519: "5Fy1xpe81NRpEBMc8h4wVwmcHBM3W7L6W16qLiaJpzcTVv7A",
        ed25519: "5F2BcLPjTQJWFgZZPCjR5YSb9CRUGqnKg7ZJFpgbaBrSXFrY",
    };
    pub const GREG_BAEDEKER_CHARLIE: KnownSs58 = KnownSs58 {
        sr25519: "5Fe2G9aGa7izEN1XvFNc3eMGPbJBgswGScPbBVuMcwngcow8",
        ed25519: "5Gck7HYTpoK1qY6nYTDxBPPU1maSfkMnZBVY7Q2RqERoGrzX",
    };

    pub const ALL: &[KnownSs58] = &[
        ALICE,
        TESTNET_VALI_1,
        TESTNET_VALI_2,
        TESTNET_VALI_3,
        TESTNET_VALI_4,
        TESTNET_VALI_5,
        TESTNET_VALI_6,
        FINNEY_VALI_1,
        FINNEY_VALI_2,
        FINNEY_VALI_3,
        FINNEY_VALI_4,
        FINNEY_VALI_5,
        FINNEY_VALI_6,
        FINNEY_VALI_7,
        FINNEY_VALI_8,
        FINNEY_VALI_9,
        FINNEY_VALI_10,
        FINNEY_VALI_11,
        FINNEY_VALI_12,
        FINNEY_VALI_13,
        FINNEY_VALI_14,
        FINNEY_VALI_15,
        FINNEY_VALI_16,
        FINNEY_VALI_17,
        FINNEY_VALI_18,
        FINNEY_VALI_19,
        FINNEY_VALI_20,
        GREG_BAEDEKER_ALICE,
        GREG_BAEDEKER_BOB,
        GREG_BAEDEKER_CHARLIE,
    ];
}
