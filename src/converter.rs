use anyhow::Result;
use serde::Serialize;
use std::path::Path;

#[derive(Debug, Serialize)]
pub struct ConvertedProof {
    pub proof: String,
    pub pub_inputs: String,
}

// Helper function to get hex strings with 0x prefix
fn to_hex_with_prefix(bytes: &[u8]) -> String {
    let hex_string: String = bytes.iter().map(|b| format!("{:02x}", b)).collect();
    format!("0x{}", hex_string)
}

pub struct ProofConverter;

impl ProofConverter {
    pub fn new() -> Self {
        Self
    }

    pub async fn convert_proof(&self, artifact_data: &[u8], _vk: &str) -> Result<ConvertedProof> {
        // Placeholder conversion - replace with real SP1/zkV conversion when needed
        let mock_proof = bincode::serde::encode_to_vec(&artifact_data, bincode::config::legacy())
            .unwrap_or_else(|_| artifact_data.to_vec());
        let mock_pub_inputs = vec![0u8; 32];
        Ok(ConvertedProof {
            proof: to_hex_with_prefix(&mock_proof),
            pub_inputs: to_hex_with_prefix(&mock_pub_inputs),
        })
    }

    pub async fn save_proof(&self, converted_proof: &ConvertedProof, output_path: &Path) -> Result<()> {
        let json_content = serde_json::to_string_pretty(converted_proof)?;
        tokio::fs::write(output_path, json_content).await?;
        Ok(())
    }
}
