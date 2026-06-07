# Changelog

All notable changes to MyPass will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Cross-Platform TLS (Breaking Change - Replaces reqwest)

**Migrated from `reqwest` to `rustls` + `aws-lc-rs`** to permanently resolve ring/nasm/gcc compilation issues on Windows GNU toolchain.

- **Replaced reqwest** with custom HTTP/1.1 protocol implementation
- **TLS stack**: rustls 0.23 + aws-lc-rs (pure Rust, no C/asm dependencies)
- **TLS versions**: TLS 1.2 + 1.3 supported
- **Certificate verification**: skipped in dev mode (configurable for production)
- **Native-tls compatibility**: preserved as optional `native-tls-compat` feature (Windows schannel, macOS Secure Transport, Linux OpenSSL)
- **Handwritten HTTP**: zero-dependency HTTP/1.1 protocol (GET, PUT, PROPFIND, MKCOL)
- **Android support**: API 21+ via NDK r23+, no JNI bindings required

### Security Improvements
- **Replaced `thread_rng()` with `OsRng`** (CSPRNG) for all cryptographic key generation
- **Implemented constant-time comparison** via `subtle::ConstantTimeEq` to prevent timing attacks
- **PIN hashing uses Argon2id** instead of raw SHA-256 with salt
- **Added HKDF-SHA256** for subkey derivation (replaced direct SHA-256)
- **MEK zeroization** on Vault drop
- **Pre-computed search index** to avoid repeated `to_lowercase()` allocations
- **Constant-time password verification** for both master password and PIN

### Added
- RFC 6238 TOTP test vectors (6 official time-step tests)
- Vault lifecycle integration tests (create→unlock→CRUD→lock→unlock)
- Manifest diff tests (auto-merge + conflict detection)
- 86 unit and integration tests in total
- WebDAV sync implementation (reqwest + rustls)
- Chrome CSV import command (auto-imports to vault)
- `secure_random` module wrapping OS CSPRNG
- `hkdf_helper` module for subkey derivation
- `constant_time` module for timing-safe comparisons
- QuickKey file-based storage with 0600 permissions
- KeePass KDBX signature detection (with keepass crate noted as future enhancement)
- `From<mypass_core::TauriError>` conversion for IPC boundary
- `capabilities/default.json` with permissions for all Tauri plugins
- Tauri plugin registration (clipboard/dialog/fs/notification)
- Browser extension dual-track (MV2 Firefox + WXT Chrome)

### Changed
- Unified all Tauri command return types to `Result<_, TauriError>`
- `Add/Edit/Delete` UI now actually works (was missing buttons)
- Group filter `onClick` now triggers `selectGroup`
- `MainLayout` integrates mobile responsive (sidebar hides on small screens)
- `useVaultStore` and `useUIStore` synchronized with backend
- `useSyncStore` consolidated to single definition in `sync.ts`
- Tauri `bundle` enabled with `targets: "all"`
- `build.rs` skips `tauri-winres` on GNU toolchain (workaround for Windows GNU linker)

### Fixed
- All Tauri compilation errors (vault, extension, pin, quickkey, biometric commands)
- Frontend state sync between `App.tsx` and `useVaultStore`
- TOTP error state now displayed to user (was silently failing)
- TOTP shared hook to avoid duplicate setInterval per component
- Password reveal toggle in entry detail
- Lock vault correctly clears in-memory data
- WebDAV manifest path includes trailing slash
- Removed `unwrap()` calls on mutex locks in `pin.rs`

## [0.1.0] - 2026-01-01

### Added
- Initial release
- Tauri 2 desktop app with React frontend
- Rust core library with Argon2id + XChaCha20-Poly1305
- Object-based storage with manifest index
- Bitwarden JSON, Chrome CSV, KeePass KDBX importers
- TOTP generator (HMAC-SHA1)
- PIN and master password auth with rate limiting
- Tauri commands: vault, sync, import, totp, security, biometric, extension, pin, quickkey, webauthn

[Unreleased]: https://github.com/mypass/mypass/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/mypass/mypass/releases/tag/v0.1.0
