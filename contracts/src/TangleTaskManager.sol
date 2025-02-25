// SPDX-License-Identifier: UNLICENSED
pragma solidity >=0.8.13;

import "@openzeppelin-upgrades/contracts/proxy/utils/Initializable.sol";
import "@openzeppelin-upgrades/contracts/access/OwnableUpgradeable.sol";
import "eigenlayer-contracts/src/contracts/permissions/Pausable.sol";
import "eigenlayer-middleware/src/interfaces/IServiceManager.sol";
import {BLSApkRegistry} from "eigenlayer-middleware/src/BLSApkRegistry.sol";
import {RegistryCoordinator} from "eigenlayer-middleware/src/RegistryCoordinator.sol";
import {BLSSignatureChecker, IRegistryCoordinator} from "eigenlayer-middleware/src/BLSSignatureChecker.sol";
import {OperatorStateRetriever} from "eigenlayer-middleware/src/OperatorStateRetriever.sol";
import "eigenlayer-middleware/src/libraries/BN254.sol";
import "contracts/src/ITangleTaskManager.sol";

contract TangleTaskManager is
    Initializable,
    OwnableUpgradeable,
    Pausable,
    BLSSignatureChecker,
    OperatorStateRetriever,
    ITangleTaskManager
{
    using BN254 for BN254.G1Point;

    /* CONSTANT */
    // The number of blocks from the task initialization within which the aggregator has to respond to
    uint32 public immutable TASK_RESPONSE_WINDOW_BLOCK;
    uint32 public constant TASK_CHALLENGE_WINDOW_BLOCK = 100;
    uint256 internal constant _THRESHOLD_DENOMINATOR = 100;

    /* STORAGE */
    // The latest task index
    uint32 public latestTaskNum;
    mapping(uint32 => GaiaNodeConfig) private nodeConfigs;
    mapping(address => uint32[]) private operatorTasks;

    // mapping of task indices to all tasks hashes
    // when a task is created, task hash is stored here,
    // and responses need to pass the actual task,
    // which is hashed onchain and checked against this mapping
    mapping(uint32 => bytes32) public allTaskHashes;

    // mapping of task indices to hash of abi.encode(taskResponse, taskResponseMetadata)
    mapping(uint32 => bytes32) public allTaskResponses;

    mapping(uint32 => bool) public taskSuccesfullyChallenged;

    address public aggregator;
    address public generator;

    /* MODIFIERS */
    modifier onlyAggregator() {
        require(msg.sender == aggregator, "Aggregator must be the caller");
        _;
    }

    // onlyTaskGenerator is used to restrict createNewTask from only being called by a permissioned entity
    // in a real world scenario, this would be removed by instead making createNewTask a payable function
    modifier onlyTaskGenerator() {
        require(msg.sender == generator, "Task generator must be the caller");
        _;
    }

    modifier onlyTaskOperator(uint32 taskId) {
        require(nodeConfigs[taskId].operator == msg.sender, "Not task operator");
        _;
    }

    modifier validTaskId(uint32 taskId) {
        require(taskId > 0 && taskId <= latestTaskNum, "Invalid task ID");
        _;
    }

    constructor(
        IRegistryCoordinator _registryCoordinator,
        uint32 _taskResponseWindowBlock
    ) BLSSignatureChecker(_registryCoordinator) {
        TASK_RESPONSE_WINDOW_BLOCK = _taskResponseWindowBlock;
    }

    function initialize(
        IPauserRegistry _pauserRegistry,
        address initialOwner,
        address _aggregator,
        address _generator
    ) public initializer {
        _initializePauser(_pauserRegistry, UNPAUSE_ALL);
        _transferOwnership(initialOwner);
        aggregator = _aggregator;
        generator = _generator;
    }

    /* FUNCTIONS */
    // NOTE: this function creates a new Gaia node
    function startGaiaNode(
        string memory network,
        string memory dataDir
    ) external override whenNotPaused returns (uint32) {
        require(bytes(network).length > 0, "Network cannot be empty");
        require(bytes(dataDir).length > 0, "Data directory cannot be empty");

        latestTaskNum++;
        uint32 taskId = latestTaskNum;

        nodeConfigs[taskId] = GaiaNodeConfig({
            network: network,
            dataDir: dataDir,
            isRunning: true,
            startTime: block.timestamp,
            operator: msg.sender
        });

        operatorTasks[msg.sender].push(taskId);

        emit GaiaNodeStarted(
            taskId,
            network,
            dataDir,
            msg.sender,
            block.timestamp
        );

        return taskId;
    }

    function stopGaiaNode(
        uint32 taskId
    ) external override whenNotPaused validTaskId(taskId) onlyTaskOperator(taskId) {
        require(nodeConfigs[taskId].isRunning, "Node not running");
        
        nodeConfigs[taskId].isRunning = false;

        emit GaiaNodeStopped(
            taskId,
            msg.sender,
            block.timestamp
        );
    }
}
