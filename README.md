# Rusty Router

A Rust CLI application that converts Succinct SP1 proof requests to zkVerify-compatible format and optionally submits them to the zkVerify network.

## Overview

Rusty Router takes a Succinct proof request ID, downloads its proof artifact, converts it into the zkVerify-compatible format, saves the result to a JSON file, and optionally submits the proof to the zkVerify network for verification.

## Features

- Fetch proof request metadata from Succinct explorer API
- Download proof artifacts from AWS S3
- Convert SP1 proofs to zkVerify format (placeholder implementation)
- Save converted proofs in JSON format with hex encoding
- Submit proofs to zkVerify network using Substrate/Polkadot blockchain
- Command-line interface with configurable output paths and network options

## Prerequisites

- Rust (latest stable version or nightly)
- Cargo
- Access to zkVerify network (for submission feature)

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

### Basic Usage (Convert Only)

```bash
cargo run -- --request-id <PROOF_REQUEST_ID> --output proof.json
```

### Convert and Send as System Remark

```bash
# Convert proof and send as system.remark transaction
cargo run -- \
  --request-id <PROOF_REQUEST_ID> \
  --output proof.json \
  --send-remark

# Send existing proof file as system.remark (no conversion needed)
cargo run -- \
  --output proof.json \
  --send-remark
```

### Convert and Submit to zkVerify Network

```bash
# Convert proof and submit to zkVerify network
cargo run -- \
  --request-id <PROOF_REQUEST_ID> \
  --output proof.json \
  --get-and-submit

# Submit existing proof file to zkVerify network (no conversion needed)
cargo run -- \
  --output proof.json \
  --get-and-submit
```

### Arguments

- `--request-id` (optional): The Succinct proof request ID (required for conversion, optional for sending existing proof)
- `--output` (optional): Path where to save the JSON file (default: `proof.json`)
- `--api-base` (optional): Override Succinct explorer base URL (default: `https://explorer.succinct.xyz`)
- `--verbose` (optional): Enable verbose structured logs
- `--ws-url` (optional): WebSocket URL of the Substrate node (default: `wss://zkverify-volta-rpc.zkverify.io`)
- `--send-remark` (optional): Send the proof as a system.remark transaction
- `--get-and-submit` (optional): Get proof from external source and submit to zkVerify network

### Examples

#### Convert Only
```bash
cargo run -- --request-id 0xf53938b95d7f0c7ec46ac63388d6ddc1b363af86bb0e3bb5f4589b7352c0f942 --output my_proof.json --verbose
```

#### Convert and Send as System Remark
```bash
# Using mnemonic from .env file
cargo run -- --request-id 0xf53938b95d7f0c7ec46ac63388d6ddc1b363af86bb0e3bb5f4589b7352c0f942 --send-remark

# Send existing proof file (no conversion needed)
cargo run -- --output proof.json --send-remark
```

#### Convert and Submit to zkVerify Network
```bash
# Using mnemonic from .env file
cargo run -- --request-id 0xf53938b95d7f0c7ec46ac63388d6ddc1b363af86bb0e3bb5f4589b7352c0f942 --get-and-submit

# Submit existing proof file (no conversion needed)
cargo run -- --output proof.json --get-and-submit
```

## Output Format

The application generates a JSON file with the following structure:

```json
{
  "proof": "0x...",
  "pub_inputs": "0x...",
  "vk": "0x..."
}
```

Where:
- `proof`: The converted proof in hex format with 0x prefix
- `pub_inputs`: The public inputs in hex format with 0x prefix
- `vk`: The verification key in hex format with 0x prefix

## Environment Configuration

### .env File Setup

Create a `.env` file in the project root with your mnemonic phrase:

```bash
# .env file
ZKV_MNEMONIC="your twelve word mnemonic phrase here"
```

The application will automatically load this mnemonic when using blockchain features.

## zkVerify Integration

The application includes integration with the zkVerify Volta network using the `subxt` crate for Substrate/Polkadot blockchain interaction.

### zkVerify Volta Network Configuration

- **Network**: zkVerify Volta Network
- **WebSocket URL**: `wss://zkverify-volta-rpc.zkverify.io`
- **Transaction Types**: 
  - `system.remark` (sends proof data as remark)
  - `Settlementsp1pallet.submit_proof` (submits proof to zkVerify network)
- **Explorer**: [zkVerify Volta Subscan](https://zkverify-volta.subscan.io/)

### Mnemonic Configuration

The application requires a mnemonic phrase for signing transactions. Set it in your `.env` file:

```bash
# Create a .env file
echo 'ZKV_MNEMONIC="your twelve word mnemonic phrase here"' > .env
```

### Current Implementation Status

- ✅ **Basic Integration**: Connection to zkVerify Volta network
- ✅ **System Remark Transactions**: Sends proof data as system.remark transactions
- ✅ **zkVerify Proof Submission**: Submits proofs to `Settlementsp1pallet.submit_proof`
- ✅ **CLI Interface**: Command-line options for submission
- ✅ **Environment Support**: Mnemonic loading from .env files
- ✅ **Transaction Signing**: Proper transaction signing with sr25519 keypairs
- ✅ **Proof File Handling**: Reads and sends existing proof files
- ✅ **JSON Parsing**: Extracts proof and public inputs from JSON format

### Next Steps for Full zkVerify Integration

1. **Generate zkVerify Pallet Types**:
   ```bash
   subxt codegen --url https://zkverify-volta-rpc.zkverify.io --output src/zkv_types.rs
   ```

2. **Implement Actual Transaction Submission**:
   - Replace placeholder with actual `Settlementsp1pallet.submit_proof` calls
   - Handle transaction signing and submission
   - Implement proper error handling

3. **Add Event Monitoring**:
   - Monitor `Settlementsp1pallet` events
   - Verify proof acceptance/rejection
   - Handle verification status updates

## Project Structure

```
src/
├── main.rs         # CLI entrypoint
├── client.rs       # HTTP client for fetching metadata and artifacts
├── converter.rs    # Proof conversion logic
├── substrate.rs    # Substrate blockchain integration
└── lib.rs          # Module declarations
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
- `tempfile`: Temporary file handling
- `subxt`: Substrate/Polkadot blockchain interaction
- `codec`: SCALE codec for blockchain data encoding

## Current Status

✅ **Working Features:**
- CLI interface with proper argument parsing
- HTTP client for API requests
- HTML parsing for metadata extraction
- Demo mode for testing
- JSON output generation
- Error handling and logging
- Temporary file handling for artifacts
- zkVerify network connection
- Proof encoding for blockchain submission

🔄 **Next Steps (To Complete Full Functionality):**
1. **Implement Real Proof Conversion (behind `real-conversion` feature)**:
   - Deserialize SP1 artifact and call `ProverClient::from_env()`
   - Use zkVerify's `convert_proof_to_zkv(...)`
   - Extract public inputs and encode to hex

2. **Complete zkVerify Integration**:
   - Generate zkVerify pallet types
   - Implement actual transaction submission
   - Add comprehensive event monitoring

3. **API Integration (optional if not using demo)**:
   - Use `--api-base` to point to the correct explorer
   - Replace demo artifact with real S3 artifact

## Demo Mode

The application includes a demo mode for testing the conversion pipeline without requiring real proof requests.

## Error Handling

The application provides clear error messages for common failure scenarios:
- Invalid request ID
- Network connectivity issues
- Invalid proof artifacts
- File system errors
- zkVerify network connection issues
- Invalid private keys

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
