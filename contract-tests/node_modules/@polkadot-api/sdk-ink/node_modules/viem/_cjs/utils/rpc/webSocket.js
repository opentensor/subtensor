"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.getWebSocketRpcClient = getWebSocketRpcClient;
const request_js_1 = require("../../errors/request.js");
const socket_js_1 = require("./socket.js");
async function getWebSocketRpcClient(url, options = {}) {
    const { keepAlive, reconnect } = options;
    return (0, socket_js_1.getSocketRpcClient)({
        async getSocket({ onClose, onError, onOpen, onResponse }) {
            const WebSocket = await Promise.resolve().then(() => require('isows')).then((module) => module.WebSocket);
            const socket = new WebSocket(url);
            function onClose_() {
                socket.removeEventListener('close', onClose_);
                socket.removeEventListener('message', onMessage);
                socket.removeEventListener('error', onError);
                socket.removeEventListener('open', onOpen);
                onClose();
            }
            function onMessage({ data }) {
                if (typeof data === 'string' && data.trim().length === 0)
                    return;
                try {
                    const _data = JSON.parse(data);
                    onResponse(_data);
                }
                catch (error) {
                    onError(error);
                }
            }
            socket.addEventListener('close', onClose_);
            socket.addEventListener('message', onMessage);
            socket.addEventListener('error', onError);
            socket.addEventListener('open', onOpen);
            if (socket.readyState === WebSocket.CONNECTING) {
                await new Promise((resolve, reject) => {
                    if (!socket)
                        return;
                    socket.onopen = resolve;
                    socket.onerror = reject;
                });
            }
            const { close: close_ } = socket;
            return Object.assign(socket, {
                close() {
                    close_.bind(socket)();
                    onClose_();
                },
                ping() {
                    try {
                        if (socket.readyState === socket.CLOSED ||
                            socket.readyState === socket.CLOSING)
                            throw new request_js_1.WebSocketRequestError({
                                url: socket.url,
                                cause: new request_js_1.SocketClosedError({ url: socket.url }),
                            });
                        const body = {
                            jsonrpc: '2.0',
                            id: null,
                            method: 'net_version',
                            params: [],
                        };
                        socket.send(JSON.stringify(body));
                    }
                    catch (error) {
                        onError(error);
                    }
                },
                request({ body }) {
                    if (socket.readyState === socket.CLOSED ||
                        socket.readyState === socket.CLOSING)
                        throw new request_js_1.WebSocketRequestError({
                            body,
                            url: socket.url,
                            cause: new request_js_1.SocketClosedError({ url: socket.url }),
                        });
                    return socket.send(JSON.stringify(body));
                },
            });
        },
        keepAlive,
        reconnect,
        url,
    });
}
//# sourceMappingURL=webSocket.js.map