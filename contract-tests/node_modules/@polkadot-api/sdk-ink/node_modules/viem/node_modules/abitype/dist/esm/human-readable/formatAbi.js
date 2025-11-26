import { formatAbiItem } from './formatAbiItem.js';
/**
 * Parses JSON ABI into human-readable ABI
 *
 * @param abi - ABI
 * @returns Human-readable ABI
 */
export function formatAbi(abi) {
    const signatures = [];
    const length = abi.length;
    for (let i = 0; i < length; i++) {
        const abiItem = abi[i];
        const signature = formatAbiItem(abiItem);
        signatures.push(signature);
    }
    return signatures;
}
//# sourceMappingURL=formatAbi.js.map