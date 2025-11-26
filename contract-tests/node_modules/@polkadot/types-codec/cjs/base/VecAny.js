"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.VecAny = void 0;
const Array_js_1 = require("../abstract/Array.js");
/**
 * @name VecAny
 * @description
 * This manages codec arrays, assuming that the inputs are already of type Codec. Unlike
 * a vector, this can be used to manage array-like structures with variable arguments of
 * any types
 */
class VecAny extends Array_js_1.AbstractArray {
    /**
     * @description Returns the base runtime type name for this instance
     */
    toRawType() {
        // FIXME This is basically an any type, cannot instantiate via createType
        return 'Vec<Codec>';
    }
}
exports.VecAny = VecAny;
