use dirs;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::PathBuf;

pub type Result<T> = std::result::Result<T, GaiaError>;

#[derive(Debug)]
pub enum GaiaError {
    Io(String),
    CommandFailed(String),
    ParseError(String),
    InvalidState(String),
    InitializationFailed(String),
    Internal(String),
}

impl fmt::Display for GaiaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GaiaError::Io(msg) => write!(f, "IO error: {}", msg),
            GaiaError::CommandFailed(msg) => write!(f, "Command failed: {}", msg),
            GaiaError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            GaiaError::InvalidState(msg) => write!(f, "Invalid state: {}", msg),
            GaiaError::InitializationFailed(msg) => write!(f, "Initialization failed: {}", msg),
            GaiaError::Internal(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl std::error::Error for GaiaError {}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum GaiaNodeStatus {
    Running,
    Starting,
    Stopped,
    Syncing {
        current_height: u64,
        target_height: u64,
    },
    Error(String),
}

impl fmt::Display for GaiaNodeStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GaiaNodeStatus::Running => write!(f, "Running"),
            GaiaNodeStatus::Starting => write!(f, "Starting"),
            GaiaNodeStatus::Stopped => write!(f, "Stopped"),
            GaiaNodeStatus::Syncing {
                current_height,
                target_height,
            } => {
                write!(
                    f,
                    "Syncing (current_height: {}, target_height: {})",
                    current_height, target_height
                )
            }
            GaiaNodeStatus::Error(err_msg) => write!(f, "Error: {}", err_msg),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GaiaNodeConfig {
    pub data_dir: String,
    pub network: String,
    pub verbose: bool,
}

impl Default for GaiaNodeConfig {
    fn default() -> Self {
        let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));

        Self {
            data_dir: home_dir.join(".gaianet").to_string_lossy().to_string(),
            network: "mainnet".to_string(),
            verbose: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    pub version: String,
    pub network: String,
    pub node_id: String,
    pub peers: u32,
    pub sync_status: Option<SyncStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncStatus {
    pub current_height: u64,
    pub target_height: u64,
    pub progress: f64,
}
