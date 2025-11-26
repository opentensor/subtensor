import type { AnyU8a, Registry } from '@polkadot/types-codec/types';
import { U8aFixed } from '@polkadot/types-codec';
declare class BaseAccountId extends U8aFixed {
    constructor(registry: Registry, allowedBits?: number, value?: AnyU8a);
    /**
     * @description Compares the value of the input to see if there is a match
     */
    eq(other?: unknown): boolean;
    /**
     * @description Converts the Object to to a human-friendly JSON, with additional fields, expansion and formatting of information
     */
    toHuman(): string;
    /**
     * @description Converts the Object to JSON, typically used for RPC transfers
     */
    toJSON(): string;
    /**
     * @description Converts the value in a best-fit primitive form
     */
    toPrimitive(): string;
    /**
     * @description Returns the string representation of the value
     */
    toString(): string;
    /**
     * @description Returns the base runtime type name for this instance
     */
    toRawType(): string;
}
/**
 * @name GenericAccountId
 * @description
 * A wrapper around an AccountId/PublicKey representation. Since we are dealing with
 * underlying PublicKeys (32 bytes in length), we extend from U8aFixed which is
 * just a Uint8Array wrapper with a fixed length.
 * If constructed with an empty value ([], "", undefined) it will result in
 * the zero account 0x000...000.
 */
export declare class GenericAccountId extends BaseAccountId {
    constructor(registry: Registry, value?: AnyU8a);
}
export declare class GenericAccountId33 extends BaseAccountId {
    constructor(registry: Registry, value?: AnyU8a);
}
export {};
