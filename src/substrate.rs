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
        println!("📄 Reading proof file...");
        
        // Read the proof file
        let proof_data = tokio::fs::read(proof_path).await?;
        
        // Parse the JSON to extract proof and public inputs
        let proof_json: serde_json::Value = serde_json::from_slice(&proof_data)?;
        
        let proof_hex = proof_json["proof"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing 'proof' field in JSON"))?;
        
        // Try both 'pubs' and 'pub_inputs' field names for compatibility
        let pub_inputs_hex = proof_json.get("pubs")
            .or_else(|| proof_json.get("pub_inputs"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'pubs' or 'pub_inputs' field in JSON"))?;
        
        // Remove 0x prefix if present
        let proof_hex = proof_hex.strip_prefix("0x").unwrap_or(proof_hex);
        let pub_inputs_hex = pub_inputs_hex.strip_prefix("0x").unwrap_or(pub_inputs_hex);
        
        // Convert hex to bytes
        let proof_bytes = hex::decode(proof_hex)?;
        let pub_inputs_bytes = hex::decode(pub_inputs_hex)?;
        
        println!("✅ Proof decomposed: {} bytes proof, {} bytes public inputs", proof_bytes.len(), pub_inputs_bytes.len());
        
        // Create the zkVerify proof submission call using the correct pallet name and call
        // Based on successful transaction: Settlementsp1pallet.Submit_proof with 4 parameters:
        // 1. vk_or_hash (VkOrHash)
        // 2. proof (Vec<U8>)
        // 3. pubs (Vec<U8>) 
        // 4. domain_id (Option<u32>)
        
        // Get VK from proof.json
        let vk_hex = proof_json.get("vk")
            .and_then(|v| v.as_str())
            .unwrap_or("50f8a2481aff84670a96db9126c7f4533f9f7e912129edfe3d35e4e81aa32472");
        
        // Handle double-encoded VK - decode it properly
        let vk_hex_clean = vk_hex.trim_start_matches("0x");
        let vk_bytes = if vk_hex_clean.len() > 64 {
            // If VK is longer than 64 chars, it might be double-encoded
            let decoded_vk = hex::decode(vk_hex_clean).unwrap();
            let decoded_vk_str = String::from_utf8(decoded_vk).unwrap();
            hex::decode(decoded_vk_str.trim_start_matches("0x")).unwrap()
        } else {
            hex::decode(vk_hex_clean).unwrap()
        };
        
        let vk_or_hash = subxt::dynamic::Value::named_variant("Vk", vec![
            ("Vk", subxt::dynamic::Value::unnamed_composite(vec![
                subxt::dynamic::Value::unnamed_composite(vk_bytes.into_iter().map(|b| subxt::dynamic::Value::u128(b as u128)).collect::<Vec<_>>())
            ]))
        ]);
        
        println!("🔗 Connecting to zkVerify network...");
        
        let call = subxt::dynamic::tx("SettlementSp1Pallet", "submit_proof", vec![
            vk_or_hash,
            subxt::dynamic::Value::unnamed_composite(proof_bytes.into_iter().map(|b| subxt::dynamic::Value::u128(b as u128)).collect::<Vec<_>>()),
            subxt::dynamic::Value::unnamed_composite(pub_inputs_bytes.into_iter().map(|b| subxt::dynamic::Value::u128(b as u128)).collect::<Vec<_>>()),
            subxt::dynamic::Value::named_variant::<&str, &str, Vec<(&str, subxt::dynamic::Value)>>("None", vec![]), // domain_id as None
        ]);
        
        println!("📤 Submitting transaction to zkVerify...");
        let result = self
            .client
            .tx()
            .sign_and_submit_default(&call, &self.signer)
            .await;
            
        match result {
            Ok(tx_hash) => {
                println!("✅ Transaction submitted successfully!");
                Ok(format!("{:?}", tx_hash))
            }
            Err(e) => {
                println!("❌ Transaction submission failed!");
                println!("Error: {:?}", e);
                
                // Check if it's a runtime error
                if e.to_string().contains("1010") {
                    println!("Error 1010 detected - this often indicates:");
                    println!("1. Insufficient funds for transaction fees");
                    println!("2. Invalid proof format or parameters");
                    println!("3. Chain-specific validation failure");
                }
                
                Err(e.into())
            }
        }
    }
}
