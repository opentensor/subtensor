const { ApiPromise, WsProvider } = require('@polkadot/api');

async function main() {
  const wsProvider = new WsProvider('wss://entrypoint-finney.opentensor.ai:443');
  const api = await ApiPromise.create({ provider: wsProvider });

  // Wait for API to be ready
  await api.isReady;

  // Log available modules and methods to help diagnose
  console.log("Available modules:", Object.keys(api.query));

  // For SubtensorModule, the correct casing is important
  if (api.query.subtensorModule) {
    console.log("Subtensor methods:", Object.keys(api.query.subtensorModule));
  } else if (api.query.SubtensorModule) {
    console.log("Subtensor methods:", Object.keys(api.query.SubtensorModule));
  } else {
    console.log("Subtensor module not found, checking all available modules");
  }

  // Try to access the storage with the correct casing
  try {
    // Use the correct casing: SubnetTAO instead of subnetTao
    const key = api.query.subtensorModule?.subnetTAO?.key(1) ||
                api.query.SubtensorModule?.subnetTAO?.key(2);
    if (key) {
      console.log(key);
    } else {
      console.log("Storage method not found");
    }
  } catch (err) {
    console.error("Error accessing storage:", err.message);
  }
}

main().catch(err => {
  console.error("Error in main:", err);
  process.exit(1);
});