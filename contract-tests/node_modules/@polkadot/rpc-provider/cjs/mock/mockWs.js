"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.TEST_WS_URL = void 0;
exports.mockWs = mockWs;
const mock_socket_1 = require("mock-socket");
const util_1 = require("@polkadot/util");
global.WebSocket = mock_socket_1.WebSocket;
exports.TEST_WS_URL = 'ws://localhost:9955';
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
function mockWs(requests, wsUrl = exports.TEST_WS_URL) {
    const server = new mock_socket_1.Server(wsUrl);
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
            socket.send((0, util_1.stringify)(response));
        });
    });
    return scope;
}
