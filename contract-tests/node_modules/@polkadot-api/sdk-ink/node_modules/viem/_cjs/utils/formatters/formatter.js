"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.defineFormatter = defineFormatter;
function defineFormatter(type, format) {
    return ({ exclude, format: overrides, }) => {
        return {
            exclude,
            format: (args, action) => {
                const formatted = format(args, action);
                if (exclude) {
                    for (const key of exclude) {
                        delete formatted[key];
                    }
                }
                return {
                    ...formatted,
                    ...overrides(args, action),
                };
            },
            type,
        };
    };
}
//# sourceMappingURL=formatter.js.map