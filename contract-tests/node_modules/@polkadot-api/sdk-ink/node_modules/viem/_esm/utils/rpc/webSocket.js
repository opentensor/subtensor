import { SocketClosedError, WebSocketRequestError, } from '../../errors/request.js';
import { getSocketRpcClient, } from './socket.js';
export async function getWebSocketRpcClient(url, options = {}) {
    const { keepAlive, reconnect } = options;
    return getSocketRpcClient({
        async getSocket({ onClose, onError, onOpen, onResponse }) {
            const WebSocket = await import('isows').then((module) => module.WebSocket);
            const socket = new WebSocket(url);
            function onClose_() {
                socket.removeEventListener('close', onClose_);
                socket.removeEventListener('message', onMessage);
                socket.removeEventListener('error', onError);
                socket.removeEventListener('open', onOpen);
                onClose();
            }
            function onMessage({ data }) {
                // ignore empty messages
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
            // Setup event listeners for RPC & subscription responses.
            socket.addEventListener('close', onClose_);
            socket.addEventListener('message', onMessage);
            socket.addEventListener('error', onError);
            socket.addEventListener('open', onOpen);
            // Wait for the socket to open.
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
                            throw new WebSocketRequestError({
                                url: socket.url,
                                cause: new SocketClosedError({ url: socket.url }),
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
                        throw new WebSocketRequestError({
                            body,
                            url: socket.url,
                            cause: new SocketClosedError({ url: socket.url }),
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