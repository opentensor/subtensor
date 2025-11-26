import type { CodecClass, Registry } from '../types/index.js';
export declare function typesToMap(registry: Registry, [Types, keys]: [CodecClass[], string[]]): Record<string, string>;
