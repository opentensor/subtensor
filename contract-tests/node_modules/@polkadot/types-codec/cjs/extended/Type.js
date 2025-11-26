"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.Type = void 0;
const Text_js_1 = require("../native/Text.js");
const index_js_1 = require("../utils/index.js");
/**
 * @name Type
 * @description
 * This is a extended version of Text, specifically to handle types. Here we rely fully
 * on what Text provides us, however we also adjust the types received from the runtime,
 * i.e. we remove the `T::` prefixes found in some types for consistency across implementation.
 */
class Type extends Text_js_1.Text {
    constructor(registry, value = '') {
        super(registry, value);
        this.setOverride((0, index_js_1.sanitize)(this.toString()));
    }
    /**
     * @description Returns the base runtime type name for this instance
     */
    toRawType() {
        return 'Type';
    }
}
exports.Type = Type;
