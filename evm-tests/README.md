# type-test

test with ts

## polkadot api

```bash
npx papi add devnet -w ws://10.0.0.11:9944
```

## get the new metadata

```bash
sh get-metadata.sh
```

## run all tests

```bash
yarn run test
```

## To run a particular test case, you can pass an argument with the name or part of the name. For example:

```bash
yarn run test -- -g "Can set subnet parameter"
```
