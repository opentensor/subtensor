"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.claimFailedDeposit = claimFailedDeposit;
const abitype_1 = require("abitype");
const getTransaction_js_1 = require("../../actions/public/getTransaction.js");
const getTransactionReceipt_js_1 = require("../../actions/public/getTransactionReceipt.js");
const readContract_js_1 = require("../../actions/public/readContract.js");
const sendTransaction_js_1 = require("../../actions/wallet/sendTransaction.js");
const bytes_js_1 = require("../../constants/bytes.js");
const account_js_1 = require("../../errors/account.js");
const chain_js_1 = require("../../errors/chain.js");
const index_js_1 = require("../../utils/index.js");
const address_js_1 = require("../constants/address.js");
const bridge_js_1 = require("../errors/bridge.js");
const undoL1ToL2Alias_js_1 = require("../utils/bridge/undoL1ToL2Alias.js");
const getDefaultBridgeAddresses_js_1 = require("./getDefaultBridgeAddresses.js");
const getLogProof_js_1 = require("./getLogProof.js");
async function claimFailedDeposit(client, parameters) {
    const { account: account_ = client.account, chain: chain_ = client.chain, client: l2Client, depositHash, ...rest } = parameters;
    const account = account_ ? (0, index_js_1.parseAccount)(account_) : client.account;
    if (!account)
        throw new account_js_1.AccountNotFoundError({
            docsPath: '/docs/actions/wallet/sendTransaction',
        });
    if (!l2Client.chain)
        throw new chain_js_1.ClientChainNotConfiguredError();
    const receipt = (await (0, getTransactionReceipt_js_1.getTransactionReceipt)(l2Client, { hash: depositHash }));
    const successL2ToL1LogIndex = receipt.l2ToL1Logs.findIndex((l2ToL1log) => (0, index_js_1.isAddressEqual)(l2ToL1log.sender, address_js_1.bootloaderFormalAddress) &&
        l2ToL1log.key === depositHash);
    const successL2ToL1Log = receipt.l2ToL1Logs[successL2ToL1LogIndex];
    if (successL2ToL1Log?.value !== bytes_js_1.zeroHash)
        throw new bridge_js_1.CannotClaimSuccessfulDepositError({ hash: depositHash });
    const tx = await (0, getTransaction_js_1.getTransaction)(l2Client, { hash: depositHash });
    const l1BridgeAddress = (0, undoL1ToL2Alias_js_1.undoL1ToL2Alias)(receipt.from);
    const l2BridgeAddress = receipt.to;
    if (!l2BridgeAddress)
        throw new bridge_js_1.L2BridgeNotFoundError();
    const l1NativeTokenVault = (await getBridgeAddresses(client, l2Client))
        .l1NativeTokenVault;
    let depositSender;
    let assetId;
    let assetData;
    try {
        const { args } = (0, index_js_1.decodeFunctionData)({
            abi: (0, abitype_1.parseAbi)([
                'function finalizeDeposit(address _l1Sender, address _l2Receiver, address _l1Token, uint256 _amount, bytes _data)',
            ]),
            data: tx.input,
        });
        assetData = (0, index_js_1.encodeAbiParameters)([{ type: 'uint256' }, { type: 'address' }, { type: 'address' }], [args[3], args[1], args[2]]);
        assetId = await (0, readContract_js_1.readContract)(client, {
            address: l1NativeTokenVault,
            abi: (0, abitype_1.parseAbi)(['function assetId(address token) view returns (bytes32)']),
            functionName: 'assetId',
            args: [args[2]],
        });
        depositSender = args[0];
        if (assetId === bytes_js_1.zeroHash)
            throw new Error(`Token ${args[2]} not registered in NTV`);
    }
    catch (_e) {
        const { args } = (0, index_js_1.decodeFunctionData)({
            abi: (0, abitype_1.parseAbi)([
                'function finalizeDeposit(uint256 _chainId, bytes32 _assetId, bytes _transferData)',
            ]),
            data: tx.input,
        });
        assetId = args[1];
        const transferData = args[2];
        const l1TokenAddress = await (0, readContract_js_1.readContract)(client, {
            address: l1NativeTokenVault,
            abi: (0, abitype_1.parseAbi)([
                'function tokenAddress(bytes32 assetId) view returns (address)',
            ]),
            functionName: 'tokenAddress',
            args: [assetId],
        });
        const transferDataDecoded = (0, index_js_1.decodeAbiParameters)([
            { type: 'address' },
            { type: 'address' },
            { type: 'address' },
            { type: 'uint256' },
            { type: 'bytes' },
        ], transferData);
        assetData = (0, index_js_1.encodeAbiParameters)([{ type: 'uint256' }, { type: 'address' }, { type: 'address' }], [transferDataDecoded[3], transferDataDecoded[1], l1TokenAddress]);
        depositSender = transferDataDecoded[0];
    }
    const proof = await (0, getLogProof_js_1.getLogProof)(l2Client, {
        txHash: depositHash,
        index: successL2ToL1LogIndex,
    });
    if (!proof)
        throw new bridge_js_1.LogProofNotFoundError({
            hash: depositHash,
            index: successL2ToL1LogIndex,
        });
    const data = (0, index_js_1.encodeFunctionData)({
        abi: (0, abitype_1.parseAbi)([
            'function bridgeRecoverFailedTransfer(uint256 _chainId, address _depositSender, bytes32 _assetId, bytes _assetData, bytes32 _l2TxHash, uint256 _l2BatchNumber, uint256 _l2MessageIndex, uint16 _l2TxNumberInBatch, bytes32[] _merkleProof)',
        ]),
        functionName: 'bridgeRecoverFailedTransfer',
        args: [
            BigInt(l2Client.chain.id),
            depositSender,
            assetId,
            assetData,
            depositHash,
            receipt.l1BatchNumber,
            BigInt(proof.id),
            Number(receipt.l1BatchTxIndex),
            proof.proof,
        ],
    });
    return await (0, sendTransaction_js_1.sendTransaction)(client, {
        chain: chain_,
        account,
        to: l1BridgeAddress,
        data,
        ...rest,
    });
}
async function getBridgeAddresses(client, l2Client) {
    const addresses = await (0, getDefaultBridgeAddresses_js_1.getDefaultBridgeAddresses)(l2Client);
    let l1Nullifier = addresses.l1Nullifier;
    let l1NativeTokenVault = addresses.l1NativeTokenVault;
    if (!l1Nullifier)
        l1Nullifier = await (0, readContract_js_1.readContract)(client, {
            address: addresses.sharedL1,
            abi: (0, abitype_1.parseAbi)(['function L1_NULLIFIER() view returns (address)']),
            functionName: 'L1_NULLIFIER',
            args: [],
        });
    if (!l1NativeTokenVault)
        l1NativeTokenVault = await (0, readContract_js_1.readContract)(client, {
            address: addresses.sharedL1,
            abi: (0, abitype_1.parseAbi)(['function nativeTokenVault() view returns (address)']),
            functionName: 'nativeTokenVault',
            args: [],
        });
    return {
        ...addresses,
        l1Nullifier,
        l1NativeTokenVault,
    };
}
//# sourceMappingURL=claimFailedDeposit.js.map