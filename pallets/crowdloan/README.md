# Crowdloan Pallet 
A pallet allowing to create and manage generic crowdloans around a transfer of funds and a arbitrary call.

A user of this pallet can create a crowdloan by providing a deposit, a cap, an end block, a target address and a call.

Users will be able to contribute to the crowdloan by providing funds to the crowdloan they chose to contribute to.

Once the crowdloan is finalized, the funds will be transferred to the target address and the call will be dispatched with the current crowdloan id as a temporary storage item.

In case the crowdloan fails to raise the cap, the initial deposit will be returned to the creator and contributions will be returned to the contributors.

## Overview

## Interface

### Dispatchable Functions

[`Call`]: ./enum.Call.html
[`Config`]: ./trait.Config.html

License: Apache-2.0
