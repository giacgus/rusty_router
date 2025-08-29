use anyhow::Result;
use reqwest::Client;
use serde::Deserialize;
use tracing::{debug};

#[derive(Debug, Deserialize)]
pub struct ProofRequestMetadata {
    pub artifact_url: String,
    pub program: String, // This contains the VK
}

impl ProofRequestMetadata {
    pub fn vk(&self) -> &str {
        &self.program
    }
}

pub struct ProofClient {
    client: Client,
    api_base: String,
    verbose: bool,
}

impl ProofClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            api_base: "https://explorer.succinct.xyz".to_string(),
            verbose: false,
        }
    }

    pub fn new_with_options(api_base: &str, verbose: bool) -> Self {
        Self {
            client: Client::new(),
            api_base: api_base.to_string(),
            verbose,
        }
    }

    pub async fn fetch_request_metadata(&self, request_id: &str) -> Result<ProofRequestMetadata> {
        // Use headless browser to render the page and extract data
        let url = format!("{}/request/{}", self.api_base, request_id);
        println!("=== RENDERING PAGE WITH HEADLESS BROWSER ===");
        println!("URL: {}", url);
        
        // Use std::process::Command to run chromium-browser
        let output = std::process::Command::new("chromium-browser")
            .args(&[
                "--headless",
                "--disable-gpu", 
                "--no-sandbox",
                "--dump-dom",
                &url
            ])
            .output()?;
            
        if !output.status.success() {
            anyhow::bail!("Failed to render page: {}", String::from_utf8_lossy(&output.stderr));
        }
        
        let html_content = String::from_utf8_lossy(&output.stdout);
        println!("Rendered HTML length: {}", html_content.len());
        
        // Print a small snippet if verbose mode is enabled
        if self.verbose {
            let preview = html_content.chars().take(500).collect::<String>();
            println!("HTML preview (first 500 chars): {}", preview);
        }
        
        // Extract artifact URL using regex
        let artifact_pattern = r#"(https://spn-artifacts-mainnet\.s3[^"<>\s]*)"#;
        let vk_pattern = r#"(0x[0-9a-fA-F]{64,})"#;
        
        let artifact_url = if let Ok(re) = regex::Regex::new(artifact_pattern) {
            re.captures(&html_content)
                .and_then(|caps| caps.get(1))
                .map(|m| {
                    let url = m.as_str().to_string();
                    // Decode HTML entities
                    url.replace("&amp;", "&")
                       .replace("&lt;", "<")
                       .replace("&gt;", ">")
                       .replace("&quot;", "\"")
                       .replace("&#39;", "'")
                })
        } else {
            None
        };
        
        let program_vk = if let Ok(re) = regex::Regex::new(vk_pattern) {
            re.captures(&html_content)
                .and_then(|caps| caps.get(1))
                .map(|m| m.as_str().to_string())
        } else {
            None
        };
        
        match (&artifact_url, &program_vk) {
            (Some(url), Some(vk)) => {
                println!("✅ Found artifact URL: {}", url);
                println!("✅ Found verification key: {}", vk);
                Ok(ProofRequestMetadata { artifact_url: url.clone(), program: vk.clone() })
            }
            _ => {
                println!("❌ Missing data - artifact_url: {:?}, program: {:?}", artifact_url, program_vk);
                anyhow::bail!("Failed to extract metadata from rendered page")
            }
        }
    }

    pub async fn download_artifact(&self, artifact_url: &str) -> Result<Vec<u8>> {
        let response = self.client.get(artifact_url).send().await?;
        
        if !response.status().is_success() {
            anyhow::bail!("Failed to download artifact: {}", response.status());
        }
        
        let artifact_data = response.bytes().await?;
        Ok(artifact_data.to_vec())
    }
}
