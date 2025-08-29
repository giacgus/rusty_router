#!/bin/bash

# Array of request IDs
request_ids=(
    "0x921d36fb2ad5e23d521c624624e9a07e2297f70704c18661e55ccf7ca4464c78"
    "0xd13f56fdbf867e01dc7f0e2198aba92e0404e6939037e6414c2b99d5559f93e4"
    "0x565c79b8eac87c500770d078d86e37996228baa8f39a16d856380a57f2d896b9"
    "0x30127246946b4d21c5e38113e83df7bcfab073218cf6a24510f48b918530d9b4"
    "0x9e0c0d56d41bb1c548a34da64cffaf9f154d5baf9420e765fabd2d9d821094b3"
    "0x9f8465e927fcc3db9b6053ec9ee3936afe1fd9c9f9eb977fd7c82de1a253bbc3"
    "0xa619e94a03d6c8d833508015fa436885834a04a4520df1febfdcc3a35a2d6058"
)

# Create output directory
mkdir -p proofs

echo "Processing ${#request_ids[@]} proof requests..."

for i in "${!request_ids[@]}"; do
    request_id="${request_ids[$i]}"
    output_file="proofs/proof_${i}.json"
    
    echo "=========================================="
    echo "Processing request $((i+1))/${#request_ids[@]}: $request_id"
    echo "Output: $output_file"
    echo "=========================================="
    
    # Convert proof (without submission to avoid hanging)
    timeout 300 ./target/release/rusty_router --request-id "$request_id" --output "$output_file" --verbose
    
    if [ $? -eq 0 ]; then
        echo "✅ Conversion successful for $request_id"
        
        # Submit to zkVerify
        echo "Submitting to zkVerify..."
        ./target/release/rusty_router --output "$output_file" --submit-to-zkverify --verbose
        
        if [ $? -eq 0 ]; then
            echo "✅ Submission successful for $request_id"
        else
            echo "❌ Submission failed for $request_id"
        fi
    else
        echo "❌ Conversion failed for $request_id"
    fi
    
    echo ""
    # Small delay between requests
    sleep 2
done

echo "All requests processed!"
