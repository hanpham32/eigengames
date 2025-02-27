use crate::types::{GaiaError, GaiaNodeConfig, GaiaNodeStatus, NodeInfo, Result};
use blueprint_sdk::logging::{error, info, warn};
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Arc;
use tokio::process::{Child, Command};
use tokio::sync::Mutex; // Added import for tokio::process::Child

#[derive(Clone)]
pub struct GaiaNodeManager {
    // holds the node process representing the gaia node
    node_process: Arc<Mutex<Option<Child>>>,
    // manages the node states
    status: Arc<Mutex<GaiaNodeStatus>>,
    node_path: PathBuf,
}

impl GaiaNodeManager {
    pub fn new() -> Result<Self> {
        let node_path = Self::find_node_binary()?;
        blueprint_sdk::logging::info!("node path: {:?}", node_path.clone());

        Ok(Self {
            node_process: Arc::new(Mutex::new(None)),
            status: Arc::new(Mutex::new(GaiaNodeStatus::Stopped)),
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

    pub async fn start(&self, _config: GaiaNodeConfig) -> Result<()> {
        info!("Starting Gaianet Node...");
        let path = which::which("gaianet").unwrap();
        info!("Running Gaia at path: {:?}", path);

        let mut command: Child = Command::new(path)
            .arg("start")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        info!("Running command: {:?}", command);

        {
            let mut status_lock = self.status.lock().await;
            *status_lock = GaiaNodeStatus::Starting;
        }

        let stdout = command
            .stdout
            .take()
            .expect("Failed to capture child stdout");

        let _stdout_handle = tokio::spawn(async move {
            use tokio::io::{AsyncBufReadExt, BufReader};
            let mut reader = BufReader::new(stdout);
            let mut line = String::new();
            while let Ok(bytes_read) = reader.read_line(&mut line).await {
                if bytes_read == 0 {
                    break; // EOF
                }
                // Print the child's stdout line to our own stdout
                println!("[Child stdout] {}", line.trim_end());
                line.clear();
            }
        });

        let stderr = command
            .stderr
            .take()
            .expect("Failed to capture child stderr");

        // Read stderr in another task
        let _stderr_handle = tokio::spawn(async move {
            use tokio::io::{AsyncBufReadExt, BufReader};
            let mut reader = BufReader::new(stderr);
            let mut line = String::new();
            while let Ok(bytes_read) = reader.read_line(&mut line).await {
                if bytes_read == 0 {
                    break; // EOF
                }
                // Print the child's stderr line to our own stderr
                eprintln!("[Child stderr] {}", line.trim_end());
                line.clear();
            }
        });

        // let _ = tokio::join!(stdout_handle, stderr_handle);
        {
            let mut status_lock = self.status.lock().await;
            *status_lock = GaiaNodeStatus::Running;
            info!("Node status: {}", status_lock.to_string());
        }

        Ok(())
    }

    pub async fn stop(&self) -> Result<()> {
        info!("Inside stop() function!");
        {
            let status_lock = self.status.lock().await;
            info!("Node status: {}", status_lock.to_string());
        }

        let path = which::which("gaianet").unwrap();
        info!("Running Gaia at path: {:?}", path);

        let mut command: Child = Command::new(path)
            .arg("stop")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();

        let stdout = command
            .stdout
            .take()
            .expect("Failed to capture child stdout");

        let _stdout_handle = tokio::spawn(async move {
            use tokio::io::{AsyncBufReadExt, BufReader};
            let mut reader = BufReader::new(stdout);
            let mut line = String::new();
            while let Ok(bytes_read) = reader.read_line(&mut line).await {
                if bytes_read == 0 {
                    break; // EOF
                }
                // Print the child's stdout line to our own stdout
                println!("[Child stdout] {}", line.trim_end());
                line.clear();
            }
        });

        info!("Running command: {:?}", command);
        Ok(())
    }

    pub async fn get_status(&self) -> GaiaNodeStatus {
        let status = self.status.lock().await;
        status.clone()
    }

    // pub async fn get_info(&self) -> Result<NodeInfo> {
    // }
}

impl Drop for GaiaNodeManager {
    fn drop(&mut self) {
        // Try to stop the node if it's still running when the manager is dropped
        if let Ok(mut node_process) = self.node_process.try_lock() {
            if let Some(mut child) = node_process.take() {
                let _ = child.kill(); // Ignore errors during drop
                let _ = child.wait(); // Ignore errors during drop
            }
        }
    }
}
