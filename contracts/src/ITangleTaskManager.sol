// SPDX-License-Identifier: UNLICENSED
pragma solidity >=0.8.13;

import "eigenlayer-middleware/src/libraries/BN254.sol";

interface ITangleTaskManager {
    // EVENTS
    event GaiaNodeStarted(
        uint32 indexed taskId,
        string network,
        string dataDir,
        address indexed operator,
        uint256 timestamp
    );

    event GaiaNodeStopped(
        uint32 indexed taskId,
        address indexed operator,
        uint256 timestamp
    );

    // STRUCTS

    struct GaiaNodeConfig {
        string network;
        string dataDir;
        bool isRunning;
        uint256 startTime;
        address operator;
    }

    struct GaiaNodeStatus {
        bool isRunning;
        uint256 uptime;
        address operator;
    }
    

    // Task response is hashed and signed by operators.
    // these signatures are aggregated and sent to the contract as response.
    struct TaskResponse {
        // Can be obtained by the operator from the event NewTaskCreated.
        uint32 referenceTaskIndex;
        // This is just the response that the operator has to compute by itself.
        uint256 numberSquared;
    }

    // Extra information related to taskResponse, which is filled inside the contract.
    // It thus cannot be signed by operators, so we keep it in a separate struct than TaskResponse
    // This metadata is needed by the challenger, so we emit it in the TaskResponded event
    struct TaskResponseMetadata {
        uint32 taskResponsedBlock;
        bytes32 hashOfNonSigners;
    }

    // FUNCTIONS
    // NOTE: this function starts a new Gaia node.
    function startGaiaNode(
      string memory network,
      string memory dataDir
    ) external returns (uint32 taskId);

    // NOTE: this function stop the Gaia node.
    function stopGaiaNode(uint32 taskId) external;

    function getGaiaNodeStatus(uint32 taskId) external view returns (GaiaNodeStatus memory);
}
