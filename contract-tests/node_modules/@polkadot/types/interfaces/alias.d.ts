import type { OverrideModuleType, Registry } from '../types/index.js';
/**
 * @description Get types for specific modules (metadata override)
 */
export declare function getAliasTypes({ knownTypes }: Registry, section: string): OverrideModuleType;
