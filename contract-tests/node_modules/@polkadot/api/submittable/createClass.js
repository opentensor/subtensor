import { catchError, first, map, mergeMap, of, switchMap, tap } from 'rxjs';
import { identity, isBn, isFunction, isNumber, isString, isU8a, objectSpread } from '@polkadot/util';
import { filterEvents, isKeyringPair } from '../util/index.js';
import { SubmittableResult } from './Result.js';
function makeEraOptions(api, registry, partialOptions, { header, mortalLength, nonce }) {
    if (!header) {
        if (partialOptions.era && !partialOptions.blockHash) {
            throw new Error('Expected blockHash to be passed alongside non-immortal era options');
        }
        if (isNumber(partialOptions.era)) {
            // since we have no header, it is immortal, remove any option overrides
            // so we only supply the genesisHash and no era to the construction
            delete partialOptions.era;
            delete partialOptions.blockHash;
        }
        return makeSignOptions(api, partialOptions, { nonce });
    }
    return makeSignOptions(api, partialOptions, {
        blockHash: header.hash,
        era: registry.createTypeUnsafe('ExtrinsicEra', [{
                current: header.number,
                period: partialOptions.era || mortalLength
            }]),
        nonce
    });
}
function makeSignAndSendOptions(partialOptions, statusCb) {
    let options = {};
    if (isFunction(partialOptions)) {
        statusCb = partialOptions;
    }
    else {
        options = objectSpread({}, partialOptions);
    }
    return [options, statusCb];
}
function makeSignOptions(api, partialOptions, extras) {
    return objectSpread({ blockHash: api.genesisHash, genesisHash: api.genesisHash }, partialOptions, extras, { runtimeVersion: api.runtimeVersion, signedExtensions: api.registry.signedExtensions, version: api.extrinsicType });
}
function optionsOrNonce(partialOptions = {}) {
    return isBn(partialOptions) || isNumber(partialOptions)
        ? { nonce: partialOptions }
        : partialOptions;
}
export function createClass({ api, apiType, blockHash, decorateMethod }) {
    // an instance of the base extrinsic for us to extend
    const ExtrinsicBase = api.registry.createClass('Extrinsic');
    const extrinsicInfoMap = new WeakMap();
    class Submittable extends ExtrinsicBase {
        #ignoreStatusCb;
        #transformResult = (identity);
        constructor(registry, extrinsic) {
            super(registry, extrinsic, { version: api.extrinsicType });
            this.#ignoreStatusCb = apiType === 'rxjs';
        }
        get hasDryRun() {
            return isFunction(api.rpc.system?.dryRun);
        }
        get hasPaymentInfo() {
            return isFunction(api.call.transactionPaymentApi?.queryInfo);
        }
        // dry run an extrinsic
        dryRun(account, optionsOrHash) {
            if (!this.hasDryRun) {
                throw new Error('The system.dryRun RPC call is not available in your environment');
            }
            if (blockHash || isString(optionsOrHash) || isU8a(optionsOrHash)) {
                // eslint-disable-next-line @typescript-eslint/no-unsafe-return
                return decorateMethod(() => api.rpc.system.dryRun(this.toHex(), blockHash || optionsOrHash));
            }
            // eslint-disable-next-line @typescript-eslint/no-unsafe-return,@typescript-eslint/no-unsafe-call
            return decorateMethod(() => this.#observeSign(account, optionsOrHash).pipe(switchMap(() => api.rpc.system.dryRun(this.toHex()))))();
        }
        // calculate the payment info for this transaction (if signed and submitted)
        paymentInfo(account, optionsOrHash) {
            if (!this.hasPaymentInfo) {
                throw new Error('The transactionPaymentApi.queryInfo runtime call is not available in your environment');
            }
            if (blockHash || isString(optionsOrHash) || isU8a(optionsOrHash)) {
                // eslint-disable-next-line @typescript-eslint/no-unsafe-return
                return decorateMethod(() => api.callAt(blockHash || optionsOrHash).pipe(switchMap((callAt) => {
                    const u8a = this.toU8a();
                    return callAt.transactionPaymentApi.queryInfo(u8a, u8a.length);
                })));
            }
            const [allOptions] = makeSignAndSendOptions(optionsOrHash);
            const address = isKeyringPair(account) ? account.address : account.toString();
            // eslint-disable-next-line @typescript-eslint/no-unsafe-return,@typescript-eslint/no-unsafe-call
            return decorateMethod(() => api.derive.tx.signingInfo(address, allOptions.nonce, allOptions.era).pipe(first(), switchMap((signingInfo) => {
                // setup our options (same way as in signAndSend)
                const eraOptions = makeEraOptions(api, this.registry, allOptions, signingInfo);
                const signOptions = makeSignOptions(api, eraOptions, {});
                // 1. Don't use the internal objects inside the new tx (hence toU8a)
                // 2. Don't override the data from existing signed extrinsics
                // 3. Ensure that this object stays intact, with no new sign after operation
                const u8a = api.tx(this.toU8a()).signFake(address, signOptions).toU8a();
                return api.call.transactionPaymentApi.queryInfo(u8a, u8a.length);
            })))();
        }
        // send implementation for both immediate Hash and statusCb variants
        send(statusCb) {
            const isSubscription = api.hasSubscriptions && (this.#ignoreStatusCb || !!statusCb);
            const updatedInfo = extrinsicInfoMap.get(this);
            extrinsicInfoMap.delete(this);
            // eslint-disable-next-line @typescript-eslint/no-unsafe-return,@typescript-eslint/no-unsafe-call
            return decorateMethod(isSubscription
                ? () => this.#observeSubscribe(updatedInfo)
                : () => this.#observeSend(updatedInfo))(statusCb);
        }
        /**
         * @description Signs a transaction, returning `this` to allow chaining. E.g.: `signAsync(...).send()`. Like `.signAndSend` this will retrieve the nonce and blockHash to send the tx with.
        */
        signAsync(account, partialOptions) {
            // eslint-disable-next-line @typescript-eslint/no-unsafe-return,@typescript-eslint/no-unsafe-call
            return decorateMethod(() => this.#observeSign(account, partialOptions).pipe(map((info) => {
                // If we got a full signed transaction from the signer, attach it
                if (info.signedTransaction) {
                    const extrinsic = new Submittable(api.registry, info.signedTransaction);
                    extrinsicInfoMap.set(this, info);
                    return extrinsic;
                }
                // Fallback if signer didn’t return signedTransaction
                return this;
            })))();
        }
        // signAndSend implementation for all 3 cases above
        signAndSend(account, partialOptions, optionalStatusCb) {
            const [options, statusCb] = makeSignAndSendOptions(partialOptions, optionalStatusCb);
            const isSubscription = api.hasSubscriptions && (this.#ignoreStatusCb || !!statusCb);
            // eslint-disable-next-line @typescript-eslint/no-unsafe-return,@typescript-eslint/no-unsafe-call
            return decorateMethod(() => this.#observeSign(account, options).pipe(switchMap((info) => isSubscription
                ? this.#observeSubscribe(info)
                : this.#observeSend(info))) // FIXME This is wrong, SubmittableResult is _not_ a codec
            )(statusCb);
        }
        // adds a transform to the result, applied before result is returned
        withResultTransform(transform) {
            this.#transformResult = transform;
            return this;
        }
        #observeSign = (account, partialOptions) => {
            const address = isKeyringPair(account) ? account.address : account.toString();
            const options = optionsOrNonce(partialOptions);
            return api.derive.tx.signingInfo(address, options.nonce, options.era).pipe(first(), mergeMap(async (signingInfo) => {
                const eraOptions = makeEraOptions(api, this.registry, options, signingInfo);
                let updateId = -1;
                let signedTx = null;
                if (isKeyringPair(account)) {
                    this.sign(account, eraOptions);
                }
                else {
                    const result = await this.#signViaSigner(address, eraOptions, signingInfo.header);
                    updateId = result.id;
                    if (result.signedTransaction) {
                        signedTx = result.signedTransaction;
                    }
                }
                return { options: eraOptions, signedTransaction: signedTx, updateId };
            }));
        };
        #observeStatus = (txHash, status) => {
            if (!status.isFinalized && !status.isInBlock) {
                return of(this.#transformResult(new SubmittableResult({
                    status,
                    txHash
                })));
            }
            const blockHash = status.isInBlock
                ? status.asInBlock
                : status.asFinalized;
            return api.derive.tx.events(blockHash).pipe(map(({ block, events }) => this.#transformResult(new SubmittableResult({
                ...filterEvents(txHash, block, events, status),
                status,
                txHash
            }))), catchError((internalError) => of(this.#transformResult(new SubmittableResult({
                internalError,
                status,
                txHash
            })))));
        };
        #observeSend = (info) => {
            return api.rpc.author.submitExtrinsic(info?.signedTransaction || this).pipe(tap((hash) => {
                this.#updateSigner(hash, info);
            }));
        };
        #observeSubscribe = (info) => {
            const txHash = this.hash;
            return api.rpc.author.submitAndWatchExtrinsic(info?.signedTransaction || this).pipe(switchMap((status) => this.#observeStatus(txHash, status)), tap((status) => {
                this.#updateSigner(status, info);
            }));
        };
        #signViaSigner = async (address, options, header) => {
            const signer = options.signer || api.signer;
            const allowCallDataAlteration = options.allowCallDataAlteration ?? true;
            if (!signer) {
                throw new Error('No signer specified, either via api.setSigner or via sign options. You possibly need to pass through an explicit keypair for the origin so it can be used for signing.');
            }
            const payload = this.registry.createTypeUnsafe('SignerPayload', [objectSpread({}, options, {
                    address,
                    blockNumber: header ? header.number : 0,
                    method: this.method
                })]);
            let result;
            if (isFunction(signer.signPayload)) {
                result = await signer.signPayload(payload.toPayload());
                if (result.signedTransaction && !options.withSignedTransaction) {
                    throw new Error('The `signedTransaction` field may not be submitted when `withSignedTransaction` is disabled');
                }
                if (result.signedTransaction && options.withSignedTransaction) {
                    const ext = this.registry.createTypeUnsafe('Extrinsic', [result.signedTransaction]);
                    const newSignerPayload = this.registry.createTypeUnsafe('SignerPayload', [objectSpread({}, {
                            address,
                            assetId: ext.assetId && ext.assetId.isSome ? ext.assetId.toHex() : null,
                            blockHash: payload.blockHash,
                            blockNumber: header ? header.number : 0,
                            era: ext.era.toHex(),
                            genesisHash: payload.genesisHash,
                            metadataHash: ext.metadataHash ? ext.metadataHash.toHex() : null,
                            method: ext.method.toHex(),
                            mode: ext.mode ? ext.mode.toHex() : null,
                            nonce: ext.nonce.toHex(),
                            runtimeVersion: payload.runtimeVersion,
                            signedExtensions: payload.signedExtensions,
                            tip: ext.tip ? ext.tip.toHex() : null,
                            version: payload.version
                        })]);
                    if (!ext.isSigned) {
                        throw new Error(`When using the signedTransaction field, the transaction must be signed. Recieved isSigned: ${ext.isSigned}`);
                    }
                    if (!allowCallDataAlteration) {
                        this.#validateSignedTransaction(payload, ext);
                    }
                    // This is only used for signAsync - signAndSend does not need to adjust the super payload or
                    // add the signature.
                    super.addSignature(address, result.signature, newSignerPayload.toPayload());
                    return { id: result.id, signedTransaction: result.signedTransaction };
                }
            }
            else if (isFunction(signer.signRaw)) {
                result = await signer.signRaw(payload.toRaw());
            }
            else {
                throw new Error('Invalid signer interface, it should implement either signPayload or signRaw (or both)');
            }
            // Here we explicitly call `toPayload()` again instead of working with an object
            // (reference) as passed to the signer. This means that we are sure that the
            // payload data is not modified from our inputs, but the signer
            super.addSignature(address, result.signature, payload.toPayload());
            return { id: result.id };
        };
        #updateSigner = (status, info) => {
            if (info && (info.updateId !== -1)) {
                const { options, updateId } = info;
                const signer = options.signer || api.signer;
                if (signer && isFunction(signer.update)) {
                    signer.update(updateId, status);
                }
            }
        };
        /**
         * When a signer includes `signedTransaction` within the SignerResult this will validate
         * specific fields within the signed extrinsic against the original payload that was passed
         * to the signer.
         */
        #validateSignedTransaction = (signerPayload, signedExt) => {
            const payload = signerPayload.toPayload();
            const errMsg = (field) => `signAndSend: ${field} does not match the original payload`;
            if (payload.method !== signedExt.method.toHex()) {
                throw new Error(errMsg('call data'));
            }
        };
    }
    return Submittable;
}
