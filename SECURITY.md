# Security Release Policy

This document describes how Truent handles security issues and releases security updates.

## Security Advisory Process

### 1. Reporting Security Issues

**DO NOT** open public GitHub issues for security vulnerabilities.

Instead, email security@truent.dev (or equivalent contact if no dedicated security email) with:
- Description of the vulnerability
- Affected versions
- Steps to reproduce (if applicable)
- Your name and contact information

### 2. Assessment and Response Timeline

- **24 hours**: Initial acknowledgment of report
- **72 hours**: Assessment and severity determination
- **7 days**: Initial patch development and testing
- **14 days**: Coordinated disclosure and public release

### 3. Vulnerability Severity Classification

#### Critical
- Remote code execution
- Privilege escalation
- Complete bypass of invariant enforcement
- Data loss or corruption

**Response**: Emergency patch release within 24-48 hours

#### High
- Significant invariant bypass
- State corruption scenarios
- Denial of service

**Response**: Standard patch release within 7 days

#### Medium
- Minor bypass scenarios
- Information disclosure
- Performance degradation

**Response**: Included in next minor/patch release

#### Low
- Edge cases
- Low-impact information disclosure
- Documentation issues

**Response**: Included in next regular release

## Release Versioning

Follows Semantic Versioning (SemVer):

- **MAJOR.MINOR.PATCH** (e.g., 0.1.0)
- **MAJOR**: Breaking changes
- **MINOR**: Feature additions (backward compatible)
- **PATCH**: Bug fixes and security patches (backward compatible)

### Release Examples

- `0.1.0` → `0.1.1` (security patch)
- `0.1.1` → `0.2.0` (new features)
- `0.2.0` → `1.0.0` (breaking changes)

## Security Patch Release Requirements

All security patches must:

1. **Include test coverage** for the vulnerability
2. **Be reproducible** (same source → same binary)
3. **Include CVE reference** (if applicable)
4. **Have detailed commit messages** explaining the fix
5. **Be documented** in SECURITY_UPDATES.md
6. **Include migration guide** (if configuration changes required)

## Building Secure Releases

### Pre-Release Checklist

- [ ] All tests pass (including new security tests)
- [ ] No uncommitted changes
- [ ] Cargo.lock is committed
- [ ] Version bumped correctly in all Cargo.toml files
- [ ] CHANGELOG.md updated
- [ ] SECURITY_UPDATES.md updated (if security-related)
- [ ] Git tag created: `git tag -s v0.1.0`

### Release Build Process

```bash
# 1. Run full test suite
cargo test --release

# 2. Build for all platforms
cargo build --release --target x86_64-unknown-linux-gnu
cargo build --release --target x86_64-apple-darwin
cargo build --release --target aarch64-apple-darwin
cargo build --release --target x86_64-pc-windows-msvc

# 3. Compute checksums
sha256sum truent-* > truent.sha256

# 4. Create signed git tag
git tag -s -m "Release v0.1.0" v0.1.0

# 5. Push and create GitHub release
git push --tags
# Use GitHub web interface or gh CLI to create release
```

## Reproducibility Verification

For each released binary:

```bash
# 1. Extract build environment from release artifact
version="0.1.0"
rustc_version="1.70.0"

# 2. Verify Cargo.lock in release tag
git show v${version}:Cargo.lock > /tmp/Cargo.lock.release
diff /tmp/Cargo.lock.release Cargo.lock

# 3. Rebuild and compare
cargo build --release --locked
sha256sum target/release/truent > /tmp/build.sha256

# 4. Compare with published checksum
sha256sum -c truent-v${version}.sha256
```

## Supported Versions

- **Latest release**: Full support
- **Previous minor version**: Security patches only (30 days)
- **Older versions**: End of life, no updates

Example:
- Version 0.2.0: Current release (all patches)
- Version 0.1.5: Last patch of 0.1.x (security patches only)
- Version 0.0.x: No longer supported

## Vulnerability Disclosure Timeline (Coordinated)

1. Vendor receives report (Day 0)
2. Assessment complete (Day 2)
3. Patch drafted (Day 5)
4. Patch tested (Day 10)
5. **Public disclosure** (Day 14)
   - CVE published
   - GitHub security advisory
   - Release notes
   - Blog post
   - Email notification to users

## Post-Release Monitoring

After security patch release:

1. Monitor GitHub issues for unexpected behavior
2. Track download statistics
3. Verify no new reports of same vulnerability
4. Publish post-mortem (for critical issues)

## Post-Mortem Requirements (Critical Issues)

For critical security issues, publish within 14 days:

- **What happened**: Clear description
- **Root cause**: Technical analysis
- **Impact**: Who/what was affected
- **Detection**: How it was discovered
- **Response**: Actions taken
- **Prevention**: Changes made to prevent recurrence

## Compensation and Bug Bounty

Truent may offer:
- **Critical**: Up to $5,000
- **High**: Up to $2,000
- **Medium**: Up to $500
- **Low**: $0 (recognition)

Eligibility:
- First to responsibly disclose
- Not already known/patched
- Follows responsible disclosure timeline

## Questions?

Contact: security@truent.dev

