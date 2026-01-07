# PDF Finder Pro

A cross-platform desktop application built with Tauri and Rust for fast, private full-text PDF search.

## 💳 Pricing

**$5 USD** - One-time payment, lifetime license
- ✅ 14-day free trial with full features
- ✅ Works on unlimited devices
- ✅ All future updates included
- ✅ Completely offline after activation
- ✅ No subscription, no recurring fees

**[Buy PDF Finder Pro →](https://lemonsqueezy.com/checkout/pdf-finder-pro)**

*See [License Activation Guide](specs/features/2026-01-lemon-squeezy-payment/USER_GUIDE.md) for details*

## Recent Improvements (v0.2.0)

🚀 **Major performance and quality improvements** have been implemented:
- **5-200x faster indexing** through parallel processing and incremental updates
- **Better search results** with BM25 ranking and Porter stemming
- **Enhanced reliability** with comprehensive error handling
- **Improved search quality** with diacritics normalization

See [specs/performance/2026-01-indexing-optimizations/](specs/performance/2026-01-indexing-optimizations/) for complete technical details.

## Features

- 🔍 **Full-Text Search**: Fast text search across all indexed PDFs with BM25 ranking
- 🔐 **Private & Offline**: All indexing and searching happens locally on your machine
- 📁 **Recursive Scanning**: Automatically scans folders and subfolders for PDFs
- ⚡ **Parallel Indexing**: Utilizes all CPU cores for fast indexing
- 🔄 **Incremental Updates**: Only re-processes changed files
- 🎯 **Advanced Filtering**: Filter by file size, modification date, and more
- 🔤 **Boolean Operators**: Case-insensitive AND, OR, NOT operators for custom queries
- 📊 **Metadata Extraction**: Displays file size, modification date, and page count
- 🗄️ **Local Indexing**: Optimized SQLite FTS5 with WAL mode and Porter stemming
- 🖥️ **Cross-Platform**: Works on Windows, macOS, and Linux

## Technology Stack

- **Frontend**: Vanilla JavaScript, HTML5, CSS3
- **Backend**: Rust with Tauri 2.0
- **Indexing**: SQLite with FTS5 (Full-Text Search), WAL mode, BM25 ranking
- **PDF Processing**: pdf-extract library with parallel processing (Rayon)
- **Performance**: Parallel PDF extraction, batch operations, incremental indexing

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

**Commercial Software** - $5 one-time purchase required after 14-day trial.

For licensing details, see [License Activation Guide](specs/features/2026-01-lemon-squeezy-payment/USER_GUIDE.md).

Code is available for review but not for redistribution without a license.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
