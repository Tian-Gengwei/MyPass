# MyPass CI/CD 流水线验证清单

## 主 CI（`ci.yml`，已有）
- [ ] `frontend` job 在 ubuntu-latest 通过 `tsc --noEmit` + `vite build`
- [ ] `rust-lint` job 通过 `cargo fmt --check`、`cargo clippy -D warnings`、`cargo test --workspace --locked --lib --bins`
- [ ] `rust-check` job 在 Linux / Windows / macOS 三平台通过 `cargo check`
- [ ] `audit` job 容忍执行错误（`continue-on-error: true`）并输出 cargo / npm 审计结果

## mypass-core 集成测试（新增）
- [ ] `mypass-core-tests.yml` 在 ubuntu-latest 通过 `cargo test --workspace --locked`
- [ ] 包含 `crates/mypass-core/tests/vault_lifecycle_test.rs` 集成测试

## 浏览器扩展 CI（新增）
- [ ] `extension-ci.yml` 在 `extension/` 变更时构建原生 MV2 产物
- [ ] `extension-ci.yml` 在 `extensions/wxt/` 变更时构建 WXT 产物
- [ ] `npm ci`、`tsc --noEmit`、`wxt build` 全部 success
- [ ] 扩展产物作为 artifact 上传，保留 ≥ 7 天

## Qodana 静态分析（新增）
- [ ] `qodana_code_quality.yml` 与 `qodana.yaml` 中 linter 版本一致
- [ ] SARIF 与 HTML 报告均上传为 artifact
- [ ] PR 触发后 Qodana comment 出现在 Conversation tab

## CodeQL 安全扫描（新增）
- [ ] `codeql.yml` 矩阵覆盖 `javascript-typescript` 与 `rust`
- [ ] SARIF 自动上传到 Security tab
- [ ] `schedule.trigger` 设为 weekly

## Stale 管理（新增）
- [ ] 30 天无活动 issue/PR 被标记 stale
- [ ] 7 天后未跟进则关闭
- [ ] `dependencies`、`security` 标签豁免

## 发布与 Android（已有）
- [ ] `release.yml` 仅在 `v*` tag 或 `workflow_dispatch` 时构建
- [ ] 5 平台产物 + macOS Universal Binary 产出
- [ ] `android.yml` 产出 arm64-v8a / armeabi-v7a / x86_64 APK
- [ ] tag 触发时产出 AAB

## 依赖治理（已有）
- [ ] `dependabot.yml` 每周检查 `github-actions`、`cargo`、`npm`
- [ ] PR 限额 ≤ 10，分组按 tauri / serde / react / ui 聚合

## 代码质量门禁
- [ ] `pwsh scripts/validate-yaml.ps1` 通过
- [ ] 所有 workflow 包含 `concurrency` 组
- [ ] 所有 timeout 显式声明
- [ ] 无明文 secret 写入仓库