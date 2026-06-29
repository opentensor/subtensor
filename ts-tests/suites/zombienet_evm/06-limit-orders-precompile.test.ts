import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import { subtensor } from "@polkadot-api/descriptors";
import { Keyring } from "@polkadot/keyring";
import type { KeyringPair } from "@polkadot/keyring/types";
import { TypeRegistry } from "@polkadot/types";
import { u8aToHex } from "@polkadot/util";
import { blake2AsHex } from "@polkadot/util-crypto";
import { ethers } from "ethers";
import { type TypedApi } from "polkadot-api";
import {
    addNewSubnetwork,
    convertH160ToSS58,
    convertPublicKeyToSs58,
    createEthersWallet,
    disableWhiteListCheck,
    forceSetBalance,
    generateKeyringPair,
    LIMIT_ORDERS_ABI,
    LIMIT_ORDERS_ADDRESS,
    ss58ToEthAddress,
    sudoSetLockReductionInterval,
    tao,
    waitForFinalizedBlocks,
    waitForTransactionWithRetry,
} from "../../utils";

// ── Constants ─────────────────────────────────────────────────────────────────

const ORDER_TYPE_LIMIT_BUY = 0;
const FAR_FUTURE = BigInt("18446744073709551615"); // u64::MAX


// ── Type registry helpers ─────────────────────────────────────────────────────

function registerLimitOrderTypes(registry: TypeRegistry): void {
    registry.register({
        LimitOrderType: {
            _enum: ["LimitBuy", "TakeProfit", "StopLoss"],
        },
        LimitOrder: {
            signer: "AccountId",
            hotkey: "AccountId",
            netuid: "u16",
            order_type: "LimitOrderType",
            amount: "u64",
            limit_price: "u64",
            expiry: "u64",
            fee_rate: "u32",
            fee_recipient: "AccountId",
            relayer: "Option<Vec<AccountId>>",
            max_slippage: "Option<u32>",
            chain_id: "u64",
            partial_fills_enabled: "bool",
        },
        LimitVersionedOrder: {
            _enum: {
                V1: "LimitOrder",
            },
        },
        LimitSignedOrder: {
            order: "LimitVersionedOrder",
            signature: "MultiSignature",
            partial_fill: "Option<u64>",
        },
    });
}

// ── On-chain helpers ──────────────────────────────────────────────────────────

async function setPalletStatus(api: TypedApi<typeof subtensor>, enabled: boolean, sudo: KeyringPair): Promise<void> {
    const tx = api.tx.Sudo.sudo({
        call: api.tx.LimitOrders.set_pallet_status({ enabled }).decodedCall,
    });
    await waitForTransactionWithRetry(api, tx, sudo, "set_pallet_status");
}

async function enableSubtoken(api: TypedApi<typeof subtensor>, netuid: number, sudo: KeyringPair): Promise<void> {
    const internalCall = api.tx.AdminUtils.sudo_set_subtoken_enabled({ netuid, subtoken_enabled: true });
    const tx = api.tx.Sudo.sudo({ call: internalCall.decodedCall });
    await waitForTransactionWithRetry(api, tx, sudo, "enable_subtoken");
}

async function associateHotkey(api: TypedApi<typeof subtensor>, coldkey: KeyringPair, hotkeyAddress: string): Promise<void> {
    const tx = api.tx.SubtensorModule.try_associate_hotkey({ hotkey: hotkeyAddress });
    await waitForTransactionWithRetry(api, tx, coldkey, "try_associate_hotkey");
}

// ── Order building ────────────────────────────────────────────────────────────

interface OrderParams {
    signer: KeyringPair;
    hotkey: KeyringPair;
    netuid: number;
    orderType: string;
    amount: bigint;
    limitPrice: bigint;
    expiry: bigint;
    feeRate: number;
    chainId?: bigint;
}

function buildTypedSignedOrder(
    registry: TypeRegistry,
    params: OrderParams,
): {
    orderInput: Record<string, unknown>;
    orderId: `0x${string}`;
    rawSigHex: `0x${string}`;
} {
    const chainId = params.chainId ?? 42n;
    const signerH160 = ss58ToEthAddress(params.signer.address);
    const hotkeyH160 = ss58ToEthAddress(params.hotkey.address);

    const orderInput: Record<string, unknown> = {
        signer: signerH160,
        hotkey: hotkeyH160,
        netuid: params.netuid,
        order_type: ["LimitBuy", "TakeProfit", "StopLoss"].indexOf(params.orderType),
        amount: params.amount,
        limit_price: params.limitPrice,
        expiry: params.expiry,
        fee_rate: params.feeRate,
        fee_recipient: signerH160,
        relayer: [],
        has_max_slippage: false,
        max_slippage: 0,
        chain_id: chainId,
        partial_fills_enabled: false,
    };

    const orderValue: Record<string, unknown> = {
        signer: params.signer.address,
        hotkey: params.hotkey.address,
        netuid: params.netuid,
        order_type: params.orderType,
        amount: params.amount,
        limit_price: params.limitPrice,
        expiry: params.expiry,
        fee_rate: params.feeRate,
        fee_recipient: params.signer.address,
        relayer: undefined,
        max_slippage: undefined,
        chain_id: chainId,
        partial_fills_enabled: false,
    };

    const versionedOrder = { V1: orderValue };

    const encoded = registry.createType("LimitVersionedOrder", versionedOrder);
    const rawSigU8a = params.signer.sign(encoded.toU8a());
    const orderId = blake2AsHex(encoded.toU8a(), 256) as `0x${string}`;

    return {
        orderInput,
        orderId,
        rawSigHex: u8aToHex(rawSigU8a) as `0x${string}`,
    };
}

function buildSignedOrderInputForPrecompile(
    orderInput: Record<string, unknown>,
    rawSigHex: `0x${string}`,
): Record<string, unknown> {
    return {
        order: orderInput,
        signature: rawSigHex,
        has_partial_fill: false,
        partial_fill: 0n,
    };
}

// ── Test suite ────────────────────────────────────────────────────────────────

describeSuite({
    id: "limit-orders-precompile",
    title: "LimitOrders precompile E2E tests",
    foundationMethods: "zombie",
    testCases: ({ it, context }) => {
        let api: TypedApi<typeof subtensor>;
        let provider: ethers.JsonRpcProvider;
        let relayerWallet: ethers.Wallet;
        let limitOrders: ethers.Contract;
        let registry: TypeRegistry;
        let sudo: KeyringPair;
        let signer: KeyringPair;
        let hotkey: KeyringPair;
        let netuid: number;
        let setupOk = false;

        beforeAll(async () => {
            api = context.papi("Node").getTypedApi(subtensor);
            provider = context.ethers("EVM").provider as ethers.JsonRpcProvider;

            registry = new TypeRegistry();
            registerLimitOrderTypes(registry);

            sudo = new Keyring({ type: "sr25519" }).addFromUri("//Alice");

            relayerWallet = createEthersWallet(provider);
            signer = generateKeyringPair("sr25519");
            hotkey = generateKeyringPair("sr25519");

            await forceSetBalance(api, convertH160ToSS58(relayerWallet.address));
            await forceSetBalance(api, convertPublicKeyToSs58(signer.publicKey));
            await forceSetBalance(api, convertPublicKeyToSs58(hotkey.publicKey));

            await disableWhiteListCheck(api, true);
            await sudoSetLockReductionInterval(api, 1);

            netuid = await addNewSubnetwork(api, hotkey, signer);

            await setPalletStatus(api, true, sudo);
            await enableSubtoken(api, netuid, sudo);
            await associateHotkey(api, signer, convertPublicKeyToSs58(hotkey.publicKey));

            limitOrders = new ethers.Contract(LIMIT_ORDERS_ADDRESS, LIMIT_ORDERS_ABI, relayerWallet);

            setupOk = true;
        }, 600000);

        // ── View functions ────────────────────────────────────────────────────

        it({
            id: "T01",
            title: "getLimitOrdersEnabled returns true",
            test: async () => {
                expect(setupOk).toBe(true);
                expect(await limitOrders.getLimitOrdersEnabled()).toBe(true);
            },
        });

        it({
            id: "T02",
            title: "getOrderStatus returns 0 for non-existent order",
            test: async () => {
                expect(setupOk).toBe(true);
                const unknownId = "0x" + "00".repeat(32);
                expect(await limitOrders.getOrderStatus(unknownId)).toBe(0n);
            },
        });

        it({
            id: "T03",
            title: "deriveOrderId produces a non-zero 32-byte hash",
            test: async () => {
                expect(setupOk).toBe(true);

                const orderInput: Record<string, unknown> = {
                    signer: ss58ToEthAddress(signer.address),
                    hotkey: ss58ToEthAddress(hotkey.address),
                    netuid,
                    order_type: ORDER_TYPE_LIMIT_BUY,
                    amount: tao(100),
                    limit_price: FAR_FUTURE,
                    expiry: FAR_FUTURE,
                    fee_rate: 0,
                    fee_recipient: ss58ToEthAddress(signer.address),
                    relayer: [],
                    has_max_slippage: false,
                    max_slippage: 0,
                    chain_id: 42n,
                    partial_fills_enabled: false,
                };

                const orderId: string = await limitOrders.deriveOrderId(orderInput);
                expect(orderId).toMatch(/^0x[0-9a-f]{64}$/i);
                expect(orderId).not.toBe("0x" + "00".repeat(32));
            },
        });

        // ── Execute via pallet, verify via precompile ─────────────────────────

        it({
            id: "T04",
            title: "Precompile executeOrders accepts a signed order submission",
            test: async () => {
                expect(setupOk).toBe(true);

                const { orderInput, rawSigHex } = buildTypedSignedOrder(registry, {
                    signer,
                    hotkey,
                    netuid,
                    orderType: "LimitBuy",
                    amount: tao(100),
                    limitPrice: FAR_FUTURE,
                    expiry: FAR_FUTURE,
                    feeRate: 0,
                });

                const signedOrderInput = buildSignedOrderInputForPrecompile(orderInput, rawSigHex);

                const tx = await limitOrders.executeOrders([signedOrderInput], false);
                const receipt = await tx.wait();
                expect(receipt?.status).toEqual(1);
            },
        });

        it({
            id: "T05",
            title: "Precompile executeOrders with a second order submission succeeds",
            test: async () => {
                expect(setupOk).toBe(true);

                const { orderInput, rawSigHex } = buildTypedSignedOrder(registry, {
                    signer,
                    hotkey,
                    netuid,
                    orderType: "LimitBuy",
                    amount: tao(50),
                    limitPrice: FAR_FUTURE,
                    expiry: FAR_FUTURE,
                    feeRate: 0,
                });

                const signedOrderInput = buildSignedOrderInputForPrecompile(orderInput, rawSigHex);

                const tx = await limitOrders.executeOrders([signedOrderInput], false);
                const receipt = await tx.wait();
                expect(receipt?.status).toEqual(1);
            },
        });

        // ── cancelOrder via precompile ──────────────────────────────────────

        it({
            id: "T06",
            title: "cancelOrder via precompile marks order as cancelled",
            test: async () => {
                expect(setupOk).toBe(true);

                // Use relayerWallet's own address as signer so the origin check
                // (HashedAddressMapping(caller) == order.signer) passes.
                const orderInput: Record<string, unknown> = {
                    signer: relayerWallet.address,
                    hotkey: relayerWallet.address,
                    netuid,
                    order_type: ORDER_TYPE_LIMIT_BUY,
                    amount: tao(1),
                    limit_price: FAR_FUTURE,
                    expiry: FAR_FUTURE,
                    fee_rate: 0,
                    fee_recipient: relayerWallet.address,
                    relayer: [],
                    has_max_slippage: false,
                    max_slippage: 0,
                    chain_id: 42n,
                    partial_fills_enabled: false,
                };

                const tx = await limitOrders.cancelOrder(orderInput);
                const receipt = await tx.wait();
                expect(receipt?.status).toEqual(1);

                const derivedId: string = await limitOrders.deriveOrderId(orderInput);
                await waitForFinalizedBlocks(api, 2);

                const status = await limitOrders.getOrderStatus(derivedId);
                expect(status).toBe(3n); // 3 = Cancelled
            },
        });
    },
});
