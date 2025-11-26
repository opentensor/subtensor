export { packageInfo } from './packageInfo.js';
/**
 * @internal
 *
 * A getRandomValues util that detects and uses the available RN
 * random utiliy generation functions.
 **/
declare function getRandomValuesRn(output: Uint8Array): Uint8Array;
export declare const getRandomValues: typeof getRandomValuesRn;
export declare const crypto: Crypto | {
    getRandomValues: typeof getRandomValuesRn;
};
