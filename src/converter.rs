use anyhow::Result;
use serde::{Deserialize, Serialize};
use sp1_sdk::{ProverClient, SP1ProofWithPublicValues, HashableKey};
use sp1_zkv_sdk::*;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
pub struct ConvertedProof {
    pub proof: String,
    pub pubs: String,
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

    pub async fn convert_proof(&self, artifact_path: &Path, vk_from_page: &str) -> Result<ConvertedProof> {
        let proof = SP1ProofWithPublicValues::load(artifact_path)?;
        let client = ProverClient::from_env();

        let vk = if !vk_from_page.is_empty() {
            vk_from_page.to_string()
        } else {
            // Fallback to extracting from proof structure
            match &proof.proof {
                sp1_sdk::SP1Proof::Compressed(sp1_reduce_proof) => {
                    let vk_bytes = sp1_reduce_proof.vk.hash_bytes();
                    to_hex_with_prefix(&vk_bytes)
                }
                _ => {
                    "0x0000000000000000000000000000000000000000000000000000000000000000".to_string()
                }
            }
        };

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
            pubs: to_hex_with_prefix(&public_values),
            vk: vk, // Use VK extracted from proof structure
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

    pub async fn save_detailed_proof_info(&self, artifact_path: &Path, output_path: &str) -> Result<()> {
        let proof = SP1ProofWithPublicValues::load(artifact_path)?;
        
        // Create a detailed structure with all the information
        #[derive(Debug, Serialize)]
        struct DetailedProofInfo {
            sp1_version: String,
            proof_type: String,
            vk_extracted: String,
            vk_length: usize,
            public_values_debug: String,
            proof_structure: String,
            tee_proof: Option<String>,
        }

        // Extract VK from the proof structure
        let vk = match &proof.proof {
            sp1_sdk::SP1Proof::Compressed(sp1_reduce_proof) => {
                let vk_bytes = sp1_reduce_proof.vk.hash_bytes();
                to_hex_with_prefix(&vk_bytes)
            }
            _ => "0x0000000000000000000000000000000000000000000000000000000000000000".to_string()
        };

        let detailed_info = DetailedProofInfo {
            sp1_version: proof.sp1_version.clone(),
            proof_type: format!("{:?}", proof.proof),
            vk_extracted: vk.clone(),
            vk_length: vk.len() - 2, // Remove "0x" prefix
            public_values_debug: format!("{:?}", proof.public_values),
            proof_structure: format!("{:?}", proof),
            tee_proof: proof.tee_proof.as_ref().map(|_| "Present".to_string()),
        };

        let json = serde_json::to_string_pretty(&detailed_info)?;
        tokio::fs::write(output_path, json).await?;
        Ok(())
    }
}
