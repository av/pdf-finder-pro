import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import { createIcons, icons } from 'lucide';
import { initLicenseUI, shouldLimitResults, showLicenseActivationDialog } from './license-ui.js';

let searchTimeout;
let currentResults = [];

// DOM Elements
const sidebar = document.getElementById('sidebar');
const toggleSidebarBtn = document.getElementById('toggle-sidebar');
const openSidebarBtn = document.getElementById('open-sidebar');
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
  if (icon) {
    icon.setAttribute('data-lucide', sidebar.classList.contains('collapsed') ? 'panel-left-open' : 'panel-left-close');
    createIcons({ icons });
  }
});

if (openSidebarBtn) {
  openSidebarBtn.addEventListener('click', () => {
    sidebar.classList.remove('collapsed');
    // Sync the icon of the main toggle button
    const icon = toggleSidebarBtn.querySelector('i');
    if (icon) {
      icon.setAttribute('data-lucide', 'panel-left-close');
      createIcons({ icons });
    }
  });
}

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
    toggleFiltersBtn.setAttribute('aria-expanded', 'true');
    filterIcon.setAttribute('data-lucide', 'sliders-horizontal');
  } else {
    filtersPanel.style.display = 'none';
    toggleFiltersBtn.classList.remove('active');
    toggleFiltersBtn.setAttribute('aria-expanded', 'false');
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
async function indexFolder(folderPath, isReindex = false) {
  // Show loading state
  if (!isReindex) {
    const loadingMsg = document.createElement('div');
    loadingMsg.className = 'empty-state-small';
    loadingMsg.innerHTML = '<p><i data-lucide="loader-2" class="loading-icon"></i> Indexing...</p>';
    foldersList.innerHTML = '';
    foldersList.appendChild(loadingMsg);
    createIcons({ icons });
  }

  try {
    const result = await invoke('index_pdfs', { folderPath });

    // Show success toast
    showToast(`Indexed ${result.count} PDFs in ${result.duration}ms`, 'success');
  } catch (error) {
    console.error('Error indexing PDFs:', error);
    showToast('Indexing failed. Please check the folder and try again.', 'error');
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

    foldersList.innerHTML = '';

    folders.forEach(folder => {
      const folderName = folder.path.split(/[\\/]/).pop() || folder.path;

      const folderItem = document.createElement('div');
      folderItem.className = 'folder-item';
      folderItem.setAttribute('data-path', folder.path);
      folderItem.setAttribute('role', 'listitem');
      folderItem.title = folder.path;

      folderItem.innerHTML = `
        <div class="folder-info">
          <div class="folder-path-text">${escapeHtml(folderName)}</div>
          <div class="folder-meta">
            <span><i data-lucide="file-text" class="meta-icon"></i> ${folder.pdf_count} PDFs</span>
            <span><i data-lucide="clock" class="meta-icon"></i> ${formatTimestamp(folder.last_indexed)}</span>
          </div>
        </div>
        <div class="folder-actions">
          <button class="icon-btn refresh" title="Re-index" aria-label="Re-index folder ${escapeHtml(folderName)}">
            <i data-lucide="refresh-cw"></i>
          </button>
          <button class="icon-btn delete" title="Remove" aria-label="Remove folder ${escapeHtml(folderName)}">
            <i data-lucide="trash-2"></i>
          </button>
        </div>
      `;

      // Add event listeners instead of inline onclick
      const refreshBtn = folderItem.querySelector('.refresh');
      const deleteBtn = folderItem.querySelector('.delete');

      refreshBtn.addEventListener('click', async (e) => {
        e.stopPropagation();
        if (confirm(`Re-index folder: ${folder.path}?`)) {
          // Disable button and show loading state
          refreshBtn.disabled = true;
          refreshBtn.innerHTML = '<i data-lucide="loader-2" class="loading-icon"></i>';
          createIcons({ icons });

          await indexFolder(folder.path, true);
          await loadIndexedFolders();
        }
      });

      deleteBtn.addEventListener('click', async (e) => {
        e.stopPropagation();
        if (confirm(`Remove folder and all its indexed PDFs: ${folder.path}?`)) {
          try {
            await invoke('remove_indexed_folder', { folderPath: folder.path });
            showToast('Folder removed successfully', 'success');
            await loadIndexedFolders();

            if (currentResults.length > 0) {
              performSearch();
            }
          } catch (error) {
            console.error('Error removing folder:', error);
            showToast('Failed to remove folder. Please try again.', 'error');
          }
        }
      });

      foldersList.appendChild(folderItem);
    });

    createIcons({ icons });
  } catch (error) {
    console.error('Error loading folders:', error);
    foldersList.innerHTML = '<div class="empty-state-small"><p>Error loading</p></div>';
  }
}

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
    // Validate and sanitize filter inputs
    const minSizeValue = minSizeInput.value ? parseInt(minSizeInput.value, 10) : null;
    const maxSizeValue = maxSizeInput.value ? parseInt(maxSizeInput.value, 10) : null;

    // Validate filter values
    if (minSizeValue !== null && (isNaN(minSizeValue) || minSizeValue < 0)) {
      showError('Minimum size must be a positive number');
      return;
    }
    if (maxSizeValue !== null && (isNaN(maxSizeValue) || maxSizeValue < 0)) {
      showError('Maximum size must be a positive number');
      return;
    }
    if (minSizeValue !== null && maxSizeValue !== null && minSizeValue > maxSizeValue) {
      showError('Minimum size cannot be greater than maximum size');
      return;
    }

    // Validate date range
    if (dateFromInput.value && dateToInput.value) {
      const dateFrom = new Date(dateFromInput.value);
      const dateTo = new Date(dateToInput.value);
      if (dateFrom > dateTo) {
        showError('Start date cannot be after end date');
        return;
      }
    }

    const filters = {
      min_size: minSizeValue ? minSizeValue * 1024 : null,
      max_size: maxSizeValue ? maxSizeValue * 1024 : null,
      date_from: dateFromInput.value || null,
      date_to: dateToInput.value || null,
    };

    const results = await invoke('search_pdfs', { query, filters });
    currentResults = results;
    displayResults(results);
  } catch (error) {
    console.error('Error searching:', error);
    // Show user-friendly error message without exposing internals
    showError('Search failed. Please try different search terms or filters.');
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

// Auto-search when filters change
minSizeInput.addEventListener('change', () => {
  if (searchInput.value.trim()) {
    performSearch();
  }
});

maxSizeInput.addEventListener('change', () => {
  if (searchInput.value.trim()) {
    performSearch();
  }
});

dateFromInput.addEventListener('change', () => {
  if (searchInput.value.trim()) {
    performSearch();
  }
});

dateToInput.addEventListener('change', () => {
  if (searchInput.value.trim()) {
    performSearch();
  }
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

  // Check if results should be limited (trial expired)
  const limitResults = shouldLimitResults();
  const originalCount = sortedResults.length;
  if (limitResults && sortedResults.length > 10) {
    sortedResults = sortedResults.slice(0, 10);
  }

  // Show count and warn if limit reached
  const MAX_RESULTS = 100;
  let countText = `${sortedResults.length} result${sortedResults.length !== 1 ? 's' : ''}`;
  if (limitResults && originalCount > 10) {
    countText = `Showing 10 of ${originalCount} results`;
  } else if (!limitResults && originalCount >= MAX_RESULTS) {
    countText += ` (showing top ${MAX_RESULTS})`;
  }
  resultsCount.textContent = countText;
  
  // Add license notice if limited
  if (limitResults && originalCount > 10) {
    const notice = document.createElement('div');
    notice.className = 'license-limit-notice';
    notice.innerHTML = `
      <i data-lucide="alert-circle"></i>
      <span>Search results limited to 10. <a href="#" id="buy-to-unlock">Purchase a license</a> to see all ${originalCount} results.</span>
    `;
    resultsContainer.innerHTML = '';
    resultsContainer.appendChild(notice);
    
    // Re-render icons
    createIcons({ icons });
    
    // Add click handler for purchase link
    document.getElementById('buy-to-unlock')?.addEventListener('click', (e) => {
      e.preventDefault();
      showLicenseActivationDialog();
    });
  } else {
    resultsContainer.innerHTML = '';
  }

  Object.entries(groupedResults).forEach(([folder, items]) => {
    // Use a hash of the folder path for IDs to avoid collisions
    const folderId = 'folder-' + hashString(folder);

    const folderGroup = document.createElement('div');
    folderGroup.className = 'folder-group';

    const header = document.createElement('div');
    header.className = 'folder-group-header';
    header.setAttribute('role', 'button');
    header.setAttribute('aria-expanded', 'true');
    header.setAttribute('aria-controls', `results-${folderId}`);
    header.setAttribute('tabindex', '0');
    header.innerHTML = `
      <i data-lucide="chevron-down" class="folder-group-toggle" id="toggle-${folderId}" aria-hidden="true"></i>
      <span class="folder-group-title"><i data-lucide="folder" class="inline-icon"></i> ${escapeHtml(getFolderName(folder))}</span>
      <span class="folder-group-count">${items.length} result${items.length !== 1 ? 's' : ''}</span>
    `;

    const resultsDiv = document.createElement('div');
    resultsDiv.className = 'folder-group-results';
    resultsDiv.id = `results-${folderId}`;
    resultsDiv.setAttribute('role', 'list');
    resultsDiv.innerHTML = items.map(result => renderResultItem(result)).join('');

    // Add click handler for toggling
    header.addEventListener('click', () => {
      const toggleIcon = document.getElementById(`toggle-${folderId}`);
      const isCollapsed = resultsDiv.classList.contains('collapsed');

      if (isCollapsed) {
        resultsDiv.classList.remove('collapsed');
        header.setAttribute('aria-expanded', 'true');
        if (toggleIcon) toggleIcon.setAttribute('data-lucide', 'chevron-down');
      } else {
        resultsDiv.classList.add('collapsed');
        header.setAttribute('aria-expanded', 'false');
        if (toggleIcon) toggleIcon.setAttribute('data-lucide', 'chevron-right');
      }
      createIcons({ icons });
    });

    // Add keyboard handler for Enter and Space
    header.addEventListener('keydown', (e) => {
      if (e.key === 'Enter' || e.key === ' ') {
        e.preventDefault();
        header.click();
      }
    });

    folderGroup.appendChild(header);
    folderGroup.appendChild(resultsDiv);
    resultsContainer.appendChild(folderGroup);
  });

  // Add click handlers for result items
  resultsContainer.querySelectorAll('.result-item').forEach(item => {
    const path = item.getAttribute('data-path');
    if (path) {
      // Click handler
      item.addEventListener('click', async () => {
        try {
          await invoke('open_pdf', { path });
        } catch (error) {
          console.error('Error opening PDF:', error);
          showError('Failed to open PDF. The file may have been moved or deleted.');
        }
      });
      
      // Keyboard handler for Enter and Space
      item.addEventListener('keydown', async (e) => {
        if (e.key === 'Enter' || e.key === ' ') {
          e.preventDefault();
          try {
            await invoke('open_pdf', { path });
          } catch (error) {
            console.error('Error opening PDF:', error);
            showError('Failed to open PDF. The file may have been moved or deleted.');
          }
        }
      });
    }
  });

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

// Note: Folder group toggling is now handled directly in displayResults() with proper event listeners
// This function is no longer needed as we removed inline onclick handlers

// Render a single result item
function renderResultItem(result) {
  return `
    <div class="result-item" data-path="${escapeHtml(result.path)}" role="listitem button" tabindex="0" aria-label="Open ${escapeHtml(result.title || getFileName(result.path))}">
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

// Simple hash function for generating stable IDs from folder paths
function hashString(str) {
  let hash = 0;
  for (let i = 0; i < str.length; i++) {
    const char = str.charCodeAt(i);
    hash = ((hash << 5) - hash) + char;
    hash = hash & hash; // Convert to 32-bit integer
  }
  return Math.abs(hash).toString(36);
}

// Note: PDF opening is now handled directly in displayResults() with proper event listeners
// This function is no longer needed as we removed inline onclick handlers

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
        handler: () => addFolderBtn.click()
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
        <button class="btn btn-primary empty-state-action">
          ${state.action.text}
        </button>
      ` : ''}
    </div>
  `;

  // Add event listener for action button if present (no inline onclick)
  if (state.action) {
    const actionBtn = resultsContainer.querySelector('.empty-state-action');
    if (actionBtn) {
      actionBtn.addEventListener('click', state.action.handler);
    }
  }

  resultsCount.textContent = '';
  createIcons({ icons });
}

function showError(message) {
  showToast(message, 'error');
  resultsContainer.innerHTML = `<div class="empty-state error-state"><p><i data-lucide="x-circle" class="error-icon"></i> ${escapeHtml(message)}</p></div>`;
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
  
  // Validate type to prevent injection
  const validType = icons.hasOwnProperty(type) ? type : 'info';
  const iconName = icons[validType];

  const container = document.getElementById('toast-container');
  const toast = document.createElement('div');
  toast.className = `toast ${validType}`;

  toast.innerHTML = `
    <i data-lucide="${iconName}" class="toast-icon"></i>
    <div class="toast-content">
      <div class="toast-message">${escapeHtml(message)}</div>
    </div>
    <button class="toast-close" aria-label="Close notification">
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

function getFileName(path) {
  return path.split(/[\\/]/).pop();
}

function formatFileSize(bytes) {
  if (bytes < 1024) return bytes + ' B';
  if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(2) + ' KB';
  if (bytes < 1024 * 1024 * 1024) return (bytes / (1024 * 1024)).toFixed(2) + ' MB';
  return (bytes / (1024 * 1024 * 1024)).toFixed(2) + ' GB';
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
  if (!query || !snippet) return escapeHtml(snippet || '');

  // Snippet from database already contains <mark> tags for highlighting
  // Just sanitize any remaining content while preserving the mark tags
  // Replace <mark> temporarily, escape everything, then restore marks
  const marked = snippet.replace(/<mark>/g, '___MARK_START___')
                        .replace(/<\/mark>/g, '___MARK_END___');

  const escaped = escapeHtml(marked);

  return escaped.replace(/___MARK_START___/g, '<mark>')
                .replace(/___MARK_END___/g, '</mark>');
}

// Initialize
async function init() {
  // Initialize license UI first
  await initLicenseUI();
  
  await loadIndexedFolders();

  const count = await invoke('get_index_stats').catch(() => 0);
  if (count > 0) {
    showEmptyState('default');
  } else {
    showEmptyState('noFolders');
  }

  createIcons({ icons });
}

// Make createIcons and icons available globally for license-ui.js
window.createIcons = createIcons;
window.icons = icons;

init();
