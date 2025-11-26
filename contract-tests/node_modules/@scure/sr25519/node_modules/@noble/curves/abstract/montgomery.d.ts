import { type CurveLengths } from './curve.ts';
export type MontgomeryOpts = {
    P: bigint;
    type: 'x25519' | 'x448';
    adjustScalarBytes: (bytes: Uint8Array) => Uint8Array;
    powPminus2: (x: bigint) => bigint;
    randomBytes?: (bytesLength?: number) => Uint8Array;
};
export type MontgomeryECDH = {
    scalarMult: (scalar: Uint8Array, u: Uint8Array) => Uint8Array;
    scalarMultBase: (scalar: Uint8Array) => Uint8Array;
    getSharedSecret: (secretKeyA: Uint8Array, publicKeyB: Uint8Array) => Uint8Array;
    getPublicKey: (secretKey: Uint8Array) => Uint8Array;
    utils: {
        randomSecretKey: () => Uint8Array;
    };
    GuBytes: Uint8Array;
    lengths: CurveLengths;
    keygen: (seed?: Uint8Array) => {
        secretKey: Uint8Array;
        publicKey: Uint8Array;
    };
};
export declare function montgomery(curveDef: MontgomeryOpts): MontgomeryECDH;
//# sourceMappingURL=montgomery.d.ts.map