use base32::Alphabet;
use hmac::{Hmac, Mac};
use rand::Rng;
use sha2::Sha256;
use std::time::{SystemTime, UNIX_EPOCH};

type HmacSha256 = Hmac<Sha256>;

/// The secret key used for HMAC signing (must match validation.rs)
const HMAC_SECRET: &str = "pdf_finder_pro_secret_key_v1_change_before_release";

fn generate_license_key() -> String {
    let mut rng = rand::thread_rng();
    
    // Generate timestamp component (8 hex chars)
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    // Generate random component (8 hex chars)
    let random: u32 = rng.gen();
    
    // Create data string (16 chars: 8 from time, 8 from random)
    let data = format!("{:08X}{:08X}", timestamp as u32, random);
    
    // Compute HMAC signature
    let mut mac = HmacSha256::new_from_slice(HMAC_SECRET.as_bytes())
        .expect("Invalid HMAC key length");
    mac.update(data.as_bytes());
    let result = mac.finalize();
    let sig_bytes = result.into_bytes();
    
    // Take first 2 bytes, encode to base32 (gives us ~4 chars)
    let sig_b32 = base32::encode(Alphabet::Crockford, &sig_bytes[..2]);
    let signature = sig_b32.chars().take(4).collect::<String>().to_uppercase();
    
    // Format: PDFPRO-XXXX-XXXX-XXXX-XXXX-YYYY
    format!(
        "PDFPRO-{}-{}-{}-{}-{}",
        &data[0..4],
        &data[4..8],
        &data[8..12],
        &data[12..16],
        signature
    )
}

fn main() {
    // Get count from command line args or default to 10
    let args: Vec<String> = std::env::args().collect();
    let count: usize = if args.len() > 1 {
        args[1].parse().unwrap_or(10)
    } else {
        10
    };
    
    println!("Generating {} license keys...\n", count);
    println!("license_key");
    println!("{}", "-".repeat(40));
    
    for _ in 0..count {
        println!("{}", generate_license_key());
    }
    
    println!("\n{} keys generated successfully!", count);
    println!("\nNOTE: Keep these keys secure!");
    println!("Upload them to Lemon Squeezy as product license keys.");
}
