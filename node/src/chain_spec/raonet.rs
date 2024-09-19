// Allowed since it's actually better to panic during chain setup when there is an error
#![allow(clippy::unwrap_used)]

use super::*;

pub fn raonet_config() -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

    // Give front-ends necessary data to present to users
    let mut properties = sc_service::Properties::new();
    properties.insert("tokenSymbol".into(), "TAO".into());
    properties.insert("tokenDecimals".into(), 9.into());
    properties.insert("ss58Format".into(), 42.into());

    Ok(ChainSpec::builder(
        wasm_binary,
        Extensions {
            bad_blocks: Some(HashSet::from_iter(vec![
                // Example bad block
                H256::from_str(
                    "0xc174d485de4bc3813ac249fe078af605c74ff91d07b0a396cf75fa04f81fa312",
                )
                .unwrap(),
            ])),
            ..Default::default()
        },
    )
    .with_name("Bittensor")
    .with_protocol_id("bittensor")
    .with_id("bittensor")
    .with_chain_type(ChainType::Development)
    .with_genesis_config_patch(raonet_genesis(
        // Initial PoA authorities (Validators)
        // aura | grandpa
        vec![
            // Keys for debug
            authority_keys_from_seed("Alice"),
            authority_keys_from_seed("Bob"),
        ],
        // Pre-funded accounts
        true,
    ))
    .with_properties(properties)
    .build())
}

fn raonet_genesis(
    initial_authorities: Vec<(AuraId, GrandpaId)>,
    _enable_println: bool,
) -> serde_json::Value {
    let mut balances = vec![
        (
            <AccountId32 as Ss58Codec>::from_ss58check("5FkLT4UZmXn3mE3rpLyBjz94tkT6KFwME3L11S12Pdkmera1").unwrap(), // Loayei  [τ, τ]
            100_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check("5G4FseBtaQd8sqeC98ZEL7xgtF2GSdueMXwUs8vsBENs4Ysn").unwrap(), // Sai [τ, τ]
            100_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check("5CPHQqL9E4GJLxeo7r61fqiQWE2xBgcsdKrLPyZpCKRHFWt8").unwrap(), // Sarokhan - manifold labs [τ, τ]
            100_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check("5FjYhLtAzbfMqKbXp8ek2NKQKF9U5qfoDhjUQhjt4S7bhARH").unwrap(), // RonX [τ, τ] :: BittexSN
            100_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check("5GppQD1aV13DdsafHyi4KQVzhx9Zeeek1ukz2WvBfMTiTWH3").unwrap(), // Mars [τ, τ]
            100_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check("5DD7xG5TJ48W7j7DUbHG5whrAkwUrbPe1NahiFbKqzytfhVt").unwrap(), // Spiigot [τ, τ]
            100_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check("5EPYa3Krjukbft2woziQ72dU8G9mbf4q7Kq378GFxXaiKDKq").unwrap(), // !   carro [τ, ∆] - manifold labs
            100_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check("5EeBuJRFUMS3CgisL1FT2w4AdqSQVGWRGNsTdR5YrFd189PT").unwrap(), // Greg Zaitsev [τ, τ]
            100_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check("5Fuck5PH4Ug6iQpXhhNL2LvyUW4rVdi2ZRbZS6x8XV1SgNZ6").unwrap(), // josh | Manifold Labs [τ, ∆]
            100_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check("5HpE7oRTSbKZF1ZP3NEcDr4MnM1UpTmbGD61B7RNH8VdFQZd").unwrap(), // arvee [τ, τ]
            100_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check("5He3NLYn5iUwNSCfDtc43MoeydZ4VPfV5iGp14wxcRMmpYkd").unwrap(), // Pardus [τ, τ]
            100_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check("5Gdau2r445N3bpBJcncsGLwYfZch9A5tUZ7BYi1iJ245oFrp").unwrap(), // biττenleo (SocialTensor) [τ, ψ]
            100_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check("5FW6djdCybRYo8XFT5sTQowgq74ovxseJsmcDGFu6zg12Erv").unwrap(), // demon [τ, τ]
            100_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check("5CXgmrp6Ts5igz9uxSdQQy9ERUVaJFtswzaSBUXhb3Ci7drK").unwrap(), // specialK [τ, τ]
            100_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check("5F7PwP2ysRFzhBoSV3qrKwvJYJm8V8dCvXJUAAePJjX1GUjy").unwrap(), // CryptoMinedMind | [η, ז]
            100_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check("5EvAf5fDX3jNkCg158ctDnHAb7JX34eXTtHRGdreQebyFhHz").unwrap(), // 0xBomb[τ, τ]
            100_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check("5CJ2ZtY8c5UfSTTDkJMDa3HU7TvxWxHUjtWyThfvj18uFckV").unwrap(), // Cow Boy [τ, τ]
            100_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check("5DktX2XnHZUSWmBk9nPmvE3XwH3e8WiUCNszXrVzKBtSNLC8").unwrap(), // yikesawjeez [τ, τ]
            100_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check("5CtTNTV9Pk7Uy8jgESwQAPUCqVMDi1kSNBEFUHsDXbEQStnq").unwrap(), // Sangar[τ, τ]
            100_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check("5FTjgzjxd9GDqzptYSy1ERME25vrmhHkngL8vxXnDoPfCuPk").unwrap(), // watermelon[τ, τ]
            100_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check("5GdhNRf6nqhybVTuzSCm4huRrh7W2PzHo1craMyrPttZQzKK").unwrap(), // HcL-CO | [η, ז]
            100_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check("5DPKKVbMyjHPtB3jKne89JqKa12k5UTHPH73ugSiusMobQsT").unwrap(), // cosmic [ τ, ο ]
            100_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check("5EgyK4xFAokjdYGMrWn1ikXrBBGRZyq5eoF5zMP35rGHs5hL").unwrap(), // hoτes [τ, τ]
            100_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check("5FyJK1gMCShmfGcnh12VG1cdHULaXoMx4qk5mWLpXhoEDjm4").unwrap(), // τaogon [τ, τ]
            100_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check("5Ca6zA5dH6h4DUnGt1ixvY5V8mWvKM3ATvuprCUjgEFqDay9").unwrap(), // Maciej Kula [𝞃, 𝞃]
            100_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check("5Ge3DKM155dLpvEFjFcgJ6c3RVcaQAL1EBWp8G7nJSBn8Aaq").unwrap(), // James - Manifold [τ,τ]
            100_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check("5EZSWFuFQB6Avs9y7NCVTkGRcUi5eBomrUfS1SmmR5H1paCU").unwrap(), // Don [τ, τ]
            100_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check("5H4D6hMo1ozTPJ6HLdLv591sThbY37FNuBas8mRBsGWyyVDC").unwrap(), // Thanos | Datura [τ, τ)
            100_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check("5FQm9kDYyG5UcpMqNS7oeYqGtc59WQMWgdU1dpwQBZoAZ7xE").unwrap(), // !_τ$mickey_[τ, τ]
            100_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check("5GYnsEFDBoB3uaPu4LyeRrgvCsLLzsUvV5kiKGNF5Kbf72MV").unwrap(), // Ch3RNØbØG | η ז
            100_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check("5G3qVaXzKMPDm5AJ3dpzbpUC27kpccBvDwzSWXrq8M6qMmbC").unwrap(), // Ashley | SN39 [τ, τ]
            100_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check("5HgxtsYn7euSLgZZVRV7BzL5voDpDQA4GojtyPKAatKGLZ7C").unwrap(), // benton [τ, τ]
            100_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check("5EUQ6C94Nyxa7WDwNKzMnwa3Jgqv8U2ZuergAF5oVWys3RFN").unwrap(), // boogie_man [τ, τ]
            100_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check("5Fba44AeMsz3jqa1zaenvbtwE5MKBEwiNbZ41spc22sQobSt").unwrap(), // Paradigma [τ, τ]
            100_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check("5DXULyK6jUGDdRkfTLpZunkyZvzrKH8Wtyw39eosGs2nBU1R").unwrap(), // Anton.noderunner [τ, τ]
            100_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check("5EHH1MKy8vrMaJ6JzbWbdXzihPMqk25LueSaEgMn8bDgNc4t").unwrap(), // Colin [τ, τ]
            100_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check("5EEox4pimQK1aDNA3kPUGry1VWognkGY7WxSMrMXQmTKpxS7").unwrap(), // palmeidawrt [τ, τ]
            100_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check("5GKKGQfbtzELVuPgpoofwDFAzf4J92u6tsdN9NVCinsuzpdu").unwrap(), // Kenobi | BitMind
            100_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check("5GsicYaRc3yUxQSgHavMnSuriwCPD92NERDNkvsmfnnfqXyi").unwrap(), // Volodymyr Truba[τ, τ]Macrocosmos
            100_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check("5DeXR1sgZF1jE2vG7Jx8edsCr12LF4PtmScR1zSPMSzRrWCK").unwrap(), // taozen [τ, τ]
            100_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check("5DvjBoT5obrba9bemsiwgvCJd9gdx37xcS4w3iC9Ra5xP9JN").unwrap(), // Fel [τ, τ]
            100_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check("5E6tCDpY9jg25uCLTWHpDoK23UHpanes1PcTiB6dkNcB2FHb").unwrap(), // pylegend [τ, τ]
            100_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check("5FsC7s4txtDvneQUYoY2HFzGQEU9e5LbUGnEE5Jz2aLxKEbc").unwrap(), // Kao [τ, τ]
            100_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check("5DevrqGMa6ahKyRqk57qb3DTLZt29VZ8y6r14LgRRJgp5AcG").unwrap(), // Ciel [τ, τ]
            100_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check("5CPN27vJ6x6f16TESnMi6qPAunDDpze11ThCurRLXS3pDgss").unwrap(), // Alder [τ, τ]
            100_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check("5G1iCdwUBjnXxGfJYzho1dToWTCkYyBF6Vq5sAJP7ftHKE1b").unwrap(), // Jordan | Taoshi [τ, θ]
            100_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check("5CarroqMMihUTGHcy46iAaaMYwsSZvmjmH69za4eTmYh2CJ4").unwrap(), // μ | coldint | SN29 [τ, ה] 
            100_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check("5DsnfJs84niPnWuuXxwqGdb6smcx67fjR9aK3HnSxUUprHgV").unwrap(), // !SuNiX [τ, τ]!
            100_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check("5ES2Ua8d4o5NChiFiqDrPJbaisvHnwWQbnmnrZKcEZcwcFCw").unwrap(), // Dubs  :: τaosτaτs  [τ, τ]
            100_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check("5G3fBbCmWcMygdy3oK3HyiDNSCZim3TvLguRxHfxg4U7TwQg").unwrap(), // UltraShiny [τ, τ]
            100_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check("5GYNpL3UuQej8bYQvXWSw5dQqmEmgJ4oJ51G9uvEeq42b2nS").unwrap(), // Vlad [τ, τ)
            100_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check("5HNp2h9C4A4BWP42b1eKmjszUC335WaLCJdXkSSA8PYdUBMx").unwrap(), // X_Blazer [τ, ⛏]
            100_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check("5CLUM696NNNk8XtquxJCAvJ6S3FGDofZnZ3z8BUJKBcQYDCS").unwrap(), // EchoStorm [τ, τ]
            100_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check("5G3aQtA1ckoVCSgEWTbK6TFvMiQD6j8k4KoDA4iXcvVWH859").unwrap(), // 𝒜𝑒𝓉𝒽 [τ, τ]
            100_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check("5Dy7aZFuVqisdDAvKGyQq1JdNjd4XjZ1kCbutz16k6asLqnJ").unwrap(), // Buddy [τ, τ]
            100_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check("5C51vbY3SvPPiLrxTviJ1kZBmjjMz5VzR2anbiHFtb2JuqQC").unwrap(), // [ τ ] passion [ τ ]
            100_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check("5EABa2cSSgsSajF43uhrsxB3nDnGt5cz7aqGD7bRzBBnDRN2").unwrap(), // Rapido [τ, ⛏] | MUV
            100_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check("5Des8SvYMitS1x9wp37Rdhqyv6nbk8iwfT7uXijCBv3E4M14").unwrap(), // hodler0 [τ , τ]
            100_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check("5HgQA9WkXr3Hey1GS23BCxoWUpzia8NsKx18fmo4tDYEGFYp").unwrap(), // Lucian [τ, נ] SN41
            100_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check("5Hgv9NxxFepLC1RZGGD2iosAtTh8NKwVrGUBEoT4TYb9zvMU").unwrap(), // isebarn | Datura [τ, τ)
            100_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check("5DEv67Dd7EXZEcpfSsPrSUVmHgBQmtrybTNPNdDov48CDCVU").unwrap(), // Professor [τ, τ] 
            100_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check("5GRA4pktvTrDoVuMn8Xo1JCQC8CCJkVHtL1q2QTJ9gVhpcm3").unwrap(), // Platwsup  [τ,τ]
            100_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check("5Co51B3Po5uauyuby9WYHqTcEysVeJwyBTTuzPx98XUaBFso").unwrap(), // Waris | [τ, τ]
            100_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check("5D8Kiee7DBcSZGMKVYAadVxeJnh1MEaQaeSTF8GEu2ZJDa9w").unwrap(), // Freebird | Datura [τ, τ)
            100_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check("5GvT34BeeL8Gjtw3kTjytYg3qE3mYJh2SPBDuKW4qgJKNTHa").unwrap(), // Cipher [τ , ⛏]
            100_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check("5CiCXcx8FEhWZHCLLJm8rtvsaubinZUnjcq8SaSeAekEkDAZ").unwrap(), // ADally [𝞃, 𝞃]
            100_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check("5GTZtWVdLT8CQB2nidJszn6THFMDcnNf9udta4mrNJnDpkuX").unwrap(), // Roman | Corτex [τ, τ]
            100_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check("5FBitd8W9mr16bME3jPYxqZscKNFrQM5wZkPBXebmmhW3ZiX").unwrap(), // shahjab [τ, τ]
            100_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check("5Hj53nAGMgtpLHCT7x7VrHDvD9oWpggDyp5rCWJEFE27aHXd").unwrap(), // 0xUnicorn [ τ, τ ]
            100_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check("5GWYVx2HB3Z5Y1bbvAnsbQH3Wm324LHwTrhjSuL24fQarcVP").unwrap(), // MrNiche [τ, ψ]
            100_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check("5Gumfmxcd9BkLVC5dE8ZqLyiS7yk4pE12C8EsR4684ZEfHBF").unwrap(), // mrbovo
            100_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check("5DcsPiqn4Y3YY4AwrPvgtoqm1mznYvxmJPKQfL2tA25CZSvj").unwrap(), // Von [τ τ]
            100_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check("5ELAdQ1m24q4t13UYmSVuWj887iZSxF51hWztZjKNrp5t1pn").unwrap(), // Ailith (τ_τ)
            100_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check("5HdctsgE4uoqvjywLMWyvaaR9b7JpXCAt8BuhcbkoeU8g1WW").unwrap(), // Echo | cortex.foundation [τ, с]
            100_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check("5FkFULMPhLBJFh2yDzeFSa6vtMm2a3ZK8B2RhsHSpszkVGPK").unwrap(), // CreativeEssence [τ, τ]
            100_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check("5DS3HKzvCeqeoHfStTGS7PnHVoCAACsiuisx5mfvQjswSZcH").unwrap(), // Rovertter [τ, τ]
            100_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check("5DiKdaWbhZieXLvYv72RYLUVT88ZG197iiYeDKS7Ss72k259").unwrap(), // RavenClaw [τ, τ]
            100_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check("5Ebcyb3gfWZ2JMkBMjZuDNr1MaeoW5hJVH92M9dwn4HVBN6t").unwrap(), // ulτrashiny [τ, ⛏]
            100_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check("5Eh77jcKp6RoggfecBCEB5KEsPiX8XaW2kwgH6KPgXyBQm6J").unwrap(), // BτGuy | τ, ψ/ך
            100_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check("5DUFdkP4rJrkXq9pfrWMHQS8zgiwXBZRgw2MMEAnBot59Taz").unwrap(), // τribbiτ [τ, τ]
            100_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check("5CML2G4gwAv3WLctozwivVTWW2Q1HHbszxWLEPM2o2e4KFeG").unwrap(), // prop_physics [τ, τ]
            100_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check("5HgbfjJLUmoBJ78o1dc6ubZJ1nGA2kXzVrgTFhrSz7yUbftE").unwrap(), // Rhef
            100_000_000_000u128,
        ),
    ];

    // Check if the environment variable is set
    if let Ok(bt_wallet) = env::var("BT_DEFAULT_TOKEN_WALLET") {
        if let Ok(decoded_wallet) = Ss58Codec::from_ss58check(&bt_wallet) {
            balances.push((decoded_wallet, 1_000_000_000_000_000u128));
        } else {
            eprintln!("Invalid format for BT_DEFAULT_TOKEN_WALLET.");
        }
    }

    let trimvirate_members: Vec<AccountId> = bounded_vec![
        <AccountId32 as Ss58Codec>::from_ss58check("5FRo4vab84LM3aiK4DijnVawGDKagLGLzfn95j9tjDaHja8Z").unwrap(),
        <AccountId32 as Ss58Codec>::from_ss58check("5HjEUemUaXSkxPcxGYiLykHmi5VfXBh5NCeNXYMbj9akYHbn").unwrap(),
        <AccountId32 as Ss58Codec>::from_ss58check("5DUFdkP4rJrkXq9pfrWMHQS8zgiwXBZRgw2MMEAnBot59Taz").unwrap(),
    ];

    let senate_members: Vec<AccountId> = bounded_vec![
        get_account_id_from_seed::<sr25519::Public>("Dave"),
        get_account_id_from_seed::<sr25519::Public>("Eve"),
        get_account_id_from_seed::<sr25519::Public>("Ferdie"),
    ];

    serde_json::json!({
        "balances": { "balances": balances },
        "aura": {
            "authorities": initial_authorities.iter().map(|x| (x.0.clone())).collect::<Vec<_>>()
        },
        "grandpa": {
            "authorities": initial_authorities
                .iter()
                .map(|x| (x.1.clone(), 1))
                .collect::<Vec<_>>()
        },
        "sudo": {
            "key": <AccountId32 as Ss58Codec>::from_ss58check("5FRo4vab84LM3aiK4DijnVawGDKagLGLzfn95j9tjDaHja8Z").unwrap()
        },
        "triumvirateMembers": {
            "members": trimvirate_members
        },
        "senateMembers": {
            "members": senate_members,
        },
    })
}
