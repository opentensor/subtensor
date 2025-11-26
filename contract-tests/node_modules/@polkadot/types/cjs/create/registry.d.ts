import type { AnyString, Codec, CodecClass, IU8a, LookupString } from '@polkadot/types-codec/types';
import type { CreateOptions, TypeDef } from '@polkadot/types-create/types';
import type { ExtDef } from '../extrinsic/signedExtensions/types.js';
import type { ChainProperties, DispatchErrorModule, DispatchErrorModuleU8, DispatchErrorModuleU8a, Hash, MetadataLatest, SiLookupTypeId } from '../interfaces/types.js';
import type { CallFunction, CodecHasher, DetectCodec, RegisteredTypes, Registry, RegistryError, RegistryTypes } from '../types/index.js';
import { GenericEventData } from '../generic/Event.js';
import { Metadata } from '../metadata/Metadata.js';
import { PortableRegistry } from '../metadata/PortableRegistry/index.js';
export declare class TypeRegistry implements Registry {
    #private;
    createdAtHash?: Hash;
    constructor(createdAtHash?: Hash | Uint8Array | string);
    get chainDecimals(): number[];
    get chainIsEthereum(): boolean;
    get chainSS58(): number | undefined;
    get chainTokens(): string[];
    get firstCallIndex(): Uint8Array;
    /**
     * @description Returns true if the type is in a Compat format
     */
    isLookupType(value: string): value is LookupString;
    /**
     * @description Creates a lookup string from the supplied id
     */
    createLookupType(lookupId: SiLookupTypeId | number): LookupString;
    get knownTypes(): RegisteredTypes;
    get lookup(): PortableRegistry;
    get metadata(): MetadataLatest;
    get unknownTypes(): string[];
    get signedExtensions(): string[];
    clearCache(): void;
    /**
     * @describe Creates an instance of the class
     */
    createClass<T extends Codec = Codec, K extends string = string>(type: K): CodecClass<DetectCodec<T, K>>;
    /**
     * @describe Creates an instance of the class
     */
    createClassUnsafe<T extends Codec = Codec, K extends string = string>(type: K): CodecClass<T>;
    /**
     * @description Creates an instance of a type as registered
     */
    createType<T extends Codec = Codec, K extends string = string>(type: K, ...params: unknown[]): DetectCodec<T, K>;
    /**
     * @description Creates an instance of a type as registered
     */
    createTypeUnsafe<T extends Codec = Codec, K extends string = string>(type: K, params: unknown[], options?: CreateOptions): T;
    findMetaCall(callIndex: Uint8Array): CallFunction;
    findMetaError(errorIndex: Uint8Array | DispatchErrorModule | DispatchErrorModuleU8 | DispatchErrorModuleU8a): RegistryError;
    findMetaEvent(eventIndex: Uint8Array): CodecClass<GenericEventData>;
    get<T extends Codec = Codec, K extends string = string>(name: K, withUnknown?: boolean, knownTypeDef?: TypeDef): CodecClass<T> | undefined;
    getUnsafe<T extends Codec = Codec, K extends string = string>(name: K, withUnknown?: boolean, knownTypeDef?: TypeDef): CodecClass<T> | undefined;
    getChainProperties(): ChainProperties | undefined;
    getClassName(Type: CodecClass): string | undefined;
    getDefinition(typeName: string): string | undefined;
    getModuleInstances(specName: AnyString, moduleName: string): string[] | undefined;
    getOrThrow<T extends Codec = Codec, K extends string = string, R = DetectCodec<T, K>>(name: K): CodecClass<R>;
    getOrUnknown<T extends Codec = Codec, K extends string = string, R = DetectCodec<T, K>>(name: K): CodecClass<R>;
    getTransactionExtensionVersion(): number;
    getSignedExtensionExtra(): Record<string, string>;
    getSignedExtensionTypes(): Record<string, string>;
    hasClass(name: string): boolean;
    hasDef(name: string): boolean;
    hasType(name: string): boolean;
    hash(data: Uint8Array): IU8a;
    register(type: CodecClass | RegistryTypes): void;
    register(name: string, type: CodecClass): void;
    setChainProperties(properties?: ChainProperties): void;
    setHasher(hasher?: CodecHasher | null): void;
    setKnownTypes(knownTypes: RegisteredTypes): void;
    setLookup(lookup: PortableRegistry): void;
    setMetadata(metadata: Metadata, signedExtensions?: string[], userExtensions?: ExtDef, noInitWarn?: boolean): void;
    setSignedExtensions(signedExtensions?: string[], userExtensions?: ExtDef, noInitWarn?: boolean): void;
}
