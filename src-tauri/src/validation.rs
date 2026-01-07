use anyhow::{Context, Result};
use base32::Alphabet;
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::license::License;

type HmacSha256 = Hmac<Sha256>;

/// License validation status
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status")]
pub enum LicenseStatus {
    /// License is valid and active
    Valid {
        key: String,
        activated_at: i64,
    },
    /// In trial period
    Trial {
        days_remaining: i32,
    },
    /// Trial expired, needs license
    Expired,
    /// License key is invalid
    Invalid {
        reason: String,
    },
}

/// The secret key used for HMAC signing (embedded in the binary)
/// In production, this should be generated and kept private
/// For now, using a placeholder that should be changed before release
const HMAC_SECRET: &str = "pdf_finder_pro_secret_key_v1_change_before_release";

/// Validates a license key using cryptographic signature verification
pub fn verify_license_key_signature(key: &str) -> Result<bool> {
    // Parse: PDFPRO-AAAA-BBBB-CCCC-DDDD-EEEE
    let parts: Vec<&str> = key.split('-').collect();
    
    if parts.len() != 6 {
        return Ok(false);
    }
    
    if parts[0] != "PDFPRO" {
        return Ok(false);
    }
    
    // Groups 1-4 contain the data (16 characters total)
    if parts[1].len() != 4 || parts[2].len() != 4 || parts[3].len() != 4 || parts[4].len() != 4 {
        return Ok(false);
    }
    
    let data = format!("{}{}{}{}", parts[1], parts[2], parts[3], parts[4]);
    
    // Group 5 is the signature (4 characters)
    let provided_sig = parts[5];
    if provided_sig.len() != 4 {
        return Ok(false);
    }
    
    // Compute expected signature
    let computed_sig = compute_signature(&data)?;
    
    // Constant-time comparison to prevent timing attacks
    Ok(constant_time_compare(provided_sig, &computed_sig))
}

/// Compute HMAC-SHA256 signature and truncate to 4 base32 characters
fn compute_signature(data: &str) -> Result<String> {
    let mut mac = HmacSha256::new_from_slice(HMAC_SECRET.as_bytes())
        .context("Invalid HMAC key length")?;
    mac.update(data.as_bytes());
    let result = mac.finalize();
    let sig_bytes = result.into_bytes();
    
    // Take first 2 bytes, encode to base32 (gives us ~4 chars)
    let sig_b32 = base32::encode(Alphabet::Crockford, &sig_bytes[..2]);
    
    // Take first 4 characters and uppercase
    Ok(sig_b32.chars().take(4).collect::<String>().to_uppercase())
}

/// Constant-time string comparison to prevent timing attacks
fn constant_time_compare(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }
    
    let mut result = 0u8;
    for (byte_a, byte_b) in a.bytes().zip(b.bytes()) {
        result |= byte_a ^ byte_b;
    }
    
    result == 0
}

/// Get the first launch timestamp (for trial tracking)
pub fn get_first_launch_timestamp() -> Result<i64> {
    let path = get_trial_timestamp_path()?;
    
    if path.exists() {
        let content = fs::read_to_string(&path)
            .context("Failed to read trial timestamp")?;
        let timestamp: i64 = content.trim().parse()
            .context("Invalid trial timestamp")?;
        Ok(timestamp)
    } else {
        // First launch - create the timestamp file
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .context("System time error")?
            .as_secs() as i64;
        
        fs::write(&path, now.to_string())
            .context("Failed to write trial timestamp")?;
        
        log::info!("First launch detected, trial started");
        Ok(now)
    }
}

/// Get the path to the trial timestamp file
fn get_trial_timestamp_path() -> Result<PathBuf> {
    let mut path = dirs::data_local_dir()
        .context("Could not find data directory")?;
    path.push("pdf-finder-pro");
    fs::create_dir_all(&path)
        .context("Failed to create data directory")?;
    path.push("trial.timestamp");
    Ok(path)
}

/// Get the number of days remaining in the trial period
pub fn get_trial_days_remaining() -> Result<i32> {
    let install_timestamp = get_first_launch_timestamp()?;
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("System time error")?
        .as_secs() as i64;
    
    let days_elapsed = (now - install_timestamp) / 86400;
    let remaining = 14 - days_elapsed;
    
    Ok(std::cmp::max(0, remaining as i32))
}

/// Check if currently in trial period
pub fn is_in_trial() -> Result<bool> {
    Ok(!License::exists() && get_trial_days_remaining()? > 0)
}

/// Check if trial has expired
pub fn is_expired() -> Result<bool> {
    Ok(!License::exists() && get_trial_days_remaining()? == 0)
}

/// Main license validation function
pub fn validate_license() -> Result<LicenseStatus> {
    // 1. Check if we have a valid license
    if License::exists() {
        match License::load() {
            Ok(license) => {
                // Validate the signature
                match verify_license_key_signature(&license.key) {
                    Ok(true) => {
                        log::info!("License validated successfully");
                        return Ok(LicenseStatus::Valid {
                            key: license.key.clone(),
                            activated_at: license.activated_at,
                        });
                    }
                    Ok(false) => {
                        log::warn!("License signature verification failed");
                        return Ok(LicenseStatus::Invalid {
                            reason: "Invalid license signature".to_string(),
                        });
                    }
                    Err(e) => {
                        log::error!("Error verifying license: {}", e);
                        return Ok(LicenseStatus::Invalid {
                            reason: format!("Verification error: {}", e),
                        });
                    }
                }
            }
            Err(e) => {
                log::warn!("Failed to load license file: {}", e);
                // License file exists but can't be loaded - treat as invalid
                return Ok(LicenseStatus::Invalid {
                    reason: "Corrupted license file".to_string(),
                });
            }
        }
    }
    
    // 2. No license - check trial status
    match get_trial_days_remaining() {
        Ok(days) if days > 0 => {
            log::info!("In trial period: {} days remaining", days);
            Ok(LicenseStatus::Trial { days_remaining: days })
        }
        Ok(_) => {
            log::info!("Trial expired");
            Ok(LicenseStatus::Expired)
        }
        Err(e) => {
            log::error!("Error checking trial status: {}", e);
            // If we can't determine trial status, assume expired
            Ok(LicenseStatus::Expired)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signature_computation() {
        let data = "A7B2C9D4E1F6G8H3";
        let sig = compute_signature(data).unwrap();
        assert_eq!(sig.len(), 4);
        // Signature should be deterministic
        let sig2 = compute_signature(data).unwrap();
        assert_eq!(sig, sig2);
    }

    #[test]
    fn test_constant_time_compare() {
        assert!(constant_time_compare("TEST", "TEST"));
        assert!(!constant_time_compare("TEST", "FAIL"));
        assert!(!constant_time_compare("TEST", "TES"));
    }

    #[test]
    fn test_invalid_key_formats() {
        // Too few parts
        assert!(!verify_license_key_signature("PDFPRO-AAAA").unwrap());
        
        // Wrong prefix
        assert!(!verify_license_key_signature("WRONGP-AAAA-BBBB-CCCC-DDDD-EEEE").unwrap());
        
        // Wrong length parts
        assert!(!verify_license_key_signature("PDFPRO-AAA-BBBB-CCCC-DDDD-EEEE").unwrap());
    }

    #[test]
    fn test_trial_days_calculation() {
        // This test verifies the calculation logic
        // Actual value depends on when first_launch was set
        let days = get_trial_days_remaining();
        if days.is_ok() {
            let d = days.unwrap();
            assert!(d >= 0 && d <= 14);
        }
    }
}
