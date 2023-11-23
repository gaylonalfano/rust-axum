use anyhow::Result;
use rand::RngCore;

// NOTE: Run this using: `cargo run --example gen_key`
// to get the encoded string output to save into .cargo/config.toml
// NOTE: Want to create a key that we can store in our config to use
// for testing passwords and tokens encryption.
fn main() -> Result<()> {
    let mut key = [0u8; 64]; // 512 bits = 64 bytes
    rand::thread_rng().fill_bytes(&mut key);
    println!("\nGenerated key for HMAC:\n{key:?}");

    // Normalize the bytes array into base64url string for safe character set
    let b64u = base64_url::encode(&key);
    println!("\nKey b64u encoded:\n{b64u}");

    Ok(())
}
