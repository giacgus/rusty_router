use anyhow::Result;
use reqwest::Client;
use serde::Deserialize;


#[derive(Debug, Deserialize)]
pub struct ProofRequestMetadata {
    pub artifact_url: String,
    pub vk: String,
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
        println!("DEBUG: Starting VK extraction...");
        
        // Print a small snippet if verbose mode is enabled
        if self.verbose {
            let preview = html_content.chars().take(500).collect::<String>();
            println!("HTML preview (first 500 chars): {}", preview);
        }
        

        
        // Extract artifact URL using regex
        let artifact_pattern = r#"(https://spn-artifacts-mainnet\.s3[^"<>\s]*)"#;
        
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

        // Extract VK from "Program Blobstream" section
        let possible_keywords = ["Program Blobstream", "Blobstream", "Program"];
        let mut found_keyword = None;
        let mut found_position = 0;
        
        for keyword in &possible_keywords {
            if let Some(pos) = html_content.find(keyword) {
                found_keyword = Some(keyword);
                found_position = pos;
                break;
            }
        }
        
        let vk = if let Some(keyword) = found_keyword {
            // Look for VK pattern around this section (look back and forward)
            let search_start = found_position.saturating_sub(1000);
            let search_end = (found_position + 2000).min(html_content.len());
            let search_section = &html_content[search_start..search_end];
            
            // Look for VK pattern (32 bytes = 64 hex chars)
            let vk_pattern = r#"0x[0-9a-fA-F]{64}"#;
            if let Ok(re) = regex::Regex::new(vk_pattern) {
                if let Some(captures) = re.captures(search_section) {
                    let found_vk = captures[0].to_string();
                    println!("✅ VK found: {}", found_vk);
                    Some(found_vk)
                } else {
                    // Let's also search the entire HTML for any VK pattern
                    if let Some(captures) = re.captures(&html_content) {
                        let found_vk = captures[0].to_string();
                        println!("✅ VK found: {}", found_vk);
                        Some(found_vk)
                    } else {
                        println!("❌ No VK pattern found");
                        None
                    }
                }
            } else {
                println!("❌ Failed to compile VK regex");
                None
            }
        } else {
            println!("❌ No keywords found in HTML");
            None
        };
        

        
        match artifact_url {
            Some(url) => {
                println!("✅ Found artifact URL: {}", url);
                Ok(ProofRequestMetadata { 
                    artifact_url: url.clone(),
                    vk: vk.unwrap_or_default(),
                })
            }
            None => {
                println!("❌ Missing artifact URL");
                anyhow::bail!("Failed to extract artifact URL from rendered page")
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
