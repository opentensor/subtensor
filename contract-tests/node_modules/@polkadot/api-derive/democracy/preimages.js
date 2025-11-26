import { map, of, switchMap } from 'rxjs';
import { BN_ZERO, isFunction } from '@polkadot/util';
import { firstMemo, memo } from '../util/index.js';
import { getImageHashBounded } from './util.js';
function getUnrequestedTicket(status) {
    return status.ticket || status.deposit;
}
function getRequestedTicket(status) {
    return (status.maybeTicket || status.deposit).unwrapOrDefault();
}
function isDemocracyPreimage(api, imageOpt) {
    return !!imageOpt && !api.query.democracy['dispatchQueue'];
}
function constructProposal(api, [bytes, proposer, balance, at]) {
    let proposal;
    try {
        proposal = api.registry.createType('Call', bytes.toU8a(true));
    }
    catch (error) {
        console.error(error);
    }
    return { at, balance, proposal, proposer };
}
function parseDemocracy(api, imageOpt) {
    if (imageOpt.isNone) {
        return;
    }
    if (isDemocracyPreimage(api, imageOpt)) {
        const status = imageOpt.unwrap();
        if (status.isMissing) {
            return;
        }
        const { data, deposit, provider, since } = status.asAvailable;
        return constructProposal(api, [data, provider, deposit, since]);
    }
    return constructProposal(api, imageOpt.unwrap());
}
function parseImage(api, [proposalHash, status, bytes]) {
    if (!status) {
        return undefined;
    }
    const [proposer, balance] = status.isUnrequested
        ? getUnrequestedTicket(status.asUnrequested)
        : getRequestedTicket(status.asRequested);
    let proposal;
    if (bytes) {
        try {
            proposal = api.registry.createType('Call', bytes.toU8a(true));
        }
        catch (error) {
            console.error(error);
        }
    }
    return { at: BN_ZERO, balance, proposal, proposalHash, proposer };
}
function getDemocracyImages(api, bounded) {
    const hashes = bounded.map((b) => getImageHashBounded(b));
    return api.query.democracy['preimages'].multi(hashes).pipe(map((images) => images.map((imageOpt) => parseDemocracy(api, imageOpt))));
}
function getImages(api, bounded) {
    const hashes = bounded.map((b) => getImageHashBounded(b));
    const bytesType = api.registry.lookup.getTypeDef(api.query.preimage.preimageFor.creator.meta.type.asMap.key).type;
    return api.query.preimage.statusFor.multi(hashes).pipe(switchMap((optStatus) => {
        const statuses = optStatus.map((o) => o.unwrapOr(null));
        const keys = statuses
            .map((s, i) => s
            ? bytesType === 'H256'
                // first generation
                ? hashes[i]
                // current generation (H256,u32)
                : s.isRequested
                    ? [hashes[i], s.asRequested.len.unwrapOr(0)]
                    : [hashes[i], s.asUnrequested.len]
            : null)
            .filter((p) => !!p);
        return api.query.preimage.preimageFor.multi(keys).pipe(map((optBytes) => {
            let ptr = -1;
            return statuses
                .map((s, i) => s
                ? [hashes[i], s, optBytes[++ptr].unwrapOr(null)]
                : [hashes[i], null, null])
                .map((v) => parseImage(api, v));
        }));
    }));
}
/**
 * @name preimages
 * @description Retrieves the full details (preimages) of governance proposals using their on-chain hashes.
 * @param { (Hash | Uint8Array | string | FrameSupportPreimagesBounded)[] } hashes An array of hashes representing governance proposals.
 * @example
 * ```javascript
 * const preimages = await api.derive.democracy.preimages([HASH1, HASH2]);
 * ```
 */
export function preimages(instanceId, api) {
    return memo(instanceId, (hashes) => hashes.length
        ? isFunction(api.query.democracy['preimages'])
            ? getDemocracyImages(api, hashes)
            : isFunction(api.query.preimage.preimageFor)
                ? getImages(api, hashes)
                : of([])
        : of([]));
}
/**
 * @name preimage
 * @description Retrieves the full details (preimage) of a governance proposal using its on-chain hash.
 * @param { Hash | Uint8Array | string | FrameSupportPreimagesBounded } hash Hash that represents governance proposals.
 *  * @example
 * ```javascript
 * const preimage = await api.derive.democracy.preimage(HASH);
 * ```
 */
export const preimage = /*#__PURE__*/ firstMemo((api, hash) => api.derive.democracy.preimages([hash]));
