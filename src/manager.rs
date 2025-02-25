use crate::types::{GaiaError, GaiaNodeConfig, GaiaNodeStatus, NodeInfo, Result};
use std::path::PathBuf;
use std::process::{Child, Command};
use std::sync::{Arc, Mutex};
use tokio::sync::Mutex as TokioMutex;

pub struct GaiaNodeManager {
    node_process: Arc<Mutex<Option<Child>>>,
    status: Arc<TokioMutex<GaiaNodeStatus>>,
    node_path: PathBuf,
}

impl GaiaNodeManager {
    pub fn new() -> Result<Self> {
        let node_path = Self::find_node_binary()?;

        Ok(Self {
            node_process: Arc::new(Mutex::new(None)),
            status: Arc::new(TokioMutex::new(GaiaNodeStatus::Stopped)),
            node_path,
        })
    }

    fn find_node_binary() -> Result<PathBuf> {
        // Try common locations for the binary
        let locations = ["/usr/local/bin/gaianet", "/usr/bin/gaianet"];

        for location in locations {
            let path = PathBuf::from(location);
            if path.exists() {
                return Ok(path);
            }
        }

        // If not found in common locations, try to find it in PATH
        match which::which("gaianet") {
            Ok(path) => Ok(path),
            Err(_) => Err(GaiaError::InitializationFailed(
                "GaiaNet binary not found. Please install GaiaNet first.".into(),
            )),
        }
    }

    pub async fn start(&self, config: GaiaNodeConfig) -> Result<()> {
        let mut status = self.status.lock().await;

        if *status != GaiaNodeStatus::Stopped {
            return Err(GaiaError::InvalidState("Node is already running".into()));
        }

        blueprint_sdk::logging::info!("Starting GaiaNet node with config: {:?}", config);

        let data_dir = PathBuf::from(&config.data_dir);
        if !data_dir.exists() {
            std::fs::create_dir_all(&data_dir)
                .map_err(|e| GaiaError::Io(format!("Failed to create data directory: {}", e)))?;
        }

        let mut cmd = Command::new(&self.node_path);
        cmd.arg("start")
            .arg("--data-dir")
            .arg(&config.data_dir)
            .arg("--network")
            .arg(&config.network);

        if config.verbose {
            cmd.arg("--verbose");
        }

        let child = cmd
            .spawn()
            .map_err(|e| GaiaError::Io(format!("Failed to start node process: {}", e)))?;

        let mut node_process = self
            .node_process
            .lock()
            .map_err(|_| GaiaError::Internal("Lock poisoned".into()))?;
        *node_process = Some(child);

        // Update status
        *status = GaiaNodeStatus::Running;

        Ok(())
    }

    pub async fn stop(&self) -> Result<()> {
        let mut status = self.status.lock().await;

        if *status == GaiaNodeStatus::Stopped {
            return Err(GaiaError::InvalidState("Node is not running".into()));
        }

        blueprint_sdk::logging::info!("Stopping GaiaNet node");

        let mut node_process = self
            .node_process
            .lock()
            .map_err(|_| GaiaError::Internal("Lock poisoned".into()))?;

        if let Some(mut child) = node_process.take() {
            // Try graceful shutdown first
            let output = Command::new(&self.node_path)
                .arg("stop")
                .output()
                .map_err(|e| GaiaError::Io(format!("Failed to execute stop command: {}", e)))?;

            if !output.status.success() {
                // If graceful shutdown fails, kill the process
                blueprint_sdk::logging::warn!("Graceful shutdown failed, killing node process");
                child
                    .kill()
                    .map_err(|e| GaiaError::Io(format!("Failed to kill node process: {}", e)))?;
            }

            // Wait for the process to exit
            child
                .wait()
                .map_err(|e| GaiaError::Io(format!("Failed to wait for node process: {}", e)))?;
        }

        // Update status
        *status = GaiaNodeStatus::Stopped;

        Ok(())
    }

    pub async fn get_status(&self) -> GaiaNodeStatus {
        let status = self.status.lock().await;
        status.clone()
    }

    pub async fn get_info(&self) -> Result<NodeInfo> {
        let status = self.status.lock().await;

        if *status == GaiaNodeStatus::Stopped {
            return Err(GaiaError::InvalidState("Node is not running".into()));
        }

        let output = Command::new(&self.node_path)
            .arg("info")
            .output()
            .map_err(|e| GaiaError::Io(format!("Failed to get node info: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(GaiaError::CommandFailed(format!(
                "Failed to get node info: {}",
                stderr
            )));
        }

        let info_str = String::from_utf8_lossy(&output.stdout);
        let info: NodeInfo = serde_json::from_str(&info_str)
            .map_err(|e| GaiaError::ParseError(format!("Failed to parse node info: {}", e)))?;

        Ok(info)
    }
}

impl Drop for GaiaNodeManager {
    fn drop(&mut self) {
        // Try to stop the node if it's still running when the manager is dropped
        if let Ok(mut node_process) = self.node_process.lock() {
            if let Some(mut child) = node_process.take() {
                let _ = child.kill(); // Ignore errors during drop
                let _ = child.wait(); // Ignore errors during drop
            }
        }
    }
}
