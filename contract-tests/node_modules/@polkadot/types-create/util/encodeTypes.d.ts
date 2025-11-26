import type { Registry } from '@polkadot/types-codec/types';
import type { TypeDef } from '@polkadot/types-create/types';
interface ToString {
    toString: () => string;
}
export declare function paramsNotation<T extends ToString>(outer: string, inner?: T | T[], transform?: (_: T) => string): string;
export declare function encodeTypeDef(registry: Registry, typeDef: TypeDef): string;
export declare function withTypeString(registry: Registry, typeDef: Omit<TypeDef, 'type'> & {
    type?: string;
}): TypeDef;
export {};
