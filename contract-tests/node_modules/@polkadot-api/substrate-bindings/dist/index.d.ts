import * as scale_ts from 'scale-ts';
import { Codec, Encoder, Decoder, StringRecord, CodecType, EncoderType, DecoderType, ResultPayload } from 'scale-ts';
export { Bytes, Codec, CodecType, Decoder, DecoderType, Encoder, EncoderType, ResultPayload, StringRecord, _void, bool, compact, createCodec, createDecoder, enhanceCodec, enhanceDecoder, enhanceEncoder, i128, i16, i256, i32, i64, i8, str, u128, u16, u256, u32, u64, u8 } from 'scale-ts';

type SS58String = string & {
    __SS58String?: unknown;
};
type SS58AddressInfo = {
    isValid: false;
} | {
    isValid: true;
    ss58Format: number;
    publicKey: Uint8Array;
};
declare const getSs58AddressInfo: (address: SS58String) => SS58AddressInfo;
declare const fromBufferToBase58: (ss58Format: number) => (publicKey: Uint8Array) => SS58String;

declare const AccountId: (ss58Format?: number, nBytes?: 32 | 33) => scale_ts.Codec<SS58String>;

type HexString = string & {
    __hexString?: unknown;
};
declare const Hex: {
    (nBytes?: number): Codec<HexString>;
    enc: (nBytes?: number) => Encoder<HexString>;
    dec: (nBytes?: number) => Decoder<HexString>;
};

declare const getMultisigAccountId: ({ threshold, signatories, }: {
    threshold: number;
    signatories: Uint8Array[];
}) => Uint8Array<ArrayBufferLike>;
declare const sortMultisigSignatories: (signatories: Uint8Array[]) => Uint8Array<ArrayBufferLike>[];

declare class Binary {
    #private;
    constructor(data: Uint8Array, opaque?: boolean);
    asText: () => string;
    asHex: () => `0x${string}`;
    asOpaqueHex: () => `0x${string}`;
    asBytes: () => Uint8Array<ArrayBufferLike>;
    asOpaqueBytes: () => Uint8Array<ArrayBufferLike>;
    static fromText(input: string): Binary;
    static fromHex(input: HexString): Binary;
    static fromOpaqueHex(input: HexString): Binary;
    static fromBytes(input: Uint8Array): Binary;
    static fromOpaqueBytes(input: Uint8Array): Binary;
}
declare class FixedSizeBinary<_L extends number> extends Binary {
    constructor(data: Uint8Array);
    static fromArray<L extends number, I extends Array<number> & {
        length: L;
    }>(input: I): FixedSizeBinary<L>;
    static fromAccountId32<L extends number>(input: L extends 32 ? SS58String : never): FixedSizeBinary<L>;
}
declare const Bin: {
    (nBytes?: number): Codec<Binary>;
    enc: (nBytes?: number) => Encoder<Binary>;
    dec: (nBytes?: number) => Decoder<Binary>;
};

interface BitSequence {
    bitsLen: number;
    bytes: Uint8Array;
}
declare const bitSequence: scale_ts.Codec<BitSequence>;

declare const char: scale_ts.Codec<string>;

declare const compactNumber: scale_ts.Codec<number>;
declare const compactBn: scale_ts.Codec<bigint>;

declare const fixedStr: (nBytes: number) => scale_ts.Codec<string>;

declare const selfEncoder: <T>(value: () => Encoder<T>) => Encoder<T>;
declare const selfDecoder: <T>(value: () => Decoder<T>) => Decoder<T>;
declare const Self: <T>(value: () => Codec<T>) => Codec<T>;

type EnumVariant<T extends {
    type: string;
    value?: any;
}, K extends T["type"]> = T & {
    type: K;
};
type ExtractEnumValue<T extends {
    type: string;
    value?: any;
}, K extends string> = EnumVariant<T, K>["value"];
type ValueArg<V> = undefined extends V ? [value?: V] : [value: V];
interface Discriminant {
    is<T extends {
        type: string;
        value: any;
    }, K extends T["type"]>(value: T, type: K): value is T & {
        type: K;
    };
    as<T extends {
        type: string;
        value: any;
    }, K extends T["type"]>(value: T, type: K): ExtractEnumValue<T, K>;
}
interface EnumFn extends Discriminant {
    <T extends {
        type: string;
        value: any;
    }, K extends T["type"]>(type: K, ...[value]: ValueArg<ExtractEnumValue<T, K>>): EnumVariant<T, K>;
}
type Enum<T extends {}> = {
    [K in keyof T & string]: {
        type: K;
        value: T[K];
    };
}[keyof T & string];
declare const Enum: EnumFn;
type GetEnum<T extends Enum<any>> = {
    [K in T["type"]]: (...args: ExtractEnumValue<T, K> extends undefined ? [] : [value: ExtractEnumValue<T, K>]) => EnumVariant<T, K>;
};
declare const _Enum: {};

type Tuple$1<T, N extends number> = readonly [T, ...T[]] & {
    length: N;
};
type Push<T extends any[], V> = [...T, V];
type UnionToIntersection<U> = (U extends any ? (k: U) => void : never) extends (k: infer I) => void ? I : never;
type LastOf<T> = UnionToIntersection<T extends any ? () => T : never> extends () => infer R ? R : never;
type TuplifyUnion<T, L = LastOf<T>, N = [T] extends [never] ? true : false> = true extends N ? [] : Push<TuplifyUnion<Exclude<T, L>>, L>;
type RestrictedLenTuple<T, O extends StringRecord<any>> = Tuple$1<T, TuplifyUnion<keyof O> extends Tuple$1<any, infer V> ? V : 0>;
declare const Variant: {
    <O extends StringRecord<Codec<any>>>(inner: O, indexes?: RestrictedLenTuple<number, O> | undefined): Codec<Enum<{
        [K in keyof O]: CodecType<O[K]>;
    }>> & {
        inner: O;
    };
    enc: <O_1 extends StringRecord<Encoder<any>>>(inner: O_1, x?: RestrictedLenTuple<number, O_1> | undefined) => Encoder<Enum<{
        [K in keyof O_1]: EncoderType<O_1[K]>;
    }>> & {
        inner: O_1;
    };
    dec: <O_2 extends StringRecord<Decoder<any>>>(inner: O_2, x?: RestrictedLenTuple<number, O_2> | undefined) => Decoder<Enum<{
        [K in keyof O_2]: DecoderType<O_2[K]>;
    }>> & {
        inner: O_2;
    };
};
declare const ScaleEnum: {
    <O extends StringRecord<Codec<any>>>(inner: O, indexes?: RestrictedLenTuple<number, O> | undefined): Codec<{
        [K in keyof O]: {
            tag: K;
            value: CodecType<O[K]>;
        };
    }[keyof O]> & {
        inner: O;
    };
    enc: <O_1 extends StringRecord<Encoder<any>>>(inner: O_1, x?: RestrictedLenTuple<number, O_1> | undefined) => Encoder<{
        [K_1 in keyof O_1]: {
            tag: K_1;
            value: EncoderType<O_1[K_1]>;
        };
    }[keyof O_1]> & {
        inner: O_1;
    };
    dec: <O_2 extends StringRecord<Decoder<any>>>(inner: O_2, x?: RestrictedLenTuple<number, O_2> | undefined) => Decoder<{
        [K_2 in keyof O_2]: {
            tag: K_2;
            value: DecoderType<O_2[K_2]>;
        };
    }[keyof O_2]> & {
        inner: O_2;
    };
};

declare const ethAccount: scale_ts.Codec<string>;

declare const Struct: {
    <A extends StringRecord<Codec<any>>>(codecs: A): Codec<{
        [K in keyof A]: CodecType<A[K]>;
    }> & {
        inner: A;
    };
    enc: <A_1 extends StringRecord<Encoder<any>>>(encoders: A_1) => Encoder<{
        [K_1 in keyof A_1]: EncoderType<A_1[K_1]>;
    }> & {
        inner: A_1;
    };
    dec: <A_2 extends StringRecord<Decoder<any>>>(decoders: A_2) => Decoder<{
        [K_2 in keyof A_2]: DecoderType<A_2[K_2]>;
    }> & {
        inner: A_2;
    };
};
declare const Tuple: {
    <A extends Codec<any>[]>(...inner: A): Codec<{
        [K in keyof A]: A[K] extends Codec<infer D> ? D : unknown;
    }> & {
        inner: A;
    };
    enc: <A_1 extends Encoder<any>[]>(...encoders: A_1) => Encoder<{
        [K_1 in keyof A_1]: A_1[K_1] extends Encoder<infer D_1> ? D_1 : unknown;
    }> & {
        inner: A_1;
    };
    dec: <A_2 extends Decoder<any>[]>(...decoders: A_2) => Decoder<{
        [K_2 in keyof A_2]: A_2[K_2] extends Decoder<infer D_2> ? D_2 : unknown;
    }> & {
        inner: A_2;
    };
};
declare const Vector: {
    <T>(inner: Codec<T>, size?: number | undefined): Codec<T[]> & {
        inner: Codec<T>;
    };
    enc: <T_1>(inner: Encoder<T_1>, size?: number | undefined) => Encoder<T_1[]> & {
        inner: Encoder<T_1>;
    };
    dec: <T_2>(getter: Decoder<T_2>, size?: number | undefined) => Decoder<T_2[]> & {
        inner: Decoder<T_2>;
    };
};
declare const Result: {
    <OK, KO>(okCodec: Codec<OK>, koCodec: Codec<KO>): Codec<ResultPayload<OK, KO>> & {
        inner: {
            ok: Codec<OK>;
            ko: Codec<KO>;
        };
    };
    dec: <OK_1, KO_1>(okDecoder: Decoder<OK_1>, koDecoder: Decoder<KO_1>) => Decoder<ResultPayload<OK_1, KO_1>> & {
        inner: {
            ok: Decoder<OK_1>;
            ko: Decoder<KO_1>;
        };
    };
    enc: <OK_2, KO_2>(okEncoder: Encoder<OK_2>, koEncoder: Encoder<KO_2>) => Encoder<ResultPayload<OK_2, KO_2>> & {
        inner: {
            ok: Encoder<OK_2>;
            ko: Encoder<KO_2>;
        };
    };
};
declare const Option: {
    <T>(inner: Codec<T>): Codec<T | undefined> & {
        inner: Codec<T>;
    };
    enc: <T_1>(inner: Encoder<T_1>) => Encoder<T_1 | undefined> & {
        inner: Encoder<T_1>;
    };
    dec: <T_2>(inner: Decoder<T_2>) => Decoder<T_2 | undefined> & {
        inner: Decoder<T_2>;
    };
};

type BitSeq = Array<0 | 1>;
declare const BitSeq: {
    (isLsb?: boolean): Codec<BitSeq>;
    enc: (isLsb?: boolean) => Encoder<BitSeq>;
    dec: (isLsb?: boolean) => Decoder<BitSeq>;
};

declare const blockHeader: [scale_ts.Encoder<{
    parentHash: HexString;
    number: number;
    stateRoot: HexString;
    extrinsicRoot: HexString;
    digests: Enum<{
        other: Uint8Array<ArrayBufferLike>;
        consensus: {
            engine: string;
            payload: HexString;
        };
        seal: {
            engine: string;
            payload: HexString;
        };
        preRuntime: {
            engine: string;
            payload: HexString;
        };
        runtimeUpdated: undefined;
    }>[];
}>, scale_ts.Decoder<{
    parentHash: HexString;
    number: number;
    stateRoot: HexString;
    extrinsicRoot: HexString;
    digests: Enum<{
        other: Uint8Array<ArrayBufferLike>;
        consensus: {
            engine: string;
            payload: HexString;
        };
        seal: {
            engine: string;
            payload: HexString;
        };
        preRuntime: {
            engine: string;
            payload: HexString;
        };
        runtimeUpdated: undefined;
    }>[];
}>] & {
    enc: scale_ts.Encoder<{
        parentHash: HexString;
        number: number;
        stateRoot: HexString;
        extrinsicRoot: HexString;
        digests: Enum<{
            other: Uint8Array<ArrayBufferLike>;
            consensus: {
                engine: string;
                payload: HexString;
            };
            seal: {
                engine: string;
                payload: HexString;
            };
            preRuntime: {
                engine: string;
                payload: HexString;
            };
            runtimeUpdated: undefined;
        }>[];
    }>;
    dec: scale_ts.Decoder<{
        parentHash: HexString;
        number: number;
        stateRoot: HexString;
        extrinsicRoot: HexString;
        digests: Enum<{
            other: Uint8Array<ArrayBufferLike>;
            consensus: {
                engine: string;
                payload: HexString;
            };
            seal: {
                engine: string;
                payload: HexString;
            };
            preRuntime: {
                engine: string;
                payload: HexString;
            };
            runtimeUpdated: undefined;
        }>[];
    }>;
} & {
    inner: {
        parentHash: scale_ts.Codec<HexString>;
        number: scale_ts.Codec<number>;
        stateRoot: scale_ts.Codec<HexString>;
        extrinsicRoot: scale_ts.Codec<HexString>;
        digests: [scale_ts.Encoder<Enum<{
            other: Uint8Array<ArrayBufferLike>;
            consensus: {
                engine: string;
                payload: HexString;
            };
            seal: {
                engine: string;
                payload: HexString;
            };
            preRuntime: {
                engine: string;
                payload: HexString;
            };
            runtimeUpdated: undefined;
        }>[]>, scale_ts.Decoder<Enum<{
            other: Uint8Array<ArrayBufferLike>;
            consensus: {
                engine: string;
                payload: HexString;
            };
            seal: {
                engine: string;
                payload: HexString;
            };
            preRuntime: {
                engine: string;
                payload: HexString;
            };
            runtimeUpdated: undefined;
        }>[]>] & {
            enc: scale_ts.Encoder<Enum<{
                other: Uint8Array<ArrayBufferLike>;
                consensus: {
                    engine: string;
                    payload: HexString;
                };
                seal: {
                    engine: string;
                    payload: HexString;
                };
                preRuntime: {
                    engine: string;
                    payload: HexString;
                };
                runtimeUpdated: undefined;
            }>[]>;
            dec: scale_ts.Decoder<Enum<{
                other: Uint8Array<ArrayBufferLike>;
                consensus: {
                    engine: string;
                    payload: HexString;
                };
                seal: {
                    engine: string;
                    payload: HexString;
                };
                preRuntime: {
                    engine: string;
                    payload: HexString;
                };
                runtimeUpdated: undefined;
            }>[]>;
        } & {
            inner: scale_ts.Codec<Enum<{
                other: Uint8Array<ArrayBufferLike>;
                consensus: {
                    engine: string;
                    payload: HexString;
                };
                seal: {
                    engine: string;
                    payload: HexString;
                };
                preRuntime: {
                    engine: string;
                    payload: HexString;
                };
                runtimeUpdated: undefined;
            }>>;
        };
    };
};
type BlockHeader = CodecType<typeof blockHeader>;

declare const metadata: Codec<{
    magicNumber: number;
    metadata: {
        tag: "v0";
        value: unknown;
    } | {
        tag: "v1";
        value: unknown;
    } | {
        tag: "v2";
        value: unknown;
    } | {
        tag: "v3";
        value: unknown;
    } | {
        tag: "v4";
        value: unknown;
    } | {
        tag: "v5";
        value: unknown;
    } | {
        tag: "v6";
        value: unknown;
    } | {
        tag: "v7";
        value: unknown;
    } | {
        tag: "v8";
        value: unknown;
    } | {
        tag: "v9";
        value: unknown;
    } | {
        tag: "v10";
        value: unknown;
    } | {
        tag: "v11";
        value: unknown;
    } | {
        tag: "v12";
        value: unknown;
    } | {
        tag: "v13";
        value: unknown;
    } | {
        tag: "v14";
        value: {
            lookup: {
                id: number;
                path: string[];
                params: {
                    name: string;
                    type: number | undefined;
                }[];
                def: {
                    tag: "composite";
                    value: {
                        name: string | undefined;
                        type: number;
                        typeName: string | undefined;
                        docs: string[];
                    }[];
                } | {
                    tag: "variant";
                    value: {
                        name: string;
                        fields: {
                            name: string | undefined;
                            type: number;
                            typeName: string | undefined;
                            docs: string[];
                        }[];
                        index: number;
                        docs: string[];
                    }[];
                } | {
                    tag: "sequence";
                    value: number;
                } | {
                    tag: "array";
                    value: {
                        len: number;
                        type: number;
                    };
                } | {
                    tag: "tuple";
                    value: number[];
                } | {
                    tag: "primitive";
                    value: {
                        tag: "bool";
                        value: undefined;
                    } | {
                        tag: "char";
                        value: undefined;
                    } | {
                        tag: "str";
                        value: undefined;
                    } | {
                        tag: "u8";
                        value: undefined;
                    } | {
                        tag: "u16";
                        value: undefined;
                    } | {
                        tag: "u32";
                        value: undefined;
                    } | {
                        tag: "u64";
                        value: undefined;
                    } | {
                        tag: "u128";
                        value: undefined;
                    } | {
                        tag: "u256";
                        value: undefined;
                    } | {
                        tag: "i8";
                        value: undefined;
                    } | {
                        tag: "i16";
                        value: undefined;
                    } | {
                        tag: "i32";
                        value: undefined;
                    } | {
                        tag: "i64";
                        value: undefined;
                    } | {
                        tag: "i128";
                        value: undefined;
                    } | {
                        tag: "i256";
                        value: undefined;
                    };
                } | {
                    tag: "compact";
                    value: number;
                } | {
                    tag: "bitSequence";
                    value: {
                        bitStoreType: number;
                        bitOrderType: number;
                    };
                };
                docs: string[];
            }[];
            pallets: {
                docs: string[];
                name: string;
                storage: {
                    prefix: string;
                    items: {
                        name: string;
                        modifier: number;
                        type: {
                            tag: "map";
                            value: {
                                hashers: ({
                                    tag: "Blake2128";
                                    value: undefined;
                                } | {
                                    tag: "Blake2256";
                                    value: undefined;
                                } | {
                                    tag: "Blake2128Concat";
                                    value: undefined;
                                } | {
                                    tag: "Twox128";
                                    value: undefined;
                                } | {
                                    tag: "Twox256";
                                    value: undefined;
                                } | {
                                    tag: "Twox64Concat";
                                    value: undefined;
                                } | {
                                    tag: "Identity";
                                    value: undefined;
                                })[];
                                key: number;
                                value: number;
                            };
                        } | {
                            tag: "plain";
                            value: number;
                        };
                        fallback: HexString;
                        docs: string[];
                    }[];
                } | undefined;
                calls: number | undefined;
                events: number | undefined;
                constants: {
                    name: string;
                    type: number;
                    value: HexString;
                    docs: string[];
                }[];
                errors: number | undefined;
                index: number;
            }[];
            extrinsic: {
                type: number;
                version: number;
                signedExtensions: {
                    identifier: string;
                    type: number;
                    additionalSigned: number;
                }[];
            };
            type: number;
            apis: {
                name: string;
                methods: {
                    deprecationInfo: {
                        tag: "NotDeprecated";
                        value: undefined;
                    } | {
                        tag: "DeprecatedWithoutNote";
                        value: undefined;
                    } | {
                        tag: "Deprecated";
                        value: {
                            note: string;
                            since: string | undefined;
                        };
                    };
                    name: string;
                    inputs: {
                        name: string;
                        type: number;
                    }[];
                    output: number;
                    docs: string[];
                }[];
                docs: string[];
                version: number;
                deprecationInfo: {
                    tag: "NotDeprecated";
                    value: undefined;
                } | {
                    tag: "DeprecatedWithoutNote";
                    value: undefined;
                } | {
                    tag: "Deprecated";
                    value: {
                        note: string;
                        since: string | undefined;
                    };
                };
            }[];
        };
    } | {
        tag: "v15";
        value: {
            lookup: {
                id: number;
                path: string[];
                params: {
                    name: string;
                    type: number | undefined;
                }[];
                def: {
                    tag: "composite";
                    value: {
                        name: string | undefined;
                        type: number;
                        typeName: string | undefined;
                        docs: string[];
                    }[];
                } | {
                    tag: "variant";
                    value: {
                        name: string;
                        fields: {
                            name: string | undefined;
                            type: number;
                            typeName: string | undefined;
                            docs: string[];
                        }[];
                        index: number;
                        docs: string[];
                    }[];
                } | {
                    tag: "sequence";
                    value: number;
                } | {
                    tag: "array";
                    value: {
                        len: number;
                        type: number;
                    };
                } | {
                    tag: "tuple";
                    value: number[];
                } | {
                    tag: "primitive";
                    value: {
                        tag: "bool";
                        value: undefined;
                    } | {
                        tag: "char";
                        value: undefined;
                    } | {
                        tag: "str";
                        value: undefined;
                    } | {
                        tag: "u8";
                        value: undefined;
                    } | {
                        tag: "u16";
                        value: undefined;
                    } | {
                        tag: "u32";
                        value: undefined;
                    } | {
                        tag: "u64";
                        value: undefined;
                    } | {
                        tag: "u128";
                        value: undefined;
                    } | {
                        tag: "u256";
                        value: undefined;
                    } | {
                        tag: "i8";
                        value: undefined;
                    } | {
                        tag: "i16";
                        value: undefined;
                    } | {
                        tag: "i32";
                        value: undefined;
                    } | {
                        tag: "i64";
                        value: undefined;
                    } | {
                        tag: "i128";
                        value: undefined;
                    } | {
                        tag: "i256";
                        value: undefined;
                    };
                } | {
                    tag: "compact";
                    value: number;
                } | {
                    tag: "bitSequence";
                    value: {
                        bitStoreType: number;
                        bitOrderType: number;
                    };
                };
                docs: string[];
            }[];
            pallets: {
                docs: string[];
                name: string;
                storage: {
                    prefix: string;
                    items: {
                        name: string;
                        modifier: number;
                        type: {
                            tag: "map";
                            value: {
                                hashers: ({
                                    tag: "Blake2128";
                                    value: undefined;
                                } | {
                                    tag: "Blake2256";
                                    value: undefined;
                                } | {
                                    tag: "Blake2128Concat";
                                    value: undefined;
                                } | {
                                    tag: "Twox128";
                                    value: undefined;
                                } | {
                                    tag: "Twox256";
                                    value: undefined;
                                } | {
                                    tag: "Twox64Concat";
                                    value: undefined;
                                } | {
                                    tag: "Identity";
                                    value: undefined;
                                })[];
                                key: number;
                                value: number;
                            };
                        } | {
                            tag: "plain";
                            value: number;
                        };
                        fallback: HexString;
                        docs: string[];
                    }[];
                } | undefined;
                calls: number | undefined;
                events: number | undefined;
                constants: {
                    name: string;
                    type: number;
                    value: HexString;
                    docs: string[];
                }[];
                errors: number | undefined;
                index: number;
            }[];
            extrinsic: {
                version: number;
                address: number;
                call: number;
                signature: number;
                extra: number;
                signedExtensions: {
                    identifier: string;
                    type: number;
                    additionalSigned: number;
                }[];
            };
            type: number;
            apis: {
                name: string;
                methods: {
                    name: string;
                    inputs: {
                        name: string;
                        type: number;
                    }[];
                    output: number;
                    docs: string[];
                }[];
                docs: string[];
            }[];
            outerEnums: {
                call: number;
                event: number;
                error: number;
            };
            custom: [string, {
                type: number;
                value: HexString;
            }][];
        };
    } | {
        tag: "v16";
        value: {
            lookup: {
                id: number;
                path: string[];
                params: {
                    name: string;
                    type: number | undefined;
                }[];
                def: {
                    tag: "composite";
                    value: {
                        name: string | undefined;
                        type: number;
                        typeName: string | undefined;
                        docs: string[];
                    }[];
                } | {
                    tag: "variant";
                    value: {
                        name: string;
                        fields: {
                            name: string | undefined;
                            type: number;
                            typeName: string | undefined;
                            docs: string[];
                        }[];
                        index: number;
                        docs: string[];
                    }[];
                } | {
                    tag: "sequence";
                    value: number;
                } | {
                    tag: "array";
                    value: {
                        len: number;
                        type: number;
                    };
                } | {
                    tag: "tuple";
                    value: number[];
                } | {
                    tag: "primitive";
                    value: {
                        tag: "bool";
                        value: undefined;
                    } | {
                        tag: "char";
                        value: undefined;
                    } | {
                        tag: "str";
                        value: undefined;
                    } | {
                        tag: "u8";
                        value: undefined;
                    } | {
                        tag: "u16";
                        value: undefined;
                    } | {
                        tag: "u32";
                        value: undefined;
                    } | {
                        tag: "u64";
                        value: undefined;
                    } | {
                        tag: "u128";
                        value: undefined;
                    } | {
                        tag: "u256";
                        value: undefined;
                    } | {
                        tag: "i8";
                        value: undefined;
                    } | {
                        tag: "i16";
                        value: undefined;
                    } | {
                        tag: "i32";
                        value: undefined;
                    } | {
                        tag: "i64";
                        value: undefined;
                    } | {
                        tag: "i128";
                        value: undefined;
                    } | {
                        tag: "i256";
                        value: undefined;
                    };
                } | {
                    tag: "compact";
                    value: number;
                } | {
                    tag: "bitSequence";
                    value: {
                        bitStoreType: number;
                        bitOrderType: number;
                    };
                };
                docs: string[];
            }[];
            pallets: {
                name: string;
                storage: {
                    prefix: string;
                    items: {
                        deprecationInfo: {
                            tag: "NotDeprecated";
                            value: undefined;
                        } | {
                            tag: "DeprecatedWithoutNote";
                            value: undefined;
                        } | {
                            tag: "Deprecated";
                            value: {
                                note: string;
                                since: string | undefined;
                            };
                        };
                        name: string;
                        modifier: number;
                        type: {
                            tag: "map";
                            value: {
                                hashers: ({
                                    tag: "Blake2128";
                                    value: undefined;
                                } | {
                                    tag: "Blake2256";
                                    value: undefined;
                                } | {
                                    tag: "Blake2128Concat";
                                    value: undefined;
                                } | {
                                    tag: "Twox128";
                                    value: undefined;
                                } | {
                                    tag: "Twox256";
                                    value: undefined;
                                } | {
                                    tag: "Twox64Concat";
                                    value: undefined;
                                } | {
                                    tag: "Identity";
                                    value: undefined;
                                })[];
                                key: number;
                                value: number;
                            };
                        } | {
                            tag: "plain";
                            value: number;
                        };
                        fallback: HexString;
                        docs: string[];
                    }[];
                } | undefined;
                calls: {
                    type: number;
                    deprecationInfo: {
                        index: number;
                        deprecation: {
                            tag: "DeprecatedWithoutNote";
                            value: undefined;
                        } | {
                            tag: "Deprecated";
                            value: {
                                note: string;
                                since: string | undefined;
                            };
                        };
                    }[];
                } | undefined;
                events: {
                    type: number;
                    deprecationInfo: {
                        index: number;
                        deprecation: {
                            tag: "DeprecatedWithoutNote";
                            value: undefined;
                        } | {
                            tag: "Deprecated";
                            value: {
                                note: string;
                                since: string | undefined;
                            };
                        };
                    }[];
                } | undefined;
                constants: {
                    name: string;
                    type: number;
                    value: HexString;
                    docs: string[];
                    deprecationInfo: {
                        tag: "NotDeprecated";
                        value: undefined;
                    } | {
                        tag: "DeprecatedWithoutNote";
                        value: undefined;
                    } | {
                        tag: "Deprecated";
                        value: {
                            note: string;
                            since: string | undefined;
                        };
                    };
                }[];
                errors: {
                    type: number;
                    deprecationInfo: {
                        index: number;
                        deprecation: {
                            tag: "DeprecatedWithoutNote";
                            value: undefined;
                        } | {
                            tag: "Deprecated";
                            value: {
                                note: string;
                                since: string | undefined;
                            };
                        };
                    }[];
                } | undefined;
                associatedTypes: {
                    name: string;
                    type: number;
                    docs: string[];
                }[];
                viewFns: {
                    deprecationInfo: {
                        tag: "NotDeprecated";
                        value: undefined;
                    } | {
                        tag: "DeprecatedWithoutNote";
                        value: undefined;
                    } | {
                        tag: "Deprecated";
                        value: {
                            note: string;
                            since: string | undefined;
                        };
                    };
                    name: string;
                    inputs: {
                        name: string;
                        type: number;
                    }[];
                    output: number;
                    docs: string[];
                    id: HexString;
                }[];
                index: number;
                docs: string[];
                deprecationInfo: {
                    tag: "NotDeprecated";
                    value: undefined;
                } | {
                    tag: "DeprecatedWithoutNote";
                    value: undefined;
                } | {
                    tag: "Deprecated";
                    value: {
                        note: string;
                        since: string | undefined;
                    };
                };
            }[];
            extrinsic: {
                version: number[];
                address: number;
                call: number;
                signature: number;
                signedExtensionsByVersion: [number, number[]][];
                signedExtensions: {
                    identifier: string;
                    type: number;
                    additionalSigned: number;
                }[];
            };
            apis: {
                name: string;
                methods: {
                    deprecationInfo: {
                        tag: "NotDeprecated";
                        value: undefined;
                    } | {
                        tag: "DeprecatedWithoutNote";
                        value: undefined;
                    } | {
                        tag: "Deprecated";
                        value: {
                            note: string;
                            since: string | undefined;
                        };
                    };
                    name: string;
                    inputs: {
                        name: string;
                        type: number;
                    }[];
                    output: number;
                    docs: string[];
                }[];
                docs: string[];
                version: number;
                deprecationInfo: {
                    tag: "NotDeprecated";
                    value: undefined;
                } | {
                    tag: "DeprecatedWithoutNote";
                    value: undefined;
                } | {
                    tag: "Deprecated";
                    value: {
                        note: string;
                        since: string | undefined;
                    };
                };
            }[];
            outerEnums: {
                call: number;
                event: number;
                error: number;
            };
            custom: [string, {
                type: number;
                value: HexString;
            }][];
        };
    };
}>;
type Metadata = CodecType<typeof metadata>;
declare const decAnyMetadata: (input: Uint8Array | HexString) => CodecType<typeof metadata>;

declare const v14: scale_ts.Codec<{
    lookup: {
        id: number;
        path: string[];
        params: {
            name: string;
            type: number | undefined;
        }[];
        def: {
            tag: "composite";
            value: {
                name: string | undefined;
                type: number;
                typeName: string | undefined;
                docs: string[];
            }[];
        } | {
            tag: "variant";
            value: {
                name: string;
                fields: {
                    name: string | undefined;
                    type: number;
                    typeName: string | undefined;
                    docs: string[];
                }[];
                index: number;
                docs: string[];
            }[];
        } | {
            tag: "sequence";
            value: number;
        } | {
            tag: "array";
            value: {
                len: number;
                type: number;
            };
        } | {
            tag: "tuple";
            value: number[];
        } | {
            tag: "primitive";
            value: {
                tag: "bool";
                value: undefined;
            } | {
                tag: "char";
                value: undefined;
            } | {
                tag: "str";
                value: undefined;
            } | {
                tag: "u8";
                value: undefined;
            } | {
                tag: "u16";
                value: undefined;
            } | {
                tag: "u32";
                value: undefined;
            } | {
                tag: "u64";
                value: undefined;
            } | {
                tag: "u128";
                value: undefined;
            } | {
                tag: "u256";
                value: undefined;
            } | {
                tag: "i8";
                value: undefined;
            } | {
                tag: "i16";
                value: undefined;
            } | {
                tag: "i32";
                value: undefined;
            } | {
                tag: "i64";
                value: undefined;
            } | {
                tag: "i128";
                value: undefined;
            } | {
                tag: "i256";
                value: undefined;
            };
        } | {
            tag: "compact";
            value: number;
        } | {
            tag: "bitSequence";
            value: {
                bitStoreType: number;
                bitOrderType: number;
            };
        };
        docs: string[];
    }[];
    pallets: {
        docs: string[];
        name: string;
        storage: {
            prefix: string;
            items: {
                name: string;
                modifier: number;
                type: {
                    tag: "map";
                    value: {
                        hashers: ({
                            tag: "Blake2128";
                            value: undefined;
                        } | {
                            tag: "Blake2256";
                            value: undefined;
                        } | {
                            tag: "Blake2128Concat";
                            value: undefined;
                        } | {
                            tag: "Twox128";
                            value: undefined;
                        } | {
                            tag: "Twox256";
                            value: undefined;
                        } | {
                            tag: "Twox64Concat";
                            value: undefined;
                        } | {
                            tag: "Identity";
                            value: undefined;
                        })[];
                        key: number;
                        value: number;
                    };
                } | {
                    tag: "plain";
                    value: number;
                };
                fallback: HexString;
                docs: string[];
            }[];
        } | undefined;
        calls: number | undefined;
        events: number | undefined;
        constants: {
            name: string;
            type: number;
            value: HexString;
            docs: string[];
        }[];
        errors: number | undefined;
        index: number;
    }[];
    extrinsic: {
        type: number;
        version: number;
        signedExtensions: {
            identifier: string;
            type: number;
            additionalSigned: number;
        }[];
    };
    type: number;
    apis: {
        name: string;
        methods: {
            deprecationInfo: {
                tag: "NotDeprecated";
                value: undefined;
            } | {
                tag: "DeprecatedWithoutNote";
                value: undefined;
            } | {
                tag: "Deprecated";
                value: {
                    note: string;
                    since: string | undefined;
                };
            };
            name: string;
            inputs: {
                name: string;
                type: number;
            }[];
            output: number;
            docs: string[];
        }[];
        docs: string[];
        version: number;
        deprecationInfo: {
            tag: "NotDeprecated";
            value: undefined;
        } | {
            tag: "DeprecatedWithoutNote";
            value: undefined;
        } | {
            tag: "Deprecated";
            value: {
                note: string;
                since: string | undefined;
            };
        };
    }[];
}>;
type V14 = CodecType<typeof v14>;

declare const v15: scale_ts.Codec<{
    lookup: {
        id: number;
        path: string[];
        params: {
            name: string;
            type: number | undefined;
        }[];
        def: {
            tag: "composite";
            value: {
                name: string | undefined;
                type: number;
                typeName: string | undefined;
                docs: string[];
            }[];
        } | {
            tag: "variant";
            value: {
                name: string;
                fields: {
                    name: string | undefined;
                    type: number;
                    typeName: string | undefined;
                    docs: string[];
                }[];
                index: number;
                docs: string[];
            }[];
        } | {
            tag: "sequence";
            value: number;
        } | {
            tag: "array";
            value: {
                len: number;
                type: number;
            };
        } | {
            tag: "tuple";
            value: number[];
        } | {
            tag: "primitive";
            value: {
                tag: "bool";
                value: undefined;
            } | {
                tag: "char";
                value: undefined;
            } | {
                tag: "str";
                value: undefined;
            } | {
                tag: "u8";
                value: undefined;
            } | {
                tag: "u16";
                value: undefined;
            } | {
                tag: "u32";
                value: undefined;
            } | {
                tag: "u64";
                value: undefined;
            } | {
                tag: "u128";
                value: undefined;
            } | {
                tag: "u256";
                value: undefined;
            } | {
                tag: "i8";
                value: undefined;
            } | {
                tag: "i16";
                value: undefined;
            } | {
                tag: "i32";
                value: undefined;
            } | {
                tag: "i64";
                value: undefined;
            } | {
                tag: "i128";
                value: undefined;
            } | {
                tag: "i256";
                value: undefined;
            };
        } | {
            tag: "compact";
            value: number;
        } | {
            tag: "bitSequence";
            value: {
                bitStoreType: number;
                bitOrderType: number;
            };
        };
        docs: string[];
    }[];
    pallets: {
        docs: string[];
        name: string;
        storage: {
            prefix: string;
            items: {
                name: string;
                modifier: number;
                type: {
                    tag: "map";
                    value: {
                        hashers: ({
                            tag: "Blake2128";
                            value: undefined;
                        } | {
                            tag: "Blake2256";
                            value: undefined;
                        } | {
                            tag: "Blake2128Concat";
                            value: undefined;
                        } | {
                            tag: "Twox128";
                            value: undefined;
                        } | {
                            tag: "Twox256";
                            value: undefined;
                        } | {
                            tag: "Twox64Concat";
                            value: undefined;
                        } | {
                            tag: "Identity";
                            value: undefined;
                        })[];
                        key: number;
                        value: number;
                    };
                } | {
                    tag: "plain";
                    value: number;
                };
                fallback: HexString;
                docs: string[];
            }[];
        } | undefined;
        calls: number | undefined;
        events: number | undefined;
        constants: {
            name: string;
            type: number;
            value: HexString;
            docs: string[];
        }[];
        errors: number | undefined;
        index: number;
    }[];
    extrinsic: {
        version: number;
        address: number;
        call: number;
        signature: number;
        extra: number;
        signedExtensions: {
            identifier: string;
            type: number;
            additionalSigned: number;
        }[];
    };
    type: number;
    apis: {
        name: string;
        methods: {
            name: string;
            inputs: {
                name: string;
                type: number;
            }[];
            output: number;
            docs: string[];
        }[];
        docs: string[];
    }[];
    outerEnums: {
        call: number;
        event: number;
        error: number;
    };
    custom: [string, {
        type: number;
        value: HexString;
    }][];
}>;
type V15 = CodecType<typeof v15>;

declare const v16: scale_ts.Codec<{
    lookup: {
        id: number;
        path: string[];
        params: {
            name: string;
            type: number | undefined;
        }[];
        def: {
            tag: "composite";
            value: {
                name: string | undefined;
                type: number;
                typeName: string | undefined;
                docs: string[];
            }[];
        } | {
            tag: "variant";
            value: {
                name: string;
                fields: {
                    name: string | undefined;
                    type: number;
                    typeName: string | undefined;
                    docs: string[];
                }[];
                index: number;
                docs: string[];
            }[];
        } | {
            tag: "sequence";
            value: number;
        } | {
            tag: "array";
            value: {
                len: number;
                type: number;
            };
        } | {
            tag: "tuple";
            value: number[];
        } | {
            tag: "primitive";
            value: {
                tag: "bool";
                value: undefined;
            } | {
                tag: "char";
                value: undefined;
            } | {
                tag: "str";
                value: undefined;
            } | {
                tag: "u8";
                value: undefined;
            } | {
                tag: "u16";
                value: undefined;
            } | {
                tag: "u32";
                value: undefined;
            } | {
                tag: "u64";
                value: undefined;
            } | {
                tag: "u128";
                value: undefined;
            } | {
                tag: "u256";
                value: undefined;
            } | {
                tag: "i8";
                value: undefined;
            } | {
                tag: "i16";
                value: undefined;
            } | {
                tag: "i32";
                value: undefined;
            } | {
                tag: "i64";
                value: undefined;
            } | {
                tag: "i128";
                value: undefined;
            } | {
                tag: "i256";
                value: undefined;
            };
        } | {
            tag: "compact";
            value: number;
        } | {
            tag: "bitSequence";
            value: {
                bitStoreType: number;
                bitOrderType: number;
            };
        };
        docs: string[];
    }[];
    pallets: {
        name: string;
        storage: {
            prefix: string;
            items: {
                deprecationInfo: {
                    tag: "NotDeprecated";
                    value: undefined;
                } | {
                    tag: "DeprecatedWithoutNote";
                    value: undefined;
                } | {
                    tag: "Deprecated";
                    value: {
                        note: string;
                        since: string | undefined;
                    };
                };
                name: string;
                modifier: number;
                type: {
                    tag: "map";
                    value: {
                        hashers: ({
                            tag: "Blake2128";
                            value: undefined;
                        } | {
                            tag: "Blake2256";
                            value: undefined;
                        } | {
                            tag: "Blake2128Concat";
                            value: undefined;
                        } | {
                            tag: "Twox128";
                            value: undefined;
                        } | {
                            tag: "Twox256";
                            value: undefined;
                        } | {
                            tag: "Twox64Concat";
                            value: undefined;
                        } | {
                            tag: "Identity";
                            value: undefined;
                        })[];
                        key: number;
                        value: number;
                    };
                } | {
                    tag: "plain";
                    value: number;
                };
                fallback: HexString;
                docs: string[];
            }[];
        } | undefined;
        calls: {
            type: number;
            deprecationInfo: {
                index: number;
                deprecation: {
                    tag: "DeprecatedWithoutNote";
                    value: undefined;
                } | {
                    tag: "Deprecated";
                    value: {
                        note: string;
                        since: string | undefined;
                    };
                };
            }[];
        } | undefined;
        events: {
            type: number;
            deprecationInfo: {
                index: number;
                deprecation: {
                    tag: "DeprecatedWithoutNote";
                    value: undefined;
                } | {
                    tag: "Deprecated";
                    value: {
                        note: string;
                        since: string | undefined;
                    };
                };
            }[];
        } | undefined;
        constants: {
            name: string;
            type: number;
            value: HexString;
            docs: string[];
            deprecationInfo: {
                tag: "NotDeprecated";
                value: undefined;
            } | {
                tag: "DeprecatedWithoutNote";
                value: undefined;
            } | {
                tag: "Deprecated";
                value: {
                    note: string;
                    since: string | undefined;
                };
            };
        }[];
        errors: {
            type: number;
            deprecationInfo: {
                index: number;
                deprecation: {
                    tag: "DeprecatedWithoutNote";
                    value: undefined;
                } | {
                    tag: "Deprecated";
                    value: {
                        note: string;
                        since: string | undefined;
                    };
                };
            }[];
        } | undefined;
        associatedTypes: {
            name: string;
            type: number;
            docs: string[];
        }[];
        viewFns: {
            deprecationInfo: {
                tag: "NotDeprecated";
                value: undefined;
            } | {
                tag: "DeprecatedWithoutNote";
                value: undefined;
            } | {
                tag: "Deprecated";
                value: {
                    note: string;
                    since: string | undefined;
                };
            };
            name: string;
            inputs: {
                name: string;
                type: number;
            }[];
            output: number;
            docs: string[];
            id: HexString;
        }[];
        index: number;
        docs: string[];
        deprecationInfo: {
            tag: "NotDeprecated";
            value: undefined;
        } | {
            tag: "DeprecatedWithoutNote";
            value: undefined;
        } | {
            tag: "Deprecated";
            value: {
                note: string;
                since: string | undefined;
            };
        };
    }[];
    extrinsic: {
        version: number[];
        address: number;
        call: number;
        signature: number;
        signedExtensionsByVersion: [number, number[]][];
        signedExtensions: {
            identifier: string;
            type: number;
            additionalSigned: number;
        }[];
    };
    apis: {
        name: string;
        methods: {
            deprecationInfo: {
                tag: "NotDeprecated";
                value: undefined;
            } | {
                tag: "DeprecatedWithoutNote";
                value: undefined;
            } | {
                tag: "Deprecated";
                value: {
                    note: string;
                    since: string | undefined;
                };
            };
            name: string;
            inputs: {
                name: string;
                type: number;
            }[];
            output: number;
            docs: string[];
        }[];
        docs: string[];
        version: number;
        deprecationInfo: {
            tag: "NotDeprecated";
            value: undefined;
        } | {
            tag: "DeprecatedWithoutNote";
            value: undefined;
        } | {
            tag: "Deprecated";
            value: {
                note: string;
                since: string | undefined;
            };
        };
    }[];
    outerEnums: {
        call: number;
        event: number;
        error: number;
    };
    custom: [string, {
        type: number;
        value: HexString;
    }][];
}>;
type V16 = CodecType<typeof v16>;

declare const lookup: scale_ts.Codec<{
    id: number;
    path: string[];
    params: {
        name: string;
        type: number | undefined;
    }[];
    def: {
        tag: "composite";
        value: {
            name: string | undefined;
            type: number;
            typeName: string | undefined;
            docs: string[];
        }[];
    } | {
        tag: "variant";
        value: {
            name: string;
            fields: {
                name: string | undefined;
                type: number;
                typeName: string | undefined;
                docs: string[];
            }[];
            index: number;
            docs: string[];
        }[];
    } | {
        tag: "sequence";
        value: number;
    } | {
        tag: "array";
        value: {
            len: number;
            type: number;
        };
    } | {
        tag: "tuple";
        value: number[];
    } | {
        tag: "primitive";
        value: {
            tag: "bool";
            value: undefined;
        } | {
            tag: "char";
            value: undefined;
        } | {
            tag: "str";
            value: undefined;
        } | {
            tag: "u8";
            value: undefined;
        } | {
            tag: "u16";
            value: undefined;
        } | {
            tag: "u32";
            value: undefined;
        } | {
            tag: "u64";
            value: undefined;
        } | {
            tag: "u128";
            value: undefined;
        } | {
            tag: "u256";
            value: undefined;
        } | {
            tag: "i8";
            value: undefined;
        } | {
            tag: "i16";
            value: undefined;
        } | {
            tag: "i32";
            value: undefined;
        } | {
            tag: "i64";
            value: undefined;
        } | {
            tag: "i128";
            value: undefined;
        } | {
            tag: "i256";
            value: undefined;
        };
    } | {
        tag: "compact";
        value: number;
    } | {
        tag: "bitSequence";
        value: {
            bitStoreType: number;
            bitOrderType: number;
        };
    };
    docs: string[];
}[]>;
type V14Lookup = CodecType<typeof lookup>;

declare const storageMap: scale_ts.Codec<{
    hashers: ({
        tag: "Blake2128";
        value: undefined;
    } | {
        tag: "Blake2256";
        value: undefined;
    } | {
        tag: "Blake2128Concat";
        value: undefined;
    } | {
        tag: "Twox128";
        value: undefined;
    } | {
        tag: "Twox256";
        value: undefined;
    } | {
        tag: "Twox64Concat";
        value: undefined;
    } | {
        tag: "Identity";
        value: undefined;
    })[];
    key: number;
    value: number;
}>;

declare const itemDeprecation: scale_ts.Codec<{
    tag: "NotDeprecated";
    value: undefined;
} | {
    tag: "DeprecatedWithoutNote";
    value: undefined;
} | {
    tag: "Deprecated";
    value: {
        note: string;
        since: string | undefined;
    };
}>;
declare const variantDeprecation: scale_ts.Codec<{
    index: number;
    deprecation: {
        tag: "DeprecatedWithoutNote";
        value: undefined;
    } | {
        tag: "Deprecated";
        value: {
            note: string;
            since: string | undefined;
        };
    };
}[]>;

declare const viewFunction: scale_ts.Codec<{
    deprecationInfo: {
        tag: "NotDeprecated";
        value: undefined;
    } | {
        tag: "DeprecatedWithoutNote";
        value: undefined;
    } | {
        tag: "Deprecated";
        value: {
            note: string;
            since: string | undefined;
        };
    };
    name: string;
    inputs: {
        name: string;
        type: number;
    }[];
    output: number;
    docs: string[];
    id: HexString;
}>;

type EnumRef<T> = ({
    type: number;
} & (T extends 16 ? {
    deprecationInfo: CodecType<typeof variantDeprecation>;
} : {})) | undefined;
type DeprecationInfo<T> = T extends 16 ? {
    deprecationInfo: CodecType<typeof itemDeprecation>;
} : {};
type UnifiedMetadata<T extends 14 | 15 | 16 = 14 | 15 | 16> = {
    version: T;
    lookup: V14Lookup;
    pallets: Array<{
        name: string;
        storage: {
            prefix: string;
            items: Array<{
                name: string;
                modifier: number;
                type: {
                    tag: "plain";
                    value: number;
                } | {
                    tag: "map";
                    value: CodecType<typeof storageMap>;
                };
                fallback: HexString;
                docs: string[];
            } & DeprecationInfo<T>>;
        } | undefined;
        calls: EnumRef<T>;
        events: EnumRef<T>;
        constants: Array<{
            name: string;
            type: number;
            value: HexString;
            docs: string[];
        } & DeprecationInfo<T>>;
        errors: EnumRef<T>;
        associatedTypes: Array<{
            name: string;
            type: number;
            docs: string[];
        }>;
        viewFns: Array<CodecType<typeof viewFunction>>;
        index: number;
        docs: string[];
    } & DeprecationInfo<T>>;
    extrinsic: {
        version: number[];
        signedExtensions: Array<{
            identifier: string;
            type: number;
            additionalSigned: number;
        }>;
    } & (T extends 14 ? {
        type: number;
    } : {
        address: number;
        call: number;
        signature: number;
    }) & (T extends 16 ? {
        signedExtensionsByVersion: Array<[number, number[]]>;
    } : {});
    apis: Array<{
        name: string;
        methods: Array<{
            name: string;
            inputs: Array<{
                name: string;
                type: number;
            }>;
            output: number;
            docs: string[];
        } & DeprecationInfo<T>>;
        docs: string[];
    } & (T extends 16 ? {
        version: number;
    } : {}) & DeprecationInfo<T>>;
} & (T extends 14 ? {} : {
    outerEnums: {
        call: number;
        event: number;
        error: number;
    };
    custom: Array<[string, {
        type: number;
        value: HexString;
    }]>;
});
declare const unifyMetadata: (metadata: Metadata | Metadata["metadata"] | V14 | V15 | V16) => UnifiedMetadata;

type ExtrinsicFormat = {
    version: 4;
    type: "bare" | "signed";
} | {
    version: 5;
    type: "bare" | "general";
};
declare const extrinsicFormat: scale_ts.Codec<ExtrinsicFormat>;

declare const Blake2256: (encoded: Uint8Array) => Uint8Array<ArrayBufferLike>;
declare const Blake2128: (encoded: Uint8Array) => Uint8Array<ArrayBufferLike>;
declare const Blake2128Concat: (encoded: Uint8Array) => Uint8Array<ArrayBufferLike>;

declare const Blake3256: (encoded: Uint8Array) => Uint8Array<ArrayBufferLike>;
declare const Blake3256Concat: (encoded: Uint8Array) => Uint8Array<ArrayBufferLike>;

declare const Identity: (encoded: Uint8Array) => Uint8Array;

declare const Twox128: (input: Uint8Array) => Uint8Array;
declare const Twox256: (input: Uint8Array) => Uint8Array;
declare const Twox64Concat: (encoded: Uint8Array) => Uint8Array;

declare function h64(input: Uint8Array, seed?: bigint): bigint;

declare const Keccak256: (input: Uint8Array) => Uint8Array;

type EncoderWithHash<T> = [Codec<T>, (input: Uint8Array) => Uint8Array];
type OpaqueKeyHash = string & {
    __opaqueKeyHash?: unknown;
};
declare const Storage: (pallet: string) => <A extends Array<EncoderWithHash<any>>>(name: string, ...encoders: [...A]) => {
    enc: (...args: { [K in keyof A]: A[K] extends EncoderWithHash<infer V> ? V : unknown; }) => string;
    dec: (value: string) => { [K in keyof A]: A[K] extends EncoderWithHash<infer V> ? V : unknown; };
};

declare const TrieNodeHeaders: {
    readonly Leaf: "Leaf";
    readonly Branch: "Branch";
    readonly BranchWithVal: "BranchWithVal";
    readonly LeafWithHash: "LeafWithHash";
    readonly BranchWithHash: "BranchWithHash";
    readonly Empty: "Empty";
    readonly Reserved: "Reserved";
};
type TrieNodeHeaders = typeof TrieNodeHeaders;
type TrieNodeHeaderKey = (typeof TrieNodeHeaders)[keyof typeof TrieNodeHeaders];
type Nibble = "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "a" | "b" | "c" | "d" | "e" | "f";
type TrieNode = {
    partialKey: string;
} & ({
    type: TrieNodeHeaders["Empty"] | TrieNodeHeaders["Reserved"];
} | {
    type: TrieNodeHeaders["Leaf"] | TrieNodeHeaders["LeafWithHash"];
    value: HexString;
} | ({
    children: Record<Nibble, HexString>;
} & ({
    type: TrieNodeHeaders["Branch"];
} | {
    type: TrieNodeHeaders["BranchWithHash"] | TrieNodeHeaders["BranchWithVal"];
    value: HexString;
})));
type ProofTrieNode = {
    hash: HexString;
    parent?: HexString;
} & (TrieNode | {
    type: "Raw";
    value: HexString;
});

declare const trieNodeDec: scale_ts.Decoder<TrieNode>;

declare const TrieNodeWithHash: (hasher: (input: Uint8Array) => Uint8Array) => scale_ts.Decoder<ProofTrieNode>;
declare const validateProofs: <T extends HexString | Uint8Array>(proofs: Array<T>, hasher?: (input: Uint8Array) => Uint8Array) => {
    rootHash: HexString;
    proofs: Record<HexString, ProofTrieNode>;
} | null;

export { AccountId, Bin, Binary, BitSeq, Blake2128, Blake2128Concat, Blake2256, Blake3256, Blake3256Concat, Enum, FixedSizeBinary, Hex, Identity, Keccak256, Option, Result, ScaleEnum, Self, Storage, Struct, TrieNodeHeaders, TrieNodeWithHash, Tuple, Twox128, Twox256, Twox64Concat, Variant, Vector, _Enum, bitSequence, blockHeader, char, compactBn, compactNumber, decAnyMetadata, ethAccount, extrinsicFormat, fixedStr, fromBufferToBase58, getMultisigAccountId, getSs58AddressInfo, h64, metadata, selfDecoder, selfEncoder, sortMultisigSignatories, trieNodeDec, unifyMetadata, v14, lookup as v14Lookup, v15, v16, validateProofs };
export type { BitSequence, BlockHeader, EncoderWithHash, EnumVariant, ExtractEnumValue, ExtrinsicFormat, GetEnum, HexString, Metadata, Nibble, OpaqueKeyHash, ProofTrieNode, SS58AddressInfo, SS58String, TrieNode, TrieNodeHeaderKey, UnifiedMetadata, V14, V14Lookup, V15, V16 };
