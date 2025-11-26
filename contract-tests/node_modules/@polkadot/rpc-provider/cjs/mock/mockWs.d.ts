import { Server } from 'mock-socket';
interface Scope {
    body: Record<string, Record<string, unknown>>;
    requests: number;
    server: Server;
    done: any;
}
interface ErrorDef {
    id: number;
    error: {
        code: number;
        message: string;
    };
}
interface ReplyDef {
    id: number;
    reply: {
        result: unknown;
    };
}
export type Request = {
    method: string;
} & (ErrorDef | ReplyDef);
export declare const TEST_WS_URL = "ws://localhost:9955";
export declare function mockWs(requests: Request[], wsUrl?: string): Scope;
export {};
