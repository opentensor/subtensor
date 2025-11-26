declare function toHex(bytes: Uint8Array): string;
declare function fromHex(hexString: string): Uint8Array;

declare function mapObject<K extends string | number | symbol, I, O>(input: Record<K, I>, mapper: (i: I, k: K) => O): Record<K, O>;
type StringRecord<T> = {
    [Sym: symbol]: never;
    [Num: number]: never;
    [Str: string]: T;
};
declare const mapStringRecord: <I, O>(input: StringRecord<I>, mapper: (value: I, key: string) => O) => StringRecord<O>;

declare function filterObject<K extends string | number | symbol, I>(input: Record<K, I>, filterFn: (i: I, k: K) => boolean): Record<K, I>;

interface MergeUint8 {
    /**
     * @deprecated This overload will be removed in PAPI v2. Migrate as
     *             follows:
     *             mergeUint8(arr1, arr2) => mergeUint8([arr1, arr2])
     */
    (...inputs: Array<Uint8Array>): Uint8Array;
    (inputs: Array<Uint8Array>): Uint8Array;
}
declare const mergeUint8: MergeUint8;

declare const noop: () => void;

declare class AbortError extends Error {
    constructor();
}

declare const jsonPrint: (value: any, indent?: number) => string;

export { AbortError, filterObject, fromHex, jsonPrint, mapObject, mapStringRecord, mergeUint8, noop, toHex };
