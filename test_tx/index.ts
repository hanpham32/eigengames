import { ethers, Contract } from "ethers";
import yargs from "yargs";
import { hideBin } from "yargs/helpers";

async function main(): Promise<void> {
  // Parse command-line arguments
  const argv = await yargs(hideBin(process.argv))
    .option("start", {
      alias: "s",
      description: "Start a Gaia node",
      type: "boolean",
    })
    .option("stop", {
      alias: "x",
      description: "Stop a Gaia node",
      type: "boolean",
    })
    .option("status", {
      description: "Get the status of a Gaia node by taskId",
      type: "number",
    })
    .option("port", {
      alias: "p",
      description: "Port number for the Ethereum provider",
      type: "number",
      default: 55002, // Default port if not provided
    })
    .help()
    .alias("help", "h").argv;

  // Connect to the Ethereum provider using the provided port
  const provider = new ethers.JsonRpcProvider(`http://localhost:${argv.port}`);

  // Set up wallet with private key
  const privateKey =
    "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
  const wallet = new ethers.Wallet(privateKey, provider);

  // Define the contract ABI (just the function we need)
  const abi = [
    "function startGaiaNode(string memory network, string memory dataDir) external returns (uint32)",
    "function stopGaiaNode(uint32 taskId) external",
    "function latestTaskNum() external view returns (uint32)",
    "function getGaiaNodeStatus(uint32 taskId) external view returns (bool, uint256, address)",
  ];

  // Create contract instance
  const contractAddress = "0x07882Ae1ecB7429a84f1D53048d35c4bB2056877";
  const contract = new ethers.Contract(contractAddress, abi, wallet);

  console.log("Sending transaction...");

  try {
    if (argv.start) {
      const receipt = await startGaiaNode(contract);
      console.log("Node started successfully:", receipt);
    } else if (argv.stop) {
      const latestTaskId = await contract.latestTaskNum();
      console.log("Latest task id:", latestTaskId);
      const receipt = await stopGaiaNode(contract, latestTaskId);
      console.log("Node stopped successfully:", receipt);
    } else if (argv.status) {
      const status = await getGaiaNodeStatus(contract, argv.status);
      console.log("Node status:", status);
    } else {
      console.error(
        "No action specified. Use --start, --stop, or --status <taskId>."
      );
      process.exit(1);
    }
  } catch (error) {
    console.error("Transaction failed:", error);
  }
}

// Function to send a transaction to start a Gaia node
// Returns a tx receipt
async function startGaiaNode(contract: Contract): Promise<any> {
  // Call the contract function
  const tx = await contract.startGaiaNode("testnet", "data/gaia/node1");
  console.log("Transaction hash:", tx.hash);

  // Wait for transaction to be mined
  const receipt = await tx.wait();
  console.log("Transaction mined in block:", receipt.blockNumber);
  return receipt;
}

// Function to send a transaction to stop a Gaia node
// Returns a tx receipt
async function stopGaiaNode(contract: Contract, taskId: number): Promise<any> {
  // Call the contract function
  const tx = await contract.stopGaiaNode(taskId);
  console.log("Transaction hash:", tx.hash);

  // Wait for transaction to be mined
  const receipt = await tx.wait();
  console.log("Transaction mined in block:", receipt.blockNumber);
  return receipt;
}

// Function to retrieve the status of a Gaia node
// Returns the GaiaNodeStatus struct
async function getGaiaNodeStatus(
  contract: ethers.Contract,
  taskId: number
): Promise<{ isRunning: boolean; uptime: number; operator: string }> {
  // Call the contract function
  const [isRunning, uptime, operator] = await contract.getGaiaNodeStatus(
    taskId
  );
  return { isRunning, uptime, operator: operator.toString() };
}

main().catch((error: any) => {
  console.error("Unhandled error:", error);
  process.exit(1);
});
