use crate::database::{Database, PdfDocument};
use anyhow::{Context, Result};
use rayon::prelude::*;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use walkdir::WalkDir;

pub struct PdfIndexer {
    db: Database,
}

impl PdfIndexer {
    pub fn new(db: Database) -> Self {
        PdfIndexer { db }
    }

    /// Index a folder with improved performance and reliability
    /// - Uses parallel processing for PDF extraction
    /// - Implements incremental indexing (only processes changed files)
    /// - Batches database operations for better I/O performance
    pub fn index_folder(&self, folder_path: &str) -> Result<usize> {
        log::info!("Starting indexing for folder: {}", folder_path);

        // Collect all PDF files to process
        let pdf_files = self.collect_pdf_files(folder_path)?;
        log::info!("Found {} PDF files", pdf_files.len());

        if pdf_files.is_empty() {
            self.db.add_indexed_folder(folder_path)?;
            return Ok(0);
        }

        // Get existing files from database for incremental indexing
        let existing_files = self.db.get_files_in_folder(folder_path)?;
        
        // Determine which files need processing
        let files_to_process = self.filter_files_to_process(&pdf_files, &existing_files)?;
        log::info!("Processing {} files (skipping {} unchanged)", 
                   files_to_process.len(), 
                   pdf_files.len() - files_to_process.len());

        // Remove files that no longer exist
        self.remove_deleted_files(folder_path, &pdf_files, &existing_files)?;

        if files_to_process.is_empty() {
            self.db.add_indexed_folder(folder_path)?;
            return Ok(0);
        }

        // Process PDFs in parallel using Rayon
        let processed_docs = Arc::new(Mutex::new(Vec::new()));
        let errors = Arc::new(Mutex::new(Vec::new()));

        files_to_process.par_iter().for_each(|path| {
            match self.extract_pdf_data(path, folder_path) {
                Ok(doc) => {
                    processed_docs.lock().unwrap().push(doc);
                }
                Err(e) => {
                    log::warn!("Failed to process {}: {}", path.display(), e);
                    errors.lock().unwrap().push((path.clone(), e.to_string()));
                }
            }
        });

        // Batch insert all documents in a single transaction
        let docs = processed_docs.lock().unwrap();
        let count = docs.len();
        
        if count > 0 {
            log::info!("Inserting {} documents into database", count);
            self.db.batch_insert_pdfs(&docs, folder_path)?;
        }

        // Update folder timestamp
        self.db.add_indexed_folder(folder_path)?;

        // Log any errors
        let error_list = errors.lock().unwrap();
        if !error_list.is_empty() {
            log::warn!("Completed with {} errors", error_list.len());
        }

        log::info!("Indexing complete: {} documents processed", count);
        Ok(count)
    }

    /// Collect all PDF files in the folder recursively
    fn collect_pdf_files(&self, folder_path: &str) -> Result<Vec<PathBuf>> {
        let mut pdf_files = Vec::new();

        for entry in WalkDir::new(folder_path)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.is_file() && is_pdf_file(path) {
                pdf_files.push(path.to_path_buf());
            }
        }

        Ok(pdf_files)
    }

    /// Filter files to only process new or modified files (incremental indexing)
    fn filter_files_to_process(
        &self,
        all_files: &[PathBuf],
        existing_files: &HashMap<String, (i64, i64)>,
    ) -> Result<Vec<PathBuf>> {
        let mut files_to_process = Vec::new();

        for path in all_files {
            let path_str = path.to_string_lossy().to_string();
            
            // Get current file metadata
            let metadata = fs::metadata(path)
                .context(format!("Failed to read metadata for {}", path.display()))?;
            let modified = metadata
                .modified()?
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs() as i64;
            let size = metadata.len() as i64;

            // Check if file is new or modified
            let needs_processing = match existing_files.get(&path_str) {
                Some((existing_modified, existing_size)) => {
                    // Process if size or modification time changed
                    *existing_modified != modified || *existing_size != size
                }
                None => true, // New file
            };

            if needs_processing {
                files_to_process.push(path.clone());
            }
        }

        Ok(files_to_process)
    }

    /// Remove files from database that no longer exist on disk
    fn remove_deleted_files(
        &self,
        folder_path: &str,
        current_files: &[PathBuf],
        existing_files: &HashMap<String, (i64, i64)>,
    ) -> Result<()> {
        let current_paths: std::collections::HashSet<String> = current_files
            .iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect();

        let deleted_files: Vec<String> = existing_files
            .keys()
            .filter(|path| !current_paths.contains(*path))
            .cloned()
            .collect();

        if !deleted_files.is_empty() {
            log::info!("Removing {} deleted files from database", deleted_files.len());
            for path in deleted_files {
                if let Err(e) = self.db.remove_pdf_by_path(&path) {
                    log::warn!("Failed to remove {}: {}", path, e);
                }
            }
        }

        Ok(())
    }

    /// Extract data from a single PDF (used in parallel processing)
    fn extract_pdf_data(&self, path: &Path, folder_path: &str) -> Result<PdfDocument> {
        let metadata = fs::metadata(path)
            .context(format!("Failed to read metadata for {}", path.display()))?;
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

        // Extract text from PDF with improved error handling
        let (content, pages) = extract_text_from_pdf(path)?;

        Ok(PdfDocument {
            id: None,
            path: path.to_string_lossy().to_string(),
            title,
            content,
            size,
            modified,
            pages: Some(pages),
        })
    }
}

fn is_pdf_file(path: &Path) -> bool {
    path.extension()
        .and_then(|s| s.to_str())
        .map(|s| s.eq_ignore_ascii_case("pdf"))
        .unwrap_or(false)
}

/// Extract text from PDF with improved error handling and validation
fn extract_text_from_pdf(path: &Path) -> Result<(String, i32)> {
    // Validate file before processing
    if !path.exists() {
        anyhow::bail!("File does not exist: {}", path.display());
    }

    // Check file size - skip if too large (>100MB) or too small (<100 bytes)
    let metadata = fs::metadata(path)?;
    let size = metadata.len();
    
    if size < 100 {
        log::warn!("File too small, likely corrupt: {}", path.display());
        return Ok((String::new(), 0));
    }
    
    if size > 100 * 1024 * 1024 {
        log::warn!("File too large (>100MB), skipping: {}", path.display());
        return Ok((String::new(), 0));
    }

    // Try to extract text using pdf-extract, catching panics
    let path_buf = path.to_path_buf();
    let result = std::panic::catch_unwind(|| {
        pdf_extract::extract_text(&path_buf)
    });

    match result {
        Ok(Ok(text)) => {
            // Successfully extracted text
            if text.is_empty() {
                log::debug!("No text content extracted from {}", path.display());
            } else {
                log::debug!("Extracted {} bytes from {}", text.len(), path.display());
            }
            let pages = estimate_page_count(&text);
            Ok((text, pages))
        }
        Ok(Err(e)) => {
            // Extraction returned an error
            log::warn!("Could not extract text from {}: {}", path.display(), e);
            // Return empty content rather than failing the entire indexing
            Ok((String::new(), 0))
        }
        Err(_) => {
            // Extraction panicked (e.g., unsupported PDF encoding)
            log::warn!("PDF extraction panicked for {} (possibly unsupported encoding or corrupt file)", path.display());
            // Return empty content rather than failing the entire indexing
            Ok((String::new(), 0))
        }
    }
}

/// Estimate page count from extracted text
/// Uses heuristics based on average character density per page
fn estimate_page_count(text: &str) -> i32 {
    if text.is_empty() {
        return 0;
    }
    
    // Count form feeds which often indicate page breaks
    let form_feeds = text.chars().filter(|&c| c == '\x0C').count();
    if form_feeds > 0 {
        return (form_feeds + 1) as i32;
    }
    
    // Fallback: rough estimate based on character count
    // Average page: ~2000-4000 characters depending on density
    // Use 3000 as a middle ground
    let chars = text.len();
    let pages = (chars / 3000).max(1);
    pages as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_pdf_file() {
        assert!(is_pdf_file(Path::new("test.pdf")));
        assert!(is_pdf_file(Path::new("test.PDF")));
        assert!(is_pdf_file(Path::new("/path/to/file.pdf")));
        assert!(!is_pdf_file(Path::new("test.txt")));
        assert!(!is_pdf_file(Path::new("test")));
    }

    #[test]
    fn test_estimate_page_count() {
        assert_eq!(estimate_page_count(""), 0);
        assert_eq!(estimate_page_count("a"), 1);
        assert_eq!(estimate_page_count(&"a".repeat(3000)), 1);
        assert_eq!(estimate_page_count(&"a".repeat(6000)), 2);
        
        // Test form feed detection
        let text_with_breaks = "page1\x0Cpage2\x0Cpage3";
        assert_eq!(estimate_page_count(text_with_breaks), 3);
    }
}
