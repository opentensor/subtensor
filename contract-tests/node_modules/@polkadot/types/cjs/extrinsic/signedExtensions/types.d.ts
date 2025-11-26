export type ExtTypes = Record<string, string>;
export interface ExtInfo {
    extrinsic: ExtTypes;
    payload: ExtTypes;
}
export type ExtDef = Record<string, ExtInfo>;
