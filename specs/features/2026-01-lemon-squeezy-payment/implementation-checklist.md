# Implementation Checklist - Lemon Squeezy Payment System

This checklist provides a step-by-step guide for implementing the payment and license validation system.

## Pre-Implementation

- [ ] Review complete specification (README.md)
- [ ] Set up development environment
- [ ] Create test Lemon Squeezy account
- [ ] Prepare domain for backend API (if using serverless)
- [ ] Review security best practices

---

## Phase 1: Lemon Squeezy & Backend (Week 1)

### Lemon Squeezy Store Setup
- [ ] Create Lemon Squeezy account
- [ ] Verify email and complete onboarding
- [ ] Create store: "PDF Finder Pro"
- [ ] Add product:
  - [ ] Name: "PDF Finder Pro - Lifetime License"
  - [ ] Price: $5.00 USD
  - [ ] Type: Single Payment
  - [ ] Description: "Lifetime license for PDF Finder Pro desktop application"
- [ ] Enable license keys:
  - [ ] Format: Custom prefix "PDFPRO-"
  - [ ] Length: 4 groups of 4 characters
  - [ ] Activation limit: 3
  - [ ] Disable characters: 0, O, 1, I, l
- [ ] Configure checkout:
  - [ ] Collect email only
  - [ ] Add logo
  - [ ] Set success URL
  - [ ] Customize email templates
- [ ] Test mode purchase
- [ ] Verify license key generation

### Backend API Setup
- [ ] Choose backend platform (Vercel recommended)
- [ ] Create new project/repository
- [ ] Set up environment variables:
  - [ ] `LEMON_SQUEEZY_API_KEY`
  - [ ] `LEMON_SQUEEZY_WEBHOOK_SECRET`
  - [ ] `LICENSE_SIGNING_SECRET`
- [ ] Install dependencies:
  - [ ] `@lemonsqueezy/lemonsqueezy.js`
  - [ ] Other required packages
- [ ] Implement webhook endpoint (`api/webhook.js`):
  - [ ] Signature verification
  - [ ] Event logging
  - [ ] Order processing
  - [ ] Test with Lemon Squeezy test mode
- [ ] Implement validation endpoint (`api/validate-license.js`):
  - [ ] License key validation
  - [ ] Activation limit check
  - [ ] Rate limiting
  - [ ] Error handling
  - [ ] Test with test license keys
- [ ] Deploy backend
- [ ] Verify endpoints are accessible
- [ ] Configure webhook in Lemon Squeezy dashboard

### Landing Page Update
- [ ] Add "Buy Now" button
- [ ] Link to Lemon Squeezy checkout
- [ ] Update pricing section
- [ ] Add FAQ about licensing
- [ ] Test purchase flow end-to-end

### Verification
- [ ] Complete test purchase
- [ ] Verify license key received via email
- [ ] Verify webhook triggered successfully
- [ ] Verify validation endpoint works
- [ ] Document API endpoints

---

## Phase 2: License System (Rust) (Week 2-3)

### Dependencies
- [ ] Add to `Cargo.toml`:
  ```toml
  aes-gcm = "0.10"
  hmac = "0.12"
  sha2 = "0.10"
  base64 = "0.21"
  reqwest = { version = "0.11", features = ["json"] }
  ```
- [ ] Run `cargo build` to verify

### Module: license.rs
- [ ] Create `src-tauri/src/license.rs`
- [ ] Define `License` struct:
  - [ ] All required fields
  - [ ] Serde serialization
- [ ] Implement encryption:
  - [ ] `encrypt_license()`
  - [ ] `decrypt_license()`
  - [ ] Key derivation from instance_id
- [ ] Implement signing:
  - [ ] `sign_license()`
  - [ ] `verify_signature()`
  - [ ] HMAC-SHA256
- [ ] Implement storage:
  - [ ] `save_license()`
  - [ ] `load_license()`
  - [ ] Platform-specific paths
  - [ ] File permissions (read/write for user only)
- [ ] Write unit tests:
  - [ ] Encryption/decryption roundtrip
  - [ ] Signature verification
  - [ ] Storage/loading
  - [ ] Error handling

### Module: validation.rs
- [ ] Create `src-tauri/src/validation.rs`
- [ ] Implement `LicenseStatus` enum
- [ ] Implement hardware fingerprinting:
  - [ ] `generate_instance_id()`
  - [ ] OS name
  - [ ] Username
  - [ ] Machine ID
  - [ ] SHA-256 hash
- [ ] Implement trial tracking:
  - [ ] `get_first_launch_timestamp()`
  - [ ] `get_trial_days_remaining()`
  - [ ] `is_in_trial()`
  - [ ] `is_expired()`
- [ ] Implement online validation:
  - [ ] `validate_with_api()`
  - [ ] HTTP request to validation endpoint
  - [ ] Response parsing
  - [ ] Error handling (network, invalid response)
- [ ] Implement offline validation:
  - [ ] Load license file
  - [ ] Verify signature
  - [ ] Check timestamps
- [ ] Implement periodic validation:
  - [ ] `should_validate_online()` (30-day check)
  - [ ] `update_last_validated()`
- [ ] Main validation function:
  - [ ] `validate_license()` - orchestrates all checks
- [ ] Write unit tests:
  - [ ] Trial period calculation
  - [ ] Instance ID generation (stable)
  - [ ] Validation logic branches
  - [ ] Mock API responses

### Module: license_manager.rs
- [ ] Create `src-tauri/src/license_manager.rs`
- [ ] Implement `LicenseManager` struct
- [ ] Activation flow:
  - [ ] `activate_license(key, email)`
  - [ ] Format validation
  - [ ] API call
  - [ ] License storage
  - [ ] Error handling
- [ ] Deactivation flow:
  - [ ] `deactivate_license()`
  - [ ] API call to free slot
  - [ ] Delete local license file
- [ ] Status queries:
  - [ ] `get_license_status()`
  - [ ] `get_trial_info()`
  - [ ] `is_licensed()`
- [ ] Write integration tests

### Tauri Commands (lib.rs)
- [ ] Add license commands:
  - [ ] `get_license_status()`
  - [ ] `activate_license(key, email)`
  - [ ] `deactivate_license()`
  - [ ] `get_trial_days_remaining()`
  - [ ] `get_license_info()`
- [ ] Register commands in `invoke_handler!`
- [ ] Add `LicenseManager` to app state
- [ ] Initialize license manager on startup

### Testing
- [ ] Unit tests pass (`cargo test`)
- [ ] Test with valid test license key
- [ ] Test with invalid license key
- [ ] Test trial period logic
- [ ] Test offline validation
- [ ] Test encryption/decryption
- [ ] Test signature verification
- [ ] Test cross-platform (if possible)

---

## Phase 3: Frontend UI (Week 4)

### New Files
- [ ] Create `license-ui.js`
- [ ] Create `license-modal.css` (or add to styles.css)

### Welcome Screen
- [ ] Design modal layout
- [ ] Implement first-launch detection
- [ ] Add "Start Free Trial" button
- [ ] Add "Enter License" button
- [ ] Show on first launch only
- [ ] Store preference (don't show again after choice)

### License Activation Dialog
- [ ] Design modal layout
- [ ] Implement license key input:
  - [ ] Auto-formatting (PDFPRO-XXXX-XXXX-XXXX-XXXX)
  - [ ] Character validation
  - [ ] Uppercase conversion
- [ ] Email input (optional)
- [ ] Activate button with loading state
- [ ] Cancel button
- [ ] "Buy Now" link
- [ ] Error message display
- [ ] Success confirmation

### Trial Expiration Notice
- [ ] Design modal layout
- [ ] Show on startup if trial expired
- [ ] Display message about trial expiration
- [ ] Display current limitations (10 results)
- [ ] "Buy Now" button
- [ ] "Enter License" button
- [ ] "Remind me later" button (closes modal)
- [ ] Don't show more than once per day

### Settings Panel
- [ ] Add "License" section to settings
- [ ] Show license status:
  - [ ] Active (with checkmark)
  - [ ] Trial (with days remaining)
  - [ ] Expired
- [ ] Show license key (masked)
- [ ] Show activation date
- [ ] "Deactivate" button (with confirmation)
- [ ] "Manage License" link (opens Lemon Squeezy)

### Main Window Updates
- [ ] Add trial banner:
  - [ ] Show during trial at top of window
  - [ ] Display "X days remaining"
  - [ ] "Buy Now" button
  - [ ] Hide when licensed
- [ ] Add Help menu items:
  - [ ] "Enter License Key..."
  - [ ] "Purchase License..."
- [ ] Update search result display:
  - [ ] Limit to 10 when expired
  - [ ] Show message: "Showing 10 of X results. Purchase license for unlimited results."

### Startup Logic (main.js)
- [ ] Check license status on app launch:
  - [ ] Call `get_license_status()`
  - [ ] Show welcome screen if first launch
  - [ ] Show trial banner if in trial
  - [ ] Show expiration notice if expired
  - [ ] Enable full features if licensed
- [ ] Periodic validation:
  - [ ] Check every 30 days in background
  - [ ] Update UI if status changes

### Styling
- [ ] License modal styles
- [ ] Trial banner styles
- [ ] Settings panel styles
- [ ] Button states (loading, disabled)
- [ ] Error/success message styles
- [ ] Responsive design
- [ ] Match existing app theme

### User Flow Testing
- [ ] First launch → welcome screen
- [ ] Start trial → banner appears
- [ ] Trial countdown updates daily
- [ ] Trial expiration → nag screen
- [ ] Expired → search limited to 10
- [ ] Enter license → activation works
- [ ] Licensed → full features unlocked
- [ ] Deactivate → returns to trial/expired
- [ ] Purchase → receive email → activate

---

## Phase 4: Integration & Testing (Week 5)

### End-to-End Testing
- [ ] Fresh install on Windows:
  - [ ] First launch → welcome screen
  - [ ] Trial mode → 14 days
  - [ ] All features work
- [ ] Fresh install on macOS:
  - [ ] Same tests as Windows
- [ ] Fresh install on Linux:
  - [ ] Same tests as Windows
- [ ] Purchase flow:
  - [ ] Buy on landing page
  - [ ] Receive email with license key
  - [ ] Activate in app
  - [ ] Verify full access
- [ ] Offline mode:
  - [ ] Activate while online
  - [ ] Disconnect internet
  - [ ] Restart app → still licensed
  - [ ] Wait 31 days (mock timestamp) → prompt to reconnect
- [ ] 30-day revalidation:
  - [ ] Mock last_validated = 31 days ago
  - [ ] Connect to internet
  - [ ] Verify revalidation happens
- [ ] Deactivation/reactivation:
  - [ ] Activate on device 1
  - [ ] Deactivate on device 1
  - [ ] Activate on device 2
  - [ ] Verify works
- [ ] Trial expiration:
  - [ ] Mock install date = 15 days ago
  - [ ] Launch app
  - [ ] Verify nag screen
  - [ ] Search → limited to 10 results
  - [ ] Verify other features work
- [ ] License revocation (refund):
  - [ ] Simulate refund in Lemon Squeezy
  - [ ] Trigger revalidation
  - [ ] Verify license marked as revoked

### Edge Case Testing
- [ ] No internet on first launch:
  - [ ] Can still start trial
  - [ ] Can't activate license (show clear error)
- [ ] Invalid license key format:
  - [ ] Show validation error immediately
- [ ] Invalid license key (wrong key):
  - [ ] Show error from API
- [ ] Activation limit reached:
  - [ ] Show clear error
  - [ ] Explain deactivation process
- [ ] Clock manipulation:
  - [ ] Set system clock backwards
  - [ ] Verify trial doesn't extend
  - [ ] Verify license still validated
- [ ] Corrupted license file:
  - [ ] Manually corrupt file
  - [ ] Launch app
  - [ ] Should fallback to trial/expired
  - [ ] Allow re-activation
- [ ] Network timeout during activation:
  - [ ] Simulate slow/timeout network
  - [ ] Verify clear error message
  - [ ] Verify can retry
- [ ] Backend downtime:
  - [ ] Simulate API down
  - [ ] Verify offline validation works
  - [ ] Verify graceful error messages

### Performance Testing
- [ ] Measure startup time:
  - [ ] Without license check: X ms
  - [ ] With license check: X ms
  - [ ] Target: <100ms difference
- [ ] License validation:
  - [ ] Offline check: <10ms
  - [ ] Online check: <2 seconds
- [ ] No UI blocking during validation
- [ ] No stuttering or freezing

### Security Audit
- [ ] Encryption verification:
  - [ ] License file is encrypted
  - [ ] Can't read plaintext
  - [ ] Requires instance_id to decrypt
- [ ] Signature verification:
  - [ ] Manually modify license file
  - [ ] Verify fails validation
  - [ ] Verify can't forge signature
- [ ] License portability:
  - [ ] Copy license from machine A to B
  - [ ] Verify fails due to instance_id mismatch
- [ ] API keys:
  - [ ] Not in client code
  - [ ] Only in backend environment
- [ ] Webhook security:
  - [ ] Signature verification works
  - [ ] Invalid signature rejected

### Documentation
- [ ] Update README.md:
  - [ ] Add pricing information
  - [ ] Link to purchase page
  - [ ] Mention trial period
- [ ] Create LICENSING.md:
  - [ ] How to purchase
  - [ ] How to activate
  - [ ] How to deactivate
  - [ ] Troubleshooting common issues
  - [ ] Support contact
- [ ] Create FAQ.md:
  - [ ] What happens after trial?
  - [ ] Can I use on multiple computers?
  - [ ] What if I get a new computer?
  - [ ] Refund policy
  - [ ] Does it work offline?
- [ ] Update landing page:
  - [ ] Add detailed FAQs
  - [ ] Add activation instructions
  - [ ] Add support email
- [ ] Internal documentation:
  - [ ] API endpoints documentation
  - [ ] Webhook handling
  - [ ] Manual license verification process
  - [ ] Customer support procedures

---

## Phase 5: Launch & Monitoring (Ongoing)

### Production Deployment
- [ ] Backend:
  - [ ] Switch environment variables to production
  - [ ] Deploy to production platform
  - [ ] Verify endpoints accessible
  - [ ] Test webhook delivery
- [ ] Lemon Squeezy:
  - [ ] Switch to production mode
  - [ ] Verify payment processing works
  - [ ] Test real purchase ($5)
  - [ ] Verify email delivery
- [ ] Landing page:
  - [ ] Update checkout links to production
  - [ ] Publish updated page
  - [ ] Verify purchase button works
- [ ] Desktop app:
  - [ ] Build release versions (Windows, macOS, Linux)
  - [ ] Code sign binaries (if possible)
  - [ ] Test installation on clean systems
  - [ ] Upload to GitHub releases
  - [ ] Update download links

### Monitoring Setup
- [ ] Lemon Squeezy dashboard:
  - [ ] Set up email alerts for sales
  - [ ] Set up webhook failure alerts
  - [ ] Monitor revenue metrics
- [ ] Backend monitoring:
  - [ ] Set up uptime monitoring (UptimeRobot, Pingdom)
  - [ ] Monitor API response times
  - [ ] Set up error logging (Sentry, LogRocket)
  - [ ] Alert on 5xx errors
- [ ] Webhook monitoring:
  - [ ] Log all webhook events
  - [ ] Alert on delivery failures
  - [ ] Manual verification process for failed webhooks
- [ ] License activation:
  - [ ] Log activation attempts
  - [ ] Track success/failure rates
  - [ ] Monitor for abuse patterns

### Customer Support
- [ ] Set up support email (support@pdffinderpro.com)
- [ ] Create email templates:
  - [ ] Welcome email (after purchase)
  - [ ] Activation help
  - [ ] Refund request response
  - [ ] General inquiries
- [ ] Create support documentation:
  - [ ] Activation troubleshooting
  - [ ] Common error messages
  - [ ] Deactivation instructions
  - [ ] Refund policy
- [ ] Manual license verification process:
  - [ ] How to verify a purchase in Lemon Squeezy
  - [ ] How to manually reset activations
  - [ ] How to issue refunds
- [ ] Response time target: <24 hours

### Marketing & Announcement
- [ ] Update main README:
  - [ ] Add pricing section
  - [ ] Add "Buy Now" badge
  - [ ] Link to landing page
- [ ] Landing page announcement:
  - [ ] Banner: "Now available for $5"
  - [ ] Highlight trial period
- [ ] Social media (if applicable):
  - [ ] Twitter/X announcement
  - [ ] Reddit (r/SideProject, r/software)
  - [ ] Hacker News (Show HN)
  - [ ] Product Hunt (optional)
- [ ] Blog post:
  - [ ] Why $5?
  - [ ] What's included
  - [ ] Future roadmap
- [ ] Email existing users (if applicable):
  - [ ] Notify of pricing change
  - [ ] Offer grandfathering option

### Post-Launch Monitoring
- [ ] Week 1:
  - [ ] Monitor for critical bugs
  - [ ] Respond to support emails
  - [ ] Track conversion rate
  - [ ] Fix urgent issues
- [ ] Week 2-4:
  - [ ] Analyze metrics
  - [ ] Gather user feedback
  - [ ] Iterate on pain points
  - [ ] Optimize conversion funnel
- [ ] Monthly:
  - [ ] Review revenue
  - [ ] Review refund rate
  - [ ] Review support volume
  - [ ] Plan improvements

---

## Success Criteria

### Technical Success
- [ ] 95%+ activation success rate
- [ ] 98%+ payment success rate
- [ ] <100ms startup time impact
- [ ] <5% support tickets for license issues
- [ ] 99.9% backend uptime

### Business Success
- [ ] 15%+ trial-to-paid conversion rate
- [ ] <2% refund rate
- [ ] Positive user reviews about pricing
- [ ] Sustainable revenue for development

### User Experience Success
- [ ] "Activation was easy" feedback
- [ ] "Trial period was fair" feedback
- [ ] "Price is reasonable" feedback
- [ ] No complaints about DRM/restrictions

---

## Rollback Plan

If critical issues arise:

1. **Disable License Checks**:
   - [ ] Deploy hotfix that always returns `LicenseStatus::Valid`
   - [ ] Allows all users full access
   - [ ] Fix issues offline

2. **Pause Sales**:
   - [ ] Disable Lemon Squeezy product
   - [ ] Add notice to landing page
   - [ ] Process refunds if necessary

3. **Communication**:
   - [ ] Email affected users
   - [ ] Post status update
   - [ ] Provide timeline for fix

---

## Post-Implementation

- [ ] Mark specification as implemented
- [ ] Archive implementation notes
- [ ] Document lessons learned
- [ ] Update INDEX.md
- [ ] Clean up any temporary files
- [ ] Celebrate! 🎉

---

**Estimated Total Time**: 4-5 weeks (46-62 hours)
- Phase 1: 8-12 hours
- Phase 2: 16-20 hours
- Phase 3: 12-16 hours
- Phase 4: 10-14 hours
- Phase 5: Ongoing (2-4 hours/week)

**Last Updated**: 2026-01-07
