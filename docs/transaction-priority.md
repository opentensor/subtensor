## Transaction Priority

### Overview
In Subtensor, transaction priority is determined by custom transaction extensions, which alter or override the default Substrate SDK behavior. Extensions affecting transaction priority are:

- **`ChargeTransactionPaymentWrapper`** (wraps `ChargeTransactionPayment`)
- **`SubtensorTransactionExtension`**

Substrate SDK combines priorities from all transaction extensions using addition. 

---

### 1. `ChargeTransactionPaymentWrapper`
In the Substrate SDK, `ChargeTransactionPayment` normally calculates transaction priority based on:
- **Tip** — an extra fee paid by the sender.
- **Weight** — computational complexity of the transaction.
- **Dispatch class** — category of the transaction (`Normal`, `Operational`, `Mandatory`).

However, in Subtensor, `ChargeTransactionPaymentWrapper` **overrides** this logic.  
It replaces the dynamic calculation with a **flat priority scale** based only on the dispatch class.

#### Current priority values:
| Dispatch Class      | Priority Value    | Notes |
|---------------------|-------------------|-------|
| `Normal`            | `1`               | Standard transactions |
| `Mandatory`         | `1`               | Rarely used, same as `Normal` |
| `Operational`       | `10_000_000_000`  | Reserved for critical system extrinsics (e.g., `sudo` calls, `drand` pulses) |

---

### 2. `SubtensorTransactionExtension`
This extension introduces **special priority rules** for certain extrinsics, such as:
- `commit_weights`
- `reveal_weights`
- etc.

For these, priority is **boosted** using a time-sensitive factor:
1. Retrieve the **current block number**.
2. Retrieve the **block number of the last axon registration**.
3. Calculate the **difference** between them.
4. **Add** this difference to the transaction’s priority.

---