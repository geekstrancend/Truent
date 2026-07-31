# Versioning and Stability

## Versioning Policy

Truent follows **Semantic Versioning** as defined at https://semver.org/

```
Version: MAJOR.MINOR.PATCH
Example: 0.1.5

MAJOR = Breaking changes
MINOR = New features (backward compatible)
PATCH = Bug fixes
```

## Pre-Release Versions

Before 1.0.0, breaking changes are more frequent:

```
0.1.0 - First beta release
0.2.0 - DSL enhancements (may break invariants from 0.1)
0.5.0 - Near stable
1.0.0 - First stable release
```

## Stability Levels

### Alpha (0.0.x)

- Unstable API
- Frequent breaking changes
- Not recommended for production
- Not published to crates.io

### Beta (0.1.x - 0.9.x)

- Relatively stable
- Occasional breaking changes noted in changelog
- Safe for development
- Breaking changes documented with migration guide

### Release Candidate (0.9.x)

- Nearly stable
- Minimal breaking changes
- All critical bugs fixed
- Ready for careful production use with monitoring

### Stable (1.0.0+)

- Stable DSL and API
- Breaking changes require MAJOR version bump
- Backward compatibility guaranteed within major version
- Safe for production with stable support

## What's Considered a Breaking Change?

### DSL Breaking Changes (Always MAJOR)

```invar
// BREAKING: Syntax change
invariant balance_check:        // Old
invariant: balance_check        // New (requires colon)

// BREAKING: Keyword removal
forall x:Item in items: ...     // Old
forall x in items: ...          // New (type removed)

// BREAKING: Semantics change
sum(values) > 100               // Old: 100 minimum
sum(values) >= 100              // New: 101 minimum
```

### CLI Breaking Changes (Always MAJOR)

```bash
# BREAKING: Command removal
truent check file.invar          # Old command removed

# BREAKING: Flag removal
truent check --old-flag file.invar

# BREAKING: Output format change
# JSON structure significantly reorganized
```

### API Breaking Changes (Always MAJOR)

```rust
// BREAKING: Function signature
fn check(ast: &Ast) -> Result        // Old
fn check(ast: &Ast, config: Config) -> Result  // New

// BREAKING: Type removal
pub struct Invariant { ... }    // Removed
```

## What's NOT a Breaking Change

### New Features (MINOR)

```invar
// OK: New keyword compatible with existing code
match expression when:
    condition: true
    other: false
```

```rust
// OK: New optional parameter
pub fn check(ast: &Ast, config: Option<Config>) -> Result
```

### Bug Fixes (PATCH)

```
OK: Parsing error corrected
OK: Performance improved
OK: Error message clarity enhanced
OK: Internal refactoring
```

### New Chain Support (MINOR)

```
OK: Added Aptos chain support
OK: Enhanced Cosmos invariant library
OK: EVM2 optimization
```

## Release Schedule

### Patch Releases (x.x.Z)

- **Frequency**: As needed
- **Cycle**: Off-cycle
- **Content**: Bug fixes only
- **Compatibility**: Fully compatible
- **Testing**: Standard suite

Example timeline:
```
1.0.0 - Jan 1
1.0.1 - Jan 15 (bug fix)
1.0.2 - Jan 28 (security fix)
```

### Minor Releases (x.Y.0)

- **Frequency**: Every 4-6 weeks
- **Cycle**: On schedule
- **Content**: New features, improvements
- **Compatibility**: Backward compatible
- **Testing**: Extended suite

Example timeline:
```
1.0.0 - Jan 1
1.1.0 - Feb 12 (new features)
1.2.0 - Mar 26 (more features)
```

### Major Releases (X.0.0)

- **Frequency**: Every 6-12 months
- **Cycle**: Planned in advance
- **Content**: Breaking changes
- **Compatibility**: Migration required
- **Testing**: Comprehensive

Example timeline:
```
1.0.0 - Jan 1
2.0.0 - Dec 1 (breaking changes)
3.0.0 - Q4 next year (major revision)
```

## Deprecation Policy

### Three-Phase Deprecation

When removing or changing functionality:

**Phase 1 - Announcement** (Release X.Y.0)
```
Deprecation warning added in code
Documentation updated
Migration guide published
```

**Phase 2 - Warning** (Release X.(Y+1).0)
```
Deprecated feature still works
Warning shown when used
Clear error messages with alternatives
```

**Phase 3 - Removal** (Release (X+1).0.0)
```
Feature completely removed
Causes compile error if used
Migration guide easily found
```

### Example Deprecation Timeline

Release 0.2.0:
```
"The 'forall' keyword with type annotations is deprecated.
 Use pattern matching instead. See migration guide:
 https://truent.sh/docs/0.3/migration"
```

Release 0.3.0:
```
warning: deprecated syntax: 'forall x:Type in items'
help: use 'forall x in items' instead

The old syntax still works but shows warnings.
```

Release 1.0.0:
```
error: removed syntax: 'forall x:Type in items'

The old type annotation syntax was removed.
Update your invariants to: 'forall x in items'
See https://truent.sh/docs/1.0/migration
```

## Backward Compatibility Guarantee

### Within Major Version

From 1.0.0 onwards:

✅ **Guaranteed backward compatible:**
- DSL syntax stays the same
- CLI interface unchanged
- JSON output structure preserved
- API signatures stable
- Bug fixes always safe

❌ **Not guaranteed:**
- Performance characteristics
- Error message exact wording
- Internal implementation details
- Diagnostic output format (unless documented)

### Migration Guides

For each major version, publish:

```markdown
# Migration Guide: 1.x → 2.0

## Breaking Changes

### DSL Syntax
- Old: `invariant balance_check`
- New: `invariant: balance_check`

### CLI
- Old: `truent check file.invar`
- New: `truent analyze file.invar`

### JSON Output
- Field `violations` renamed to `failed_invariants`
```

## Experimental Features

Features marked as experimental (EXPERIMENTAL):

Can change without major version bump:

```rust
/// EXPERIMENTAL: Move chain support is beta.
/// This API may change until Move reaches production status.
pub mod move_analyzer;
```

Used by:
- Move chain support (until production ready)
- Advanced DSL features (under evaluation)
- New command-line flags (under testing)

## Stability Tiers

### Tier 1 - Stable

Fully stable, breaking changes require major version.

- DSL core syntax (invariant, forall, global)
- Solana chain support
- EVM chain support
- CLI core commands (check, init, report)
- JSON output (documented schema)

### Tier 2 - Beta

Stable enough for production, rare breaking changes.

- Advanced DSL features
- Move chain support
- Performance optimization flags
- Extended CLI options

### Tier 3 - Experimental

May change significantly, use with caution.

- Upcoming DSL extensions
- New chain integrations
- Research features
- Proposed but unstable APIs

## Feature Flags

Control experimental features:

```toml
[features]
default = ["solana", "evm"]
solana = []
evm = []
move = []  # Experimental
advanced-dsl = []  # Experimental

[[bin]]
name = "truent-debug"
required-features = ["advanced-dsl"]
```

## Documentation Versioning

Documentation for each major version:

```
https://truent.sh/docs/0.2/     # Previous
https://truent.sh/docs/1.0/     # Current
https://truent.sh/docs/latest   # Alias to current
```

API documentation:

```bash
# Specific version
cargo doc --version 1.0.0

# Current development
cargo doc
```

## Changelog Format

Follow [Keep a Changelog](https://keepachangelog.com/):

```markdown
## [1.0.0] - 2024-01-01

### Added
- New feature X
- New feature Y

### Changed
- Changed behavior of Z

### Deprecated
- Old API A (use B instead)

### Removed
- Removed feature that was deprecated in 0.9.0

### Fixed
- Bug in parser causing crash on empty input

### Security
- Fixed vulnerability in path handling
```

## Support Policy

| Version | Status | Support Ends |
|---------|--------|--------------|
| 0.1.x | Beta | 0.2.0 release |
| 0.2.x | Beta | 0.3.0 release |
| 0.9.x | RC | 1.0.0 release |
| 1.0.x | Stable | 2.0.0 release + 12 months |
| 1.1.x | Stable | 2.0.0 release + 12 months |
| 2.0.x | Stable | 3.0.0 release + 12 months |

Critical security patches backported one version.

## Version Pinning Recommendations

For production environments:

```toml
# Good - Accept patch updates
truent = "1.0"    # Accepts 1.0.x

# Good - Pin minor version
truent = "1.1"    # Accepts 1.1.x

# Caution - Pin everything
truent = "=1.0.5" # Only 1.0.5

# Bad - Too permissive
truent = "*"      # Accepts any version
```

## Communicating Changes

### In Code

```rust
#[deprecated(
    since = "0.2.0",
    note = "Use `new_function()` instead"
)]
pub fn old_function() { ... }
```

### In CHANGELOG

```markdown
### Deprecated
- Old API marked as deprecated in 0.2.0
  - Migration path documented
  - Removal scheduled for 1.0.0
```

### In Documentation

Clear "Deprecated" sections with migration examples.

### In Release Notes

Prominent section on breaking changes and migration.

## Questions?

For versioning questions:
- GitHub Issues with label `versioning`
- Email: release@truent-project.dev
