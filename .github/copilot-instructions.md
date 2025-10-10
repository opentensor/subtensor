When reviewing pull requests, please adhere to and keep in mind the following:
* Unless this PR is a hotfix or a deployment from `devnet-ready` => `devnet`, `devnet` =>
  `testnet`, or `testnet` => `main`, all new PRs should be opened against the `devnet-ready`
  branch. The `devnet` and `testnet` branches should only be updated via merges from
  `devnet-ready` and `testnet-ready`, respectively, and are designed to track the current state
  of the `devnet` and `testnet` networks. Feel free in your review to encourage people to
  change the target to `devnet-ready` if they are targeting `main` and this is not a hotfix PR.
* Bittensor is a substrate-based blockchain. It is critical that there are never panics in the
  runtime, as this can brick the chain. You should be flagging any runtime/pallet code
  (extrinsics, code that could be called by extrinsics, code in migrations, etc) that could
  panic because of changes introduced by the PR. We do have clippy lints in place to detect
  this, but these do not always catch everything. We CANNOT have panics in the runtime.
* Remember that things like direct indexing (i.e. `arr[3]`) can panic and are dangerous when
  used in a substrate runtime. Everything should be using `get` / `set` and `Result` / `Option`
  wherever possible.
* Similarly you should look out for unsafe math, such as scenarios where a value might
  overflow, potential divide by zero errors, etc., programmers should always be using checked
  or saturating math ops.
* If one of the lints is failing (clippy / cargo fmt / cargo fix / etc), you should suggest
  that they run `./scripts/fix_rust.sh` as this will typically auto-resolve the issue. This
  includes issues like uncommitted `Cargo.lock`.
* There should never be unsafe code in the runtime.
* You should be extra careful with contributions from external contributors (users without the
  "Nucleus" role)
* Be extra vigilent for PRs that introduce a security vulnerability or exploit. Always call out
  potential security vulnerabilities.
* Code should be performant and maintainable, with maximum safety and security. $4B market cap
  is on the line.
* You should be concise in your review, only report legitimate issues, no fluff.