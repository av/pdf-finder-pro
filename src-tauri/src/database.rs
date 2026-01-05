use rusqlite::{params, Connection, Result as SqliteResult, Transaction};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfDocument {
    pub id: Option<i64>,
    pub path: String,
    pub title: String,
    pub content: String,
    pub size: i64,
    pub modified: i64,
    pub pages: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub path: String,
    pub title: String,
    pub size: i64,
    pub modified: i64,
    pub pages: Option<i32>,
    pub snippet: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchFilters {
    pub min_size: Option<i64>,
    pub max_size: Option<i64>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexedFolder {
    pub path: String,
    pub last_indexed: i64,
    pub pdf_count: i64,
}

#[derive(Clone)]
pub struct Database {
    conn: Arc<Mutex<Connection>>,
}

impl Database {
    pub fn new(db_path: PathBuf) -> anyhow::Result<Self> {
        let conn = Connection::open(db_path)?;

        // Enable optimizations for better write performance
        // Reference: "Managing Gigabytes" Ch. 5 - Index Construction
        // Reference: "Systems Performance" Ch. 9 - Disk I/O
        // PRAGMA settings optimized for:
        // - WAL mode for better concurrency
        // - 64MB cache, 256MB mmap for read performance
        // - Balanced durability vs speed
        conn.execute_batch(
            "PRAGMA journal_mode=WAL;
             PRAGMA synchronous=NORMAL;
             PRAGMA cache_size=-64000;
             PRAGMA temp_store=MEMORY;
             PRAGMA mmap_size=268435456;
             PRAGMA page_size=4096;
             PRAGMA wal_autocheckpoint=1000;
             PRAGMA busy_timeout=5000;
             PRAGMA optimize;"
        )?;

        // Create folders table to track indexed folders
        conn.execute(
            "CREATE TABLE IF NOT EXISTS indexed_folders (
                path TEXT PRIMARY KEY NOT NULL,
                last_indexed INTEGER NOT NULL
            )",
            [],
        )?;

        // Create tables with FTS5 for full-text search
        conn.execute(
            "CREATE TABLE IF NOT EXISTS pdfs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                path TEXT UNIQUE NOT NULL,
                title TEXT NOT NULL,
                content TEXT NOT NULL,
                size INTEGER NOT NULL,
                modified INTEGER NOT NULL,
                pages INTEGER,
                folder_path TEXT DEFAULT ''
            )",
            [],
        )?;

        // Try to add folder_path column if it doesn't exist (migration for existing databases)
        // This will fail silently if the column already exists
        let _ = conn.execute(
            "ALTER TABLE pdfs ADD COLUMN folder_path TEXT DEFAULT ''",
            [],
        );

        // Create FTS5 virtual table with optimized tokenizer
        // Using porter tokenizer for better stemming support
        conn.execute(
            "CREATE VIRTUAL TABLE IF NOT EXISTS pdfs_fts USING fts5(
                path UNINDEXED,
                title,
                content,
                content=pdfs,
                content_rowid=id,
                tokenize='porter unicode61 remove_diacritics 1'
            )",
            [],
        )?;

        // Create triggers to keep FTS index in sync
        conn.execute(
            "CREATE TRIGGER IF NOT EXISTS pdfs_ai AFTER INSERT ON pdfs BEGIN
                INSERT INTO pdfs_fts(rowid, path, title, content)
                VALUES (new.id, new.path, new.title, new.content);
            END",
            [],
        )?;

        conn.execute(
            "CREATE TRIGGER IF NOT EXISTS pdfs_ad AFTER DELETE ON pdfs BEGIN
                DELETE FROM pdfs_fts WHERE rowid = old.id;
            END",
            [],
        )?;

        conn.execute(
            "CREATE TRIGGER IF NOT EXISTS pdfs_au AFTER UPDATE ON pdfs BEGIN
                DELETE FROM pdfs_fts WHERE rowid = old.id;
                INSERT INTO pdfs_fts(rowid, path, title, content)
                VALUES (new.id, new.path, new.title, new.content);
            END",
            [],
        )?;

        // Create indexes for better query performance
        // Reference: "Introduction to Information Retrieval" Ch. 4 - Index Construction
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_pdfs_folder_path ON pdfs(folder_path)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_pdfs_modified ON pdfs(modified)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_pdfs_size ON pdfs(size)",
            [],
        )?;

        // Optimize FTS5 index for better search performance
        let _ = conn.execute("INSERT INTO pdfs_fts(pdfs_fts) VALUES('optimize')", []);

        // Analyze tables to update query planner statistics
        let _ = conn.execute("ANALYZE", []);

        Ok(Database {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    pub fn insert_pdf(&self, doc: &PdfDocument, folder_path: &str) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();

        conn.execute(
            "INSERT OR REPLACE INTO pdfs (path, title, content, size, modified, pages, folder_path)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                &doc.path,
                &doc.title,
                &doc.content,
                doc.size,
                doc.modified,
                doc.pages,
                folder_path
            ],
        )?;

        Ok(())
    }

    /// Batch insert multiple PDFs in a single transaction for better performance
    /// This is significantly faster than individual inserts
    pub fn batch_insert_pdfs(&self, docs: &[PdfDocument], folder_path: &str) -> anyhow::Result<()> {
        let mut conn = self.conn.lock().unwrap();

        let tx = conn.transaction()?;

        {
            let mut stmt = tx.prepare(
                "INSERT OR REPLACE INTO pdfs (path, title, content, size, modified, pages, folder_path)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)"
            )?;

            for doc in docs {
                stmt.execute(params![
                    &doc.path,
                    &doc.title,
                    &doc.content,
                    doc.size,
                    doc.modified,
                    doc.pages,
                    folder_path
                ])?;
            }
        }

        tx.commit()?;

        Ok(())
    }

    /// Get existing files in a folder with their metadata for incremental indexing
    pub fn get_files_in_folder(&self, folder_path: &str) -> anyhow::Result<HashMap<String, (i64, i64)>> {
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn.prepare(
            "SELECT path, modified, size FROM pdfs WHERE folder_path = ?1"
        )?;

        let rows = stmt.query_map(params![folder_path], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, i64>(2)?,
            ))
        })?;

        let mut result = HashMap::new();
        for row in rows {
            if let Ok((path, modified, size)) = row {
                result.insert(path, (modified, size));
            }
        }

        Ok(result)
    }

    /// Remove a specific PDF by path
    pub fn remove_pdf_by_path(&self, path: &str) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM pdfs WHERE path = ?1", params![path])?;
        Ok(())
    }

    pub fn search(&self, query: &str, filters: &SearchFilters) -> anyhow::Result<Vec<SearchResult>> {
        let conn = self.conn.lock().unwrap();

        // Validate and optimize query
        // Reference: "Introduction to Information Retrieval" Ch. 2 - Query Processing
        let optimized_query = optimize_search_query(query);

        // Build the search query with filters
        // Use BM25 ranking for better relevance
        // Reference: "Introduction to Information Retrieval" Ch. 6 - Scoring and Ranking
        let mut sql = String::from(
            "SELECT p.path, p.title, p.size, p.modified, p.pages,
                    snippet(pdfs_fts, 2, '<mark>', '</mark>', '...', 64) as snippet
             FROM pdfs p
             INNER JOIN pdfs_fts ON p.id = pdfs_fts.rowid
             WHERE pdfs_fts MATCH ?1"
        );

        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(optimized_query)];

        if let Some(min_size) = filters.min_size {
            sql.push_str(" AND p.size >= ?");
            params_vec.push(Box::new(min_size));
        }

        if let Some(max_size) = filters.max_size {
            sql.push_str(" AND p.size <= ?");
            params_vec.push(Box::new(max_size));
        }

        if let Some(date_from) = &filters.date_from {
            if let Ok(timestamp) = parse_date_to_timestamp(date_from) {
                sql.push_str(" AND p.modified >= ?");
                params_vec.push(Box::new(timestamp));
            }
        }

        if let Some(date_to) = &filters.date_to {
            if let Ok(timestamp) = parse_date_to_timestamp(date_to) {
                sql.push_str(" AND p.modified <= ?");
                params_vec.push(Box::new(timestamp + 86400)); // Add 1 day to include entire day
            }
        }

        // Order by BM25 rank (best matches first) and limit results
        sql.push_str(" ORDER BY bm25(pdfs_fts) LIMIT 100");

        let params_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();

        let mut stmt = conn.prepare(&sql)?;
        let results = stmt.query_map(params_refs.as_slice(), |row| {
            Ok(SearchResult {
                path: row.get(0)?,
                title: row.get(1)?,
                size: row.get(2)?,
                modified: row.get(3)?,
                pages: row.get(4)?,
                snippet: row.get(5).ok(),
            })
        })?;

        let mut search_results = Vec::new();
        for result in results {
            if let Ok(r) = result {
                search_results.push(r);
            }
        }

        Ok(search_results)
    }

    pub fn clear(&self) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM pdfs", [])?;
        Ok(())
    }

    pub fn get_count(&self) -> anyhow::Result<i64> {
        let conn = self.conn.lock().unwrap();
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM pdfs", [], |row| row.get(0))?;
        Ok(count)
    }

    pub fn add_indexed_folder(&self, folder_path: &str) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs() as i64;

        conn.execute(
            "INSERT OR REPLACE INTO indexed_folders (path, last_indexed) VALUES (?1, ?2)",
            params![folder_path, timestamp],
        )?;
        Ok(())
    }

    pub fn get_indexed_folders(&self) -> anyhow::Result<Vec<IndexedFolder>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT f.path, f.last_indexed, COUNT(p.id) as pdf_count
             FROM indexed_folders f
             LEFT JOIN pdfs p ON p.folder_path = f.path
             GROUP BY f.path
             ORDER BY f.last_indexed DESC"
        )?;

        let folders = stmt.query_map([], |row| {
            Ok(IndexedFolder {
                path: row.get(0)?,
                last_indexed: row.get(1)?,
                pdf_count: row.get(2)?,
            })
        })?;

        let mut result = Vec::new();
        for folder in folders {
            if let Ok(f) = folder {
                result.push(f);
            }
        }
        Ok(result)
    }

    pub fn remove_indexed_folder(&self, folder_path: &str) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM pdfs WHERE folder_path = ?1", params![folder_path])?;
        conn.execute("DELETE FROM indexed_folders WHERE path = ?1", params![folder_path])?;
        Ok(())
    }

    pub fn remove_pdfs_for_folder(&self, folder_path: &str) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM pdfs WHERE folder_path = ?1", params![folder_path])?;
        Ok(())
    }
}

fn parse_date_to_timestamp(date_str: &str) -> anyhow::Result<i64> {
    use chrono::NaiveDate;
    let date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d")?;
    Ok(date.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp())
}

/// Optimize search query for better FTS5 performance
/// Reference: "Introduction to Information Retrieval" Ch. 2 - Query Processing
fn optimize_search_query(query: &str) -> String {
    if query.is_empty() {
        return query.to_string();
    }

    // Remove excessive whitespace and normalize query structure
    // This improves consistency and ensures predictable FTS5 behavior
    // Note: We preserve all FTS5 operators (AND, OR, NOT, quotes) as-is
    // Future enhancement: Could add stop word removal or query expansion here
    query.trim()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn create_test_db() -> Database {
        let temp_dir = std::env::temp_dir();
        let db_path = temp_dir.join(format!("test_pdf_db_{}.db", uuid::Uuid::new_v4()));
        Database::new(db_path).unwrap()
    }

    fn create_test_document(path: &str) -> PdfDocument {
        PdfDocument {
            id: None,
            path: path.to_string(),
            title: "Test Document".to_string(),
            content: "This is test content for searching".to_string(),
            size: 1024,
            modified: 1000000,
            pages: Some(5),
        }
    }

    #[test]
    fn test_insert_and_get_count() {
        let db = create_test_db();
        let doc = create_test_document("/test/doc1.pdf");

        db.insert_pdf(&doc, "/test").unwrap();
        let count = db.get_count().unwrap();

        assert_eq!(count, 1);
    }

    #[test]
    fn test_batch_insert() {
        let db = create_test_db();

        let docs = vec![
            create_test_document("/test/doc1.pdf"),
            create_test_document("/test/doc2.pdf"),
            create_test_document("/test/doc3.pdf"),
        ];

        db.batch_insert_pdfs(&docs, "/test").unwrap();
        let count = db.get_count().unwrap();

        assert_eq!(count, 3);
    }

    #[test]
    fn test_get_files_in_folder() {
        let db = create_test_db();

        let doc1 = create_test_document("/test/doc1.pdf");
        let doc2 = create_test_document("/test/doc2.pdf");

        db.insert_pdf(&doc1, "/test").unwrap();
        db.insert_pdf(&doc2, "/test").unwrap();

        let files = db.get_files_in_folder("/test").unwrap();

        assert_eq!(files.len(), 2);
        assert!(files.contains_key("/test/doc1.pdf"));
        assert!(files.contains_key("/test/doc2.pdf"));
    }

    #[test]
    fn test_search_basic() {
        let db = create_test_db();

        let doc = PdfDocument {
            id: None,
            path: "/test/doc1.pdf".to_string(),
            title: "Machine Learning".to_string(),
            content: "This document discusses machine learning algorithms".to_string(),
            size: 2048,
            modified: 1000000,
            pages: Some(10),
        };

        db.insert_pdf(&doc, "/test").unwrap();

        let filters = SearchFilters {
            min_size: None,
            max_size: None,
            date_from: None,
            date_to: None,
        };

        let results = db.search("machine", &filters).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Machine Learning");
    }

    #[test]
    fn test_remove_pdf_by_path() {
        let db = create_test_db();
        let doc = create_test_document("/test/doc1.pdf");

        db.insert_pdf(&doc, "/test").unwrap();
        assert_eq!(db.get_count().unwrap(), 1);

        db.remove_pdf_by_path("/test/doc1.pdf").unwrap();
        assert_eq!(db.get_count().unwrap(), 0);
    }

    #[test]
    fn test_indexed_folders() {
        let db = create_test_db();

        db.add_indexed_folder("/test/folder1").unwrap();
        db.add_indexed_folder("/test/folder2").unwrap();

        let folders = db.get_indexed_folders().unwrap();
        assert_eq!(folders.len(), 2);
    }

    #[test]
    fn test_optimize_search_query() {
        assert_eq!(optimize_search_query(""), "");
        assert_eq!(optimize_search_query("  hello  world  "), "hello world");
        assert_eq!(optimize_search_query("test\tquery"), "test query");

        // Test quote handling
        assert_eq!(optimize_search_query("\"exact phrase\""), "\"exact phrase\"");
        assert_eq!(optimize_search_query("before \"exact phrase\" after"),
                   "before \"exact phrase\" after");
    }

    #[test]
    fn test_search_with_filters() {
        let db = create_test_db();

        // Add test documents with different sizes
        let doc1 = PdfDocument {
            id: None,
            path: "/test/small.pdf".to_string(),
            title: "Small Document".to_string(),
            content: "This is a small test document".to_string(),
            size: 1000,
            modified: 1000000,
            pages: Some(1),
        };

        let doc2 = PdfDocument {
            id: None,
            path: "/test/large.pdf".to_string(),
            title: "Large Document".to_string(),
            content: "This is a large test document".to_string(),
            size: 10000,
            modified: 2000000,
            pages: Some(10),
        };

        db.insert_pdf(&doc1, "/test").unwrap();
        db.insert_pdf(&doc2, "/test").unwrap();

        // Test size filter
        let filters = SearchFilters {
            min_size: Some(5000),
            max_size: None,
            date_from: None,
            date_to: None,
        };

        let results = db.search("document", &filters).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Large Document");
    }
}

// Add uuid dependency only for tests
#[cfg(test)]
use uuid;
