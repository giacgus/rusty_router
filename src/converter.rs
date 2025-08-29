use anyhow::Result;
use serde::{Deserialize, Serialize};
use sp1_sdk::{ProverClient, SP1ProofWithPublicValues};
use sp1_zkv_sdk::*;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
pub struct ConvertedProof {
    pub proof: String,
    pub pub_inputs: String,
    pub vk: String,
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

    pub async fn convert_proof(&self, artifact_path: &Path, vk: &str) -> Result<ConvertedProof> {
        let proof = SP1ProofWithPublicValues::load(artifact_path)?;
        let client = ProverClient::from_env();

        // Convert proof and vk into a zkVerify-compatible proof.
        let SP1ZkvProofWithPublicValues {
            proof: shrunk_proof,
            public_values,
        } = client
            .convert_proof_to_zkv(proof, Default::default())
            .unwrap();

        // Serialize the proof
        let serialized_proof =
            bincode::serde::encode_to_vec(&shrunk_proof, bincode::config::legacy())
                .expect("failed to serialize proof");

        // Convert to required struct
        let output = ConvertedProof {
            proof: to_hex_with_prefix(&serialized_proof),
            pub_inputs: to_hex_with_prefix(&public_values),
            vk: vk.to_string(), // Keep VK as hex string, don't double-encode
        };
        Ok(output)
    }

    pub async fn save_proof(
        &self,
        converted_proof: &ConvertedProof,
        output_path: &Path,
    ) -> Result<()> {
        let json_content = serde_json::to_string_pretty(converted_proof)?;
        tokio::fs::write(output_path, json_content).await?;
        Ok(())
    }
}
