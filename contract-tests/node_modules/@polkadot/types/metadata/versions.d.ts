export declare const KNOWN_VERSIONS: readonly [16, 15, 14, 13, 12, 11, 10, 9];
export declare const LATEST_VERSION: 16;
export declare const TO_CALLS_VERSION = 14;
export type MetaVersionAll = typeof KNOWN_VERSIONS[number];
export type MetaVersionAsX = `asV${MetaVersionAll}`;
