use anyhow::Result;
use rand::RngCore;

// NOTE: Want to create a key that we can store in our config.
// This will help with our tests.
fn main() -> Result<()> {
    let mut fx_key = [0u8; 64]; // 512 bits = 64 bytes
    rand::thread_rng().fill_bytes(&mut fx_key);

    Ok(())
}
