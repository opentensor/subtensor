import { getDevnetApi, getRandomSubstrateKeypair, getSignerFromKeypair } from "../src/substrate";
import { devnet } from "@polkadot-api/descriptors";
import { Binary, TypedApi, type FixedSizeBinary } from "polkadot-api";
import type { ResultPayload } from "polkadot-api";
import * as assert from "assert";
import { contracts } from "../.papi/descriptors";
import { getInkClient, InkClient } from "@polkadot-api/ink-contracts";
import { forceSetBalanceToSs58Address } from "../src/subtensor";
import fs from "fs";
import { convertPublicKeyToSs58 } from "../src/address-utils";
import { KeyPair } from "@polkadot-labs/hdkd-helpers";

const directWasmPath = "./direct-call/target/ink/direct_call.wasm";
const delegateWasmPath = "./delegate-call/target/ink/delegate_call.wasm";

/** Decode ink `Result<Result<(), E>, LangError>` from a `dummy` message RPC result. */
function assertDummyDoubleOk(decoded: unknown) {
    const outer = decoded as ResultPayload<
        ResultPayload<undefined, { type: string; value: unknown }>,
        unknown
    >;
    assert.equal(outer.success, true, "expected outer ink Result::Ok (no LangError)");
    const inner = outer.value as ResultPayload<undefined, { type: string; value: unknown }>;
    assert.equal(inner.success, true, "expected inner Result::Ok (chain extension dummy)");
}

describe("direct_call and delegate_call ink contracts", () => {
    let api: TypedApi<typeof devnet>;
    let coldkey: KeyPair;
    let directClient: InkClient<typeof contracts.direct_call>;
    let delegateClient: InkClient<typeof contracts.delegate_call>;

    let directCodeHash: FixedSizeBinary<32>;

    before(async () => {
        api = await getDevnetApi();
        directClient = getInkClient(contracts.direct_call);
        delegateClient = getInkClient(contracts.delegate_call);
    });

    beforeEach(async () => {
        coldkey = getRandomSubstrateKeypair();
        await forceSetBalanceToSs58Address(api, convertPublicKeyToSs58(coldkey.publicKey));
    });

    async function instantiateDirectCall(): Promise<string> {
        const bytecode = fs.readFileSync(directWasmPath);
        const signer = getSignerFromKeypair(coldkey);
        console.log("direct_call: signing with", convertPublicKeyToSs58(coldkey.publicKey));
        const ctor = directClient.constructor("new");
        const data = ctor.encode();
        const tx = await api.tx.Contracts.instantiate_with_code({
            code: Binary.fromBytes(bytecode),
            storage_deposit_limit: BigInt(10000000),
            value: BigInt(0),
            gas_limit: {
                ref_time: BigInt(1000000000),
                proof_size: BigInt(1000000),
            },
            data: Binary.fromBytes(data.asBytes()),
            salt: Binary.fromHex("0x01"),
        }).signAndSubmit(signer);

        const evts = await api.event.Contracts.Instantiated.filter(tx.events);
        if (evts.length === 0) {
            throw new Error("direct_call: no Contracts.Instantiated event");
        }

        const contractInfo = await api.query.Contracts.ContractInfoOf.getValue(evts[0].contract);
        if (contractInfo === undefined) {
            throw new Error("direct_call: contract info should exist");
        }
        directCodeHash = contractInfo.code_hash;
        console.log("direct_call: instantiated at", evts[0].contract);
        return evts[0].contract;
    }

    async function instantiateDelegateCall(): Promise<string> {
        const bytecode = fs.readFileSync(delegateWasmPath);
        const signer = getSignerFromKeypair(coldkey);
        console.log("delegate_call: signing with", convertPublicKeyToSs58(coldkey.publicKey));
        const ctor = delegateClient.constructor("new");
        const data = ctor.encode();
        const tx = await api.tx.Contracts.instantiate_with_code({
            code: Binary.fromBytes(bytecode),
            storage_deposit_limit: BigInt(10000000),
            value: BigInt(0),
            gas_limit: {
                ref_time: BigInt(1000000000),
                proof_size: BigInt(1000000),
            },
            data: Binary.fromBytes(data.asBytes()),
            salt: Binary.fromHex("0x02"),
        }).signAndSubmit(signer);

        const evts = await api.event.Contracts.Instantiated.filter(tx.events);
        if (evts.length === 0) {
            throw new Error("delegate_call: no Contracts.Instantiated event");
        }
        console.log("delegate_call: instantiated at", evts[0].contract);
        return evts[0].contract;
    }

    it("direct_call: instantiate and dummy chain extension succeeds", async () => {
        const contractAddress = await instantiateDirectCall();

        const message = directClient.message("dummy");
        const callData = message.encode();
        const response = await api.apis.ContractsApi.call(
            convertPublicKeyToSs58(coldkey.publicKey),
            contractAddress,
            BigInt(0),
            undefined,
            undefined,
            Binary.fromBytes(callData.asBytes()),
        );

        assert.ok(response.result.success, "ContractsApi.call should succeed");
        const decoded = message.decode(response.result.value);
        assertDummyDoubleOk(decoded);
    });

    it("delegate_call: instantiate with direct_call code hash and dummy delegates successfully", async () => {
        const directAddress = await instantiateDirectCall();
        const contractInfo = await api.query.Contracts.ContractInfoOf.getValue(directAddress);
        assert.ok(contractInfo !== undefined, "direct_call contract info should exist");
        const codeHash = contractInfo.code_hash;

        const delegateAddress = await instantiateDelegateCall();

        console.log("delegate_call: delegating to", directAddress);
        const message = delegateClient.message("dummy");
        const callData = message.encode({ code_hash: codeHash });
        const response = await api.apis.ContractsApi.call(
            convertPublicKeyToSs58(coldkey.publicKey),
            delegateAddress,
            BigInt(0),
            undefined,
            undefined,
            Binary.fromBytes(callData.asBytes()),
        );

        assert.ok(response.result.success, "ContractsApi.call should succeed");
        const decoded = message.decode(response.result.value);
        assertDummyDoubleOk(decoded);
    });
});
