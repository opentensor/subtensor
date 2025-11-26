import { blake2AsU8a } from '@polkadot/util-crypto';
export function sign(_registry, signerPair, u8a, options) {
    const encoded = u8a.length > 256
        ? blake2AsU8a(u8a)
        : u8a;
    return signerPair.sign(encoded, options);
}
export function signGeneral(registry, u8a) {
    const encoded = registry.hash(u8a);
    return encoded;
}
