# Lemon Squeezy Payment & License Validation System

---
**Metadata**
- **Type**: feature
- **Status**: approved
- **Created**: 2026-01-07
- **Updated**: 2026-01-07
- **Author**: @av
- **Reviewers**: TBD
- **Related Specs**: 
  - [Landing Page Implementation](../../deployment/)
  - [Security Specifications](../../security/)
---

## Overview

Implementation of a payment and license validation system for PDF Finder Pro using Lemon Squeezy as the payment gateway. The application will transition from a free-to-use model to a commercial $5 one-time purchase with license key validation.

### Goals
- Implement secure payment processing through Lemon Squeezy
- Add offline-first license key validation (self-contained in the app)
- Provide trial/demo mode with limited functionality
- Maintain privacy-first architecture (all validation happens locally, zero backend)
- Create seamless purchase and activation flow
- Ensure cross-platform compatibility (Windows, macOS, Linux)
- Zero recurring costs after purchase (no backend validation service)

### Non-Goals
- Subscription-based pricing (this is one-time purchase only)
- Cloud-based license validation (completely offline after purchase)
- Backend validation service (no server infrastructure needed)
- Periodic online checks (never contacts servers after activation)
- DRM or aggressive anti-piracy measures (focus on honest users)
- Mobile applications (desktop only)
- Refund automation (handled manually via Lemon Squeezy dashboard)

---

## Background / Context

### Problem Statement
PDF Finder Pro is currently a free desktop application. To sustain development and provide ongoing support, the application needs to generate revenue through a one-time $5 purchase. The payment system must be:
1. **Simple**: One-time purchase with lifetime license
2. **Private**: Minimal data collection, no tracking, no backend
3. **Secure**: Prevent trivial piracy without invasive DRM
4. **Cross-platform**: Work identically on Windows, macOS, Linux
5. **Offline-first**: Zero cost after purchase, no server dependencies

### Current State
- Free, open-source desktop application
- No payment processing
- No license validation
- No usage restrictions
- Full functionality available to all users
- Landing page exists but has no purchase flow

### Desired State
- $5 one-time purchase via Lemon Squeezy
- 14-day free trial with full functionality
- Simple license key activation in the app (self-validating)
- License validated completely offline using cryptographic signatures
- Purchase flow integrated into landing page
- Graceful degradation if license invalid (show nag screen, allow basic search)
- Zero backend costs or dependencies after purchase

---

## Proposal / Solution

### High-Level Approach

**Payment & License Flow**:
1. User visits landing page
2. Clicks "Buy Now" → redirected to Lemon Squeezy checkout
3. After purchase → receives license key via email
4. Opens PDF Finder Pro → prompted to enter license key
5. App validates license key **cryptographically offline** (no API call)
6. License stored locally with verification data
7. App runs **completely offline forever** (never contacts servers again)

**Trial Mode**:
- First 14 days: Full functionality, no restrictions
- After 14 days: Nag screen on startup, search limited to 10 results
- Can still view indexed PDFs and access basic features

**Architecture Principles**:
- **Offline-first**: License validation is purely cryptographic, no server needed
- **Privacy-first**: Zero data collection, zero network calls after purchase
- **Zero-cost**: No backend infrastructure or ongoing costs
- **Non-invasive**: No background processes, no telemetry, no phone-home
- **Honest-user focused**: Simple validation, not aggressive DRM
- **Transparent**: User can verify license status without internet

### Detailed Design

#### Component 1: Lemon Squeezy Integration

**Payment Gateway Setup**:
- Create Lemon Squeezy account and store
- Create product: "PDF Finder Pro - Lifetime License"
- Price: $5 USD (one-time)
- Configure license keys with cryptographic format
- NO backend service needed (Lemon Squeezy handles payment only)

**Product Configuration**:
```
Product Name: PDF Finder Pro - Lifetime License
Price: $5.00 USD
Type: Single Payment
License Keys: Enabled (auto-generate on purchase)
Variants: None (single SKU for all platforms)
```

**Checkout Configuration**:
- Collect: Email only (required for license delivery)
- Dark mode support: Match app theme
- Logo: PDF Finder Pro logo
- Success URL: Link to landing page with activation instructions
- Email template: Contains license key and activation instructions

#### Component 2: License Key System (Offline-First)

**License Key Format** (Self-Validating):
```
PDFPRO-XXXX-XXXX-XXXX-XXXX-YYYY
Where:
- XXXX groups = Random data + timestamp
- YYYY = HMAC signature of the data
Example: PDFPRO-A7B2-C9D4-E1F6-G8H3-K2M4
```

**License Key Structure** (Cryptographically Signed):
- Format: 30 characters total (PDFPRO- prefix + 5 groups of 4 chars)
- Groups 1-4: Encoded data (timestamp, random seed)
- Group 5: HMAC-SHA256 signature (truncated to 4 chars for brevity)
- Character set: Base32 (A-Z, 2-7, no confusing chars)
- **Validation**: Completely offline by verifying HMAC signature
- **Secret key**: Embedded in application (rotated per version)

**License Data Stored Locally**:
```rust
struct License {
    key: String,                    // "PDFPRO-XXXX-XXXX-XXXX-XXXX-YYYY"
    activated_at: i64,              // Unix timestamp when activated
    // That's it! No server IDs, no validation timestamps, no backend data
}
```

**Storage Location**:
- Platform-specific storage (simple file)
- Windows: `%APPDATA%/pdf-finder-pro/license.key`
- macOS: `~/Library/Application Support/pdf-finder-pro/license.key`
- Linux: `~/.local/share/pdf-finder-pro/license.key`
- Plain text file (license key itself contains all security)
- User can backup, copy, or view this file

#### Component 3: License Validation System (Offline-Only)

**Validation Mode** (Single Mode):

1. **Offline Validation** (always, no internet required):
   - User enters license key
   - App decodes the key structure
   - Verifies HMAC signature using embedded secret
   - Checks timestamp is reasonable (not from year 1970 or 2099)
   - Stores validated key locally
   - **No API calls, no hardware binding, no server checks**

**Validation Logic**:
```rust
enum LicenseStatus {
    Valid,                          // License is valid
    Trial(days_remaining: i32),     // In trial period
    Expired,                        // Trial expired, needs purchase
    Invalid,                        // License key invalid or corrupted
}

fn validate_license() -> LicenseStatus {
    // 1. Check trial period (never expires, just counts days)
    if is_within_first_14_days() {
        return LicenseStatus::Trial(remaining_days());
    }
    
    // 2. Check local license file
    if let Some(license_key) = load_local_license() {
        // Validate cryptographic signature (offline)
        if verify_license_key_signature(&license_key) {
            return LicenseStatus::Valid;
        } else {
            return LicenseStatus::Invalid;
        }
    }
    
    // 3. No license found after trial
    LicenseStatus::Expired
}

fn verify_license_key_signature(key: &str) -> bool {
    // Extract components from key
    let parts: Vec<&str> = key.split('-').skip(1).collect(); // Skip "PDFPRO"
    if parts.len() != 5 { return false; }
    
    // Groups 1-4 contain the data
    let data = parts[..4].join("");
    
    // Group 5 is the signature
    let provided_sig = parts[4];
    
    // Compute expected signature
    let secret = get_embedded_secret(); // Compiled into app
    let computed_sig = hmac_sha256_truncated(&secret, &data);
    
    // Compare (constant-time to prevent timing attacks)
    provided_sig == computed_sig
}
```

**Key Properties**:
- **No expiration**: Once valid, always valid (lifetime license)
- **No revocation**: Cannot be revoked (accept this trade-off)
- **No activation limits**: Same key works on unlimited devices
- **Completely offline**: Never contacts any server
- **Zero cost**: No backend infrastructure whatsoever

#### Component 4: User Interface Changes

**New UI Components**:

1. **Welcome Screen** (first launch):
   ```
   ┌─────────────────────────────────────┐
   │  Welcome to PDF Finder Pro!         │
   │                                      │
   │  Start your 14-day free trial       │
   │  Full access, no credit card        │
   │                                      │
   │  [Start Free Trial]  [Enter License]│
   └─────────────────────────────────────┘
   ```

2. **License Activation Dialog**:
   ```
   ┌─────────────────────────────────────┐
   │  Enter Your License Key             │
   │                                      │
   │  PDFPRO-[____]-[____]-[____]-[____] │
   │                                      │
   │  Email (optional): [______________] │
   │                                      │
   │  [Activate] [Cancel]                │
   │                                      │
   │  Don't have a license?              │
   │  [Buy PDF Finder Pro - $5]         │
   └─────────────────────────────────────┘
   ```

3. **Trial Expiration Notice**:
   ```
   ┌─────────────────────────────────────┐
   │  ⚠️ Trial Expired                   │
   │                                      │
   │  Your 14-day trial has ended.       │
   │  Purchase a license to continue.    │
   │                                      │
   │  Search results limited to 10.      │
   │                                      │
   │  [Buy Now - $5] [Enter License]     │
   │                                      │
   │  [Remind me later]                  │
   └─────────────────────────────────────┘
   ```

4. **Settings Panel Addition**:
   ```
   Settings
   ├── License
   │   ├── Status: Licensed ✓
   │   ├── Key: PDFPRO-****-****-****-****-K2M4
   │   ├── Activated: Jan 7, 2026
   │   └── [View License] [Buy Another License]
   ```

**Main Window Updates**:
- Add "Help" menu item: "Enter License Key"
- Add "Help" menu item: "Purchase License"
- Trial banner at top during trial: "14 days remaining - Buy Now"
- No banner when licensed
- Status display: "Licensed" or "Trial (X days)" in UI corner

#### Component 5: License Key Generation (Build-Time)

**Key Generation Tool** (Run when creating releases):
- Simple Rust CLI tool: `cargo run --bin generate-license-keys -- --count 1000`
- Generates batch of license keys
- Uploads to Lemon Squeezy as product variants/codes
- Each key is cryptographically valid
- Keys stored in Lemon Squeezy, distributed via email on purchase

**Generation Logic**:
```rust
fn generate_license_key() -> String {
    let timestamp = SystemTime::now().timestamp();
    let random_seed = random_u64();
    
    // Encode data into 4 groups
    let data = encode_license_data(timestamp, random_seed);
    
    // Compute HMAC signature
    let secret = get_embedded_secret();
    let signature = hmac_sha256_truncated(&secret, &data);
    
    format!("PDFPRO-{}-{}", data, signature)
}
```

**Benefits**:
- Generate keys offline during build/release
- Upload batch to Lemon Squeezy manually or via API
- No runtime key generation service needed
- Can generate keys years in advance

**Alternative**: Use Lemon Squeezy's built-in random key generation and validate those using a simple checksum (no signature). Trade-off: Easier setup but less secure validation.

#### Component 6: Trial System

**Trial Logic**:
```rust
fn get_trial_days_remaining() -> i32 {
    let install_date = get_first_launch_timestamp();
    let now = SystemTime::now().timestamp();
    let days_elapsed = (now - install_date) / 86400;
    let remaining = 14 - days_elapsed;
    
    max(0, remaining as i32)
}

fn is_in_trial() -> bool {
    !has_valid_license() && get_trial_days_remaining() > 0
}

fn is_expired() -> bool {
    !has_valid_license() && get_trial_days_remaining() == 0
}
```

**Trial Restrictions After Expiration**:
- Show modal on startup (closable)
- Limit search results to 10 items
- All other features work normally
- No time-based locks or feature removal
- User can still index, browse indexed folders
- Focus on gentle encouragement, not frustration

#### Component 7: Security Measures (Offline-First)

**License File Encryption**:
```rust
// Encrypt license file with AES-256-GCM
fn encrypt_license(license: &License) -> Vec<u8> {
    let key = derive_key_from_instance_id();
    let nonce = generate_random_nonce();
    
    let cipher = Aes256Gcm::new(key);
    let plaintext = serde_json::to_vec(license)?;
    
    cipher.encrypt(&nonce, plaintext.as_ref())
}

fn decrypt_license(encrypted: &[u8]) -> Result<License> {
    let key = derive_key_from_instance_id();
    // ... decrypt and deserialize
}
```

**HMAC Signature**:
```rust
fn sign_license(license: &License) -> String {
    let message = format!(
        "{}|{}|{}|{}",
        license.key,
        license.activated_at,
        license.order_id,
        license.instance_id
    );
    
    let secret = get_signing_secret(); // Embedded in app
    let signature = hmac_sha256(&secret, &message);
    
    base64_encode(signature)
}

fn verify_signature(license: &License) -> bool {
    let expected = sign_license(license);
    expected == license.signature
}
```

**API Security**:
- Lemon Squeezy API keys stored as environment variables
- Webhook signatures verified
- Rate limiting on validation endpoint
- HTTPS only
- No sensitive data logged

**Anti-Tampering** (basic):
- License file encrypted
- Signature verification
- Timestamp checks (can't set trial date backwards)
- No aggressive measures (we trust users)

---

## Implementation Plan

### Phase 1: Lemon Squeezy Setup & Key Generation
**Duration**: 3-5 days 
**Effort**: 4-6 hours

- [x] Create Lemon Squeezy account
- [ ] Set up store and product ($5 PDF Finder Pro)
- [ ] Generate batch of license keys (offline tool)
- [ ] Upload keys to Lemon Squeezy as product codes
- [ ] Configure email template with activation instructions
- [ ] Test purchase flow in test mode
- [ ] Update landing page with Lemon Squeezy checkout link

**Deliverables**:
- Lemon Squeezy store configured
- License keys generated and uploaded
- Landing page updated with "Buy Now" button
- Test purchase successful (receive key via email)

### Phase 2: License Validation (Rust)
**Duration**: 1 week  
**Effort**: 8-12 hours

- [ ] Add dependencies: `hmac`, `sha2`, `base32`
- [ ] Implement key generation tool (`src-tauri/src/bin/generate-keys.rs`):
  - [ ] Key encoding logic
  - [ ] HMAC signature generation
  - [ ] Batch generation (output CSV for Lemon Squeezy)
- [ ] Implement `license.rs` module:
  - [ ] License struct (just key + timestamp)
  - [ ] Storage functions (save/load plaintext file)
  - [ ] Platform-specific paths
- [ ] Implement `validation.rs` module:
  - [ ] Offline signature verification
  - [ ] Trial period tracking
  - [ ] Timestamp validation
- [ ] Add Tauri commands:
  - [ ] `get_license_status()`
  - [ ] `activate_license(key)`
  - [ ] `get_trial_days_remaining()`
- [ ] Write unit tests for all validation logic
- [ ] Test with generated keys

**Deliverables**:
- License validation working completely offline
- Key generation tool functional
- Trial period tracking accurate
- All tests passing

### Phase 3: UI Implementation (Frontend)
**Duration**: 4-5 days  
**Effort**: 8-10 hours

- [ ] Create `license-modal.js` for license activation UI
- [ ] Add welcome screen for first launch
- [ ] Add license activation dialog (simpler - just key input)
- [ ] Add trial expiration notice
- [ ] Add Settings → License panel (no deactivation button)
- [ ] Add trial banner to main window
- [ ] Update main.js to check license on startup
- [ ] Implement search result limiting (10 items) when expired
- [ ] Add "Buy Now" buttons (link to Lemon Squeezy checkout)
- [ ] Add "Enter License" menu item to Help menu
- [ ] Style all license-related UI components
- [ ] Test all user flows:
  - [ ] First launch → start trial
  - [ ] First launch → enter license
  - [ ] Trial expiration → nag screen
  - [ ] Purchase → activate → use app
  - [ ] License works across reboots (offline)

**Deliverables**:
- All license UI components implemented
- User flows smooth and intuitive
- Trial and licensed modes working
- Visual design matches app style
- No internet required after activation

### Phase 4: Integration & Testing
**Duration**: 1 week  
**Effort**: 10-14 hours

- [ ] End-to-end testing:
  - [ ] Fresh install → trial mode
  - [ ] Purchase on landing page → receive email
  - [ ] Activate license in app → full access
  - [ ] Test offline mode (disconnect internet)
  - [ ] Test 30-day revalidation
  - [ ] Test deactivation → reactivation
  - [ ] Test trial expiration → limited mode
  - [ ] Test license revocation (refund)
- [ ] Cross-platform testing:
  - [ ] Windows 10/11
  - [ ] macOS (Intel & Apple Silicon)
  - [ ] Linux (Ubuntu, Fedora)
- [ ] Edge case testing:
  - [ ] No internet on first launch
  - [ ] Invalid license key
  - [ ] Expired license key
  - [ ] Activation limit reached (3 devices)
  - [ ] Clock manipulation attempts
  - [ ] Corrupted license file
- [ ] Performance testing:
  - [ ] License check should add <100ms to startup
  - [ ] No stuttering or blocking on validation
- [ ] Security audit:
  - [ ] Verify encryption is working
  - [ ] Verify signatures can't be forged
  - [ ] Verify license can't be copied between machines easily
- [ ] Documentation:
  - [ ] Update README with purchase info
  - [ ] Create activation guide
  - [ ] Create troubleshooting guide
  - [ ] Update landing page with clear instructions

**Deliverables**:
- All tests passing
- Works on all platforms
- Security measures validated
- Documentation complete

### Phase 5: Launch & Monitoring
**Duration**: Ongoing  
**Effort**: 1-2 hours/week

- [ ] Deploy to production:
  - [ ] Lemon Squeezy in production mode
  - [ ] Landing page updated
  - [ ] Build and sign desktop apps
  - [ ] Upload binaries to GitHub releases
- [ ] Set up monitoring:
  - [ ] Lemon Squeezy dashboard for sales tracking
  - [ ] Simple analytics (purchase count only)
- [ ] Customer support preparation:
  - [ ] License activation FAQ
  - [ ] Common issues troubleshooting
  - [ ] Email templates for support
  - [ ] Refund policy documentation
- [ ] Marketing:
  - [ ] Announce on landing page
  - [ ] Post on relevant communities
  - [ ] Update GitHub README
  - [ ] Social media announcement (if applicable)

**Deliverables**:
- Application live with payment system
- Support infrastructure ready
- Launch announcement published
- Zero ongoing costs (except Lemon Squeezy fees)

### Dependencies
- **Lemon Squeezy Account**: Must be approved before testing payments
- **Code Signing Certificate**: For Windows/macOS builds (optional but recommended)

### Risks & Mitigation

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| License keys leaked/shared | Medium | Low | Accept risk, $5 price reduces piracy incentive, focus on honest users |
| Secret extracted from binary | Medium | Low | Acceptable for offline-first approach, can rotate secret in updates |
| Trial period bypass | Medium | Low | Basic timestamp validation, acceptable losses for simplicity |
| Key generation tool misuse | Low | Medium | Keep tool private, generate keys in batches, monitor unusual patterns |
| Cross-platform bugs | Medium | Medium | Test thoroughly on all platforms before release |
| Refund abuse | Low | Medium | Manual refund approval via Lemon Squeezy, limit window to 14 days |
| Lemon Squeezy service changes | Low | Low | Only use for payment processing, app works independently |

---

## Success Metrics

### Quantitative Metrics
| Metric | Current | Target | How to Measure |
|--------|---------|--------|----------------|
| Conversion Rate (trial → paid) | 0% | 15% | Lemon Squeezy analytics |
| Activation Success Rate | N/A | >98% | Simple offline validation, should be near perfect |
| Payment Success Rate | N/A | >98% | Lemon Squeezy checkout analytics |
| Support Tickets (license issues) | 0 | <3% of sales | Support email volume |
| Refund Rate | N/A | <2% | Lemon Squeezy refund stats |
| Average Activation Time | N/A | <30 seconds | User feedback (offline = instant) |

### Qualitative Metrics
- User feedback on purchase flow (should be "simple" and "painless")
- No negative reviews about payment/license system
- License activation "just works" without support
- Trial period feels generous, not restrictive
- Limited mode (after trial) is fair, not crippled

---

## Alternatives Considered

### Alternative 1: Gumroad
**Pros**: 
- Simple setup
- Good for digital products
- Email-based license delivery
- Lower fees than some alternatives

**Cons**:
- No native license API (would need custom implementation)
- Less developer-friendly than Lemon Squeezy
- Limited webhook capabilities
- No built-in license key generation

**Why not chosen**: For offline-first approach, we don't need Lemon Squeezy's license API - just their payment processing and email delivery. Their built-in license management would add unnecessary complexity and ongoing API dependencies.

### Alternative 2: Paddle
**Pros**:
- Merchant of record (handles VAT/taxes)
- Comprehensive API
- Good analytics

**Cons**:
- Higher fees (~10% vs Lemon Squeezy's 5%)
- More complex setup
- Requires approval process
- Overkill for $5 product

**Why not chosen**: Too expensive and complex for a simple one-time $5 purchase.

### Alternative 3: Stripe + Custom License System
**Pros**:
- Maximum flexibility
- Lower fees (2.9% + 30¢)
- Complete control

**Cons**:
- Must build entire license system from scratch
- Must handle tax compliance manually
- Must handle refunds manually
- Significantly more development time (3-4x)

**Why not chosen**: Development time and complexity not justified. For offline validation, we still need custom license key generation anyway, and Lemon Squeezy provides simpler payment integration.

### Alternative 4: Honor System (Pay What You Want)
**Pros**:
- No implementation needed
- Zero friction
- Good publicity

**Cons**:
- Unpredictable revenue
- Likely <1% conversion
- No license to grant "ownership"
- No value signal

**Why not chosen**: Need predictable revenue for sustainable development. $5 fixed price is low enough to be accessible while providing value.

### Alternative 5: GitHub Sponsors
**Pros**:
- No fees
- Built into GitHub
- Simple setup

**Cons**:
- Monthly subscription only (we want one-time)
- No license key system
- Less "ownership" feeling
- US-based only initially

**Why not chosen**: Subscription model doesn't fit our "lifetime license" value proposition.

---

## Open Questions

- [ ] Should we offer refunds within 14 days? (Recommendation: Yes, to match trial period)
- [ ] How to handle users sharing keys? (Recommendation: Accept some sharing, $5 price reduces incentive)
- [ ] Should we rotate the HMAC secret between major versions? (Recommendation: Yes, in major updates)
- [ ] Should we offer volume licensing for teams/organizations? (Recommendation: Later, if demand exists)
- [ ] Should we collect any analytics on feature usage? (Recommendation: No, stay privacy-first)
- [ ] Should we implement automatic updates notification? (Recommendation: Yes, but separate feature)
- [ ] What's the upgrade path if we add a "Pro" tier later? (Recommendation: Grandfather existing users into Pro features)

---

## Resources

### External Documentation
- [Lemon Squeezy API Docs](https://docs.lemonsqueezy.com/)
- [Tauri Security Best Practices](https://tauri.app/security/)
- [HMAC Best Practices](https://en.wikipedia.org/wiki/HMAC)

### Rust Crates Needed
- `hmac` (0.12) - HMAC signature generation/verification
- `sha2` (0.10) - SHA-256 hashing
- `base32` (0.4) - Base32 encoding for keys
- `serde` (1.0) - Serialization
- `serde_json` (1.0) - JSON for license file

### Frontend Libraries
- No new dependencies needed (use existing Tauri APIs)
- Native OS dialogs for license input
- CSS for modal styling

### Design References
- [Sublime Text License Activation](https://www.sublimetext.com/) - Simple and elegant
- [Panic Apps](https://panic.com/) - Clean license management
- [Sketch App](https://www.sketch.com/) - Trial + activation flow

---

## Appendix

### Code Examples

#### Rust: License Activation Command (Offline)
```rust
#[tauri::command]
async fn activate_license(
    key: String,
    state: State<'_, AppState>
) -> Result<String, String> {
    // 1. Validate format
    if !is_valid_format(&key) {
        return Err("Invalid license key format".to_string());
    }
    
    // 2. Verify cryptographic signature (completely offline)
    if !verify_license_key_signature(&key) {
        return Err("Invalid license key".to_string());
    }
    
    // 3. Create license object (minimal data)
    let license = License {
        key: key.clone(),
        activated_at: SystemTime::now().timestamp(),
    };
    
    // 4. Save (simple plaintext file)
    save_license(&license)?;
    
    Ok("License activated successfully!".to_string())
}

fn verify_license_key_signature(key: &str) -> bool {
    // Parse: PDFPRO-AAAA-BBBB-CCCC-DDDD-EEEE
    let parts: Vec<&str> = key.split('-').skip(1).collect();
    if parts.len() != 5 { return false; }
    
    let data = parts[..4].join("");
    let provided_sig = parts[4];
    
    // Compute expected signature
    let secret = include_str!("../secret.key"); // Embedded at compile time
    let computed_sig = hmac_sha256_base32_truncated(secret, &data, 4);
    
    // Constant-time comparison
    constant_time_compare(provided_sig, &computed_sig)
}
```

#### JavaScript: License Activation UI (Simplified)
```javascript
async function showLicenseActivation() {
  const modal = document.createElement('div');
  modal.className = 'license-modal';
  modal.innerHTML = `
    <div class="license-modal-content">
      <h2>Enter Your License Key</h2>
      <input 
        type="text" 
        id="license-key"
        placeholder="PDFPRO-XXXX-XXXX-XXXX-XXXX-XXXX"
        maxlength="34"
      />
      <div class="license-actions">
        <button id="activate-btn" class="btn btn-primary">
          Activate
        </button>
        <button id="cancel-btn" class="btn btn-secondary">
          Cancel
        </button>
      </div>
      <p class="license-help">
        Don't have a license? 
        <a href="https://app.lemonsqueezy.com/checkout/..." target="_blank">
          Buy PDF Finder Pro - $5
        </a>
      </p>
      <p class="offline-note">
        ✓ Works completely offline
      </p>
    </div>
  `;
  
  document.body.appendChild(modal);
  
  // Format license key as user types
  document.getElementById('license-key').addEventListener('input', (e) => {
    let value = e.target.value.toUpperCase().replace(/[^A-Z2-7]/g, '');
    if (value.startsWith('PDFPRO')) value = value.substring(7);
    
    // Format: PDFPRO-XXXX-XXXX-XXXX-XXXX-XXXX (5 groups)
    const groups = value.match(/.{1,4}/g) || [];
    e.target.value = 'PDFPRO-' + groups.join('-');
  });
  
  document.getElementById('activate-btn').addEventListener('click', async () => {
    const key = document.getElementById('license-key').value;
    
    try {
      const result = await invoke('activate_license', { key });
      alert('License activated successfully! Works completely offline.');
      modal.remove();
      location.reload(); // Refresh to show licensed state
    } catch (error) {
      alert(`Activation failed: ${error}`);
    }
  });
  
  document.getElementById('cancel-btn').addEventListener('click', () => {
    modal.remove();
  });
}
```

#### Key Generation Tool (Build-Time)
```rust
// src-tauri/src/bin/generate-keys.rs
use hmac::{Hmac, Mac};
use sha2::Sha256;
use base32::Alphabet;

type HmacSha256 = Hmac<Sha256>;

fn generate_license_key(secret: &str) -> String {
    let timestamp = SystemTime::now().timestamp();
    let random = rand::random::<u64>();
    
    // Encode data (timestamp + random) into 16 chars
    let data = format!("{:08x}{:08x}", timestamp, random);
    let data_b32 = base32::encode(Alphabet::RFC4648 { padding: false }, data.as_bytes());
    let data_clean = &data_b32[..16]; // Take first 16 chars
    
    // Compute HMAC signature
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
    mac.update(data_clean.as_bytes());
    let result = mac.finalize();
    let sig_bytes = result.into_bytes();
    
    // Take first 2 bytes, encode to base32 = 4 chars
    let sig_b32 = base32::encode(Alphabet::RFC4648 { padding: false }, &sig_bytes[..2]);
    let sig_clean = &sig_b32[..4];
    
    // Format: PDFPRO-AAAA-BBBB-CCCC-DDDD-EEEE
    let formatted = format!(
        "PDFPRO-{}-{}-{}-{}-{}",
        &data_clean[0..4],
        &data_clean[4..8],
        &data_clean[8..12],
        &data_clean[12..16],
        sig_clean
    );
    
    formatted
}

fn main() {
    let secret = std::env::var("LICENSE_SECRET").expect("LICENSE_SECRET not set");
    let count: usize = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(100);
    
    println!("license_key");
    for _ in 0..count {
        println!("{}", generate_license_key(&secret));
    }
}
```


### File Structure After Implementation

```
pdf-finder-pro/
├── src-tauri/
│   └── src/
│       ├── lib.rs                   # Add license commands
│       ├── license.rs               # New: License struct & storage (simple)
│       ├── validation.rs            # New: Offline validation logic
│       ├── bin/
│       │   └── generate-keys.rs     # New: Key generation tool
│       └── secret.key               # New: HMAC secret (gitignored)
├── main.js                          # Add license checks
├── license-ui.js                    # New: License activation UI
├── landing/
│   └── index.html                   # Update with checkout link
└── specs/
    └── features/
        └── 2026-01-lemon-squeezy-payment/
            ├── README.md            # This file
            ├── implementation-checklist.md
            └── QUICK_REFERENCE.md
```

**Note**: No backend API needed! All validation is offline.

---

## Change Log

| Date | Author | Change |
|------|--------|--------|
| 2026-01-07 | @av | Initial specification created |

---

## Review & Approval

### Reviewers
- [ ] Technical Lead - Rust implementation feasibility
- [ ] Frontend Developer - UI/UX implementation review
- [ ] Product Owner - Business model validation
- [ ] Security Reviewer - License validation security

### Sign-off
- [ ] Technical feasibility reviewed
- [ ] UX/Design reviewed
- [ ] Product requirements met
- [ ] Security reviewed
- [ ] Performance implications considered
- [ ] Privacy implications considered
- [ ] Ready for implementation

---

*This spec follows the PDF Finder Pro specification system. All documentation must remain in `specs/` directory per AGENTS.md guidelines.*
