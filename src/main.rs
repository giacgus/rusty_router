use clap::Parser;
use rusty_router::client::ProofClient;
use rusty_router::converter::ProofConverter;
use rusty_router::substrate::SubstrateClient;

use std::path::PathBuf;
use tempfile::NamedTempFile;
use tracing::{debug, info};
use dotenv::dotenv;

#[derive(Parser)]
#[command(name = "rusty_router")]
#[command(about = "Convert Succinct proof requests to zkVerify format")]
struct Args {
    /// The Succinct proof request ID
    #[arg(long)]
    request_id: Option<String>,

    /// Path where to save the JSON file
    #[arg(long, default_value = "proof.json")]
    output: PathBuf,

    /// Override explorer API base URL
    #[arg(long, default_value = "https://explorer.succinct.xyz")]
    api_base: String,

    /// Enable verbose logging
    #[arg(long, default_value_t = false)]
    verbose: bool,

    /// WebSocket URL of the Substrate node
    #[arg(long, default_value = "wss://zkverify-volta-rpc.zkverify.io")]
    ws_url: String,



    /// Send the proof as a system.remark transaction
    #[arg(long, default_value_t = false)]
    send_remark: bool,

    /// Submit existing proof to zkVerify network for validation
    #[arg(long, default_value_t = false)]
    submit_to_zkverify: bool,

    /// Extract and save detailed proof information without submitting
    #[arg(long, default_value_t = false)]
    get_proof: bool,

    /// List available pallets (for debugging)
    #[arg(long, default_value_t = false)]
    list_pallets: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load environment variables from .env file
    dotenv().ok();

    let args = Args::parse();

    if args.verbose {
        tracing_subscriber::fmt()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .with_target(false)
            .compact()
            .init();
        debug!("Verbose logging enabled");
    }



    // Handle proof conversion (original functionality) - only if request_id is provided
    if let Some(request_id) = args.request_id {
        info!("Fetching proof request metadata...");
        let client = ProofClient::new_with_options(&args.api_base, args.verbose);
        let metadata = client.fetch_request_metadata(&request_id).await?;

        info!("Downloading proof artifact...");
        let artifact_data = client.download_artifact(&metadata.artifact_url).await?;

        // Create a temporary file to store the artifact
        let temp_file = NamedTempFile::new()?;
        let temp_file_path = temp_file.path().to_path_buf();

        info!("Saving artifact to temporary file...");
        tokio::fs::write(&temp_file_path, artifact_data).await?;

        info!("Converting proof to zkVerify format...");
        let converter = ProofConverter::new();
        let converted_proof = converter
            .convert_proof(&temp_file_path, &metadata.vk)
            .await?;

        info!("Saving converted proof...");
        converter.save_proof(&converted_proof, &args.output).await?;

        info!("Proof converted successfully: {}", args.output.display());

        // If --get-proof is specified, also save detailed proof information
        if args.get_proof {
            info!("Extracting detailed proof information...");
            converter.save_detailed_proof_info(&temp_file_path, "proof_details.json").await?;
            info!("Detailed proof information saved to proof_details.json");
        }

        // Explicitly clean up the temporary file
        drop(temp_file);
    } else {
        info!("No request_id provided, skipping proof conversion");
    }

    // Handle blockchain transactions (system.remark, zkVerify submission, or pallet listing)
    if args.send_remark || args.submit_to_zkverify || args.list_pallets {
        // Get mnemonic from environment
        let mnemonic = std::env::var("ZKV_MNEMONIC")
            .expect("ZKV_MNEMONIC environment variable not found. Please set it in your .env file");

        info!("Connecting to Substrate node...");
        let substrate_client = SubstrateClient::new(&args.ws_url, &mnemonic).await?;

        if args.send_remark {
            info!("Sending proof as system.remark transaction...");
            let tx_hash = substrate_client.send_proof_as_remark(&args.output).await?;
            info!("Proof sent successfully! Transaction hash: {}", tx_hash);
        }

        if args.submit_to_zkverify {
            info!("Submitting proof to zkVerify network...");
            let tx_hash = substrate_client.submit_proof_to_zkverify(&args.output).await?;
            info!("Proof submitted successfully to zkVerify! Transaction hash: {}", tx_hash);
        }

        if args.list_pallets {
            substrate_client.list_available_pallets().await?;
        }
    }

    Ok(())
}
