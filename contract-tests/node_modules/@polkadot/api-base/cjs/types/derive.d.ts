import type { Observable } from 'rxjs';
type DeriveCreator = (instanceId: string, api: unknown) => (...args: unknown[]) => Observable<any>;
export type DeriveCustom = Record<string, Record<string, DeriveCreator>>;
export {};
