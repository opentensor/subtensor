/**
 * bn254, previously known as alt_bn_128, when it had 128-bit security.

Barbulescu-Duquesne 2017 shown it's weaker: just about 100 bits,
so the naming has been adjusted to its prime bit count:
https://hal.science/hal-01534101/file/main.pdf.
Compatible with EIP-196 and EIP-197.

There are huge compatibility issues in the ecosystem:

1. Different libraries call it in different ways: "bn254", "bn256", "alt_bn128", "bn128".
2. libff has bn128, but it's a different curve with different G2:
   https://github.com/scipr-lab/libff/blob/a44f482e18b8ac04d034c193bd9d7df7817ad73f/libff/algebra/curves/bn128/bn128_init.cpp#L166-L169
3. halo2curves bn256 is also incompatible and returns different outputs

We don't implement Point methods toHex / toBytes.
To work around this limitation, has to initialize points on their own from BigInts.
Reason it's not implemented is because [there is no standard](https://github.com/privacy-scaling-explorations/halo2curves/issues/109).
Points of divergence:

- Endianness: LE vs BE (byte-swapped)
- Flags as first hex bits (similar to BLS) vs no-flags
- Imaginary part last in G2 vs first (c0, c1 vs c1, c0)

The goal of our implementation is to support "Ethereum" variant of the curve,
because it at least has specs:

- EIP196 (https://eips.ethereum.org/EIPS/eip-196) describes bn254 ECADD and ECMUL opcodes for EVM
- EIP197 (https://eips.ethereum.org/EIPS/eip-197) describes bn254 pairings
- It's hard: EIPs don't have proper tests. EIP-197 returns boolean output instead of Fp12
- The existing implementations are bad. Some are deprecated:
    - https://github.com/paritytech/bn (old version)
    - https://github.com/ewasm/ethereum-bn128.rs (uses paritytech/bn)
    - https://github.com/zcash-hackworks/bn
    - https://github.com/arkworks-rs/curves/blob/master/bn254/src/lib.rs
- Python implementations use different towers and produce different Fp12 outputs:
    - https://github.com/ethereum/py_pairing
    - https://github.com/ethereum/py_ecc/tree/main/py_ecc/bn128
- Points are encoded differently in different implementations

### Params
Seed (X): 4965661367192848881
Fr: (36x⁴+36x³+18x²+6x+1)
Fp: (36x⁴+36x³+24x²+6x+1)
(E  / Fp ): Y² = X³+3
(Et / Fp²): Y² = X³+3/(u+9) (D-type twist)
Ate loop size: 6x+2

### Towers
- Fp²[u] = Fp/u²+1
- Fp⁶[v] = Fp²/v³-9-u
- Fp¹²[w] = Fp⁶/w²-v

 * @module
 */
/*! noble-curves - MIT License (c) 2022 Paul Miller (paulmillr.com) */
import { type BlsCurvePair, type BlsPostPrecomputeFn } from './abstract/bls.ts';
import { type IField } from './abstract/modular.ts';
export declare const bn254_Fr: IField<bigint>;
export declare const _postPrecompute: BlsPostPrecomputeFn;
/**
 * bn254 (a.k.a. alt_bn128) pairing-friendly curve.
 * Contains G1 / G2 operations and pairings.
 */
export declare const bn254: BlsCurvePair;
//# sourceMappingURL=bn254.d.ts.map