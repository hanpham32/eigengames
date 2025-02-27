use alloy_rpc_client::ReqwestClient;
use color_eyre::Result;
use eigensdk::crypto_bls::{OperatorId, Signature};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::time::{sleep, Duration};
use tracing::{debug, info};

use crate::ITangleTaskManager::TaskResponse;

const MAX_RETRIES: u32 = 5;
const INITIAL_RETRY_DELAY: Duration = Duration::from_secs(1);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedTaskResponse {
    pub task_response: TaskResponse,
    pub signature: Signature,
    pub operator_id: OperatorId,
}

/// Client for interacting with the Aggregator RPC server
#[derive(Debug, Clone)]
pub struct AggregatorClient {
    client: ReqwestClient,
}
impl AggregatorClient {
    /// Creates a new AggregatorClient
    pub fn new(aggregator_address: &str) -> Result<Self> {
        let url = Url::parse(&format!("http://{}", aggregator_address))?;
        let client = ReqwestClient::new_http(url);
        Ok(Self { client })
    }
}
