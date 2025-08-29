# Rusty Router

A Rust CLI application that converts Succinct SP1 proof requests to zkVerify-compatible format.

## Overview

Rusty Router takes a Succinct proof request ID, downloads its proof artifact, converts it into the zkVerify-compatible format, and saves the result to a JSON file.

## Features

- Fetch proof request metadata from Succinct explorer API
- Download proof artifacts from AWS S3
- Convert SP1 proofs to zkVerify format (placeholder implementation)
- Save converted proofs in JSON format with hex encoding
- Command-line interface with configurable output paths
 

## Prerequisites

- Rust (latest stable version or nightly)
- Cargo

## Installation

1. Clone the repository:
```bash
git clone <repository-url>
cd rusty_router
```

2. Build the project:
```bash
cargo +nightly build --release
```

## Usage

### Basic Usage

```bash
cargo run -- --request-id <PROOF_REQUEST_ID> --output proof.json
```

### Arguments

- `--request-id` (required): The Succinct proof request ID
- `--output` (optional): Path where to save the JSON file (default: `proof.json`)
- `--api-base` (optional): Override Succinct explorer base URL (default: `https://explorer.succinct.xyz`)
- `--verbose` (optional): Enable verbose structured logs

### Example

```bash
cargo run -- --request-id 0xf53938b95d7f0c7ec46ac63388d6ddc1b363af86bb0e3bb5f4589b7352c0f942 --output my_proof.json --verbose
```

## Output Format

The application generates a JSON file with the following structure:

```json
{
  "proof": "0x...",
  "pub_inputs": "0x..."
}
```

Where:
- `proof`: The converted proof in hex format with 0x prefix
- `pub_inputs`: The public inputs in hex format with 0x prefix

## Project Structure

```
src/
├── main.rs      # CLI entrypoint
├── client.rs    # HTTP client for fetching metadata and artifacts
├── converter.rs # Proof conversion logic (placeholder implementation)
└── lib.rs       # Module declarations
```

## Dependencies

- `reqwest`: HTTP client for API requests and downloads
- `serde`/`serde_json`: JSON serialization/deserialization
- `bincode`: Binary serialization
- `clap`: CLI argument parsing
- `tokio`: Async runtime
- `anyhow`: Error handling
- `hex`: Hex encoding/decoding
- `regex`: Pattern matching for HTML parsing

## Current Status

✅ **Working Features:**
- CLI interface with proper argument parsing
- HTTP client for API requests
- HTML parsing for metadata extraction
- Demo mode for testing
- JSON output generation
- Error handling and logging

🔄 **Next Steps (To Complete Full Functionality):**
1. **Implement Real Proof Conversion (behind `real-conversion` feature)**:
   - Deserialize SP1 artifact and call `ProverClient::from_env()`
   - Use zkVerify's `convert_proof_to_zkv(...)`
   - Extract public inputs and encode to hex

2. **API Integration (optional if not using demo)**:
   - Use `--api-base` to point to the correct explorer
   - Replace demo artifact with real S3 artifact

## Demo Mode

 

## Error Handling

The application provides clear error messages for common failure scenarios:
- Invalid request ID
- Network connectivity issues
- Invalid proof artifacts
- File system errors

## Development

To run in development mode:

```bash
cargo +nightly run -- --request-id <PROOF_REQUEST_ID>
```

To run tests:

```bash
cargo +nightly test
```

## License

[Add your license information here]
