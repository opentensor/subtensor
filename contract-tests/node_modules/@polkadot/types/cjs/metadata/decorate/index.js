"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.filterEventsSome = exports.filterCallsSome = exports.decorateStorage = exports.decorateExtrinsics = exports.decorateEvents = exports.decorateErrors = exports.decorateConstants = void 0;
exports.expandMetadata = expandMetadata;
const Metadata_js_1 = require("../Metadata.js");
const index_js_1 = require("./constants/index.js");
Object.defineProperty(exports, "decorateConstants", { enumerable: true, get: function () { return index_js_1.decorateConstants; } });
const index_js_2 = require("./errors/index.js");
Object.defineProperty(exports, "decorateErrors", { enumerable: true, get: function () { return index_js_2.decorateErrors; } });
const index_js_3 = require("./events/index.js");
Object.defineProperty(exports, "decorateEvents", { enumerable: true, get: function () { return index_js_3.decorateEvents; } });
Object.defineProperty(exports, "filterEventsSome", { enumerable: true, get: function () { return index_js_3.filterEventsSome; } });
const index_js_4 = require("./extrinsics/index.js");
Object.defineProperty(exports, "decorateExtrinsics", { enumerable: true, get: function () { return index_js_4.decorateExtrinsics; } });
Object.defineProperty(exports, "filterCallsSome", { enumerable: true, get: function () { return index_js_4.filterCallsSome; } });
const index_js_5 = require("./storage/index.js");
Object.defineProperty(exports, "decorateStorage", { enumerable: true, get: function () { return index_js_5.decorateStorage; } });
/**
 * Expands the metadata by decoration into consts, query and tx sections
 */
function expandMetadata(registry, metadata) {
    if (!(metadata instanceof Metadata_js_1.Metadata)) {
        throw new Error('You need to pass a valid Metadata instance to Decorated');
    }
    const latest = metadata.asLatest;
    const version = metadata.version;
    return {
        consts: (0, index_js_1.decorateConstants)(registry, latest, version),
        errors: (0, index_js_2.decorateErrors)(registry, latest, version),
        events: (0, index_js_3.decorateEvents)(registry, latest, version),
        query: (0, index_js_5.decorateStorage)(registry, latest, version),
        registry,
        tx: (0, index_js_4.decorateExtrinsics)(registry, latest, version)
    };
}
