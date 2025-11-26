"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.TypedData = exports.TypedDataParameter = exports.TypedDataType = exports.TypedDataDomain = exports.Abi = exports.AbiItemType = exports.AbiError = exports.AbiEvent = exports.AbiReceive = exports.AbiFallback = exports.AbiConstructor = exports.AbiFunction = exports.AbiStateMutability = exports.AbiEventParameter = exports.AbiParameter = exports.SolidityArray = exports.SolidityArrayWithTuple = exports.SolidityArrayWithoutTuple = exports.SolidityInt = exports.SolidityTuple = exports.SolidityString = exports.SolidityFunction = exports.SolidityBytes = exports.SolidityBool = exports.SolidityAddress = exports.Address = void 0;
const zod_1 = require("zod");
const utils_js_1 = require("./human-readable/runtime/utils.js");
const regex_js_1 = require("./regex.js");
const Identifier = zod_1.z.string().regex(/[a-zA-Z$_][a-zA-Z0-9$_]*/);
exports.Address = zod_1.z.string().transform((val, ctx) => {
    const regex = /^0x[a-fA-F0-9]{40}$/;
    if (!regex.test(val)) {
        ctx.addIssue({
            code: zod_1.z.ZodIssueCode.custom,
            message: `Invalid Address ${val}`,
        });
    }
    return val;
});
exports.SolidityAddress = zod_1.z.literal('address');
exports.SolidityBool = zod_1.z.literal('bool');
exports.SolidityBytes = zod_1.z.string().regex(regex_js_1.bytesRegex);
exports.SolidityFunction = zod_1.z.literal('function');
exports.SolidityString = zod_1.z.literal('string');
exports.SolidityTuple = zod_1.z.literal('tuple');
exports.SolidityInt = zod_1.z.string().regex(regex_js_1.integerRegex);
exports.SolidityArrayWithoutTuple = zod_1.z
    .string()
    .regex(/^(address|bool|function|string|bytes([1-9]|1[0-9]|2[0-9]|3[0-2])?|u?int(8|16|24|32|40|48|56|64|72|80|88|96|104|112|120|128|136|144|152|160|168|176|184|192|200|208|216|224|232|240|248|256)?)(\[[0-9]{0,}\])+$/);
exports.SolidityArrayWithTuple = zod_1.z
    .string()
    .regex(/^tuple(\[[0-9]{0,}\])+$/);
exports.SolidityArray = zod_1.z.union([
    exports.SolidityArrayWithTuple,
    exports.SolidityArrayWithoutTuple,
]);
exports.AbiParameter = zod_1.z.lazy(() => zod_1.z.intersection(zod_1.z.object({
    name: zod_1.z.union([Identifier.optional(), zod_1.z.literal('')]),
    internalType: zod_1.z.string().optional(),
}), zod_1.z.union([
    zod_1.z.object({
        type: zod_1.z.union([
            exports.SolidityAddress,
            exports.SolidityBool,
            exports.SolidityBytes,
            exports.SolidityFunction,
            exports.SolidityString,
            exports.SolidityInt,
            exports.SolidityArrayWithoutTuple,
        ]),
    }),
    zod_1.z.object({
        type: zod_1.z.union([exports.SolidityTuple, exports.SolidityArrayWithTuple]),
        components: zod_1.z.array(exports.AbiParameter).readonly(),
    }),
])));
exports.AbiEventParameter = zod_1.z.intersection(exports.AbiParameter, zod_1.z.object({ indexed: zod_1.z.boolean().optional() }));
exports.AbiStateMutability = zod_1.z.union([
    zod_1.z.literal('pure'),
    zod_1.z.literal('view'),
    zod_1.z.literal('nonpayable'),
    zod_1.z.literal('payable'),
]);
exports.AbiFunction = zod_1.z.preprocess((val) => {
    const abiFunction = val;
    if (abiFunction.stateMutability === undefined) {
        if (abiFunction.constant)
            abiFunction.stateMutability = 'view';
        else if (abiFunction.payable)
            abiFunction.stateMutability = 'payable';
        else
            abiFunction.stateMutability = 'nonpayable';
    }
    return val;
}, zod_1.z.object({
    type: zod_1.z.literal('function'),
    constant: zod_1.z.boolean().optional(),
    gas: zod_1.z.number().optional(),
    inputs: zod_1.z.array(exports.AbiParameter).readonly(),
    name: Identifier,
    outputs: zod_1.z.array(exports.AbiParameter).readonly(),
    payable: zod_1.z.boolean().optional(),
    stateMutability: exports.AbiStateMutability,
}));
exports.AbiConstructor = zod_1.z.preprocess((val) => {
    const abiFunction = val;
    if (abiFunction.stateMutability === undefined) {
        if (abiFunction.payable)
            abiFunction.stateMutability = 'payable';
        else
            abiFunction.stateMutability = 'nonpayable';
    }
    return val;
}, zod_1.z.object({
    type: zod_1.z.literal('constructor'),
    inputs: zod_1.z.array(exports.AbiParameter).readonly(),
    payable: zod_1.z.boolean().optional(),
    stateMutability: zod_1.z.union([zod_1.z.literal('nonpayable'), zod_1.z.literal('payable')]),
}));
exports.AbiFallback = zod_1.z.preprocess((val) => {
    const abiFunction = val;
    if (abiFunction.stateMutability === undefined) {
        if (abiFunction.payable)
            abiFunction.stateMutability = 'payable';
        else
            abiFunction.stateMutability = 'nonpayable';
    }
    return val;
}, zod_1.z.object({
    type: zod_1.z.literal('fallback'),
    payable: zod_1.z.boolean().optional(),
    stateMutability: zod_1.z.union([zod_1.z.literal('nonpayable'), zod_1.z.literal('payable')]),
}));
exports.AbiReceive = zod_1.z.object({
    type: zod_1.z.literal('receive'),
    stateMutability: zod_1.z.literal('payable'),
});
exports.AbiEvent = zod_1.z.object({
    type: zod_1.z.literal('event'),
    anonymous: zod_1.z.boolean().optional(),
    inputs: zod_1.z.array(exports.AbiEventParameter).readonly(),
    name: Identifier,
});
exports.AbiError = zod_1.z.object({
    type: zod_1.z.literal('error'),
    inputs: zod_1.z.array(exports.AbiParameter).readonly(),
    name: zod_1.z.string(),
});
exports.AbiItemType = zod_1.z.union([
    zod_1.z.literal('constructor'),
    zod_1.z.literal('event'),
    zod_1.z.literal('error'),
    zod_1.z.literal('fallback'),
    zod_1.z.literal('function'),
    zod_1.z.literal('receive'),
]);
exports.Abi = zod_1.z
    .array(zod_1.z.union([
    exports.AbiError,
    exports.AbiEvent,
    zod_1.z.preprocess((val) => {
        const abiItem = val;
        if (abiItem.type === 'receive')
            return abiItem;
        if (val
            .stateMutability === undefined) {
            if (abiItem.type === 'function' &&
                abiItem.constant)
                abiItem.stateMutability = 'view';
            else if (abiItem.payable)
                abiItem.stateMutability = 'payable';
            else
                abiItem.stateMutability = 'nonpayable';
        }
        return val;
    }, zod_1.z.intersection(zod_1.z.object({
        constant: zod_1.z.boolean().optional(),
        gas: zod_1.z.number().optional(),
        payable: zod_1.z.boolean().optional(),
    }), zod_1.z.discriminatedUnion('type', [
        zod_1.z.object({
            type: zod_1.z.literal('function'),
            inputs: zod_1.z.array(exports.AbiParameter).readonly(),
            name: zod_1.z.string().regex(/[a-zA-Z$_][a-zA-Z0-9$_]*/),
            outputs: zod_1.z.array(exports.AbiParameter).readonly(),
            stateMutability: exports.AbiStateMutability,
        }),
        zod_1.z.object({
            type: zod_1.z.literal('constructor'),
            inputs: zod_1.z.array(exports.AbiParameter).readonly(),
            stateMutability: zod_1.z.union([
                zod_1.z.literal('payable'),
                zod_1.z.literal('nonpayable'),
            ]),
        }),
        zod_1.z.object({
            type: zod_1.z.literal('fallback'),
            inputs: zod_1.z.tuple([]).optional(),
            stateMutability: zod_1.z.union([
                zod_1.z.literal('payable'),
                zod_1.z.literal('nonpayable'),
            ]),
        }),
        zod_1.z.object({
            type: zod_1.z.literal('receive'),
            stateMutability: zod_1.z.literal('payable'),
        }),
    ]))),
]))
    .readonly();
exports.TypedDataDomain = zod_1.z.object({
    chainId: zod_1.z.union([zod_1.z.number(), zod_1.z.bigint()]).optional(),
    name: Identifier.optional(),
    salt: zod_1.z.string().optional(),
    verifyingContract: exports.Address.optional(),
    version: zod_1.z.string().optional(),
});
exports.TypedDataType = zod_1.z.union([
    exports.SolidityAddress,
    exports.SolidityBool,
    exports.SolidityBytes,
    exports.SolidityString,
    exports.SolidityInt,
    exports.SolidityArray,
]);
exports.TypedDataParameter = zod_1.z.object({
    name: Identifier,
    type: zod_1.z.string(),
});
exports.TypedData = zod_1.z
    .record(Identifier, zod_1.z.array(exports.TypedDataParameter))
    .transform((val, ctx) => validateTypedDataKeys(val, ctx));
function validateTypedDataKeys(typedData, zodContext) {
    const keys = Object.keys(typedData);
    for (let i = 0; i < keys.length; i++) {
        if ((0, utils_js_1.isSolidityType)(keys[i])) {
            zodContext.addIssue({
                code: 'custom',
                message: `Invalid key. ${keys[i]} is a solidity type.`,
            });
            return zod_1.z.NEVER;
        }
        validateTypedDataParameters(keys[i], typedData, zodContext);
    }
    return typedData;
}
const typeWithoutTupleRegex = /^(?<type>[a-zA-Z$_][a-zA-Z0-9$_]*?)(?<array>(?:\[\d*?\])+?)?$/;
function validateTypedDataParameters(key, typedData, zodContext, ancestors = new Set()) {
    const val = typedData[key];
    const length = val.length;
    for (let i = 0; i < length; i++) {
        if (val[i]?.type === key) {
            zodContext.addIssue({
                code: 'custom',
                message: `Invalid type. ${key} is a self reference.`,
            });
            return zod_1.z.NEVER;
        }
        const match = (0, regex_js_1.execTyped)(typeWithoutTupleRegex, val[i]?.type);
        if (!match?.type) {
            zodContext.addIssue({
                code: 'custom',
                message: `Invalid type. ${key} does not have a type.`,
            });
            return zod_1.z.NEVER;
        }
        if (match.type in typedData) {
            if (ancestors.has(match.type)) {
                zodContext.addIssue({
                    code: 'custom',
                    message: `Invalid type. ${match.type} is a circular reference.`,
                });
                return zod_1.z.NEVER;
            }
            validateTypedDataParameters(match.type, typedData, zodContext, new Set([...ancestors, match.type]));
        }
        else if (!(0, utils_js_1.isSolidityType)(match.type)) {
            zodContext.addIssue({
                code: 'custom',
                message: `Invalid type. ${match.type} is not a valid EIP-712 type.`,
            });
        }
    }
    return;
}
//# sourceMappingURL=zod.js.map