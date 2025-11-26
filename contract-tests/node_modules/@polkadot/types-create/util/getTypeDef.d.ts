import type { AnyString } from '@polkadot/types-codec/types';
import type { TypeDef } from '@polkadot/types-create/types';
interface TypeDefOptions {
    name?: string;
    displayName?: string;
}
export declare function getTypeDef(_type: AnyString, { displayName, name }?: TypeDefOptions, count?: number): TypeDef;
export {};
