pub const MAX_CHUNK_SIZE: usize = 2000; // Maximum characters per chunk
pub const BATCH_SIZE: usize = 3; // Number of chunks to process at once

use anyhow::{Context, Result};
use regex::Regex;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize, Deserialize)]
struct Block {
    block_type: BlockType,
    content: String,
}

#[derive(Debug, Serialize, Deserialize)]
enum BlockType {
    Code,
    Text,
}

#[derive(Debug, Serialize, Deserialize)]
struct EmbeddingData {
    text: String,
    embedding: Vec<f32>,
}

pub struct DynamicRAG {
    client: Client,
}

impl DynamicRAG {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    fn is_code_block(&self, text: &str) -> bool {
        let code_indicators = vec![
            Regex::new(r"^import\s+").unwrap(),
            Regex::new(r"^const\s+").unwrap(),
            Regex::new(r"^let\s+").unwrap(),
            Regex::new(r"^function\s+").unwrap(),
            Regex::new(r"^class\s+").unwrap(),
            Regex::new(r"=>").unwrap(),
            Regex::new(r"{\s*$").unwrap(),
            Regex::new(r"^\s*}").unwrap(),
            Regex::new(r"^\s*return\s+").unwrap(),
            Regex::new(r"^\s*if\s*\(").unwrap(),
            Regex::new(r"^\s*for\s*\(").unwrap(),
            Regex::new(r"^\s*while\s*\(").unwrap(),
        ];

        code_indicators.iter().any(|pattern| pattern.is_match(text))
    }

    fn find_matching_brace(&self, text: &str, start_index: usize) -> usize {
        let chars: Vec<char> = text.chars().collect();
        let mut count = 1;

        for i in (start_index + 1)..chars.len() {
            if chars[i] == '{' {
                count += 1;
            }
            if chars[i] == '}' {
                count -= 1;
            }
            if count == 0 {
                return i;
            }
        }

        text.len()
    }

    pub fn create_chunks(&self, text: &str) -> Vec<String> {
        let mut blocks = Vec::new();
        let mut current_block = String::new();
        let lines: Vec<&str> = text.lines().collect();

        let mut i = 0;
        while i < lines.len() {
            let line = lines[i];

            if self.is_code_block(line) {
                if !current_block.is_empty() && !self.is_code_block(&current_block) {
                    blocks.push(Block {
                        block_type: BlockType::Text,
                        content: current_block.trim().to_string(),
                    });
                    current_block.clear();
                }

                current_block.push_str(if current_block.is_empty() { "" } else { "\n" });
                current_block.push_str(line);

                if line.contains('{') {
                    let mut brace_count = 1;
                    while i + 1 < lines.len() && brace_count > 0 {
                        i += 1;
                        current_block.push_str("\n");
                        current_block.push_str(lines[i]);
                        brace_count += lines[i].matches('{').count();
                        brace_count -= lines[i].matches('}').count();
                    }
                }
            } else {
                if !current_block.is_empty() && self.is_code_block(&current_block) {
                    blocks.push(Block {
                        block_type: BlockType::Code,
                        content: current_block.trim().to_string(),
                    });
                    current_block.clear();
                }
                current_block.push_str(if current_block.is_empty() { "" } else { "\n" });
                current_block.push_str(line);
            }
            i += 1;
        }

        if !current_block.is_empty() {
            blocks.push(Block {
                block_type: if self.is_code_block(&current_block) {
                    BlockType::Code
                } else {
                    BlockType::Text
                },
                content: current_block.trim().to_string(),
            });
        }

        let mut chunks = Vec::new();
        for block in blocks {
            match block.block_type {
                BlockType::Code => {
                    if block.content.len() <= MAX_CHUNK_SIZE {
                        chunks.push(block.content);
                    } else {
                        let lines: Vec<&str> = block.content.lines().collect();
                        let mut current_chunk = String::new();

                        for line in lines {
                            if current_chunk.len() + line.len() > MAX_CHUNK_SIZE {
                                if !current_chunk.is_empty() {
                                    chunks.push(current_chunk.trim().to_string());
                                }
                                current_chunk = line.to_string();
                            } else {
                                if !current_chunk.is_empty() {
                                    current_chunk.push_str("\n");
                                }
                                current_chunk.push_str(line);
                            }
                        }
                        if !current_chunk.is_empty() {
                            chunks.push(current_chunk.trim().to_string());
                        }
                    }
                }
                BlockType::Text => {
                    if block.content.len() <= MAX_CHUNK_SIZE {
                        chunks.push(block.content);
                    } else {
                        let re = Regex::new(r"[^.!?]+[.!?]+").unwrap();
                        let sentences: Vec<String> = re
                            .find_iter(&block.content)
                            .map(|m| m.as_str().to_string())
                            .collect();

                        let mut current_chunk = String::new();
                        for sentence in sentences {
                            if current_chunk.len() + sentence.len() > MAX_CHUNK_SIZE {
                                if !current_chunk.is_empty() {
                                    chunks.push(current_chunk.trim().to_string());
                                }
                                current_chunk = sentence;
                            } else {
                                current_chunk.push_str(" ");
                                current_chunk.push_str(&sentence);
                            }
                        }
                        if !current_chunk.is_empty() {
                            chunks.push(current_chunk.trim().to_string());
                        }
                    }
                }
            }
        }

        chunks
            .into_iter()
            .filter(|chunk| !chunk.is_empty())
            .collect()
    }

    pub async fn process_batch(
        &self,
        chunks: &[String],
        start_idx: usize,
    ) -> Result<Vec<EmbeddingData>> {
        let batch_chunks: Vec<String> =
            chunks[start_idx..std::cmp::min(start_idx + BATCH_SIZE, chunks.len())].to_vec();

        let response = self
            .client
            .post("http://localhost:8080/v1/embeddings")
            .json(&serde_json::json!({
                "model": "nomic-embed",
                "input": batch_chunks
            }))
            .send()
            .await
            .context("Failed to get embeddings")?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            anyhow::bail!("Embedding creation failed: {}", error_text);
        }

        let embeddings: serde_json::Value = response.json().await?;

        let results: Vec<EmbeddingData> = batch_chunks
            .iter()
            .enumerate()
            .filter_map(|(i, chunk)| {
                embeddings["data"].get(i).and_then(|data| {
                    Some(EmbeddingData {
                        text: chunk.clone(),
                        embedding: serde_json::from_value(data["embedding"].clone()).ok()?,
                    })
                })
            })
            .collect();

        Ok(results)
    }

    pub async fn create_snapshot(&self, embeddings_data: &[EmbeddingData]) -> Result<String> {
        let collection_name = format!(
            "temp_{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs()
        );

        self.client
            .put(&format!(
                "http://localhost:6333/collections/{}",
                collection_name
            ))
            .json(&serde_json::json!({
                "vectors": {
                    "size": 768,
                    "distance": "Cosine"
                }
            }))
            .send()
            .await?;

        let points: Vec<serde_json::Value> = embeddings_data
            .iter()
            .enumerate()
            .map(|(i, item)| {
                serde_json::json!({
                    "id": i,
                    "vector": item.embedding,
                    "payload": { "text": item.text }
                })
            })
            .collect();

        self.client
            .put(&format!(
                "http://localhost:6333/collections/{}/points",
                collection_name
            ))
            .json(&serde_json::json!({ "points": points }))
            .send()
            .await?;

        Ok(collection_name)
    }

    pub async fn query_llm(&self, user_query: &str, context: &str) -> Result<String> {
        let response = self.client.post("http://localhost:8080/v1/chat/completions")
            .json(&serde_json::json!({
                "model": "llama",
                "messages": [
                    {
                        "role": "system",
                        "content": "You are a helpful assistant. Use the provided context to answer questions."
                    },
                    {
                        "role": "user",
                        "content": format!("Context: {}\n\nQuestion: {}", context, user_query)
                    }
                ]
            }))
            .send()
            .await?;

        let completion: serde_json::Value = response.json().await?;
        Ok(completion["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string())
    }
}
