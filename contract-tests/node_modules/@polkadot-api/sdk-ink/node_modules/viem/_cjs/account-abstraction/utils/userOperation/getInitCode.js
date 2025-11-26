"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.getInitCode = getInitCode;
const concat_js_1 = require("../../../utils/data/concat.js");
function getInitCode(userOperation) {
    const { authorization, factory, factoryData } = userOperation;
    if (factory === '0x7702' ||
        factory === '0x7702000000000000000000000000000000000000') {
        if (!authorization)
            return '0x7702000000000000000000000000000000000000';
        const delegation = authorization.address;
        return (0, concat_js_1.concat)([delegation, factoryData ?? '0x']);
    }
    if (!factory)
        return '0x';
    return (0, concat_js_1.concat)([factory, factoryData ?? '0x']);
}
//# sourceMappingURL=getInitCode.js.map