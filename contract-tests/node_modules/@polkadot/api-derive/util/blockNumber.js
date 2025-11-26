import { isCompact } from '@polkadot/util';
export function unwrapBlockNumber(hdr) {
    return isCompact(hdr.number)
        ? hdr.number.unwrap()
        : hdr.number;
}
