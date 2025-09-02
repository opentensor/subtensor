# Crowdloan Pallet

## Overview

A pallet that enables the creation and management of generic crowdloans for transferring funds and executing an arbitrary call.

Users of this pallet can create a crowdloan by providing a deposit, a cap, an end block, an optional target address and an optional call.

Users can contribute to a crowdloan by providing funds to the crowdloan they choose to support. The contribution can be withdrawn while the crowdloan is not finalized.

Once the crowdloan is finalized, the funds will be transferred to the target address if provided; otherwise, the end user is expected to transfer them manually on-chain if the call is a pallet extrinsic. The call will be dispatched with the current crowdloan ID stored as a temporary item.

If the crowdloan fails to reach the cap, the creator can decide to refund all contributors and dissolve the crowdloan. The initial deposit will be refunded.

*The call or target address provided when creating the crowdloan is guaranteed to never change. Only the minimum contribution, end block and cap can be updated from the crowdloan creator.*

## Interface

- `create`: Create a crowdloan that will raise funds up to a maximum cap and if successful, will transfer funds to the target address if provided and/or dispatch the call (using creator origin). The initial deposit will be transfered to the crowdloan account and will be refunded in case the crowdloan fails to raise the cap. Additionally, the creator will pay for the execution of the call.

- `contribute`: Contribute to an active crowdloan. The contribution will be transfered to the crowdloan account and will be refunded if the crowdloan fails to raise the cap. If the contribution would raise the amount above the cap, the contribution will be set to the amount that is left to be raised.

- `withdraw`: Withdraw a contribution from an active (not yet finalized or dissolved) crowdloan. Only contributions over the deposit can be withdrawn by the creator.

- `refund`: Try to refund all contributors (excluding the creator) up to the limit defined by a runtime parameter *RefundContributorsLimit* (currently set to 5). If the limit is reached, the call will stop and the crowdloan will be marked as partially refunded. It may be needed to dispatch this call multiple times to refund all contributors.

The following functions are only callable by the creator of the crowdloan:

- `finalize`: Finalize a successful crowdloan. The call will transfer the raised amount to the target address if it was provided when the crowdloan was created and dispatch the call that was provided using the creator origin. 

- `dissolve`: Dissolve a crowdloan. The crowdloan will be removed from the storage. All contributions must have been refunded before the crowdloan can be dissolved (except the creator's one).

- `update_min_contribution`: Update the minimum contribution of a non-finalized crowdloan.

- `update_end`: Update the end block of a non-finalized crowdloan.

- `update_cap`: Update the cap of a non-finalized crowdloan.

## Integration with subnet leasing (from the subtensor pallet)

The `crowdloan` pallet can be used to create a crowdloan that will be used to register a new leased network through a crowdloan using the `register_leased_network` extrinsic from the `subtensor` pallet as a call parameter to the crowdloan pallet `create` extrinsic. A new subnet will be registered paying the lock cost using the crowdloan funds and a proxy will be created for the beneficiary to operate the subnet.

When active, the lease will distribute dividends to the contributors according to their contribution to the crowdloan and the lease can be operated by the beneficiary using the proxy created `SubnetLeaseBeneficiary`.

If the lease is perpetual, the lease will never be terminated and emissions will continue to be distributed to the contributors.

If the lease has an end block, the lease can be terminated when end block has passed and the subnet ownership will be transferred to the beneficiary.

License: Apache-2.0
