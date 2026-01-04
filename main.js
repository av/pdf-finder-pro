import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';

let searchTimeout;
let currentResults = [];

// DOM Elements
const guideSection = document.getElementById('guide-section');
const toggleGuideBtn = document.getElementById('toggle-guide');
const guideContent = document.getElementById('guide-content');
const addFolderBtn = document.getElementById('add-folder');
const foldersList = document.getElementById('folders-list');
const searchInput = document.getElementById('search-input');
const searchBtn = document.getElementById('search-btn');
const resultsContainer = document.getElementById('results-container');
const resultsCount = document.getElementById('results-count');
const clearFiltersBtn = document.getElementById('clear-filters');
const toggleFiltersBtn = document.getElementById('toggle-filters');
const filtersPanel = document.getElementById('filters-panel');
const filterIcon = document.getElementById('filter-icon');
const sortBySelect = document.getElementById('sort-by');

// Filters
const minSizeInput = document.getElementById('min-size');
const maxSizeInput = document.getElementById('max-size');
const dateFromInput = document.getElementById('date-from');
const dateToInput = document.getElementById('date-to');

// Toggle guide visibility
toggleGuideBtn.addEventListener('click', () => {
  if (guideContent.style.display === 'none') {
    guideContent.style.display = 'block';
    toggleGuideBtn.textContent = 'Hide';
  } else {
    guideContent.style.display = 'none';
    toggleGuideBtn.textContent = 'Show';
  }
});

// Toggle filters visibility
toggleFiltersBtn.addEventListener('click', () => {
  if (filtersPanel.style.display === 'none') {
    filtersPanel.style.display = 'flex';
    filterIcon.classList.add('open');
  } else {
    filtersPanel.style.display = 'none';
    filterIcon.classList.remove('open');
  }
});

// Add folder for indexing
addFolderBtn.addEventListener('click', async () => {
  try {
    const selected = await open({
      directory: true,
      multiple: false,
      title: 'Select folder to index PDFs'
    });

    if (selected) {
      await indexFolder(selected);
      await loadIndexedFolders();
    }
  } catch (error) {
    console.error('Error selecting folder:', error);
    showError('Failed to select folder');
  }
});

// Index a folder
async function indexFolder(folderPath) {
  const loadingMsg = document.createElement('div');
  loadingMsg.className = 'empty-state';
  loadingMsg.innerHTML = '<p>⏳ Indexing PDFs...</p>';
  foldersList.innerHTML = '';
  foldersList.appendChild(loadingMsg);

  try {
    const result = await invoke('index_pdfs', { folderPath });
    
    // Show success message
    const successMsg = document.createElement('div');
    successMsg.style.cssText = 'padding: 1rem; background: #d1fae5; color: #065f46; border-radius: 6px; margin-bottom: 1rem;';
    successMsg.textContent = `✅ Indexed ${result.count} PDFs in ${result.duration}ms`;
    foldersList.insertBefore(successMsg, foldersList.firstChild);
    
    setTimeout(() => successMsg.remove(), 3000);
  } catch (error) {
    console.error('Error indexing PDFs:', error);
    showError(`Indexing failed: ${error}`);
  }
}

// Load and display indexed folders
async function loadIndexedFolders() {
  try {
    const folders = await invoke('get_indexed_folders');
    
    if (!folders || folders.length === 0) {
      foldersList.innerHTML = '<div class="empty-state"><p>No folders indexed yet. Click "Add Folder" to get started.</p></div>';
      return;
    }

    foldersList.innerHTML = folders.map(folder => `
      <div class="folder-item" data-path="${escapeHtml(folder.path)}">
        <div class="folder-info">
          <div class="folder-path-text">${escapeHtml(folder.path)}</div>
          <div class="folder-meta">
            <span>📄 ${folder.pdf_count} PDFs</span>
            <span>🕒 Last indexed: ${formatTimestamp(folder.last_indexed)}</span>
          </div>
        </div>
        <div class="folder-actions">
          <button class="icon-btn refresh" onclick="window.__reindexFolder('${escapePath(folder.path)}')" title="Re-index this folder">
            🔄
          </button>
          <button class="icon-btn delete" onclick="window.__removeFolder('${escapePath(folder.path)}')" title="Remove this folder">
            🗑️
          </button>
        </div>
      </div>
    `).join('');
  } catch (error) {
    console.error('Error loading folders:', error);
    foldersList.innerHTML = '<div class="empty-state"><p>Error loading folders</p></div>';
  }
}

// Re-index a folder
window.__reindexFolder = async (path) => {
  if (!confirm(`Re-index folder: ${path}?`)) return;
  
  await indexFolder(path);
  await loadIndexedFolders();
};

// Remove a folder
window.__removeFolder = async (path) => {
  if (!confirm(`Remove folder and all its indexed PDFs: ${path}?`)) return;
  
  try {
    await invoke('remove_indexed_folder', { folderPath: path });
    await loadIndexedFolders();
    
    // Clear results if they were from this folder
    if (currentResults.length > 0) {
      performSearch();
    }
  } catch (error) {
    console.error('Error removing folder:', error);
    showError(`Failed to remove folder: ${error}`);
  }
};

// Search functionality
async function performSearch() {
  const query = searchInput.value.trim();
  if (!query) {
    showEmptyState('Enter a search query to find PDFs');
    currentResults = [];
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
    currentResults = results;
    displayResults(results);
  } catch (error) {
    console.error('Error searching:', error);
    showError(`Search failed: ${error}`);
    currentResults = [];
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

// Sort results
sortBySelect.addEventListener('change', () => {
  if (currentResults.length > 0) {
    displayResults(currentResults);
  }
});

// Clear filters
clearFiltersBtn.addEventListener('click', () => {
  minSizeInput.value = '';
  maxSizeInput.value = '';
  dateFromInput.value = '';
  dateToInput.value = '';
  if (searchInput.value.trim()) {
    performSearch();
  }
});

// Display results with folder grouping and sorting
function displayResults(results) {
  if (!results || results.length === 0) {
    showEmptyState('No results found');
    return;
  }

  // Sort results based on selection
  const sortBy = sortBySelect.value;
  let sortedResults = [...results];
  
  switch (sortBy) {
    case 'date-desc':
      sortedResults.sort((a, b) => b.modified - a.modified);
      break;
    case 'date-asc':
      sortedResults.sort((a, b) => a.modified - b.modified);
      break;
    case 'size-desc':
      sortedResults.sort((a, b) => b.size - a.size);
      break;
    case 'size-asc':
      sortedResults.sort((a, b) => a.size - b.size);
      break;
    // 'relevance' is default ordering from FTS5
  }

  resultsCount.textContent = `${sortedResults.length} result${sortedResults.length !== 1 ? 's' : ''}`;

  // Group results by folder
  const groupedResults = groupByFolder(sortedResults);
  
  resultsContainer.innerHTML = Object.entries(groupedResults).map(([folder, items]) => {
    const folderId = btoa(folder).replace(/[^a-zA-Z0-9]/g, '');
    return `
      <div class="folder-group">
        <div class="folder-group-header" onclick="window.__toggleFolderGroup('${folderId}')">
          <span class="folder-group-toggle" id="toggle-${folderId}">▼</span>
          <span class="folder-group-title">📁 ${escapeHtml(getFolderName(folder))}</span>
          <span class="folder-group-count">${items.length} result${items.length !== 1 ? 's' : ''}</span>
        </div>
        <div class="folder-group-results" id="results-${folderId}">
          ${items.map(result => renderResultItem(result)).join('')}
        </div>
      </div>
    `;
  }).join('');
}

// Group results by parent folder
function groupByFolder(results) {
  const grouped = {};
  
  results.forEach(result => {
    // Extract folder path (everything except filename)
    const pathParts = result.path.split(/[\\/]/);
    const folder = pathParts.slice(0, -1).join('/') || '/';
    
    if (!grouped[folder]) {
      grouped[folder] = [];
    }
    grouped[folder].push(result);
  });
  
  return grouped;
}

// Get folder name from path
function getFolderName(path) {
  if (path === '/') return 'Root';
  const parts = path.split(/[\\/]/);
  return parts[parts.length - 1] || path;
}

// Toggle folder group expansion
window.__toggleFolderGroup = (folderId) => {
  const resultsDiv = document.getElementById(`results-${folderId}`);
  const toggleIcon = document.getElementById(`toggle-${folderId}`);
  
  if (resultsDiv.classList.contains('collapsed')) {
    resultsDiv.classList.remove('collapsed');
    toggleIcon.classList.remove('collapsed');
  } else {
    resultsDiv.classList.add('collapsed');
    toggleIcon.classList.add('collapsed');
  }
};

// Render a single result item
function renderResultItem(result) {
  return `
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
  `;
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

function formatTimestamp(timestamp) {
  const date = new Date(timestamp * 1000);
  const now = new Date();
  const diffMs = now - date;
  const diffMins = Math.floor(diffMs / 60000);
  const diffHours = Math.floor(diffMs / 3600000);
  const diffDays = Math.floor(diffMs / 86400000);

  if (diffMins < 1) return 'Just now';
  if (diffMins < 60) return `${diffMins} minute${diffMins !== 1 ? 's' : ''} ago`;
  if (diffHours < 24) return `${diffHours} hour${diffHours !== 1 ? 's' : ''} ago`;
  if (diffDays < 7) return `${diffDays} day${diffDays !== 1 ? 's' : ''} ago`;
  
  return date.toLocaleDateString();
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
async function init() {
  await loadIndexedFolders();
  
  const count = await invoke('get_index_stats').catch(() => 0);
  if (count > 0) {
    showEmptyState(`${count} PDFs indexed. Enter a search query to find files.`);
    // Hide guide if already has indexed PDFs
    guideContent.style.display = 'none';
    toggleGuideBtn.textContent = 'Show';
  } else {
    showEmptyState('Add a folder to start indexing PDFs');
  }
}

init();
