import type { HexString } from '@polkadot/util/types';
import type { AnyJson, BareOpts, Codec, CodecClass, DefinitionSetter, Inspect, IStruct, IU8a, Registry } from '../types/index.js';
type TypesDef<T = Codec> = Record<string, string | CodecClass<T>>;
type Definition = [CodecClass[], string[]];
/**
 * @name Struct
 * @description
 * A Struct defines an Object with key-value pairs - where the values are Codec values. It removes
 * a lot of repetition from the actual coding, define a structure type, pass it the key/Codec
 * values in the constructor and it manages the decoding. It is important that the constructor
 * values matches 100% to the order in th Rust code, i.e. don't go crazy and make it alphabetical,
 * it needs to decoded in the specific defined order.
 * @noInheritDoc
 */
export declare class Struct<S extends TypesDef = TypesDef, V extends {
    [K in keyof S]: any;
} = {
    [K in keyof S]: any;
}, E extends {
    [K in keyof S]: string;
} = {
    [K in keyof S]: string;
}> extends Map<keyof S, Codec> implements IStruct<keyof S> {
    #private;
    readonly registry: Registry;
    createdAtHash?: IU8a | undefined;
    initialU8aLength?: number;
    isStorageFallback?: boolean;
    constructor(registry: Registry, Types: S, value?: V | Map<unknown, unknown> | unknown[] | HexString | null, jsonMap?: Map<string, string>, { definition, setDefinition }?: DefinitionSetter<Definition>);
    static with<S extends TypesDef>(Types: S, jsonMap?: Map<string, string>): CodecClass<Struct<S>>;
    /**
     * @description The available keys for this struct
     */
    get defKeys(): string[];
    /**
     * @description Checks if the value is an empty value '{}'
     */
    get isEmpty(): boolean;
    /**
     * @description The length of the value when encoded as a Uint8Array
     */
    get encodedLength(): number;
    /**
     * @description returns a hash of the contents
     */
    get hash(): IU8a;
    /**
     * @description Returns the Type description of the structure
     */
    get Type(): E;
    /**
     * @description Compares the value of the input to see if there is a match
     */
    eq(other?: unknown): boolean;
    /**
     * @description Returns a specific names entry in the structure
     * @param key The name of the entry to retrieve
     */
    get(key: keyof S): Codec | undefined;
    /**
     * @description Returns the values of a member at a specific index (Rather use get(name) for performance)
     */
    getAtIndex(index: number): Codec;
    /**
     * @description Returns the a types value by name
     */
    getT<T = Codec>(key: string): T;
    /**
     * @description Returns a breakdown of the hex encoding for this Codec
     */
    inspect(isBare?: BareOpts): Inspect;
    /**
     * @description Converts the Object to an standard JavaScript Array
     */
    toArray(): Codec[];
    /**
     * @description Returns a hex string representation of the value
     */
    toHex(): HexString;
    /**
     * @description Converts the Object to to a human-friendly JSON, with additional fields, expansion and formatting of information
     */
    toHuman(isExtended?: boolean, disableAscii?: boolean): Record<string, AnyJson>;
    /**
     * @description Converts the Object to JSON, typically used for RPC transfers
     */
    toJSON(): Record<string, AnyJson>;
    /**
     * @description Converts the value in a best-fit primitive form
     */
    toPrimitive(disableAscii?: boolean): Record<string, AnyJson>;
    /**
     * @description Returns the base runtime type name for this instance
     */
    toRawType(): string;
    /**
     * @description Returns the string representation of the value
     */
    toString(): string;
    /**
     * @description Encodes the value as a Uint8Array as per the SCALE specifications
     * @param isBare true when the value has none of the type-specific prefixes (internal)
     */
    toU8a(isBare?: BareOpts): Uint8Array;
}
export {};
