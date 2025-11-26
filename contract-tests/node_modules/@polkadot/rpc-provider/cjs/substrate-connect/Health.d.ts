import type { HealthChecker } from './types.js';
export declare function healthChecker(): HealthChecker;
export declare class HealthCheckError extends Error {
    #private;
    getCause(): unknown;
    constructor(response: unknown, message?: string);
}
