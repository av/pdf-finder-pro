use crate::database::{Database, PdfDocument};
use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub struct PdfIndexer {
    db: Database,
}

impl PdfIndexer {
    pub fn new(db: Database) -> Self {
        PdfIndexer { db }
    }
    
    pub fn index_folder(&self, folder_path: &str) -> Result<usize> {
        let mut count = 0;
        
        // Walk through directory recursively
        for entry in WalkDir::new(folder_path)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            
            // Check if file is a PDF
            if path.is_file() && is_pdf_file(path) {
                match self.index_pdf(path) {
                    Ok(_) => count += 1,
                    Err(e) => {
                        eprintln!("Error indexing {}: {}", path.display(), e);
                    }
                }
            }
        }
        
        Ok(count)
    }
    
    fn index_pdf(&self, path: &Path) -> Result<()> {
        let metadata = fs::metadata(path)?;
        let size = metadata.len() as i64;
        let modified = metadata
            .modified()?
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs() as i64;
        
        let title = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Untitled")
            .to_string();
        
        // Extract text from PDF
        let (content, pages) = extract_text_from_pdf(path)?;
        
        let doc = PdfDocument {
            id: None,
            path: path.to_string_lossy().to_string(),
            title,
            content,
            size,
            modified,
            pages: Some(pages),
        };
        
        self.db.insert_pdf(&doc)?;
        
        Ok(())
    }
}

fn is_pdf_file(path: &Path) -> bool {
    path.extension()
        .and_then(|s| s.to_str())
        .map(|s| s.eq_ignore_ascii_case("pdf"))
        .unwrap_or(false)
}

fn extract_text_from_pdf(path: &Path) -> Result<(String, i32)> {
    // Try to extract text using pdf-extract
    match pdf_extract::extract_text(path) {
        Ok(text) => {
            // Count approximate pages (rough estimate based on text length)
            let pages = estimate_page_count(&text);
            Ok((text, pages))
        }
        Err(e) => {
            // If extraction fails, return empty content but still index the file
            eprintln!("Warning: Could not extract text from {}: {}", path.display(), e);
            Ok((String::new(), 0))
        }
    }
}

fn estimate_page_count(text: &str) -> i32 {
    // Rough estimate: ~3000 characters per page
    let chars = text.len();
    let pages = (chars / 3000).max(1);
    pages as i32
}
