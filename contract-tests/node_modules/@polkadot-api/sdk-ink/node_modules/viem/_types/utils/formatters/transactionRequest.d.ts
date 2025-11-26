import type { ErrorType } from '../../errors/utils.js';
import type { Account } from '../../types/account.js';
import type { Chain, ExtractChainFormatterParameters } from '../../types/chain.js';
import type { RpcTransactionRequest } from '../../types/rpc.js';
import type { TransactionRequest } from '../../types/transaction.js';
import type { ExactPartial } from '../../types/utils.js';
import { type DefineFormatterErrorType } from './formatter.js';
export type FormattedTransactionRequest<chain extends Chain | undefined = Chain | undefined> = ExtractChainFormatterParameters<chain, 'transactionRequest', TransactionRequest>;
export declare const rpcTransactionType: {
    readonly legacy: "0x0";
    readonly eip2930: "0x1";
    readonly eip1559: "0x2";
    readonly eip4844: "0x3";
    readonly eip7702: "0x4";
};
export type FormatTransactionRequestErrorType = ErrorType;
export declare function formatTransactionRequest(request: ExactPartial<TransactionRequest> & {
    account?: Account | undefined;
}, _?: string | undefined): RpcTransactionRequest;
export type DefineTransactionRequestErrorType = DefineFormatterErrorType | ErrorType;
export declare const defineTransactionRequest: <parametersOverride, returnTypeOverride, exclude extends ("gasPrice" | "maxFeePerBlobGas" | "maxFeePerGas" | "maxPriorityFeePerGas" | "type" | "data" | "value" | "gas" | "to" | "from" | "nonce" | "blobs" | "blobVersionedHashes" | "kzg" | "accessList" | "sidecars" | "authorizationList" | "account" | keyof parametersOverride)[] = []>({ exclude, format: overrides, }: {
    exclude?: exclude | undefined;
    format: (args: parametersOverride, action?: string | undefined) => returnTypeOverride;
}) => {
    exclude: exclude | undefined;
    format: (args: parametersOverride, action?: string | undefined) => { [K in keyof returnTypeOverride]: returnTypeOverride[K]; } & { [_key in exclude[number]]: never; };
    type: "transactionRequest";
};
//# sourceMappingURL=transactionRequest.d.ts.map