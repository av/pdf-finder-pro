// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod database;
mod indexer;

use database::{Database, SearchFilters, IndexedFolder};
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
async fn get_index_stats(state: State<'_, AppState>) -> Result<i64, String> {
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
    database
        .get_count()
        .map_err(|e| format!("Failed to get count: {}", e))
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

#[tauri::command]
async fn get_indexed_folders(state: State<'_, AppState>) -> Result<Vec<IndexedFolder>, String> {
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
    database
        .get_indexed_folders()
        .map_err(|e| format!("Failed to get folders: {}", e))
}

#[tauri::command]
async fn remove_indexed_folder(folder_path: String, state: State<'_, AppState>) -> Result<(), String> {
    let db_lock = state.db.lock().unwrap();
    let db = db_lock
        .as_ref()
        .ok_or("Database not initialized")?;

    db.remove_indexed_folder(&folder_path)
        .map_err(|e| format!("Failed to remove folder: {}", e))
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
    let tokens: Vec<String> = query
        .split_whitespace()
        .map(|token| {
            let upper = token.to_uppercase();
            if upper == "AND" || upper == "OR" || upper == "NOT" {
                upper
            } else {
                token.to_string()
            }
        })
        .collect();

    let has_boolean_operator = tokens.iter().any(|t| t == "AND" || t == "OR" || t == "NOT");

    if has_boolean_operator {
        tokens.join(" ")
    } else if tokens.len() > 1 {
        tokens.join(" OR ")
    } else {
        tokens.join(" ")
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(AppState {
            db: Mutex::new(None),
        })
        .invoke_handler(tauri::generate_handler![
            index_pdfs, 
            search_pdfs, 
            open_pdf, 
            get_index_stats,
            get_indexed_folders,
            remove_indexed_folder
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
