rm -rf .papi
npx papi add devnet -w ws://localhost:9944
npx papi ink add ./bittensor/target/ink/bittensor.json 
npx papi ink add ./direct-call/target/ink/direct_call.json
npx papi ink add ./delegate-call/target/ink/delegate_call.json