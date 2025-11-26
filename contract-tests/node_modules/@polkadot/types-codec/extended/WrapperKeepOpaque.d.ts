import type { AnyJson, Codec, CodecClass, Inspect, Registry } from '../types/index.js';
import { Bytes } from './Bytes.js';
type OpaqueName = 'WrapperKeepOpaque' | 'WrapperOpaque';
interface Options {
    opaqueName?: OpaqueName;
}
export declare class WrapperKeepOpaque<T extends Codec> extends Bytes {
    #private;
    constructor(registry: Registry, typeName: CodecClass<T> | string, value?: unknown, { opaqueName }?: Options);
    static with<T extends Codec>(Type: CodecClass<T> | string): CodecClass<WrapperKeepOpaque<T>>;
    /**
     * @description Checks if the wrapper is decodable
     */
    get isDecoded(): boolean;
    /**
     * @description Returns a breakdown of the hex encoding for this Codec
     */
    inspect(): Inspect;
    /**
     * @description Converts the Object to to a human-friendly JSON, with additional fields, expansion and formatting of information
     */
    toHuman(isExtended?: boolean, disableAscii?: boolean): AnyJson;
    /**
     * @description Converts the value in a best-fit primitive form
     */
    toPrimitive(disableAscii?: boolean): any;
    /**
     * @description Returns the base runtime type name for this instance
     */
    toRawType(): string;
    /**
     * @description Converts the Object to to a string (either decoded or bytes)
     */
    toString(): string;
    /**
     * @description Returns the decoded that the WrapperKeepOpaque represents (if available), throws if non-decodable
     */
    unwrap(): T;
}
export {};
