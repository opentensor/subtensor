import type { DefaultCapabilitiesSchema } from './capabilities.js';
export interface Register {
}
export type ResolvedRegister = {
    CapabilitiesSchema: Register extends {
        CapabilitiesSchema: infer schema;
    } ? schema : DefaultRegister['CapabilitiesSchema'];
};
/** @internal */
type DefaultRegister = {
    CapabilitiesSchema: DefaultCapabilitiesSchema;
};
export {};
//# sourceMappingURL=register.d.ts.map