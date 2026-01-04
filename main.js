import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';

let selectedFolder = '';
let isIndexing = false;
let searchTimeout;

// DOM Elements
const selectFolderBtn = document.getElementById('select-folder');
const selectedFolderSpan = document.getElementById('selected-folder');
const indexPdfsBtn = document.getElementById('index-pdfs');
const indexStatus = document.getElementById('index-status');
const searchInput = document.getElementById('search-input');
const searchBtn = document.getElementById('search-btn');
const resultsContainer = document.getElementById('results-container');
const resultsCount = document.getElementById('results-count');
const clearFiltersBtn = document.getElementById('clear-filters');

// Filters
const minSizeInput = document.getElementById('min-size');
const maxSizeInput = document.getElementById('max-size');
const dateFromInput = document.getElementById('date-from');
const dateToInput = document.getElementById('date-to');

// Select folder for indexing
selectFolderBtn.addEventListener('click', async () => {
  try {
    const selected = await open({
      directory: true,
      multiple: false,
      title: 'Select folder to index PDFs'
    });
    
    if (selected) {
      selectedFolder = selected;
      selectedFolderSpan.textContent = selectedFolder;
      indexPdfsBtn.disabled = false;
    }
  } catch (error) {
    console.error('Error selecting folder:', error);
    showError('Failed to select folder');
  }
});

// Index PDFs
indexPdfsBtn.addEventListener('click', async () => {
  if (!selectedFolder || isIndexing) return;
  
  isIndexing = true;
  indexPdfsBtn.disabled = true;
  indexStatus.textContent = 'Indexing...';
  
  try {
    const result = await invoke('index_pdfs', { folderPath: selectedFolder });
    indexStatus.textContent = `Indexed ${result.count} PDFs in ${result.duration}ms`;
    isIndexing = false;
    indexPdfsBtn.disabled = false;
  } catch (error) {
    console.error('Error indexing PDFs:', error);
    indexStatus.textContent = 'Error indexing PDFs';
    showError(`Indexing failed: ${error}`);
    isIndexing = false;
    indexPdfsBtn.disabled = false;
  }
});

// Search functionality
async function performSearch() {
  const query = searchInput.value.trim();
  if (!query) {
    showEmptyState('Enter a search query to find PDFs');
    return;
  }
  
  resultsContainer.innerHTML = '<div class="empty-state"><p>Searching...</p></div>';
  
  try {
    const filters = {
      min_size: minSizeInput.value ? parseInt(minSizeInput.value) * 1024 : null,
      max_size: maxSizeInput.value ? parseInt(maxSizeInput.value) * 1024 : null,
      date_from: dateFromInput.value || null,
      date_to: dateToInput.value || null,
    };
    
    const results = await invoke('search_pdfs', { query, filters });
    displayResults(results);
  } catch (error) {
    console.error('Error searching:', error);
    showError(`Search failed: ${error}`);
  }
}

searchBtn.addEventListener('click', performSearch);

searchInput.addEventListener('keypress', (e) => {
  if (e.key === 'Enter') {
    performSearch();
  }
});

searchInput.addEventListener('input', () => {
  clearTimeout(searchTimeout);
  searchTimeout = setTimeout(() => performSearch(), 250);
});

// Clear filters
clearFiltersBtn.addEventListener('click', () => {
  minSizeInput.value = '';
  maxSizeInput.value = '';
  dateFromInput.value = '';
  dateToInput.value = '';
});

// Display results
function displayResults(results) {
  if (!results || results.length === 0) {
    showEmptyState('No results found');
    return;
  }
  
  resultsCount.textContent = `${results.length} result${results.length !== 1 ? 's' : ''}`;
  
  resultsContainer.innerHTML = results.map(result => `
    <div class="result-item" onclick="window.__openPdf('${escapePath(result.path)}')">
      <div class="result-title">${escapeHtml(result.title || getFileName(result.path))}</div>
      <div class="result-path">${escapeHtml(result.path)}</div>
      <div class="result-metadata">
        <span>📄 ${formatFileSize(result.size)}</span>
        <span>📅 Modified: ${formatDate(result.modified)}</span>
        ${result.pages ? `<span>📖 ${result.pages} pages</span>` : ''}
      </div>
      ${result.snippet ? `<div class="result-snippet">${highlightSnippet(result.snippet, searchInput.value)}</div>` : ''}
    </div>
  `).join('');
}

// Open PDF file
window.__openPdf = async (path) => {
  try {
    await invoke('open_pdf', { path });
  } catch (error) {
    console.error('Error opening PDF:', error);
    showError(`Failed to open PDF: ${error}`);
  }
};

// Utility functions
function showEmptyState(message) {
  resultsContainer.innerHTML = `<div class="empty-state"><p>${escapeHtml(message)}</p></div>`;
  resultsCount.textContent = '';
}

function showError(message) {
  resultsContainer.innerHTML = `<div class="empty-state" style="color: #ef4444;"><p>❌ ${escapeHtml(message)}</p></div>`;
  resultsCount.textContent = '';
}

function escapeHtml(text) {
  const div = document.createElement('div');
  div.textContent = text;
  return div.innerHTML;
}

function escapePath(path) {
  return path.replace(/'/g, "\\'");
}

function getFileName(path) {
  return path.split(/[\\/]/).pop();
}

function formatFileSize(bytes) {
  if (bytes < 1024) return bytes + ' B';
  if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(2) + ' KB';
  return (bytes / (1024 * 1024)).toFixed(2) + ' MB';
}

function formatDate(timestamp) {
  const date = new Date(timestamp * 1000);
  return date.toLocaleDateString() + ' ' + date.toLocaleTimeString();
}

function highlightSnippet(snippet, query) {
  if (!query) return escapeHtml(snippet);
  
  // Extract search terms (simple parsing, ignoring operators for highlighting)
  const terms = query.split(/\s+/).filter(t => 
    !['AND', 'OR', 'NOT'].includes(t.toUpperCase())
  );
  
  let highlighted = escapeHtml(snippet);
  terms.forEach(term => {
    const regex = new RegExp(`(${term})`, 'gi');
    highlighted = highlighted.replace(regex, '<span class="highlight">$1</span>');
  });
  
  return highlighted;
}

// Initialize
showEmptyState('Select a folder to index PDFs, then search for files');
