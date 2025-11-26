import { Server, WebSocket } from 'mock-socket';
import { stringify } from '@polkadot/util';
global.WebSocket = WebSocket;
export const TEST_WS_URL = 'ws://localhost:9955';
function createError({ error: { code, message }, id }) {
    return {
        error: {
            code,
            message
        },
        id,
        jsonrpc: '2.0'
    };
}
function createReply({ id, reply: { result } }) {
    return {
        id,
        jsonrpc: '2.0',
        result
    };
}
export function mockWs(requests, wsUrl = TEST_WS_URL) {
    const server = new Server(wsUrl);
    let requestCount = 0;
    const scope = {
        body: {},
        done: () => new Promise((resolve) => server.stop(resolve)),
        requests: 0,
        server
    };
    server.on('connection', (socket) => {
        socket.on('message', (body) => {
            const request = requests[requestCount];
            const response = request.error
                ? createError(request)
                : createReply(request);
            scope.body[request.method] = body;
            requestCount++;
            socket.send(stringify(response));
        });
    });
    return scope;
}
