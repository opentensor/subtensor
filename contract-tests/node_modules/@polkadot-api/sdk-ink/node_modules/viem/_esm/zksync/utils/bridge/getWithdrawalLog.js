import { getTransactionReceipt } from '../../../actions/index.js';
import { isAddressEqual, toFunctionHash } from '../../../utils/index.js';
import { l1MessengerAddress } from '../../constants/address.js';
/** @internal */
export async function getWithdrawalLog(client, parameters) {
    const { hash, index = 0 } = parameters;
    const receipt = (await getTransactionReceipt(client, {
        hash,
    }));
    const log = receipt.logs.filter((log) => isAddressEqual(log.address, l1MessengerAddress) &&
        log.topics[0] === toFunctionHash('L1MessageSent(address,bytes32,bytes)'))[index];
    return {
        log,
        l1BatchTxId: receipt.l1BatchTxIndex,
    };
}
//# sourceMappingURL=getWithdrawalLog.js.map