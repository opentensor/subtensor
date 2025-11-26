import type { Mock } from './types.js';
interface Request {
    code?: number;
    method: string;
    reply?: Record<string, unknown>;
}
export declare const TEST_HTTP_URL = "http://localhost:9944";
export declare function mockHttp(requests: Request[]): Mock;
export {};
