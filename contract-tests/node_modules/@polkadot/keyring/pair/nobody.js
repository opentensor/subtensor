const publicKey = new Uint8Array(32);
const address = '5C4hrfjw9DjXZTzV3MwzrrAr9P1MJhSrvWGWqi1eSuyUpnhM';
const meta = {
    isTesting: true,
    name: 'nobody'
};
const json = {
    address,
    encoded: '',
    encoding: {
        content: ['pkcs8', 'ed25519'],
        type: 'none',
        version: '0'
    },
    meta
};
const pair = {
    address,
    addressRaw: publicKey,
    decodePkcs8: (_passphrase, _encoded) => undefined,
    derive: (_suri, _meta) => pair,
    encodePkcs8: (_passphrase) => new Uint8Array(0),
    isLocked: true,
    lock: () => {
        // no locking, it is always locked
    },
    meta,
    publicKey,
    setMeta: (_meta) => undefined,
    sign: (_message) => new Uint8Array(64),
    toJson: (_passphrase) => json,
    type: 'ed25519',
    unlock: (_passphrase) => undefined,
    verify: (_message, _signature) => false,
    vrfSign: (_message, _context, _extra) => new Uint8Array(96),
    vrfVerify: (_message, _vrfResult, _context, _extra) => false
};
export function nobody() {
    return pair;
}
