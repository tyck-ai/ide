# App Store Submission Guide

This guide covers how to submit your Tapp to the Tyck App Store for discovery by other users.

## Prerequisites

1. **Completed App** - Your app builds successfully with `tapp build`
2. **Manifest** - Valid `manifest.json` with all required fields
3. **Account** - Tyck developer account at https://tyck.dev/developers

## Package Requirements

### Manifest Checklist

```json
{
  "id": "unique-app-id",
  "name": "Human Readable Name",
  "version": "1.0.0",
  "description": "Clear description (10-200 chars)",
  "author": "Your Name <email@example.com>",
  "homepage": "https://your-app-site.com",
  "repository": "https://github.com/username/repo",
  "license": "MIT",
  "keywords": ["relevant", "searchable", "keywords"],
  "icon": "assets/icon.svg",
  "permissions": ["only", "required", "permissions"]
}
```

### Icon Requirements

| Property | Requirement |
|----------|-------------|
| Format | SVG (preferred) or PNG |
| Size | 256x256 pixels minimum |
| Background | Transparent preferred |
| Colors | Works on light and dark themes |

### WASM Size Limits

| Tier | Max Size | Review |
|------|----------|--------|
| Standard | 5 MB | Automatic |
| Extended | 25 MB | Manual review |
| Custom | > 25 MB | Contact support |

## Submission Process

### 1. Build Release Package

```bash
# Build optimized WASM
tapp build --release

# Verify build output
ls -la target/wasm32-wasip2/release/*.wasm
```

### 2. Create Signed Package

```bash
# Generate signing key (first time only)
tapp keys generate

# Sign the package
tapp package --sign

# Output: dist/my-app-1.0.0.tapp
```

### 3. Submit to Registry

```bash
# Login to developer account
tapp login

# Submit package
tapp publish dist/my-app-1.0.0.tapp

# Output:
# ✓ Package uploaded
# ✓ Signature verified
# ✓ Submitted for review
# 
# Review typically takes 1-3 business days.
# Track status: https://tyck.dev/developers/apps/my-app
```

## Review Process

### Automatic Checks

The following are verified automatically:

- [ ] Valid manifest schema
- [ ] WASM compiles and loads
- [ ] No malware signatures
- [ ] Package signature valid
- [ ] Size within limits
- [ ] No duplicate app ID

### Manual Review

For apps requesting certain permissions:

| Permission | Review Reason |
|------------|---------------|
| `network:unrestricted` | Security review |
| `fs:system` | Security review |
| `agent:spawn` | Resource usage review |
| Size > 5MB | Performance review |

### Review Timeline

| Category | Timeline |
|----------|----------|
| Standard apps | 1-3 business days |
| Network access | 3-5 business days |
| First submission | 3-5 business days |
| Updates (no permission changes) | < 24 hours |

## Versioning and Updates

### Publishing Updates

```bash
# Update version in manifest.json
# Rebuild and repackage
tapp build --release
tapp package --sign
tapp publish dist/my-app-1.1.0.tapp
```

### Version Requirements

- Must increment version (semver)
- Cannot overwrite existing versions
- Breaking changes require major version bump

### Deprecation

```bash
# Mark a version as deprecated
tapp deprecate my-app 1.0.0 --message "Please upgrade to 2.0.0"

# Yank a version (security issues only)
tapp yank my-app 1.0.0 --reason "Security vulnerability"
```

## Pricing and Monetization

### Free Apps

Default for all submissions. No configuration needed.

### Paid Apps (Coming Soon)

```json
{
  "pricing": {
    "model": "one-time",
    "price": 9.99,
    "currency": "USD"
  }
}
```

### Subscription Apps (Coming Soon)

```json
{
  "pricing": {
    "model": "subscription",
    "price": 4.99,
    "currency": "USD",
    "period": "monthly"
  }
}
```

## App Store Listing

### Optimizing for Discovery

1. **Clear name** - Descriptive, searchable
2. **Keywords** - Include synonyms and related terms
3. **Description** - First 100 chars appear in search results
4. **Category** - Choose the most relevant primary category

### Categories

- Developer Tools
- Productivity
- Database
- AI/ML
- Debugging
- Testing
- Documentation
- Version Control
- Deployment
- Utilities

### Screenshots (Optional)

Include up to 5 screenshots:

```bash
# Add screenshots to submission
tapp publish dist/my-app-1.0.0.tapp \
  --screenshot assets/screenshot1.png \
  --screenshot assets/screenshot2.png
```

## Policies

### Prohibited Content

- Malware or security exploits
- Apps that bypass Tyck licensing
- Copyright infringing content
- Apps with excessive permissions
- Cryptocurrency miners

### Required Disclosures

If your app:
- Collects analytics → Disclose in description
- Sends data externally → List hosts in manifest
- Uses third-party services → Credit in description

### Updates Policy

- Security fixes must be submitted within 30 days of disclosure
- Apps abandoned > 2 years may be delisted
- Breaking API changes require deprecation notice

## Support and Appeals

### Getting Help

- Documentation: https://docs.tyck.dev/tapp/app-store
- Developer Forum: https://forum.tyck.dev/c/app-store
- Email: app-store@tyck.dev

### Appealing Rejection

1. Review rejection reason in developer portal
2. Make required changes
3. Resubmit with explanation
4. Or email app-store@tyck.dev to discuss

## CLI Commands Reference

```bash
# Authentication
tapp login                    # Login to developer account
tapp logout                   # Logout
tapp whoami                   # Show current user

# Key Management
tapp keys generate            # Generate signing keypair
tapp keys list               # List your keys
tapp keys revoke <id>        # Revoke a key

# Packaging
tapp package                 # Create package
tapp package --sign          # Create signed package
tapp package --verify        # Verify existing package

# Publishing
tapp publish <file>          # Submit to app store
tapp unpublish <id>          # Remove from store
tapp deprecate <id> <ver>    # Deprecate version
tapp yank <id> <ver>         # Yank version

# Analytics (for your apps)
tapp stats <id>              # View download stats
tapp reviews <id>            # View user reviews
```
