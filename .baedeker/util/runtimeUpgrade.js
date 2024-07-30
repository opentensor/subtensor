const { ApiPromise, WsProvider } = require('@polkadot/api');
const { Keyring } = require('@polkadot/keyring');
const fs = require('fs');

const NODE_URL = 'ws://127.0.0.1:9946';
const SUDO_ACCOUNT_SEED = '//Alice';
const WASM_FILE_PATH = '../../wasm/node_subtensor_runtime.wasm';

async function main() {
  // Create a provider connected to the local node
  const provider = new WsProvider(NODE_URL);
  
  // Create the API instance
  const api = await ApiPromise.create({ provider });

  // Create a keyring instance
  const keyring = new Keyring({ type: 'sr25519' });

  // Load the sudo account from the seed phrase
  const sudoAccount = keyring.addFromUri(SUDO_ACCOUNT_SEED);

  // Read the WASM file
  const wasmCode = fs.readFileSync(WASM_FILE_PATH).toString('hex');

  // Construct the sudo call to set the new code
  const sudoCall = api.tx.sudo.sudo(
    api.tx.system.setCode(`0x${wasmCode}`)
  );

  // Send the transaction using the sudo account
  await sudoCall.signAndSend(sudoAccount, ({ status }) => {
    if (status.isInBlock) {
      console.log(`Transaction included at blockHash ${status.asInBlock}`);
      process.exit(0);
    }
  });
}

main().catch((error) => {
  console.error(error);
  process.exit(1);
});
