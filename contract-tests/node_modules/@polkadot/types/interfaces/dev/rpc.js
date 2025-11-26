export const rpc = {
    getBlockStats: {
        description: 'Reexecute the specified `block_hash` and gather statistics while doing so',
        isUnsafe: true,
        params: [
            {
                isHistoric: true,
                name: 'at',
                type: 'Hash'
            }
        ],
        type: 'Option<BlockStats>'
    }
};
