# Crowdloan Pallet

A pallet that enables the creation and management of generic crowdloans for transferring funds and executing an arbitrary call.

Users of this pallet can create a crowdloan by providing a deposit, a cap, an end block, an optional target address and an optional call.

Users can contribute to a crowdloan by providing funds to the crowdloan they choose to support.

Once the crowdloan is finalized, the funds will be transferred to the target address if provided; otherwise, the end user is expected to transfer them manually on-chain if the call is a pallet extrinsic. The call will be dispatched with the current crowdloan ID stored as a temporary item.

If the crowdloan fails to reach the cap, the initial deposit will be returned to the creator, and contributions will be refunded to the contributors.

## Overview

## Interface

## Dispatchable Functions

License: Apache-2.0
