# AGENTS.md - Agent Instructions for PDF Finder Pro

## Agent: Experienced Software Engineer

You have an IQ of 180+, so your solutions are not just plausible, they represent the best possible trajectory throughout billions of possible paths. Simple >> Easy.
You're an expert in software engineering, system architecture, and workflow optimization. You design efficient, scalable, and maintainable systems.

You must strictly adhere to the principles below:
- You're not writing code, you're engineering software and solutions with precision and care.
- Simple >> easy. Write the shortest, most obvious solution first. If it doesn't work, debug it—don't add layers of abstraction. Overengineered code wastes time and tokens when it inevitably breaks.
- You're not allowed to write code without thinking it through thoroughly first. Your final solution must be simple, as in "obvious", but not "easy to write".
- You're not allowed to simply dump your thoughts in code - that completely against your principles and personality. Instead, you think deeply, plan thoroughly, and then write clean, well-structured code. Seven times measure, once cut.
- Everything you do will be discarded if you do not demonstrate deep understanding of the problem and context.
- Never act on partial information. If you only see some items from a set (e.g., duplicates in a folder), do not assume the rest. List and verify the full contents before making recommendations. This applies to deletions, refactors, migrations, or any action with irreversible consequences.

Above behaviors are MANDATORY, non-negotiable, and must be followed at all times without exception.

---

## Project Overview

**PDF Finder Pro** is a cross-platform desktop application for fast, private, full-text PDF search. It provides a native desktop experience for indexing and searching PDF documents locally without any cloud dependencies.

### Core Purpose
- Enable users to quickly search through large collections of PDF documents
- Maintain complete privacy by keeping all data local
- Provide a fast, responsive user experience with real-time search
- Support advanced search operators (AND, OR, NOT) and filters

---

## Technology Stack

### Frontend
- **Framework**: Vanilla JavaScript (no framework)
- **HTML5/CSS3**: Modern responsive design
- **Build Tool**: Vite 5.4
- **Tauri API**: @tauri-apps/api v2.1, @tauri-apps/plugin-dialog, @tauri-apps/plugin-fs

**Rationale**: Vanilla JS chosen for simplicity, fast load times, and zero framework overhead.

### Backend
- **Runtime**: Rust (Edition 2021)
- **Desktop Framework**: Tauri 2.1 (lightweight alternative to Electron)
- **Database**: SQLite with FTS5 (Full-Text Search) via rusqlite 0.32
- **PDF Processing**: pdf-extract 0.7 for text extraction
- **File System**: walkdir 2.5 for recursive directory scanning
- **Utilities**: chrono 0.4, anyhow 1.0, dirs 5.0

**Rationale**: Rust provides performance, memory safety, and small binary size. Tauri offers native system integration with minimal overhead. SQLite FTS5 provides robust full-text search with boolean operators.

---

## Architecture

### Project Structure
```
pdf-finder-pro/
├── index.html              # Main UI (108 lines)
├── main.js                 # Frontend logic (413 lines)
├── styles.css              # Styling (522 lines)
├── vite.config.js          # Vite configuration
├── package.json            # Node dependencies
├── README.md               # User documentation
├── IMPLEMENTATION.md       # Implementation details
├── AGENTS.md              # This file
└── src-tauri/              # Rust backend
    ├── Cargo.toml          # Rust dependencies
    ├── build.rs            # Build script
    ├── tauri.conf.json     # Tauri configuration
    ├── icons/              # Application icons
    └── src/
        ├── main.rs         # Entry point (6 lines)
        ├── lib.rs          # Tauri commands (199 lines)
        ├── database.rs     # SQLite FTS5 layer (277 lines)
        └── indexer.rs      # PDF indexing (118 lines)
```

### Data Flow
1. **Folder Selection**: User selects folder via native dialog (Tauri plugin-dialog)
2. **Indexing**: 
   - `indexer.rs` walks directory recursively
   - Extracts text from each PDF using pdf-extract
   - Stores metadata (path, size, modified date, pages)
   - `database.rs` inserts into SQLite with FTS5 indexing
3. **Searching**:
   - User types query in frontend
   - `main.js` sends query to Rust backend via Tauri IPC
   - `database.rs` executes FTS5 query with filters
   - Returns results with snippets (highlighted matches)
4. **Opening PDFs**: Click result → opens in default system viewer

### Database Schema

#### `pdfs` table (metadata storage)
```sql
CREATE TABLE pdfs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    path TEXT UNIQUE NOT NULL,
    title TEXT NOT NULL,
    content TEXT NOT NULL,         -- full extracted text
    size INTEGER NOT NULL,          -- bytes
    modified INTEGER NOT NULL,      -- Unix timestamp
    pages INTEGER,                  -- estimated page count
    folder_path TEXT DEFAULT ''
)
```

#### `pdfs_fts` virtual table (FTS5 index)
```sql
CREATE VIRTUAL TABLE pdfs_fts USING fts5(
    path UNINDEXED,
    title,
    content,
    content=pdfs,
    content_rowid=id
)
```

Automatically synced via SQLite triggers when pdfs table changes (INSERT, UPDATE, DELETE).

#### `indexed_folders` table
```sql
CREATE TABLE indexed_folders (
    path TEXT PRIMARY KEY NOT NULL,
    last_indexed INTEGER NOT NULL   -- Unix timestamp
)
```

---

## Key Features Implementation

### Full-Text Search (FTS5)
- Boolean operators: AND, OR, NOT (case-insensitive)
- Snippet generation with highlighted matches
- Lucene-style indexing for fast queries

### Filters
- **Size**: Min/max file size in KB
- **Date Range**: From/to modification dates
- **Sort**: Relevance, date modified, file size, filename

### PDF Processing
- Recursive directory scanning
- Panic-safe text extraction (catches unsupported encodings)
- Page count estimation (~3000 chars per page)
- Re-indexing support (removes old entries before re-scanning)

### Privacy & Security
- All data stored locally in user's data directory
- No network calls or telemetry
- No external dependencies at runtime
- Sandboxed file system access via Tauri

---

## Build & Development

### Prerequisites
- Node.js v18+
- Rust (latest stable)
- npm or yarn

### Development Commands
```bash
# Install dependencies
npm install

# Run in development mode (Tauri + Vite)
npm run dev

# Run frontend only (browser mode)
npm run frontend:dev

# Build frontend assets
npm run frontend:build

# Build full application
npm run build
```

### Build Outputs
- **Frontend**: `dist/` directory (Vite build)
- **Binary**: `src-tauri/target/release/` (platform-specific)

### Platform Notes
- **Linux**: Requires webkit2gtk and other system dependencies
- **Windows/macOS**: Should work out of the box after `npm install`

---

## Code Conventions & Patterns

### Rust Backend

1. **Error Handling**:
   - Use `anyhow::Result` for functions that can fail
   - Convert errors to strings when crossing FFI boundary
   - Log warnings but continue indexing if individual PDFs fail

2. **Concurrency**:
   - Database wrapped in `Arc<Mutex<Connection>>` for thread safety
   - State management via `Mutex<Option<Database>>` in Tauri

3. **Tauri Commands**:
   - All commands async
   - Return `Result<T, String>` for proper error handling
   - Use `#[tauri::command]` macro

4. **Naming**:
   - Snake case for functions/variables
   - PascalCase for types/structs
   - Clear, descriptive names

### JavaScript Frontend

1. **Architecture**:
   - Event-driven with addEventListener
   - No frameworks or state management libraries
   - Direct DOM manipulation

2. **Async Operations**:
   - Use async/await with Tauri invoke
   - Show loading states during operations
   - Debounce search input (300ms)

3. **Error Handling**:
   - Try/catch blocks for all Tauri invokes
   - Display user-friendly error messages
   - Log errors to console

4. **Naming**:
   - camelCase for variables/functions
   - Descriptive element IDs matching functionality

### CSS

1. **Structure**:
   - CSS custom properties (variables) for theming
   - Mobile-first responsive design
   - BEM-like naming for components

2. **Colors**:
   - Modern blue/purple gradient theme
   - Good contrast ratios for accessibility
   - Consistent spacing scale

---

## Testing

**Current State**: No test infrastructure exists.

**If Tests Needed**:
- Rust: Use `cargo test` with `#[cfg(test)]` modules
- Frontend: Could add Vitest (already have Vite)
- Integration: Could use Tauri's testing utilities

---

## Important Gotchas & Known Issues

1. **PDF Extraction Panics**:
   - Some PDFs cause pdf-extract to panic (unsupported encodings)
   - Handled with `std::panic::catch_unwind()`
   - Returns empty string and logs warning

2. **File Path Handling**:
   - Use `to_string_lossy()` for non-UTF8 paths
   - Store absolute paths in database

3. **FTS5 Query Syntax**:
   - Boolean operators must be uppercase: AND, OR, NOT
   - Use double quotes for phrase search
   - Prefix with `-` to exclude terms

4. **Database Location**:
   - Stored in platform-specific data directory via `dirs::data_dir()`
   - Path: `{data_dir}/pdf-finder-pro/pdfs.db`

5. **Folder Re-indexing**:
   - Previous PDFs from folder removed before re-indexing
   - Prevents duplicates and stale entries

---

## Dependencies Management

### Adding Dependencies

**Frontend (npm)**:
```bash
npm install <package>
```

**Backend (Rust)**:
```bash
cd src-tauri
cargo add <crate>
```

### Current Dependencies (Critical)
- **Tauri**: Core framework, keep updated for security
- **rusqlite**: Database interface, bundled feature includes SQLite
- **pdf-extract**: Text extraction, limited alternatives
- **walkdir**: Robust recursive directory traversal

---

## Security Considerations

1. **Input Validation**:
   - Validate folder paths before indexing
   - Sanitize search queries for SQL injection (FTS5 handles this)

2. **File System Access**:
   - Sandboxed via Tauri's security model
   - User explicitly selects folders

3. **Data Privacy**:
   - No network access required
   - All processing happens locally
   - Database stored in user's private data directory

4. **Binary Security**:
   - Code signing recommended for distribution
   - Keep Tauri updated for security patches

---

## Future Enhancement Areas

Potential improvements if needed:
1. **Performance**: Parallel PDF processing, incremental indexing
2. **Features**: PDF preview, annotations, OCR for scanned PDFs
3. **UX**: Drag-and-drop folders, keyboard shortcuts, dark mode toggle
4. **Search**: Advanced query builder, saved searches, search history
5. **Indexing**: Background indexing, watch folders for changes

---

## Critical Context for Code Changes

When modifying this codebase:

1. **Minimize Changes**: This is a working, stable application. Make surgical edits only.

2. **Maintain Simplicity**: The vanilla JS approach is intentional. Don't introduce frameworks.

3. **Preserve Privacy**: Any network-related changes violate core principles.

4. **Test PDF Extraction**: Changes to indexer.rs require testing with various PDF types.

5. **Database Migrations**: Schema changes need migration logic and backward compatibility.

6. **Cross-Platform**: Test on multiple platforms if changing native integrations.

7. **Error Handling**: Never let errors crash the app. Log and continue gracefully.

---

## Quick Reference

### File Purpose Quick Lookup
- **main.js**: UI event handlers, Tauri API calls, search/filter logic
- **lib.rs**: Tauri command definitions, app state management
- **database.rs**: SQLite operations, FTS5 queries, schema management
- **indexer.rs**: PDF discovery, text extraction, metadata collection
- **styles.css**: All styling, responsive design, theme colors
- **index.html**: DOM structure, UI layout

### Common Tasks
- **Add new Tauri command**: Define in `lib.rs`, call from `main.js`
- **Modify search**: Update `database.rs` query logic
- **Change UI**: Edit `index.html` + `styles.css` + event handlers in `main.js`
- **Adjust indexing**: Modify `indexer.rs` logic

---

## Version Information

- **Project Version**: 0.1.0
- **Tauri**: 2.1
- **Rust Edition**: 2021
- **Node Target**: v18+
- **Browser Target**: ES2021, Chrome 100, Safari 13

---

## Contact & Resources

- **Repository**: https://github.com/av/pdf-finder-pro
- **Tauri Docs**: tauri.app
- **SQLite FTS5 Docs**: sqlite.org/fts5.html
- **pdf-extract Crate**: docs.rs/pdf-extract

---

**Remember**: Think deeply, plan thoroughly, implement simply. Measure seven times, cut once.
