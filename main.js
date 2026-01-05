import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import { createIcons, icons } from 'lucide';

let searchTimeout;
let currentResults = [];

// DOM Elements
const sidebar = document.getElementById('sidebar');
const toggleSidebarBtn = document.getElementById('toggle-sidebar');
const showHelpBtn = document.getElementById('show-help');
const closeHelpBtn = document.getElementById('close-help');
const helpModal = document.getElementById('help-modal');
const addFolderBtn = document.getElementById('add-folder');
const foldersList = document.getElementById('folders-list');
const searchInput = document.getElementById('search-input');
const clearSearchBtn = document.getElementById('clear-search');
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

// Sidebar toggle
toggleSidebarBtn.addEventListener('click', () => {
  sidebar.classList.toggle('collapsed');
  const icon = toggleSidebarBtn.querySelector('i');
  icon.setAttribute('data-lucide', sidebar.classList.contains('collapsed') ? 'panel-left-open' : 'panel-left-close');
  createIcons({ icons });
});

// Help modal
showHelpBtn.addEventListener('click', () => {
  helpModal.style.display = 'flex';
  createIcons({ icons });
});

closeHelpBtn.addEventListener('click', () => {
  helpModal.style.display = 'none';
});

helpModal.addEventListener('click', (e) => {
  if (e.target === helpModal) {
    helpModal.style.display = 'none';
  }
});

// Show/hide clear button in search input
searchInput.addEventListener('input', () => {
  clearSearchBtn.style.display = searchInput.value ? 'flex' : 'none';
});

// Clear search input
clearSearchBtn.addEventListener('click', () => {
  searchInput.value = '';
  clearSearchBtn.style.display = 'none';
  searchInput.focus();
  showEmptyState('default');
  currentResults = [];
  resultsCount.textContent = '';
});

// Keyboard shortcuts
const shortcuts = {
  'k': (e) => {
    if (e.metaKey || e.ctrlKey) {
      e.preventDefault();
      searchInput.focus();
      searchInput.select();
    }
  },
  'Escape': () => {
    if (helpModal.style.display === 'flex') {
      helpModal.style.display = 'none';
    } else if (searchInput.value) {
      searchInput.value = '';
      clearSearchBtn.style.display = 'none';
      showEmptyState('default');
      currentResults = [];
      resultsCount.textContent = '';
    } else if (document.activeElement === searchInput) {
      searchInput.blur();
    }
  },
  '/': (e) => {
    if (document.activeElement.tagName !== 'INPUT') {
      e.preventDefault();
      searchInput.focus();
    }
  },
  '?': (e) => {
    if (document.activeElement.tagName !== 'INPUT') {
      e.preventDefault();
      helpModal.style.display = 'flex';
      createIcons({ icons });
    }
  }
};

// Register shortcuts
document.addEventListener('keydown', (e) => {
  const handler = shortcuts[e.key];
  if (handler) handler(e);
});

// Toggle filters visibility
toggleFiltersBtn.addEventListener('click', () => {
  if (filtersPanel.style.display === 'none') {
    filtersPanel.style.display = 'flex';
    toggleFiltersBtn.classList.add('active');
    filterIcon.setAttribute('data-lucide', 'sliders-horizontal');
  } else {
    filtersPanel.style.display = 'none';
    toggleFiltersBtn.classList.remove('active');
    filterIcon.setAttribute('data-lucide', 'sliders-horizontal');
  }
  createIcons({ icons });
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
  loadingMsg.className = 'empty-state-small';
  loadingMsg.innerHTML = '<p><i data-lucide="loader-2" class="loading-icon"></i> Indexing...</p>';
  foldersList.innerHTML = '';
  foldersList.appendChild(loadingMsg);
  createIcons({ icons });

  try {
    const result = await invoke('index_pdfs', { folderPath });
    
    // Show success toast
    showToast(`Indexed ${result.count} PDFs in ${result.duration}ms`, 'success');
  } catch (error) {
    console.error('Error indexing PDFs:', error);
    showToast(`Indexing failed: ${error}`, 'error');
  }
}

// Load and display indexed folders
async function loadIndexedFolders() {
  try {
    const folders = await invoke('get_indexed_folders');
    
    if (!folders || folders.length === 0) {
      foldersList.innerHTML = '<div class="empty-state-small"><p>No folders yet</p></div>';
      return;
    }

    foldersList.innerHTML = folders.map(folder => {
      const folderName = folder.path.split(/[\\/]/).pop() || folder.path;
      return `
        <div class="folder-item" data-path="${escapeHtml(folder.path)}" title="${escapeHtml(folder.path)}">
          <div class="folder-info">
            <div class="folder-path-text">${escapeHtml(folderName)}</div>
            <div class="folder-meta">
              <span><i data-lucide="file-text" class="meta-icon"></i> ${folder.pdf_count} PDFs</span>
              <span><i data-lucide="clock" class="meta-icon"></i> ${formatTimestamp(folder.last_indexed)}</span>
            </div>
          </div>
          <div class="folder-actions">
            <button class="icon-btn refresh" onclick="window.__reindexFolder('${escapePath(folder.path)}')" title="Re-index">
              <i data-lucide="refresh-cw"></i>
            </button>
            <button class="icon-btn delete" onclick="window.__removeFolder('${escapePath(folder.path)}')" title="Remove">
              <i data-lucide="trash-2"></i>
            </button>
          </div>
        </div>
      `;
    }).join('');
    createIcons({ icons });
  } catch (error) {
    console.error('Error loading folders:', error);
    foldersList.innerHTML = '<div class="empty-state-small"><p>Error loading</p></div>';
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
    showToast('Folder removed successfully', 'success');
    await loadIndexedFolders();
    
    // Clear results if they were from this folder
    if (currentResults.length > 0) {
      performSearch();
    }
  } catch (error) {
    console.error('Error removing folder:', error);
    showToast(`Failed to remove folder: ${error}`, 'error');
  }
};

// Search functionality
async function performSearch() {
  const query = searchInput.value.trim();
  if (!query) {
    showEmptyState('default');
    currentResults = [];
    return;
  }

  showSkeletonLoader();

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

// Enter key triggers immediate search
searchInput.addEventListener('keypress', (e) => {
  if (e.key === 'Enter') {
    clearTimeout(searchTimeout);
    performSearch();
  }
});

// Auto-search with debounce
searchInput.addEventListener('input', () => {
  clearTimeout(searchTimeout);
  searchTimeout = setTimeout(() => performSearch(), 300);
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
    showEmptyState('noResults');
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
          <i data-lucide="chevron-down" class="folder-group-toggle" id="toggle-${folderId}"></i>
          <span class="folder-group-title"><i data-lucide="folder" class="inline-icon"></i> ${escapeHtml(getFolderName(folder))}</span>
          <span class="folder-group-count">${items.length} result${items.length !== 1 ? 's' : ''}</span>
        </div>
        <div class="folder-group-results" id="results-${folderId}">
          ${items.map(result => renderResultItem(result)).join('')}
        </div>
      </div>
    `;
  }).join('');
  createIcons({ icons });
}

function showSkeletonLoader(count = 3) {
  const skeletons = Array.from({ length: count }, () => `
    <div class="skeleton-item">
      <div class="skeleton-title"></div>
      <div class="skeleton-text"></div>
      <div class="skeleton-text short"></div>
    </div>
  `).join('');
  
  resultsContainer.innerHTML = `<div class="skeleton-loader">${skeletons}</div>`;
  resultsCount.textContent = '';
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
    toggleIcon.setAttribute('data-lucide', 'chevron-down');
  } else {
    resultsDiv.classList.add('collapsed');
    toggleIcon.setAttribute('data-lucide', 'chevron-right');
  }
  createIcons({ icons });
};

// Render a single result item
function renderResultItem(result) {
  return `
    <div class="result-item" onclick="window.__openPdf('${escapePath(result.path)}')">
      <div class="result-title">${escapeHtml(result.title || getFileName(result.path))}</div>
      <div class="result-path">${escapeHtml(result.path)}</div>
      <div class="result-metadata">
        <span><i data-lucide="file-text" class="meta-icon"></i> ${formatFileSize(result.size)}</span>
        <span><i data-lucide="calendar" class="meta-icon"></i> Modified: ${formatDate(result.modified)}</span>
        ${result.pages ? `<span><i data-lucide="book-open" class="meta-icon"></i> ${result.pages} pages</span>` : ''}
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
function showEmptyState(type = 'default') {
  const states = {
    default: {
      icon: 'search',
      title: 'Ready to find your PDFs?',
      message: 'Try searching for a topic, keyword, or phrase.',
      action: null
    },
    noResults: {
      icon: 'search-x',
      title: 'No results found',
      message: 'We searched everywhere but couldn\'t find that. Try different keywords or check your filters.',
      action: null
    },
    noFolders: {
      icon: 'folder-plus',
      title: 'No folders indexed yet',
      message: 'Add a folder containing PDFs to start building your searchable library.',
      action: {
        text: 'Add Folder',
        onclick: 'document.getElementById("add-folder").click()'
      }
    },
    indexing: {
      icon: 'loader-2',
      title: 'Indexing your PDFs...',
      message: 'This might take a moment. You can search as soon as we\'re done.',
      action: null,
      iconClass: 'loading-icon'
    },
    searching: {
      icon: 'loader-2',
      title: 'Searching...',
      message: '',
      action: null,
      iconClass: 'loading-icon'
    }
  };

  const state = states[type] || states.default;
  
  resultsContainer.innerHTML = `
    <div class="empty-state">
      <i data-lucide="${state.icon}" class="empty-state-icon ${state.iconClass || ''}"></i>
      <h3 class="empty-state-title">${state.title}</h3>
      ${state.message ? `<p class="empty-state-message">${state.message}</p>` : ''}
      ${state.action ? `
        <button class="btn btn-primary" onclick="${state.action.onclick}">
          ${state.action.text}
        </button>
      ` : ''}
    </div>
  `;
  
  resultsCount.textContent = '';
  createIcons({ icons });
}

function showError(message) {
  showToast(message, 'error');
  resultsContainer.innerHTML = `<div class="empty-state" style="color: #ef4444;"><p><i data-lucide="x-circle" class="error-icon"></i> ${escapeHtml(message)}</p></div>`;
  resultsCount.textContent = '';
  createIcons({ icons });
}

// Toast notification system
function showToast(message, type = 'info', duration = 5000) {
  const icons = {
    success: 'check-circle-2',
    error: 'x-circle',
    warning: 'alert-circle',
    info: 'info'
  };

  const container = document.getElementById('toast-container');
  const toast = document.createElement('div');
  toast.className = `toast ${type}`;
  
  toast.innerHTML = `
    <i data-lucide="${icons[type]}" class="toast-icon"></i>
    <div class="toast-content">
      <div class="toast-message">${escapeHtml(message)}</div>
    </div>
    <button class="toast-close">
      <i data-lucide="x" style="width: 16px; height: 16px;"></i>
    </button>
  `;
  
  container.appendChild(toast);
  createIcons({ icons: icons });
  
  // Close button
  toast.querySelector('.toast-close').addEventListener('click', () => {
    removeToast(toast);
  });
  
  // Auto-dismiss
  if (duration > 0) {
    setTimeout(() => removeToast(toast), duration);
  }
  
  return toast;
}

function removeToast(toast) {
  toast.style.animation = 'slideOut 200ms ease-out';
  setTimeout(() => toast.remove(), 200);
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
    showEmptyState('default');
  } else {
    showEmptyState('noFolders');
  }
  
  createIcons({ icons });
}

init();
