use rusqlite::{params, Connection, Result as SqliteResult};
use serde::{Deserialize, Serialize};
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

#[derive(Clone)]
pub struct Database {
    conn: Arc<Mutex<Connection>>,
}

impl Database {
    pub fn new(db_path: PathBuf) -> anyhow::Result<Self> {
        let conn = Connection::open(db_path)?;
        
        // Create tables with FTS5 for full-text search
        conn.execute(
            "CREATE TABLE IF NOT EXISTS pdfs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                path TEXT UNIQUE NOT NULL,
                title TEXT NOT NULL,
                content TEXT NOT NULL,
                size INTEGER NOT NULL,
                modified INTEGER NOT NULL,
                pages INTEGER
            )",
            [],
        )?;
        
        // Create FTS5 virtual table for full-text search
        conn.execute(
            "CREATE VIRTUAL TABLE IF NOT EXISTS pdfs_fts USING fts5(
                path UNINDEXED,
                title,
                content,
                content=pdfs,
                content_rowid=id
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
        
        Ok(Database {
            conn: Arc::new(Mutex::new(conn)),
        })
    }
    
    pub fn insert_pdf(&self, doc: &PdfDocument) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();
        
        conn.execute(
            "INSERT OR REPLACE INTO pdfs (path, title, content, size, modified, pages)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                &doc.path,
                &doc.title,
                &doc.content,
                doc.size,
                doc.modified,
                doc.pages
            ],
        )?;
        
        Ok(())
    }
    
    pub fn search(&self, query: &str, filters: &SearchFilters) -> anyhow::Result<Vec<SearchResult>> {
        let conn = self.conn.lock().unwrap();
        
        // Build the search query with filters
        let mut sql = String::from(
            "SELECT p.path, p.title, p.size, p.modified, p.pages, snippet(pdfs_fts, 2, '', '', '...', 64) as snippet
             FROM pdfs p
             INNER JOIN pdfs_fts ON p.id = pdfs_fts.rowid
             WHERE pdfs_fts MATCH ?1"
        );
        
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(query.to_string())];
        
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
        
        sql.push_str(" ORDER BY rank LIMIT 100");
        
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
}

fn parse_date_to_timestamp(date_str: &str) -> anyhow::Result<i64> {
    use chrono::NaiveDate;
    let date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d")?;
    Ok(date.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp())
}
