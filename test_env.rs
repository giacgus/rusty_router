use dotenv::dotenv;
use std::env;

fn main() {
    // Load environment variables from .env file if it exists
    dotenv().ok();
    
    println!("Testing environment variable loading...");
    
    // Check if ZKV_MNEMONIC is loaded
    match env::var("ZKV_MNEMONIC") {
        Ok(mnemonic) => println!("✅ ZKV_MNEMONIC found: {}", mnemonic),
        Err(_) => println!("❌ ZKV_MNEMONIC not found"),
    }
    
    // Check if ZKV_PRIVATE_KEY is loaded
    match env::var("ZKV_PRIVATE_KEY") {
        Ok(private_key) => println!("✅ ZKV_PRIVATE_KEY found: {}", private_key),
        Err(_) => println!("❌ ZKV_PRIVATE_KEY not found"),
    }
    
    // List all environment variables that start with ZKV_
    println!("\nAll ZKV_ environment variables:");
    for (key, value) in env::vars() {
        if key.starts_with("ZKV_") {
            println!("  {}: {}", key, value);
        }
    }
}
