use crate::database::{Database, PdfDocument};
use anyhow::{Context, Result};
use rayon::prelude::*;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use walkdir::WalkDir;

/// Configuration for PDF indexing with resource limits
/// Reference: "Systems Performance" Ch. 6 - CPU Performance
pub struct IndexConfig {
    /// Maximum file size to process (bytes). Files larger are skipped.
    pub max_file_size: u64,
    /// Minimum file size to process (bytes). Files smaller are skipped.
    pub min_file_size: u64,
    /// Maximum number of parallel threads (0 = use all cores)
    pub max_threads: usize,
}

impl Default for IndexConfig {
    fn default() -> Self {
        Self {
            max_file_size: 100 * 1024 * 1024, // 100 MB
            min_file_size: 100,                 // 100 bytes
            max_threads: 0,                     // Use all available cores
        }
    }
}

pub struct PdfIndexer {
    db: Database,
    config: IndexConfig,
}

impl PdfIndexer {
    pub fn new(db: Database) -> Self {
        Self::with_config(db, IndexConfig::default())
    }
    
    pub fn with_config(db: Database, config: IndexConfig) -> Self {
        // Configure Rayon thread pool if max_threads is specified
        if config.max_threads > 0 {
            rayon::ThreadPoolBuilder::new()
                .num_threads(config.max_threads)
                .build_global()
                .ok(); // Ignore errors if pool already initialized
        }
        
        PdfIndexer { db, config }
    }

    /// Index a folder with improved performance and reliability
    /// - Uses parallel processing for PDF extraction
    /// - Implements incremental indexing (only processes changed files)
    /// - Batches database operations for better I/O performance
    /// - Provides detailed performance metrics
    /// Reference: "Systems Performance" Ch. 2 - Methodology (USE Method)
    pub fn index_folder(&self, folder_path: &str) -> Result<usize> {
        let start_time = Instant::now();
        log::info!("Starting indexing for folder: {}", folder_path);

        // Collect all PDF files to process
        let collect_start = Instant::now();
        let pdf_files = self.collect_pdf_files(folder_path)?;
        let collect_duration = collect_start.elapsed();
        log::info!("Found {} PDF files in {:?}", pdf_files.len(), collect_duration);

        if pdf_files.is_empty() {
            self.db.add_indexed_folder(folder_path)?;
            return Ok(0);
        }

        // Get existing files from database for incremental indexing
        let db_query_start = Instant::now();
        let existing_files = self.db.get_files_in_folder(folder_path)?;
        let db_query_duration = db_query_start.elapsed();
        log::debug!("Database query took {:?}", db_query_duration);
        
        // Determine which files need processing
        let filter_start = Instant::now();
        let files_to_process = self.filter_files_to_process(&pdf_files, &existing_files)?;
        let filter_duration = filter_start.elapsed();
        log::info!("Processing {} files (skipping {} unchanged) - filtering took {:?}", 
                   files_to_process.len(), 
                   pdf_files.len() - files_to_process.len(),
                   filter_duration);

        // Remove files that no longer exist
        let cleanup_start = Instant::now();
        self.remove_deleted_files(folder_path, &pdf_files, &existing_files)?;
        let cleanup_duration = cleanup_start.elapsed();
        log::debug!("Cleanup took {:?}", cleanup_duration);

        if files_to_process.is_empty() {
            self.db.add_indexed_folder(folder_path)?;
            log::info!("No files to process. Total time: {:?}", start_time.elapsed());
            return Ok(0);
        }

        // Process PDFs in parallel using Rayon
        let extract_start = Instant::now();
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
        
        let extract_duration = extract_start.elapsed();
        let docs = processed_docs.lock().unwrap();
        let count = docs.len();
        
        // Calculate and log performance metrics
        if count > 0 {
            let throughput = count as f64 / extract_duration.as_secs_f64();
            log::info!("Extracted {} documents in {:?} ({:.2} docs/sec)", 
                       count, extract_duration, throughput);
        }

        // Batch insert all documents in a single transaction
        if count > 0 {
            let insert_start = Instant::now();
            log::info!("Inserting {} documents into database", count);
            self.db.batch_insert_pdfs(&docs, folder_path)?;
            let insert_duration = insert_start.elapsed();
            log::info!("Database insertion took {:?}", insert_duration);
        }

        // Update folder timestamp
        self.db.add_indexed_folder(folder_path)?;

        // Log any errors
        let error_list = errors.lock().unwrap();
        if !error_list.is_empty() {
            log::warn!("Completed with {} errors", error_list.len());
            for (path, error) in error_list.iter().take(5) {
                log::debug!("  Error in {}: {}", path.display(), error);
            }
            if error_list.len() > 5 {
                log::debug!("  ... and {} more errors", error_list.len() - 5);
            }
        }

        let total_duration = start_time.elapsed();
        log::info!("Indexing complete: {} documents processed in {:?}", count, total_duration);
        
        // Log performance summary
        if count > 0 {
            let avg_time_per_doc = total_duration.as_secs_f64() / count as f64;
            log::info!("Performance: {:.3}s per document average", avg_time_per_doc);
        }
        
        Ok(count)
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
        let (content, pages) = extract_text_from_pdf(path, &self.config)?;

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
/// Reference: "PDF Explained" Ch. 9 - Text Extraction
/// Reference: "Systems Performance" Ch. 8 - File Systems (I/O optimization)
fn extract_text_from_pdf(path: &Path, config: &IndexConfig) -> Result<(String, i32)> {
    // Validate file before processing
    if !path.exists() {
        anyhow::bail!("File does not exist: {}", path.display());
    }

    // Check file size with configurable limits
    let metadata = fs::metadata(path)?;
    let size = metadata.len();
    
    if size < config.min_file_size {
        log::warn!("File too small (< {} bytes), likely corrupt: {}", 
                   config.min_file_size, path.display());
        return Ok((String::new(), 0));
    }
    
    if size > config.max_file_size {
        log::warn!("File too large (> {} bytes), skipping: {}", 
                   config.max_file_size, path.display());
        return Ok((String::new(), 0));
    }

    // Try to extract text using pdf-extract, catching panics
    // Reference: "Developing with PDF" Ch. 4 - Text Extraction Best Practices
    let path_buf = path.to_path_buf();
    let result = std::panic::catch_unwind(|| {
        pdf_extract::extract_text(&path_buf)
    });

    match result {
        Ok(Ok(text)) => {
            // Successfully extracted text
            if text.is_empty() {
                log::debug!("No text content extracted from {}", path.display());
                Ok((String::new(), 0))
            } else {
                log::debug!("Extracted {} bytes from {}", text.len(), path.display());
                
                // Normalize text for better indexing and search
                // Reference: "Introduction to Information Retrieval" Ch. 2 - Text Processing
                let normalized = normalize_text(&text);
                let pages = estimate_page_count(&text);
                
                Ok((normalized, pages))
            }
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

/// Normalize text for better indexing and search quality
/// Reference: "Introduction to Information Retrieval" Ch. 2.2 - Normalization
fn normalize_text(text: &str) -> String {
    // Remove excessive whitespace and normalize line breaks
    // This improves index size and search quality
    let mut result = String::with_capacity(text.len());
    let mut prev_was_space = false;
    
    for c in text.chars() {
        if c.is_whitespace() {
            if !prev_was_space {
                result.push(' ');
                prev_was_space = true;
            }
        } else {
            result.push(c);
            prev_was_space = false;
        }
    }
    
    result.trim().to_string()
}

/// Estimate page count from extracted text
/// Uses multiple heuristics for better accuracy
/// Reference: "PDF Explained" Ch. 3 - Document Structure
fn estimate_page_count(text: &str) -> i32 {
    if text.is_empty() {
        return 0;
    }
    
    // Strategy 1: Count form feeds which often indicate page breaks
    let form_feeds = text.chars().filter(|&c| c == '\x0C').count();
    if form_feeds > 0 {
        return (form_feeds + 1) as i32;
    }
    
    // Strategy 2: Look for common page break patterns
    // Some PDFs use specific strings like "Page N" or similar markers
    let newline_sequences = text.matches("\n\n\n").count();
    if newline_sequences > 5 {
        // Multiple triple-newlines often indicate page breaks
        // This is a heuristic and may not be accurate for all PDFs
        let estimated = (newline_sequences as f64 * 0.8) as i32 + 1;
        if estimated > 1 {
            return estimated;
        }
    }
    
    // Strategy 3: Character density heuristic
    // Average page: ~2000-4000 characters depending on density
    // Use 3000 as a middle ground with better rounding
    let chars = text.len();
    let pages = ((chars as f64 / 3000.0).ceil() as usize).max(1);
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
        assert_eq!(estimate_page_count(&"a".repeat(3001)), 2);
        assert_eq!(estimate_page_count(&"a".repeat(6000)), 2);
        
        // Test form feed detection
        let text_with_breaks = "page1\x0Cpage2\x0Cpage3";
        assert_eq!(estimate_page_count(text_with_breaks), 3);
        
        // Test triple-newline heuristic
        let text_with_newlines = "page1\n\n\npage2\n\n\npage3\n\n\npage4\n\n\npage5\n\n\npage6";
        let count = estimate_page_count(text_with_newlines);
        assert!(count >= 4 && count <= 6); // Should estimate around 5 pages
    }
    
    #[test]
    fn test_normalize_text() {
        // Test whitespace normalization
        assert_eq!(normalize_text("hello   world"), "hello world");
        assert_eq!(normalize_text("hello\n\nworld"), "hello world");
        assert_eq!(normalize_text("hello\tworld"), "hello world");
        assert_eq!(normalize_text("  hello  world  "), "hello world");
        
        // Test mixed whitespace
        assert_eq!(normalize_text("a\n  b\t\tc   "), "a b c");
        
        // Test empty and whitespace-only strings
        assert_eq!(normalize_text(""), "");
        assert_eq!(normalize_text("   "), "");
    }
    
    #[test]
    fn test_index_config_default() {
        let config = IndexConfig::default();
        assert_eq!(config.max_file_size, 100 * 1024 * 1024);
        assert_eq!(config.min_file_size, 100);
        assert_eq!(config.max_threads, 0);
    }
}
