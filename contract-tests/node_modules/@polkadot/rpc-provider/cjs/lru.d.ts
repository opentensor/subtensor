export declare const DEFAULT_CAPACITY = 1024;
export declare const DEFAULT_TTL = 30000;
export declare class LRUCache {
    #private;
    readonly capacity: number;
    constructor(capacity?: number, ttl?: number | null);
    get ttl(): number | null;
    get length(): number;
    get lengthData(): number;
    get lengthRefs(): number;
    entries(): [string, unknown][];
    keys(): string[];
    get<T>(key: string): T | null;
    set<T>(key: string, value: T): void;
}
