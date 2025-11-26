import type { Option, Text, Vec } from '@polkadot/types-codec';
import type { LookupString, Registry } from '@polkadot/types-codec/types';
import type { ILookup, TypeDef } from '@polkadot/types-create/types';
import type { PortableType } from '../../interfaces/metadata/index.js';
import type { SiLookupTypeId, SiType, SiTypeParameter } from '../../interfaces/scaleInfo/index.js';
import { Struct } from '@polkadot/types-codec';
interface TypeInfoParams {
    FrameSystemEventRecord: [event: SiTypeParameter, topic: SiTypeParameter];
    SpRuntimeUncheckedExtrinsic: [address: SiTypeParameter, call: SiTypeParameter, signature: SiTypeParameter, extra: SiTypeParameter];
    [key: string]: SiTypeParameter[];
}
export declare class PortableRegistry extends Struct implements ILookup {
    #private;
    constructor(registry: Registry, value?: Uint8Array, isContract?: boolean);
    /**
     * @description Returns all the available type names for this chain
     **/
    get names(): string[];
    /**
     * @description Returns all the available parameterized types for this chain
     **/
    get paramTypes(): TypeInfoParams;
    /**
     * @description The types of the registry
     */
    get types(): Vec<PortableType>;
    /**
     * @description Register all available types into the registry (generally for internal usage)
     */
    register(): void;
    /**
     * @description Returns the name for a specific lookup
     */
    getName(lookupId: SiLookupTypeId | LookupString | number): string | undefined;
    /**
     * @description Finds a specific type in the registry
     */
    getSiType(lookupId: SiLookupTypeId | LookupString | number): SiType;
    /**
     * @description Lookup the type definition for the index
     */
    getTypeDef(lookupId: SiLookupTypeId | LookupString | number): TypeDef;
    /**
     * @description For a specific field, perform adjustments to not have built-in conflicts
     */
    sanitizeField(name: Option<Text>): [string | null, string | null];
}
export {};
