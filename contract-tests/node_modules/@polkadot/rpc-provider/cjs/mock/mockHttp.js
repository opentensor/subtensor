"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.TEST_HTTP_URL = void 0;
exports.mockHttp = mockHttp;
const tslib_1 = require("tslib");
const nock_1 = tslib_1.__importDefault(require("nock"));
exports.TEST_HTTP_URL = 'http://localhost:9944';
function mockHttp(requests) {
    nock_1.default.cleanAll();
    return requests.reduce((scope, request) => scope
        .post('/')
        .reply(request.code || 200, (_uri, body) => {
        scope.body = scope.body || {};
        scope.body[request.method] = body;
        return Object.assign({ id: body.id, jsonrpc: '2.0' }, request.reply || {});
    }), (0, nock_1.default)(exports.TEST_HTTP_URL));
}
