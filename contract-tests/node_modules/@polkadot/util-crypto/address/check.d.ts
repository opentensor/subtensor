import type { Prefix } from './types.js';
/**
 * @name checkAddress
 * @summary Validates an ss58 address.
 * @description
 * From the provided input, validate that the address is a valid input.
 */
export declare function checkAddress(address: string, prefix: Prefix): [boolean, string | null];
