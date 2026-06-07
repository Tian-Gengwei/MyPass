# 🦀 Rust 与安全基线

<绝对红线>
- `#![forbid(unsafe_code)]` 是不可协商的底线，绝不引入 unsafe 块。
- 业务逻辑中严禁使用 `unwrap()` 或 `expect()`，一律使用 `?` 进行错误传播。
- 错误处理：领域/核心业务错误必须使用 `thiserror` 派生；应用顶层兜底使用 `anyhow`。
  </绝对红线>

<反模式_坚决抵制>
- ❌ 在库级别返回 `Box<dyn Error>`。
- ❌ 使用 `let _ = ...` 忽略 `Result` 结果（除非显式注明安全且无副作用）。
- ❌ 使用 `String` 存储密码或密钥；必须使用支持 `Zeroize` 的包装器（如 `Secret<String>`）。
  </反模式_坚决抵制>

<密码学原语_仅限以下选型>
- KDF：仅限 Argon2id。
- 对称加密：仅限 XChaCha20-Poly1305 (AEAD)。
- 随机数：必须使用 `rand::rngs::OsRng` 或 `rand::thread_rng()`。
- 内存清理：密钥/MEK 使用后必须立即通过 `zeroize` crate 进行内存清零。
