use clap::Parser;
use rusty_router::client::ProofClient;
use rusty_router::converter::ProofConverter;
use std::path::PathBuf;
use tempfile::NamedTempFile;
use tracing::{debug, info};

#[derive(Parser)]
#[command(name = "rusty_router")]
#[command(about = "Convert Succinct proof requests to zkVerify format")]
struct Args {
    /// The Succinct proof request ID
    #[arg(long)]
    request_id: String,

    /// Path where to save the JSON file
    #[arg(long, default_value = "proof.json")]
    output: PathBuf,

    /// Override explorer API base URL
    #[arg(long, default_value = "https://explorer.succinct.xyz")]
    api_base: String,

    /// Enable verbose logging
    #[arg(long, default_value_t = false)]
    verbose: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    if args.verbose {
        tracing_subscriber::fmt()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .with_target(false)
            .compact()
            .init();
        debug!("Verbose logging enabled");
    }

    info!("Fetching proof request metadata...");
    let client = ProofClient::new_with_options(&args.api_base, args.verbose);
    let metadata = client.fetch_request_metadata(&args.request_id).await?;

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
        .convert_proof(&temp_file_path, &metadata.program)
        .await?;

    // Explicitly clean up the temporary file
    drop(temp_file);

    info!("Saving converted proof...");
    converter.save_proof(&converted_proof, &args.output).await?;

    info!("Proof converted successfully: {}", args.output.display());
    Ok(())
}
