// send-tx.ts
import { ethers, Contract } from "ethers";

async function main(): Promise<void> {
  // Connect to the Ethereum provider
  const provider = new ethers.JsonRpcProvider("http://localhost:55000");

  // Set up wallet with private key
  const privateKey =
    "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
  const wallet = new ethers.Wallet(privateKey, provider);

  // Define the contract ABI (just the function we need)
  const abi = [
    "function startGaiaNode(string memory network, string memory dataDir) external returns (uint32)",
    "function stopGaiaNode(uint32 taskId) external",
    "function latestTaskNum() external view returns (uint32)",
  ];

  // Create contract instance
  const contractAddress = "0x07882Ae1ecB7429a84f1D53048d35c4bB2056877";
  const contract = new ethers.Contract(contractAddress, abi, wallet);

  console.log("Sending transaction...");

  try {
    const latestTaskId = await contract.latestTaskNum();
    console.log("Latest task id:", latestTaskId);
    const tx = await stopGaiaNode(contract, latestTaskId);
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

// Function to send a transaction to start a Gaia node
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

main().catch((error: any) => {
  console.error("Unhandled error:", error);
  process.exit(1);
});
