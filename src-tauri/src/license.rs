use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// License information stored locally
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct License {
    /// The license key (format: PDFPRO-XXXX-XXXX-XXXX-XXXX-YYYY)
    pub key: String,
    /// Unix timestamp when the license was activated
    pub activated_at: i64,
}

impl License {
    /// Create a new license
    pub fn new(key: String) -> Self {
        let activated_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        Self { key, activated_at }
    }

    /// Get the path to the license file
    pub fn get_license_path() -> Result<PathBuf> {
        let mut path = dirs::data_local_dir()
            .context("Could not find data directory")?;
        path.push("pdf-finder-pro");
        fs::create_dir_all(&path)
            .context("Failed to create license directory")?;
        path.push("license.key");
        Ok(path)
    }

    /// Save the license to disk
    pub fn save(&self) -> Result<()> {
        let path = Self::get_license_path()?;
        let content = serde_json::to_string_pretty(self)
            .context("Failed to serialize license")?;
        fs::write(&path, content)
            .context("Failed to write license file")?;
        log::info!("License saved to: {:?}", path);
        Ok(())
    }

    /// Load the license from disk
    pub fn load() -> Result<Self> {
        let path = Self::get_license_path()?;
        if !path.exists() {
            anyhow::bail!("License file does not exist");
        }
        let content = fs::read_to_string(&path)
            .context("Failed to read license file")?;
        let license: License = serde_json::from_str(&content)
            .context("Failed to parse license file")?;
        Ok(license)
    }

    /// Check if a license file exists
    pub fn exists() -> bool {
        if let Ok(path) = Self::get_license_path() {
            path.exists()
        } else {
            false
        }
    }

    /// Delete the license file
    pub fn delete() -> Result<()> {
        let path = Self::get_license_path()?;
        if path.exists() {
            fs::remove_file(&path)
                .context("Failed to delete license file")?;
            log::info!("License file deleted");
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_license_creation() {
        let key = "PDFPRO-A7B2-C9D4-E1F6-G8H3-K2M4".to_string();
        let license = License::new(key.clone());
        assert_eq!(license.key, key);
        assert!(license.activated_at > 0);
    }

    #[test]
    fn test_license_serialization() {
        let key = "PDFPRO-TEST-TEST-TEST-TEST-TEST".to_string();
        let license = License::new(key);
        let json = serde_json::to_string(&license).unwrap();
        let deserialized: License = serde_json::from_str(&json).unwrap();
        assert_eq!(license.key, deserialized.key);
        assert_eq!(license.activated_at, deserialized.activated_at);
    }
}
