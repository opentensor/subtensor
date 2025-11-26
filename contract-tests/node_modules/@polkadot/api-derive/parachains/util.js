export function didUpdateToBool(didUpdate, id) {
    return didUpdate.isSome
        ? didUpdate.unwrap().some((paraId) => paraId.eq(id))
        : false;
}
