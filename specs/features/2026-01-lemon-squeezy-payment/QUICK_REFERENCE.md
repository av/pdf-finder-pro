# Payment System - Quick Reference (Offline-First)

**Quick Links**:
- [Full Specification](README.md) - Complete technical details
- [Implementation Checklist](implementation-checklist.md) - Step-by-step guide

---

## Executive Summary

Transform PDF Finder Pro from free to a $5 one-time purchase using Lemon Squeezy payment gateway with **completely offline license validation** (zero backend costs after purchase).

### Business Model
- **Price**: $5 USD one-time payment
- **Trial**: 14 days, full functionality
- **License**: Lifetime, unlimited devices
- **Target Conversion**: 15% (trial → paid)
- **Ongoing Cost**: $0 (no backend service needed)

### Key Decisions Made

#### 1. Why Completely Offline?
- Zero ongoing costs for customers
- No backend validation service required
- Better privacy (no phone-home after purchase)
- Works forever without internet
- Simpler implementation and maintenance

#### 2. Why Lemon Squeezy?
- Payment processing only (not using their license API)
- 5% fees (competitive)
- Simple email delivery of keys
- Handles VAT/tax compliance

#### 3. Why 14-Day Trial?
- Industry standard
- Enough time to index large libraries
- Matches potential refund window
- Generous without being exploitable

#### 4. Why $5?
- Low barrier to entry
- Sustainable for solo developer
- Signals quality (not free)
- Fair for value provided
- Low enough to reduce piracy incentive

### Architecture Highlights

**Truly Offline-First**:
- License validation is cryptographic (HMAC signatures)
- License key contains all needed data
- No API calls after activation
- No backend validation service
- No periodic revalidation
- Works forever without internet

**Maximum Privacy**:
- Zero data collection after purchase
- Zero network calls after activation
- No telemetry, no analytics
- License file is simple plaintext
- User controls their license file

**Zero Ongoing Costs**:
- No backend servers to run
- No database to maintain
- No API rate limits or quotas
- Lemon Squeezy only for payment processing
- $0/month infrastructure

**Trade-offs Accepted**:
- No license revocation (can't revoke remotely)
- No activation limits (same key unlimited devices)
- Cryptographic secret in binary (extractable with effort)
- Keys could be shared (focus on honest users)

---

## Implementation Timeline

### Phase 1: Lemon Squeezy + Key Generation (3-5 days)
- Set up Lemon Squeezy store
- Build key generation tool
- Generate and upload keys
- Update landing page

**Deliverable**: Working payment → email with key

### Phase 2: Offline Validation (1 week)
- Implement cryptographic validation (Rust)
- Simple license file storage
- Trial tracking
- Tauri commands

**Deliverable**: Offline validation working

### Phase 3: UI (4-5 days)
- Welcome screen
- License activation dialog
- Trial banner
- Settings panel
- Expiration notice

**Deliverable**: Complete user experience

### Phase 4: Testing (3-4 days)
- Cross-platform testing
- Offline verification
- Security validation
- Documentation

**Deliverable**: Production-ready

### Phase 5: Launch (Ongoing)
- Deploy to production (just Lemon Squeezy)
- Build and distribute apps
- Customer support

**Deliverable**: Live system (zero backend)

---

## Success Metrics

**Technical**:
- ✅ 98%+ activation success rate (offline = always works)
- ✅ <10ms validation time (offline crypto check)
- ✅ Works completely offline forever
- ✅ Zero infrastructure costs

**Business**:
- ✅ 15% trial-to-paid conversion
- ✅ <2% refund rate
- ✅ $4.55 net revenue per sale
- ✅ Zero ongoing costs

**User Experience**:
- ✅ "Simple and instant" activation
- ✅ "Works everywhere, even offline"
- ✅ No DRM complaints
- ✅ Minimal support burden

---

## Risk Management

### Top Risks & Mitigations

1. **License Keys Shared Publicly**
   - **Risk**: Medium
   - **Mitigation**: $5 price reduces incentive, focus on honest majority, can rotate secret in updates
   
2. **Secret Extracted from Binary**
   - **Risk**: Medium
   - **Mitigation**: Acceptable trade-off for offline-first, simpler than alternatives, rotate periodically

3. **Trial Period Bypass**
   - **Risk**: Medium
   - **Mitigation**: Basic timestamp validation, acceptable losses for simplicity

### What Could Go Wrong?

- **Keys leaked online**: Accept some sharing, $5 price minimizes damage
- **Secret reverse-engineered**: Can rotate in updates, focus on honest users
- **Lemon Squeezy service changes**: Only payment processing, minimal dependency
- **Cross-platform bugs**: Thorough testing before release

---

## Key Technical Components

### Rust Modules (New)
```
src-tauri/src/
├── license.rs          # Simple storage (plaintext file)
├── validation.rs       # Offline cryptographic validation
├── bin/
│   └── generate-keys.rs # Key generation tool (build-time)
└── secret.key          # HMAC secret (gitignored, compile-time)
```

### Frontend (New)
```
license-ui.js           # Modals, banners, settings UI
```

### No Backend Needed!
- Lemon Squeezy handles payment processing
- Email delivers license key
- App validates completely offline
- Zero infrastructure to maintain

### Rust Dependencies (New)
```toml
hmac = "0.12"        # HMAC signatures
sha2 = "0.10"        # Hashing
base32 = "0.4"       # Key encoding
serde = "1.0"        # Serialization
```

**Removed** (no longer needed):
- ~~aes-gcm~~ - No encryption needed
- ~~reqwest~~ - No HTTP calls needed
- ~~uuid~~ - No instance IDs needed

---

## User Flows

### First-Time User (Offline After Purchase)
1. Download & install PDF Finder Pro
2. Launch → Welcome screen
3. Choose "Start Free Trial"
4. Full access for 14 days
5. Trial banner shows days remaining
6. Day 15 → Nag screen, search limited to 10 results
7. Click "Buy Now" → Lemon Squeezy checkout
8. Receive license key via email
9. Enter license → **Validates offline instantly**
10. Full access restored **forever, even without internet**

### Purchased User (No Internet Required)
1. Purchase on landing page
2. Receive email with license key
3. Download & install (can be offline)
4. Launch → Enter license dialog
5. Paste key → **Activates instantly (offline)**
6. Full access immediately
7. **Works forever without internet**
8. **No revalidation ever needed**

### Multi-Device User (Same Key Everywhere)
1. Activate on device 1 (work laptop)
2. Activate on device 2 (home desktop)
3. Activate on device 3 (personal laptop)
4. **Same key works everywhere, no limits**
5. User convenience over DRM

---

## FAQ for Implementers

**Q: What if a user has no internet on first launch?**
A: Trial works offline. License activation also works offline (just validates cryptography locally).

**Q: What happens if Lemon Squeezy goes down?**
A: Only affects new purchases. Existing users completely unaffected (offline validation).

**Q: Can users share license keys?**
A: Technically yes. Accept this trade-off for simplicity. $5 price reduces incentive.

**Q: What if secret is extracted from binary?**
A: Can happen with any offline system. Rotate secret in major version updates.

**Q: How to handle refunds?**
A: Manual via Lemon Squeezy dashboard. Can't revoke keys (accept trade-off).

**Q: What about enterprise/volume licensing?**
A: Future consideration. Start simple with same key for all purchases.

**Q: Can keys be revoked?**
A: No. This is the trade-off for offline-first. Focus on honest users.

---

## Cost Breakdown

### Development
- Phase 1: 4-6 hours
- Phase 2: 8-12 hours
- Phase 3: 8-10 hours
- Phase 4: 6-8 hours
- **Total**: 26-36 hours (was 46-62 hours with backend)

### Ongoing Costs
- **Lemon Squeezy**: 5% + payment processor fees (~2.9% + 30¢)
- **Backend Hosting**: $0 (**no backend needed!**)
- **Domain**: Not needed for license system
- **Support**: 1-2 hours/week

### Break-Even Analysis
- Total fees per sale: ~$0.45 (9%)
- Net revenue per sale: ~$4.55
- Development cost: 30 hours × hourly rate
- Break-even: ~(30 × rate) / $4.55 sales

Example: At $50/hr → $1,500 dev cost → **330 sales to break even** (was 550 with backend)

---

## Next Steps

1. **Review**: Complete specification (README.md)
2. **Generate Keys**: Build key generation tool, create batch of keys
3. **Upload**: Add keys to Lemon Squeezy as product codes
4. **Implement**: Offline validation in Rust
5. **Test**: Thoroughly test cryptographic validation
6. **Deploy**: Just Lemon Squeezy + app binaries (no backend!)

---

## Resources

- [Lemon Squeezy Docs](https://docs.lemonsqueezy.com/)
- [HMAC Wikipedia](https://en.wikipedia.org/wiki/HMAC)
- [Tauri Security Best Practices](https://tauri.app/security/)

---

**Total Specification Size**: ~1,400 lines (simplified from 1,627)
**Confidence Level**: High - simpler architecture, proven cryptography
**Complexity**: Low-Medium - no backend reduces moving parts
**Ongoing Costs**: **$0/month** - truly sustainable

*Last Updated: 2026-01-07 (Revised: Offline-first, zero backend)*
