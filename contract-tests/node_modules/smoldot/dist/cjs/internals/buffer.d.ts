export declare function utf8BytesToString(buffer: Uint8Array, offset: number, length: number): string;
export declare function readUInt8(buffer: Uint8Array, offset: number): number;
export declare function readUInt16BE(buffer: Uint8Array, offset: number): number;
export declare function readUInt32LE(buffer: Uint8Array, offset: number): number;
/**
 * Sets the value of a given byte in the buffer.
 *
 * This function is equivalent to `buffer[offset] = value`, except that an exception is thrown
 * if `offset` is out of range.
 */
export declare function writeUInt8(buffer: Uint8Array, offset: number, value: number): void;
export declare function writeUInt32LE(buffer: Uint8Array, offset: number, value: number): void;
export declare function writeUInt64LE(buffer: Uint8Array, offset: number, value: bigint): void;
