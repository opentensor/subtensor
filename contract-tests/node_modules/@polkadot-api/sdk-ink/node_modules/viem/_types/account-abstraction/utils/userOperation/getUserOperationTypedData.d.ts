import type { Address } from 'abitype';
import type { TypedDataDefinition } from '../../../types/typedData.js';
import type { UserOperation } from '../../types/userOperation.js';
export type GetUserOperationTypedDataParameters = {
    chainId: number;
    entryPointAddress: Address;
    userOperation: UserOperation<'0.8'>;
};
export type GetUserOperationTypedDataReturnType = TypedDataDefinition<typeof types, 'PackedUserOperation'>;
declare const types: {
    readonly PackedUserOperation: readonly [{
        readonly type: "address";
        readonly name: "sender";
    }, {
        readonly type: "uint256";
        readonly name: "nonce";
    }, {
        readonly type: "bytes";
        readonly name: "initCode";
    }, {
        readonly type: "bytes";
        readonly name: "callData";
    }, {
        readonly type: "bytes32";
        readonly name: "accountGasLimits";
    }, {
        readonly type: "uint256";
        readonly name: "preVerificationGas";
    }, {
        readonly type: "bytes32";
        readonly name: "gasFees";
    }, {
        readonly type: "bytes";
        readonly name: "paymasterAndData";
    }];
};
export declare function getUserOperationTypedData(parameters: GetUserOperationTypedDataParameters): GetUserOperationTypedDataReturnType;
export {};
//# sourceMappingURL=getUserOperationTypedData.d.ts.map