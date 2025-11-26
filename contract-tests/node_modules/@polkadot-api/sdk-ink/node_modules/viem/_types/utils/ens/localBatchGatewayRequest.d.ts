import type { Hex } from '../../types/misc.js';
import type { CcipRequestParameters, CcipRequestReturnType } from '../ccip.js';
export declare const localBatchGatewayUrl = "x-batch-gateway:true";
export declare function localBatchGatewayRequest(parameters: {
    data: Hex;
    ccipRequest: (parameters: CcipRequestParameters) => Promise<CcipRequestReturnType>;
}): Promise<Hex>;
//# sourceMappingURL=localBatchGatewayRequest.d.ts.map