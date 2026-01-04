// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod database;
mod indexer;

use database::{Database, SearchFilters};
use indexer::PdfIndexer;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::State;

#[derive(Debug, Serialize, Deserialize)]
struct IndexResult {
    count: usize,
    duration: u128,
}

struct AppState {
    db: Mutex<Option<Database>>,
}

#[tauri::command]
async fn index_pdfs(folder_path: String, state: State<'_, AppState>) -> Result<IndexResult, String> {
    let start = std::time::Instant::now();
    
    // Get or create database
    let db = {
        let mut db_lock = state.db.lock().unwrap();
        if db_lock.is_none() {
            let db_path = get_db_path().map_err(|e| format!("Failed to get DB path: {}", e))?;
            let database = Database::new(db_path).map_err(|e| format!("Failed to create database: {}", e))?;
            *db_lock = Some(database);
        }
        db_lock.clone()
    };
    
    let database = db.ok_or("Database not initialized")?;
    let indexer = PdfIndexer::new(database);
    
    let count = indexer
        .index_folder(&folder_path)
        .map_err(|e| format!("Indexing failed: {}", e))?;
    
    let duration = start.elapsed().as_millis();
    
    Ok(IndexResult { count, duration })
}

#[tauri::command]
async fn search_pdfs(
    query: String,
    filters: SearchFilters,
    state: State<'_, AppState>,
) -> Result<Vec<database::SearchResult>, String> {
    let db_lock = state.db.lock().unwrap();
    let db = db_lock
        .as_ref()
        .ok_or("Database not initialized. Please index PDFs first.")?;
    
    // Transform query to FTS5 format
    let fts_query = transform_query(&query);
    
    let results = db
        .search(&fts_query, &filters)
        .map_err(|e| format!("Search failed: {}", e))?;
    
    Ok(results)
}

#[tauri::command]
async fn open_pdf(path: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/C", "start", "", &path])
            .spawn()
            .map_err(|e| format!("Failed to open PDF: {}", e))?;
    }
    
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("Failed to open PDF: {}", e))?;
    }
    
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("Failed to open PDF: {}", e))?;
    }
    
    Ok(())
}

fn get_db_path() -> anyhow::Result<PathBuf> {
    let mut path = dirs::data_local_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not find data directory"))?;
    path.push("pdf-finder-pro");
    std::fs::create_dir_all(&path)?;
    path.push("index.db");
    Ok(path)
}

fn transform_query(query: &str) -> String {
    // Transform user query to FTS5 format
    // Support boolean operators: AND, OR, NOT
    let mut fts_query = query.to_string();
    
    // Replace common patterns with FTS5 equivalents
    fts_query = fts_query.replace(" AND ", " AND ");
    fts_query = fts_query.replace(" OR ", " OR ");
    fts_query = fts_query.replace(" NOT ", " NOT ");
    
    // If no operators, treat as phrase or individual terms
    if !fts_query.contains(" AND ") 
        && !fts_query.contains(" OR ") 
        && !fts_query.contains(" NOT ") {
        // Split into words and OR them together for better matching
        let words: Vec<&str> = fts_query.split_whitespace().collect();
        if words.len() > 1 {
            fts_query = words.join(" OR ");
        }
    }
    
    fts_query
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(AppState {
            db: Mutex::new(None),
        })
        .invoke_handler(tauri::generate_handler![index_pdfs, search_pdfs, open_pdf])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
