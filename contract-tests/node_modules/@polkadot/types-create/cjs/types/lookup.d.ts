import type { Option, Text } from '@polkadot/types-codec';
import type { ICompact, INumber, LookupString } from '@polkadot/types-codec/types';
import type { TypeDef } from './types.js';
interface SiTypeBase {
    def: {
        asTuple: ICompact<INumber>[];
    };
}
export interface ILookup {
    getSiType(lookupId: ICompact<INumber> | LookupString | number): SiTypeBase;
    getTypeDef(lookupId: ICompact<INumber> | LookupString | number): TypeDef;
    sanitizeField(name: Option<Text>): [string | null, string | null];
}
export {};
