import { invoke } from '@tauri-apps/api/core';

/**
 * License UI Module
 * Handles all license-related UI components and interactions
 */

// State
let licenseStatus = null;
let trialDaysRemaining = 0;

/**
 * Initialize license UI on app startup
 */
export async function initLicenseUI() {
  try {
    // Check license status
    licenseStatus = await invoke('get_license_status');
    console.log('License status:', licenseStatus);
    
    // Handle different statuses
    if (licenseStatus.status === 'Trial') {
      trialDaysRemaining = licenseStatus.days_remaining;
      showTrialBanner();
      
      // Show welcome screen only on first launch (day 14)
      if (trialDaysRemaining === 14) {
        const hasSeenWelcome = localStorage.getItem('hasSeenWelcome');
        if (!hasSeenWelcome) {
          showWelcomeScreen();
          localStorage.setItem('hasSeenWelcome', 'true');
        }
      }
    } else if (licenseStatus.status === 'Expired') {
      showExpirationNotice();
    } else if (licenseStatus.status === 'Valid') {
      console.log('Licensed version - full access');
    } else if (licenseStatus.status === 'Invalid') {
      console.error('Invalid license:', licenseStatus.reason);
      showExpirationNotice();
    }
  } catch (error) {
    console.error('Failed to check license status:', error);
  }
}

/**
 * Check if search results should be limited
 */
export function shouldLimitResults() {
  return licenseStatus && licenseStatus.status === 'Expired';
}

/**
 * Get current license status
 */
export function getLicenseStatus() {
  return licenseStatus;
}

/**
 * Show welcome screen on first launch
 */
function showWelcomeScreen() {
  const modal = document.createElement('div');
  modal.className = 'modal-overlay license-modal';
  modal.innerHTML = `
    <div class="modal license-welcome-modal">
      <div class="modal-header">
        <h2><i data-lucide="sparkles"></i> Welcome to PDF Finder Pro!</h2>
      </div>
      <div class="modal-content">
        <p class="welcome-message">
          Thank you for trying PDF Finder Pro! You have <strong>14 days</strong> 
          of full access to all features.
        </p>
        <div class="welcome-features">
          <div class="feature-item">
            <i data-lucide="zap" class="feature-icon"></i>
            <span>Lightning-fast full-text search</span>
          </div>
          <div class="feature-item">
            <i data-lucide="shield" class="feature-icon"></i>
            <span>Complete privacy - everything offline</span>
          </div>
          <div class="feature-item">
            <i data-lucide="infinity" class="feature-icon"></i>
            <span>Unlimited PDFs and folders</span>
          </div>
        </div>
        <div class="welcome-actions">
          <button id="start-trial-btn" class="btn btn-primary">
            <i data-lucide="play-circle"></i>
            Start Free Trial
          </button>
          <button id="have-license-btn" class="btn btn-secondary">
            I Have a License Key
          </button>
        </div>
        <p class="welcome-note">
          No credit card required • Only $5 after trial
        </p>
      </div>
    </div>
  `;
  
  document.body.appendChild(modal);
  
  // Re-render Lucide icons
  if (window.createIcons) {
    window.createIcons({ icons: window.icons });
  }
  
  // Event listeners
  document.getElementById('start-trial-btn').addEventListener('click', () => {
    modal.remove();
  });
  
  document.getElementById('have-license-btn').addEventListener('click', () => {
    modal.remove();
    showLicenseActivationDialog();
  });
}

/**
 * Show license activation dialog
 */
export function showLicenseActivationDialog() {
  const modal = document.createElement('div');
  modal.className = 'modal-overlay license-modal';
  modal.innerHTML = `
    <div class="modal license-activation-modal">
      <div class="modal-header">
        <h2><i data-lucide="key"></i> Enter Your License Key</h2>
        <button class="icon-btn close-modal-btn" aria-label="Close">
          <i data-lucide="x"></i>
        </button>
      </div>
      <div class="modal-content">
        <div class="license-input-group">
          <label for="license-key-input">License Key</label>
          <input 
            type="text" 
            id="license-key-input"
            placeholder="PDFPRO-XXXX-XXXX-XXXX-XXXX-XXXX"
            maxlength="40"
            autocomplete="off"
            spellcheck="false"
          />
          <p class="input-hint">
            <i data-lucide="info"></i>
            Enter the license key from your purchase email
          </p>
        </div>
        <div class="license-actions">
          <button id="activate-btn" class="btn btn-primary">
            <i data-lucide="check-circle"></i>
            Activate License
          </button>
          <button id="cancel-activation-btn" class="btn btn-secondary">
            Cancel
          </button>
        </div>
        <div id="activation-message" class="activation-message"></div>
        <div class="license-footer">
          <p>Don't have a license?</p>
          <a href="https://lemonsqueezy.com/checkout/pdf-finder-pro" target="_blank" class="buy-link">
            <i data-lucide="shopping-cart"></i>
            Buy PDF Finder Pro - $5
          </a>
          <p class="offline-notice">
            <i data-lucide="wifi-off"></i>
            Works completely offline after activation
          </p>
        </div>
      </div>
    </div>
  `;
  
  document.body.appendChild(modal);
  
  // Re-render Lucide icons
  if (window.createIcons) {
    window.createIcons({ icons: window.icons });
  }
  
  const input = document.getElementById('license-key-input');
  const activateBtn = document.getElementById('activate-btn');
  const cancelBtn = document.getElementById('cancel-activation-btn');
  const closeBtn = modal.querySelector('.close-modal-btn');
  const messageDiv = document.getElementById('activation-message');
  
  // Auto-format license key as user types
  input.addEventListener('input', (e) => {
    let value = e.target.value.toUpperCase().replace(/[^A-Z0-9]/g, '');
    
    // Add PDFPRO prefix if not present
    if (!value.startsWith('PDFPRO')) {
      if (value.length > 0 && !'PDFPRO'.startsWith(value)) {
        value = 'PDFPRO' + value;
      }
    }
    
    // Remove prefix for formatting
    if (value.startsWith('PDFPRO')) {
      value = value.substring(7);
    }
    
    // Format into groups
    const groups = value.match(/.{1,4}/g) || [];
    const formatted = 'PDFPRO-' + groups.join('-');
    
    e.target.value = formatted === 'PDFPRO-' ? '' : formatted;
  });
  
  // Activate button handler
  activateBtn.addEventListener('click', async () => {
    const key = input.value.trim();
    
    if (!key || key === 'PDFPRO-') {
      showMessage('Please enter a license key', 'error');
      return;
    }
    
    // Validate format
    if (!key.match(/^PDFPRO-[A-Z0-9]{4}-[A-Z0-9]{4}-[A-Z0-9]{4}-[A-Z0-9]{4}-[A-Z0-9]{4}$/)) {
      showMessage('Invalid license key format', 'error');
      return;
    }
    
    activateBtn.disabled = true;
    activateBtn.innerHTML = '<i data-lucide="loader"></i> Activating...';
    
    try {
      const result = await invoke('activate_license', { key });
      showMessage(result, 'success');
      
      // Refresh license status
      await initLicenseUI();
      
      setTimeout(() => {
        modal.remove();
        showToast('License activated successfully!', 'success');
      }, 1500);
    } catch (error) {
      showMessage(error, 'error');
      activateBtn.disabled = false;
      activateBtn.innerHTML = '<i data-lucide="check-circle"></i> Activate License';
    }
    
    // Re-render icons
    if (window.createIcons) {
      window.createIcons({ icons: window.icons });
    }
  });
  
  // Cancel/close handlers
  const closeModal = () => modal.remove();
  cancelBtn.addEventListener('click', closeModal);
  closeBtn.addEventListener('click', closeModal);
  
  // Close on overlay click
  modal.addEventListener('click', (e) => {
    if (e.target === modal) closeModal();
  });
  
  function showMessage(text, type) {
    messageDiv.textContent = text;
    messageDiv.className = `activation-message ${type}`;
  }
  
  // Focus input
  input.focus();
}

/**
 * Show trial banner in the header
 */
function showTrialBanner() {
  // Check if banner already exists
  if (document.getElementById('trial-banner')) return;
  
  const header = document.querySelector('.app-header');
  if (!header) return;
  
  const banner = document.createElement('div');
  banner.id = 'trial-banner';
  banner.className = 'trial-banner';
  banner.innerHTML = `
    <div class="trial-info">
      <i data-lucide="clock"></i>
      <span><strong>${trialDaysRemaining}</strong> day${trialDaysRemaining !== 1 ? 's' : ''} remaining in trial</span>
    </div>
    <button id="buy-now-banner-btn" class="btn btn-small btn-primary">
      <i data-lucide="shopping-cart"></i>
      Buy Now - $5
    </button>
  `;
  
  header.insertAdjacentElement('afterend', banner);
  
  // Re-render Lucide icons
  if (window.createIcons) {
    window.createIcons({ icons: window.icons });
  }
  
  // Buy now button handler
  document.getElementById('buy-now-banner-btn').addEventListener('click', () => {
    window.open('https://lemonsqueezy.com/checkout/pdf-finder-pro', '_blank');
  });
}

/**
 * Show expiration notice modal
 */
function showExpirationNotice() {
  // Don't show more than once per day
  const lastShown = localStorage.getItem('lastExpirationNotice');
  const now = Date.now();
  if (lastShown && (now - parseInt(lastShown)) < 86400000) {
    return;
  }
  
  const modal = document.createElement('div');
  modal.className = 'modal-overlay license-modal';
  modal.innerHTML = `
    <div class="modal license-expiration-modal">
      <div class="modal-header">
        <h2><i data-lucide="alert-circle"></i> Trial Expired</h2>
      </div>
      <div class="modal-content">
        <p class="expiration-message">
          Your 14-day trial has ended. Purchase a lifetime license to continue 
          enjoying unlimited search results and all features.
        </p>
        <div class="limitation-notice">
          <i data-lucide="info"></i>
          <span>Search results are now limited to 10 items</span>
        </div>
        <div class="expiration-actions">
          <button id="buy-now-btn" class="btn btn-primary">
            <i data-lucide="shopping-cart"></i>
            Buy Now - $5
          </button>
          <button id="enter-license-btn" class="btn btn-secondary">
            <i data-lucide="key"></i>
            Enter License Key
          </button>
        </div>
        <button id="remind-later-btn" class="btn btn-text">
          Remind me later
        </button>
      </div>
    </div>
  `;
  
  document.body.appendChild(modal);
  
  // Re-render Lucide icons
  if (window.createIcons) {
    window.createIcons({ icons: window.icons });
  }
  
  // Event listeners
  document.getElementById('buy-now-btn').addEventListener('click', () => {
    window.open('https://lemonsqueezy.com/checkout/pdf-finder-pro', '_blank');
  });
  
  document.getElementById('enter-license-btn').addEventListener('click', () => {
    modal.remove();
    showLicenseActivationDialog();
  });
  
  document.getElementById('remind-later-btn').addEventListener('click', () => {
    localStorage.setItem('lastExpirationNotice', now.toString());
    modal.remove();
  });
}

/**
 * Show toast notification
 */
function showToast(message, type = 'info') {
  const container = document.getElementById('toast-container');
  if (!container) return;
  
  const toast = document.createElement('div');
  toast.className = `toast toast-${type}`;
  toast.textContent = message;
  
  container.appendChild(toast);
  
  setTimeout(() => {
    toast.classList.add('show');
  }, 10);
  
  setTimeout(() => {
    toast.classList.remove('show');
    setTimeout(() => toast.remove(), 300);
  }, 3000);
}
