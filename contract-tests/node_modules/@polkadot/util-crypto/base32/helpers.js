import { u8aToU8a } from '@polkadot/util';
/** @internal */
export function createDecode({ coder, ipfs }, validate) {
    return (value, ipfsCompat) => {
        validate(value, ipfsCompat);
        return coder.decode(ipfs && ipfsCompat
            ? value.substring(1)
            : value);
    };
}
/** @internal */
export function createEncode({ coder, ipfs }) {
    return (value, ipfsCompat) => {
        const out = coder.encode(u8aToU8a(value));
        return ipfs && ipfsCompat
            ? `${ipfs}${out}`
            : out;
    };
}
/** @internal */
export function createIs(validate) {
    return (value, ipfsCompat) => {
        try {
            return validate(value, ipfsCompat);
        }
        catch {
            return false;
        }
    };
}
/** @internal */
export function createValidate({ chars, ipfs, type, withPadding }) {
    return (value, ipfsCompat) => {
        if (typeof value !== 'string') {
            throw new Error(`Expected ${type} string input`);
        }
        else if (ipfs && ipfsCompat && !value.startsWith(ipfs)) {
            throw new Error(`Expected ipfs-compatible ${type} to start with '${ipfs}'`);
        }
        for (let i = (ipfsCompat ? 1 : 0), count = value.length; i < count; i++) {
            if (chars.includes(value[i])) {
                // all ok, character found
            }
            else if (withPadding && value[i] === '=') {
                if (i === count - 1) {
                    // last character, everything ok
                }
                else if (value[i + 1] === '=') {
                    // next one is also padding, sequence ok
                }
                else {
                    throw new Error(`Invalid ${type} padding sequence "${value[i]}${value[i + 1]}" at index ${i}`);
                }
            }
            else {
                throw new Error(`Invalid ${type} character "${value[i]}" (0x${value.charCodeAt(i).toString(16)}) at index ${i}`);
            }
        }
        return true;
    };
}
