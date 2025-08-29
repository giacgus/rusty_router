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
    
    fn extract_from_next_data(&self, html_content: &str) -> Option<ProofRequestMetadata> {
        // Look for the __NEXT_DATA__ script tag which contains the page data
        if let Some(data_start) = html_content.find("__NEXT_DATA__") {
            println!("Found __NEXT_DATA__ at position {}", data_start);
            if let Some(script_start) = html_content[data_start..].find(">") {
                let script_content = &html_content[data_start + script_start + 1..];
                if let Some(script_end) = script_content.find("</script>") {
                    let json_str = &script_content[..script_end];
                    if let Some(json_start) = json_str.find('{') {
                        let json_content = &json_str[json_start..];
                        println!("Found JSON content in __NEXT_DATA__: {}", &json_content[..json_content.len().min(500)]);
                        if let Ok(page_data) = serde_json::from_str::<serde_json::Value>(json_content) {
                            // Extract the request data from the page props
                            if let Some(props) = page_data.get("props") {
                                if let Some(page_props) = props.get("pageProps") {
                                    if let Some(request_data) = page_props.get("request") {
                                        return self.parse_request_data(request_data).ok();
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        None
    }
    
    fn extract_from_script_tags(&self, html_content: &str) -> Option<ProofRequestMetadata> {
        // Look for any script tag that might contain the data
        let script_patterns = [
            r#"window\.__INITIAL_STATE__\s*=\s*({.*?});"#,
            r#"window\.__PRELOADED_STATE__\s*=\s*({.*?});"#,
            r#"data\s*=\s*({.*?});"#,
        ];
        
        for pattern in &script_patterns {
            if let Some(captures) = regex::Regex::new(pattern).ok().and_then(|re| re.captures(html_content)) {
                if let Some(json_str) = captures.get(1) {
                    if let Ok(data) = serde_json::from_str::<serde_json::Value>(json_str.as_str()) {
                        if let Some(metadata) = self.parse_request_data(&data).ok() {
                            return Some(metadata);
                        }
                    }
                }
            }
        }
        None
    }
    
    fn extract_via_regex_scan(&self, html_content: &str) -> Option<ProofRequestMetadata> {
        // Candidates for artifact URL
        let artifact_patterns = [
            r#"artifactUrl"\s*:\s*"([^"]+)"#,
            r#"artifact_url"\s*:\s*"([^"]+)"#,
            r#"artifact"\s*:\s*"([^"]+)"#,
            r#"(https?://[^"]*amazonaws\.com/[^"]+)"#,
            r#"(https?://spn-artifacts-mainnet\.s3\.us-east-2\.amazonaws\.com/[^"]+)"#,
            r#"(https?://[^"]*s3\.us-east-2\.amazonaws\.com/[^"]+)"#,
        ];
        // Candidates for verification key string (0x...)
        let vk_patterns = [
            r#"\bprogram\b"\s*:\s*"(0x[0-9a-fA-F]+)"#,
            r#"verificationKey"\s*:\s*"(0x[0-9a-fA-F]+)"#,
            r#"\bvk\b"\s*:\s*"(0x[0-9a-fA-F]+)"#,
        ];

        let mut artifact_url: Option<String> = None;
        for pat in &artifact_patterns {
            if let Ok(re) = regex::Regex::new(pat) {
                if let Some(caps) = re.captures(html_content) {
                    if let Some(m) = caps.get(1) {
                        let found_url = m.as_str().to_string();
                        println!("Found artifact URL with pattern: {}", found_url);
                        artifact_url = Some(found_url);
                        break;
                    }
                }
            }
        }

        let mut program_vk: Option<String> = None;
        for pat in &vk_patterns {
            if let Ok(re) = regex::Regex::new(pat) {
                if let Some(caps) = re.captures(html_content) {
                    if let Some(m) = caps.get(1) {
                        let found_vk = m.as_str().to_string();
                        println!("Found verification key with pattern: {}", found_vk);
                        program_vk = Some(found_vk);
                        break;
                    }
                }
            }
        }

        match (artifact_url, program_vk) {
            (Some(a), Some(vk)) => Some(ProofRequestMetadata { artifact_url: a, program: vk }),
            _ => None,
        }
    }
    
    fn parse_request_data(&self, request_data: &serde_json::Value) -> Result<ProofRequestMetadata> {
        println!("Parsing request data: {}", serde_json::to_string_pretty(request_data)?);
        
        // Try different possible field names
        let artifact_url = request_data
            .get("artifactUrl")
            .or_else(|| request_data.get("artifact_url"))
            .or_else(|| request_data.get("artifact"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing artifactUrl"))?
            .to_string();
            
        let program = request_data
            .get("program")
            .or_else(|| request_data.get("verificationKey"))
            .or_else(|| request_data.get("vk"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing program"))?
            .to_string();
            
        Ok(ProofRequestMetadata {
            artifact_url,
            program,
        })
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
