use anyhow::Result;
use subxt::{
    config::PolkadotConfig,
    OnlineClient,
};
use subxt_signer::sr25519::Keypair;
use bip39::Mnemonic;
use std::path::Path;
use tracing::{debug, info, error};

pub struct SubstrateClient {
    client: OnlineClient<PolkadotConfig>,
    signer: Keypair,
}

impl SubstrateClient {
    pub async fn new(ws_url: &str, mnemonic: &str) -> Result<Self> {
        info!("Connecting to Substrate node at: {}", ws_url);
        
        // Create the client
        let client = OnlineClient::<PolkadotConfig>::from_url(ws_url).await?;
        
        // Create the signer from mnemonic
        let mnemonic = Mnemonic::parse_normalized(mnemonic)?;
        let keypair = Keypair::from_phrase(&mnemonic, None)?;
        
        info!("Connected to Substrate node successfully");
        
        Ok(Self { client, signer: keypair })
    }
    
    pub async fn list_available_pallets(&self) -> Result<()> {
        info!("Fetching available pallets from the network...");
        
        // For now, just log that we're connected
        info!("Successfully connected to Substrate node");
        info!("Note: To see available pallets, you may need to check the network documentation");
        
        Ok(())
    }
    
    pub async fn send_system_remark(&self, remark: &[u8]) -> Result<String> {
        info!("Preparing system.remark transaction...");
        
        // Create the system.remark call
        let call = subxt::dynamic::tx("System", "remark", vec![remark.to_vec()]);
        
        // Submit the transaction
        let tx_hash = self
            .client
            .tx()
            .sign_and_submit_default(&call, &self.signer)
            .await?;
            
        info!("Transaction submitted successfully with hash: {:?}", tx_hash);
        
        Ok(format!("{:?}", tx_hash))
    }
    
    pub async fn send_proof_as_remark(&self, proof_path: &Path) -> Result<String> {
        info!("Reading proof file from: {}", proof_path.display());
        
        // Read the proof file
        let proof_data = tokio::fs::read(proof_path).await?;
        debug!("Proof file size: {} bytes", proof_data.len());
        
        // Send as system.remark
        self.send_system_remark(&proof_data).await
    }
    
    pub async fn submit_proof_to_zkverify(&self, proof_path: &Path) -> Result<String> {
        info!("=== Starting zkVerify proof submission ===");
        info!("Reading proof file from: {}", proof_path.display());
        
        // Read the proof file
        let proof_data = tokio::fs::read(proof_path).await?;
        info!("Proof file size: {} bytes", proof_data.len());
        
        // Parse the JSON to extract proof and public inputs
        let proof_json: serde_json::Value = serde_json::from_slice(&proof_data)?;
        info!("Successfully parsed proof JSON");
        
        // Log available fields in the JSON
        if let Some(obj) = proof_json.as_object() {
            info!("Available fields in proof JSON: {:?}", obj.keys().collect::<Vec<_>>());
        }
        
        let proof_hex = proof_json["proof"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing 'proof' field in JSON"))?;
        info!("Found proof field, length: {} chars", proof_hex.len());
        
        // Try both 'pubs' and 'pub_inputs' field names for compatibility
        let pub_inputs_hex = proof_json.get("pubs")
            .or_else(|| proof_json.get("pub_inputs"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'pubs' or 'pub_inputs' field in JSON"))?;
        
        let field_name = if proof_json.get("pubs").is_some() { "pubs" } else { "pub_inputs" };
        info!("Using field '{}' for public inputs, length: {} chars", field_name, pub_inputs_hex.len());
        
        // Remove 0x prefix if present
        let proof_hex = proof_hex.strip_prefix("0x").unwrap_or(proof_hex);
        let pub_inputs_hex = pub_inputs_hex.strip_prefix("0x").unwrap_or(pub_inputs_hex);
        
        // Convert hex to bytes
        let proof_bytes = hex::decode(proof_hex)?;
        let pub_inputs_bytes = hex::decode(pub_inputs_hex)?;
        
        info!("=== Proof data summary ===");
        info!("Proof hex length: {} chars", proof_hex.len());
        info!("Proof bytes: {} bytes", proof_bytes.len());
        info!("Public inputs hex length: {} chars", pub_inputs_hex.len());
        info!("Public inputs bytes: {} bytes", pub_inputs_bytes.len());
        
        // Create the zkVerify proof submission call using the correct pallet name and call
        // Based on successful transaction: Settlementsp1pallet.Submit_proof with 4 parameters:
        // 1. vk_or_hash (VkOrHash)
        // 2. proof (Vec<U8>)
        // 3. pubs (Vec<U8>) 
        // 4. domain_id (Option<u32>)
        
        // Create the VkOrHash value from the proof file
        // Try to get Vk from proof.json, fallback to default if not found
        let vk_hex = proof_json.get("vk")
            .and_then(|v| v.as_str())
            .unwrap_or("50f8a2481aff84670a96db9126c7f4533f9f7e912129edfe3d35e4e81aa32472");
        
        info!("=== VK processing ===");
        info!("VK hex from JSON: {}", vk_hex);
        info!("VK hex length: {} chars", vk_hex.len());
        
        // Handle double-encoded VK - decode it properly
        let vk_hex_clean = vk_hex.trim_start_matches("0x");
        let vk_bytes = if vk_hex_clean.len() > 64 {
            info!("VK appears to be double-encoded ({} chars), decoding...", vk_hex_clean.len());
            // If VK is longer than 64 chars, it might be double-encoded
            // Decode it once to get the actual VK
            let decoded_vk = hex::decode(vk_hex_clean).unwrap();
            let decoded_vk_str = String::from_utf8(decoded_vk).unwrap();
            let final_vk = hex::decode(decoded_vk_str.trim_start_matches("0x")).unwrap();
            info!("Double-decoded VK length: {} bytes", final_vk.len());
            final_vk
        } else {
            info!("VK appears to be single-encoded ({} chars)", vk_hex_clean.len());
            let decoded = hex::decode(vk_hex_clean).unwrap();
            info!("Single-decoded VK length: {} bytes", decoded.len());
            decoded
        };
        
        info!("Final VK bytes: {} bytes", vk_bytes.len());
        if vk_bytes.len() <= 32 {
            info!("VK hex: {}", hex::encode(&vk_bytes));
        }
        
        let vk_or_hash = subxt::dynamic::Value::named_variant("Vk", vec![
            ("Vk", subxt::dynamic::Value::unnamed_composite(vec![
                subxt::dynamic::Value::unnamed_composite(vk_bytes.into_iter().map(|b| subxt::dynamic::Value::u128(b as u128)).collect::<Vec<_>>())
            ]))
        ]);
        
        info!("=== Creating transaction call ===");
        info!("Pallet: SettlementSp1Pallet");
        info!("Call: submit_proof");
        info!("Parameters: vk_or_hash, proof ({} bytes), pubs ({} bytes), domain_id (None)", 
              proof_bytes.len(), pub_inputs_bytes.len());
        
        let call = subxt::dynamic::tx("SettlementSp1Pallet", "submit_proof", vec![
            vk_or_hash,
            subxt::dynamic::Value::unnamed_composite(proof_bytes.into_iter().map(|b| subxt::dynamic::Value::u128(b as u128)).collect::<Vec<_>>()),
            subxt::dynamic::Value::unnamed_composite(pub_inputs_bytes.into_iter().map(|b| subxt::dynamic::Value::u128(b as u128)).collect::<Vec<_>>()),
            subxt::dynamic::Value::named_variant::<&str, &str, Vec<(&str, subxt::dynamic::Value)>>("None", vec![]), // domain_id as None
        ]);
        
        info!("=== Submitting transaction to chain ===");
        info!("Using signer account: {}", hex::encode(self.signer.public_key().0));
        
        // Submit the transaction
        info!("Signing and submitting transaction...");
        let result = self
            .client
            .tx()
            .sign_and_submit_default(&call, &self.signer)
            .await;
            
        match result {
            Ok(tx_hash) => {
                info!("=== Transaction submitted successfully! ===");
                info!("Transaction hash: {:?}", tx_hash);
                info!("You can view this transaction on the zkVerify explorer");
                info!("Note: The transaction may take a moment to be processed by the chain");
                Ok(format!("{:?}", tx_hash))
            }
            Err(e) => {
                error!("=== Transaction submission failed! ===");
                error!("Error: {:?}", e);
                
                // Check if it's a runtime error
                if e.to_string().contains("1010") {
                    error!("Error 1010 detected - this often indicates:");
                    error!("1. Insufficient funds for transaction fees");
                    error!("2. Invalid proof format or parameters");
                    error!("3. Chain-specific validation failure");
                }
                
                Err(e.into())
            }
        }
    }
}
