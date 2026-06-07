# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |

## Reporting a Vulnerability

We take the security of MyPass seriously. If you discover a security vulnerability, please report it privately.

**Please DO NOT file a public GitHub issue for security vulnerabilities.**

### Contact

Send an email to: **security@mypass.app**

Include the following in your report:
- Description of the vulnerability
- Steps to reproduce
- Potential impact
- Suggested fix (if any)

### What to Expect

- **Acknowledgment** within 48 hours
- **Initial assessment** within 7 days
- **Status updates** every 14 days until resolved
- **Public disclosure** after a fix is released

## Security Architecture

### Encryption

| Layer | Algorithm | Parameters |
|-------|-----------|------------|
| Key Derivation | Argon2id | 64 MiB, 3 iterations, 4 parallel lanes, 16-byte salt |
| Symmetric | XChaCha20-Poly1305 | 256-bit key, 192-bit nonce, 16-byte auth tag |
| Subkey Derivation | HKDF-SHA256 | RFC 5869 compliant |
| Random Number | OS CSPRNG | Linux: getrandom, macOS: SecRandomCopyBytes, Windows: BCryptGenRandom |

### Memory Safety

- All cryptographic key material is zeroized on drop using `zeroize` crate
- Tauri-side errors are converted to typed `TauriError` to avoid information leakage
- No `unsafe` code (enforced by `#![forbid(unsafe_code)]`)

### Authentication

- Master password: rate-limited (5 failures → 5 min lockout), constant-time comparison
- PIN: rate-limited separately, Argon2id hash, constant-time comparison
- Auto-lock: 5 min default, configurable

### Sync (WebDAV)

- Manifest-driven incremental sync
- Conflict detection when same version has different content
- TLS via rustls
- HTTP Basic Auth over HTTPS

## Threat Model

### Protected

- ✅ Local file system attackers (encryption-at-rest)
- ✅ Shoulder surfing (password not displayed in UI)
- ✅ Brute force on master password (Argon2id is slow)
- ✅ Memory dump attacks after Vault lock (MEK zeroized)
- ✅ Timing side-channels (constant-time comparison)
- ✅ Network MitM during WebDAV sync (TLS)

### Out of Scope

- ❌ Compromised endpoint (keylogger on unlocked device)
- ❌ Physical access while Vault is unlocked
- ❌ Side-channel on the host OS (Spectre, etc.)
- ❌ Compromise of the build toolchain (supply chain)

## Security Best Practices for Users

1. **Use a strong master password** (≥ 12 characters, random)
2. **Enable QuickKey** (biometric/PIN) for convenience + security
3. **Keep auto-lock enabled** (default 5 minutes)
4. **Use HTTPS** for WebDAV sync
5. **Don't disable** Windows Hello / Touch ID for vaults
6. **Back up your vault** regularly (the encrypted file is safe to copy)

## Acknowledgments

We thank security researchers who help us improve MyPass. With your permission, we will credit you in our security advisories.

## Bug Bounty

A bug bounty program is not currently active. We may offer rewards for critical vulnerabilities in the future.
