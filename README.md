# PDF Finder Pro

A cross-platform desktop application built with Tauri and Rust for fast, private full-text PDF search.

## Features

- 🔍 **Full-Text Search**: Fast text search across all indexed PDFs
- 🔐 **Private & Offline**: All indexing and searching happens locally on your machine
- 📁 **Recursive Scanning**: Automatically scans folders and subfolders for PDFs
- 🎯 **Advanced Filtering**: Filter by file size, modification date, and more
- 🔤 **Boolean Operators**: Case-insensitive AND, OR, NOT operators for custom queries
- 📊 **Metadata Extraction**: Displays file size, modification date, and page count
- ⚡ **Real-Time Results**: Searches update as you type for instant feedback
- 🗄️ **Local Indexing**: Lightweight SQLite FTS5 (Lucene-style) index with minimal resource usage
- 🖥️ **Cross-Platform**: Works on Windows, macOS, and Linux

## Technology Stack

- **Frontend**: Vanilla JavaScript, HTML5, CSS3
- **Backend**: Rust with Tauri 2.0
- **Indexing**: SQLite with FTS5 (Full-Text Search)
- **PDF Processing**: pdf-extract library for text extraction

## Development

### Prerequisites

- Node.js (v18 or later)
- Rust (latest stable)
- npm or yarn

### Installation

1. Clone the repository:
```bash
git clone https://github.com/av/pdf-finder-pro.git
cd pdf-finder-pro
```

2. Install dependencies:
```bash
npm install
```

3. Run in development mode:
```bash
npm run dev
```

To run just the frontend (in a browser):
```bash
npm run frontend:dev
```

### Building

Build the application for your platform:
```bash
npm run build
```

To build just the frontend assets:
```bash
npm run frontend:build
```

The built application will be available in `src-tauri/target/release`.

## Usage

1. **Select Folder**: Click "Select Folder to Index" and choose a directory containing PDFs
2. **Index PDFs**: Click "Index PDFs" to scan and index all PDF files recursively
3. **Search**: Enter search terms in the search box
   - Use boolean operators: `rust AND programming`
   - Combine terms: `machine OR learning`
   - Exclude terms: `python NOT django`
4. **Filter Results**: Use the filter options to narrow down results by size and date
5. **Open PDF**: Click on any result to open the PDF in your default viewer

## Features in Detail

### Full-Text Search
All PDF text content is extracted and indexed using SQLite's FTS5 engine, enabling fast and efficient searches even with large document collections.

### Boolean Operators
- **AND**: Find documents containing all terms
- **OR**: Find documents containing any of the terms
- **NOT**: Exclude documents containing specific terms

### Filters
- **Min/Max Size**: Filter by file size in KB
- **Date Range**: Filter by modification date (from/to)

### Privacy
All data stays on your local machine. No data is sent to external servers.

## License

MIT

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
