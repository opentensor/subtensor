import { createRuntimeFunction } from './util.js';
const prefix = 'Substrate';
const section = 'substrate';
function createSubstrateFn(method, key, meta) {
    return createRuntimeFunction({ method, prefix, section }, key, meta);
}
export const substrate = {
    changesTrieConfig: createSubstrateFn('changesTrieConfig', ':changes_trie', {
        docs: 'Changes trie configuration is stored under this key.',
        type: 'u32'
    }),
    childStorageKeyPrefix: createSubstrateFn('childStorageKeyPrefix', ':child_storage:', {
        docs: 'Prefix of child storage keys.',
        type: 'u32'
    }),
    code: createSubstrateFn('code', ':code', {
        docs: 'Wasm code of the runtime.',
        type: 'Bytes'
    }),
    defaultChildStorageKeyPrefix: createSubstrateFn('defaultChildStorageKeyPrefix', ':child_storage:default:', {
        docs: 'Prefix of the default child storage keys in the top trie.',
        type: 'u32'
    }),
    extrinsicIndex: createSubstrateFn('extrinsicIndex', ':extrinsic_index', {
        docs: 'Current extrinsic index (u32) is stored under this key.',
        type: 'u32'
    }),
    heapPages: createSubstrateFn('heapPages', ':heappages', {
        docs: 'Number of wasm linear memory pages required for execution of the runtime.',
        type: 'u64'
    }),
    intrablockEntropy: createSubstrateFn('intrablockEntropy', ':intrablock_entropy', {
        docs: 'Current intra-block entropy (a universally unique `[u8; 32]` value) is stored here.',
        type: '[u8; 32]'
    }),
    storageVersionStorageKeyPostfix: createSubstrateFn('storageVersionStorageKeyPostfix', ':__STORAGE_VERSION__:', {
        docs: 'The storage key postfix that is used to store the [`StorageVersion`] per pallet.',
        type: 'u16'
    }),
    transactionLevelKey: createSubstrateFn('transactionLevelKey', ':transaction_level:', {
        docs: 'The key that holds the current number of active layers.',
        type: 'u32'
    })
};
