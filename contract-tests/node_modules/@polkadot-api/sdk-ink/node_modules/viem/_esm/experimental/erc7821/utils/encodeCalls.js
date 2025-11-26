import * as AbiParameters from 'ox/AbiParameters';
import { encodeFunctionData, } from '../../../utils/abi/encodeFunctionData.js';
export function encodeCalls(calls_, opData) {
    const calls = calls_.map((call_) => {
        const call = call_;
        return {
            data: call.abi ? encodeFunctionData(call) : (call.data ?? '0x'),
            value: call.value ?? 0n,
            target: call.to,
        };
    });
    return AbiParameters.encode(AbiParameters.from([
        'struct Call { address target; uint256 value; bytes data; }',
        'Call[] calls',
        ...(opData ? ['bytes opData'] : []),
    ]), [calls, ...(opData ? [opData] : [])]);
}
//# sourceMappingURL=encodeCalls.js.map