const { ApiPromise, WsProvider } = require('@polkadot/api');
const { Keyring } = require('@polkadot/keyring');
const fs = require('fs');

const NODE_URL = 'ws://127.0.0.1:9946';
const SUDO_ACCOUNT_SEED = '//Alice';
const WASM_FILE_PATH = '../../wasm/node_subtensor_runtime.wasm';

function sendTransaction(call, signer) {
  return new Promise((resolve, reject) => {
    call.signAndSend(signer, ({ status, events, dispatchError }) => {
      // Check for transaction errors
      if (dispatchError) {
        if (dispatchError.isModule) {
          // for module errors, we have the section indexed, lookup
          const decoded = api.registry.findMetaError(dispatchError.asModule);
          const { docs, name, section } = decoded;
          console.error(`${section}.${name}: ${docs.join(' ')}`);
        } else {
          // Other, such as system, errors
          console.error(dispatchError.toString());
        }
        reject(dispatchError.toString());
      }
      // Log and resolve when the transaction is included in a block
      if (status.isInBlock) {
        console.log(`Transaction included at blockHash ${status.asInBlock}`);
        resolve(status.asInBlock);
      }
    }).catch((error) => {
      console.error('Failed to send transaction:', error);
      reject(error);
    });
  });
}

async function main() {
  // Create a provider connected to the local node
  const provider = new WsProvider(NODE_URL);
  
  // Create the API instance
  const api = await ApiPromise.create({ provider });

  // Create a keyring instance
  const keyring = new Keyring({ type: 'sr25519' });

  // Load the sudo account from the seed phrase
  const sudoAccount = keyring.addFromUri(SUDO_ACCOUNT_SEED);

  // Construct and execute the sudo call to increase Alice's balance
  const sudoCallSetBalance = api.tx.sudo.sudo(
    api.tx.balances.forceSetBalance(sudoAccount.address, `1000000000000`)
  );
  console.log("Increasing Alice balance...");
  await sendTransaction(sudoCallSetBalance, sudoAccount);
  console.log("Increasing Alice balance - done");

  // Read the WASM file
  const wasmCode = fs.readFileSync(WASM_FILE_PATH).toString('hex');

  // Construct the sudo call to set the new code
  const sudoCallSetCode = api.tx.sudo.sudo(
    api.tx.system.setCode(`0x${wasmCode}`)
  );

  // Send the transaction using the sudo account
  console.log("Running runtime upgrade...");
  await sendTransaction(sudoCallSetCode, sudoAccount);

  // Sleep for a minute to make sure migrations have run
  console.log("Sleep for a minute to make sure migrations have run");
  await new Promise(r => setTimeout(r, 60000));

  // Test that chain is functionning with a balance transfer
  const bob = keyring.addFromUri("//Bob");
  const balanceTransfer = api.tx.balances.transferKeepAlive(bob.address, `1000000000`);
  console.log("Executing balance transfer...");
  await sendTransaction(balanceTransfer, sudoAccount);
  console.log("Balance transfer successful, chain is working, blocks are being produced");

  process.exit(0);
}

main().catch((error) => {
  console.error(error);
  process.exit(1);
});
