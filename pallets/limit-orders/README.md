# pallet-limit-orders

A FRAME pallet for off-chain signed limit orders on Bittensor subnets.

Users sign orders off-chain and submit them to a relayer. The relayer batches
orders targeting the same subnet and submits them via `execute_batched_orders`,
which nets the buy and sell sides, executes a single AMM pool swap for the
residual, and distributes outputs pro-rata to all participants. This minimises
price impact compared to executing each order independently against the pool.

MEV protection is available for free: any caller can wrap `execute_orders` or
`execute_batched_orders` inside `pallet_shield::submit_encrypted` to hide the
batch contents from the mempool until the block is proposed.

---

## Order lifecycle

```
User signs Order off-chain
        │
        ▼
Relayer submits via execute_orders (one-by-one)
        or execute_batched_orders (aggregated)
        │
        ├─ Invalid / expired / price-not-met → OrderSkipped (no state change)
        │
        └─ Valid → executed → OrderExecuted
                                    │
                                    └─ order_id written to Orders storage
                                       (prevents replay)

User can cancel at any time via cancel_order
        └─ order_id written to Orders as Cancelled
```

---

## Data structures

### `Order<AccountId>`

The payload that a user signs off-chain. Never stored in full on-chain — only
its `blake2_256` hash (`OrderId`) is persisted.

| Field         | Type        | Description |
|---------------|-------------|-------------|
| `signer`      | `AccountId` | Coldkey that authorises the order. For buys: pays TAO. For sells: owns the staked alpha. |
| `hotkey`      | `AccountId` | Hotkey to stake to (buy) or unstake from (sell). |
| `netuid`      | `NetUid`    | Target subnet. |
| `side`        | `OrderSide` | `Buy` or `Sell`. |
| `amount`      | `u64`       | Input amount in raw units. TAO for buys; alpha for sells. |
| `limit_price` | `u64`       | Price threshold in TAO/alpha raw units. Buy: maximum acceptable price. Sell: minimum acceptable price. |
| `expiry`      | `u64`       | Unix timestamp in milliseconds. Order must not execute after this time. |

### `SignedOrder<AccountId, Signature>`

Envelope submitted by the relayer: the `Order` payload plus the user's
sr25519/ed25519 signature over its SCALE encoding. Signature verification
uses `order.signer` as the expected public key.

### `OrderStatus`

Terminal state of a processed order, stored under its `OrderId`.

| Variant     | Meaning |
|-------------|---------|
| `Fulfilled` | Order was successfully executed. |
| `Cancelled` | User registered a cancellation intent before execution. |

---

## Storage

### `ProtocolFee: StorageValue<u32>`

Protocol fee in parts-per-billion (PPB).

- `0` = no fee.
- `1_000_000` = 0.1%.
- `1_000_000_000` = 100%.

For buy orders the fee is deducted from the TAO input before swapping. For sell
orders the fee is deducted from the TAO output after swapping. Both flows result
in the fee being collected in TAO and forwarded to `FeeCollector`.

Default: `0`.

### `Admin: StorageValue<Option<AccountId>>`

The privileged account that may call `set_protocol_fee` alongside root.
`None` means no admin is set; only root can change the fee.
Set by root via `set_admin`.

Default: absent (`None`).

### `OrderStatus: StorageMap<H256, OrderStatus>`

Maps an `OrderId` (blake2_256 of the SCALE-encoded `Order`) to its terminal
`OrderStatus`. Absence means the order has never been seen and is still
executable (provided it is valid). Presence means it is permanently closed —
neither `Fulfilled` nor `Cancelled` orders can be re-executed.

---

## Config

| Item                  | Type                                              | Description |
|-----------------------|---------------------------------------------------|-------------|
| `Signature`           | `Verify + ...`                                    | Signature type for off-chain order authorisation. Set to `sp_runtime::MultiSignature` in the subtensor runtime. |
| `SwapInterface`       | `OrderSwapInterface<Self::AccountId>`             | Full swap + balance execution interface. Implemented by `pallet_subtensor::Pallet<T>`. Provides `buy_alpha`, `sell_alpha`, `transfer_tao`, `transfer_staked_alpha`, and `current_alpha_price`. |
| `TimeProvider`        | `UnixTime`                                        | Current wall-clock time for expiry checks. |
| `FeeCollector`        | `Get<Self::AccountId>` (constant)                 | Account that receives all accumulated protocol fees in TAO. |
| `MaxOrdersPerBatch`   | `Get<u32>` (constant)                             | Maximum number of orders accepted in a single `execute_orders` or `execute_batched_orders` call. Should equal `floor(max_block_weight / per_order_weight)`. |
| `PalletId`            | `Get<PalletId>` (constant)                        | Used to derive the pallet intermediary account (`PalletId::into_account_truncating`). This account temporarily holds pooled TAO and staked alpha during `execute_batched_orders`. |
| `PalletHotkey`        | `Get<Self::AccountId>` (constant)                 | Hotkey the pallet intermediary account stakes to/from during batch execution. Must be a dedicated hotkey registered on every subnet the pallet may operate on. Operators should register it as a non-validator neuron. |

---

## Extrinsics

### `execute_orders(orders)` — call index 0

**Origin:** any signed account (typically a relayer).

Executes a list of signed limit orders one by one, each interacting with the
AMM pool independently. Orders that fail validation or whose price condition is
not met are silently skipped — a single bad order does not revert the batch.

**Fee handling:** protocol fee is deducted from each order's input before the
pool swap.

**When to use:** suitable for small batches or when orders target different
subnets. Use `execute_batched_orders` for same-subnet batches to reduce price
impact.

---

### `execute_batched_orders(netuid, orders)` — call index 4

**Origin:** any signed account (typically a relayer).

Aggregates all valid orders targeting `netuid` into a single net pool
interaction:

1. **Validate & classify** — orders with wrong netuid, invalid signature,
   already-processed id, past expiry, or price condition not met emit
   `OrderSkipped` and are dropped. The rest are split into `buys` and `sells`.

2. **Collect assets** — gross TAO is pulled from each buyer's free balance into
   the pallet intermediary account. Gross alpha stake is moved from each seller's
   `(coldkey, hotkey)` position to the pallet intermediary's `(pallet_account,
   pallet_hotkey)` position.

3. **Net pool swap** — buy TAO and sell alpha are converted to a common TAO
   basis at the current spot price and offset against each other. Only the
   residual amount touches the pool in a single swap:
   - Buy-dominant: residual TAO is sent to the pool; pool returns alpha.
   - Sell-dominant: residual alpha is sent to the pool; pool returns TAO.
   - Perfectly offset: no pool interaction.

4. **Distribute alpha pro-rata** — every buyer receives their share of the total
   available alpha (pool output + seller passthrough alpha). Share is
   proportional to each buyer's net TAO contribution. Integer division floors
   each share; any remainder stays in the pallet intermediary account as dust.

5. **Distribute TAO pro-rata** — every seller receives their share of the total
   available TAO (pool output + buyer passthrough TAO), minus the protocol fee.
   Share is proportional to each seller's alpha valued at the current spot price.
   Integer division floors each share; any remainder stays in the pallet
   intermediary account as dust.

6. **Collect fees** — total buy-side fees (withheld from TAO input) plus total
   sell-side fees (withheld from TAO output) are forwarded in a single transfer
   to `FeeCollector`.

7. **Emit `GroupExecutionSummary`.**

> **Note:** rounding dust (alpha and TAO) accumulates in the pallet intermediary
> account between batches. If an emission epoch fires while dust is present, the
> pallet earns emissions it never distributes. See the TODO in `collect_fees`.

---

### `cancel_order(order)` — call index 1

**Origin:** the order's `signer` (coldkey).

Registers a cancellation intent by writing the `OrderId` into `Orders` as
`Cancelled`. Once cancelled an order can never be executed. The full `Order`
payload is required so the pallet can derive the `OrderId`.

---

### `set_protocol_fee(fee)` — call index 3

**Origin:** root or the current admin account (see `set_admin`).

Sets `ProtocolFee` to `fee` (PPB). Emits `ProtocolFeeSet`.

---

### `set_admin(new_admin)` — call index 5

**Origin:** root.

Sets or clears the privileged admin account stored in `Admin`. Pass `None` to
remove the admin, leaving only root able to change the fee. Emits `AdminSet`.

---

## Events

| Event | Fields | Emitted when |
|-------|--------|--------------|
| `OrderExecuted` | `order_id`, `signer`, `netuid`, `side` | An individual order was successfully executed (by either extrinsic). |
| `OrderSkipped` | `order_id` | An order was dropped during batch validation (bad signature, expired, wrong netuid, already processed, or price condition not met). |
| `OrderCancelled` | `order_id`, `signer` | The signer registered a cancellation via `cancel_order`. |
| `ProtocolFeeSet` | `fee` | Root or admin updated the protocol fee. |
| `AdminSet` | `new_admin` | Root updated the admin account (`None` means admin was removed). |
| `GroupExecutionSummary` | `netuid`, `net_side`, `net_amount`, `actual_out`, `executed_count` | Emitted once per `execute_batched_orders` call summarising the net pool trade. `net_side` is `Buy` if TAO was sent to the pool, `Sell` if alpha was sent. `net_amount` and `actual_out` are zero when the two sides perfectly offset. |

---

## Errors

| Error | Cause |
|-------|-------|
| `InvalidSignature` | Signature does not match the order payload and signer. Also used as a catch-all for failed validation in `execute_orders`. |
| `OrderAlreadyProcessed` | The `OrderId` is already present in `Orders` (either `Fulfilled` or `Cancelled`). |
| `OrderExpired` | `now > order.expiry`. |
| `PriceConditionNotMet` | Current spot price is beyond the order's `limit_price`. |
| `Unauthorized` | Caller of `cancel_order` is not the order's `signer`. |
| `NotAdmin` | Caller of `set_protocol_fee` is neither root nor the current admin. |
| `SwapReturnedZero` | The pool swap returned zero output for a non-zero residual input. |

---

## Fee model

All fees are collected in TAO regardless of order side.

| Order side | Fee deducted from | Timing |
|------------|-------------------|--------|
| Buy        | TAO input         | Before pool swap (`validate_and_classify`) |
| Sell       | TAO output        | After pool swap (`distribute_tao_pro_rata`) |

Fee formula: `fee = floor(amount × fee_ppb / 1_000_000_000)`.

Accumulated fees are forwarded to `FeeCollector` at the end of each batch
execution in a single transfer.
