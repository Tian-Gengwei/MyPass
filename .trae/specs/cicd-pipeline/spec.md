# MyPass CI/CD 流水线规范

## Overview
- **Summary**: 基于 MyPass 现有 `.github/workflows/`、`dependabot.yml`、`qodana.yaml`、`scripts/validate-yaml.ps1` 等内容，统一规划并补齐 CI/CD 流水线，覆盖 Rust 工作区、前端、浏览器扩展、Android 端、桌面端发布、安全审计、依赖治理与代码质量门禁。
- **Purpose**: 为 `Dev1` / `main` / `develop` 分支及 `v*` tag 提供可重复、可观测、低成本的 CI/CD 流水线，保证每次合并前都通过自动化门禁，每个版本都按可预期产物交付。
- **Target Users**: MyPass 维护者、贡献者、Release Manager。

## Goals
- 复用现有 `ci.yml` / `android.yml` / `release.yml` 的执行单元，仅在缺失处补齐新 workflow。
- 覆盖工作区中所有可交付物：Rust 工作区、前端 SPA、双轨浏览器扩展、Android APK/AAB、桌面多平台安装包。
- 把 `qodana.yaml`、Cargo 审计、npm 审计、CodeQL 接入主流水线。
- 维护依赖自动更新（`dependabot.yml`）与 stale PR 处理。
- 所有工作流通过 `scripts/validate-yaml.ps1` 的 YAML 语法校验。

## Non-Goals (Out of Scope)
- 不修改任何应用业务逻辑代码。
- 不引入新的 CI 平台（CircleCI、Drone 等）。
- 不在 CI 中执行 GUI 端到端测试（保留为人工验收）。
- 不在公开仓库中保存签名密钥（继续使用 GitHub Secrets）。

## Background & Context
- 仓库根目录是 Cargo workspace（`crates/mypass-core` + `src-tauri`），Rust 2021 edition，`forbid(unsafe_code)`。
- 前端位于 `frontend/`（React 18 + Vite + Tailwind）。
- 浏览器扩展存在双轨：`extension/`（原生 MV2）与 `extensions/wxt/`（WXT 框架）。
- Android 通过 `cargo tauri android` 构建；NDK 26.1.10909125，API 34。
- 桌面端通过 `cargo tauri build` 输出 `app/dmg/msi/nsis/deb/rpm/appimage`。
- 现有 `qodana.yaml` 已配置 `jetbrains/qodana-<linter>:2026.1`，但尚未被任何 workflow 调用。
- `scripts/validate-yaml.ps1` 已存在，本地可作 YAML lint 入口。
- 现有 `dependabot.yml` 已覆盖 `github-actions` / `cargo`（`/src-tauri`）/ `npm`（`/frontend`）。

## Functional Requirements

### FR-1 保留并加固主 CI（基于 `ci.yml`）
- 触发：`push` / `pull_request` 到 `Dev1`、`main`、`develop`，以及 `workflow_dispatch`。
- Job 集合：`frontend`（tsc + vite build）、`rust-lint`（fmt + clippy + lib/bin 单元测试）、`rust-check`（Linux/Windows/macOS 三平台 `cargo check`）、`audit`（cargo-audit + npm audit）。
- 并发：保留 `concurrency.cancel-in-progress`，避免同一 PR 重复构建。

### FR-2 补齐 mypass-core 集成测试
- 现有 `crates/mypass-core/tests/vault_lifecycle_test.rs` 未在任何 workflow 中运行。
- 新增 workflow 在 `ubuntu-latest` 上运行 `cargo test --workspace --locked`（含 mypass-core 集成测试），与 `ci.yml` 的 `rust-lint` 形成分层。

### FR-3 补齐浏览器扩展 CI
- `extension/`：`npm ci` + `npx tsc --noEmit` + `wxt build` → 上传 `.output/` 制品。
- `extensions/wxt/`：`npm ci` + `npx tsc --noEmit` + `wxt build` → 上传 `.output/` 制品。
- 仅在对应目录变更或 workflow 自身变更时触发（`paths` 过滤）。

### FR-4 启用 Qodana 静态分析
- 基于 `qodana.yaml` 中的 `linter: jetbrains/qodana-<linter>:2026.1`。
- 与 `qodana.starter` profile 配合，失败条件可后续在 `qodana.yaml` 中按需启用 `severityThresholds`。
- 触发：PR 与 `push` 到受保护分支。

### FR-5 启用 CodeQL 安全扫描
- 覆盖 JavaScript/TypeScript（前端 + 扩展）与 Rust（src-tauri、crates）。
- 默认 schedule：每周一次 + PR 触发。

### FR-6 文档构建检查
- 仓库中包含 `tools/docs/`（含 `README.md`、`api.md`、`checks.md`、`config.md`、`install.md`、`reference.md`、`usage.md`）。
- 仅在文档变更时执行 Markdown lint（可选 `markdownlint-cli2`）与死链检查；若未来引入 mdbook，再扩展构建步骤。
- 现阶段以 lint 为主，避免引入额外重量级工具。

### FR-7 Stale 与 PR 标签管理
- `stale.yml`：30 天无活动标记 stale，7 天后关闭；仅对 issue / PR 生效，豁免 `dependencies`、`security` 标签。
- `labeler.yml`（可选）：按 `src-tauri/**` → `rust`、`frontend/**` → `frontend`、`extension/**` → `extension` 自动打标签。

### FR-8 复用现有发布与 Android 流水线
- `release.yml`、`android.yml` 保持现状，仅在末尾添加注释指明其依赖的 FR-1 / FR-3 必须先绿。

## Non-Functional Requirements
- **NFR-1**：所有新增 workflow 必须包含 `concurrency` 组，避免对同一 ref 的重复运行。
- **NFR-2**：所有 workflow 文件必须能通过 `scripts/validate-yaml.ps1` 解析。
- **NFR-3**：依赖缓存（`actions/setup-node` 的 `cache: 'npm'`、`Swatinem/rust-cache@v2`）继续作为默认配置。
- **NFR-4**：默认 timeout：lint/test job ≤ 30 分钟，build job ≤ 90 分钟，Android ≤ 90 分钟。
- **NFR-5**：所有执行 secret（`TAURI_SIGNING_PRIVATE_KEY`、`APPLE_*`、`WINDOWS_CERT_*`）维持原状，文档中显式列出但不写入文件。
- **NFR-6**：与 `dependabot.yml` 协调：CI 中的 `cargo audit` / `npm audit --audit-level=high` 容忍 transient 失败（`continue-on-error: true`），实际合并由 Dependabot 自身的 PR 校验承担。

## Constraints
- **Technical**: GitHub Actions（Ubuntu/macOS/Windows runners）、Node 24、Rust stable、Tauri CLI 2.x。
- **Business**: 公开仓库，免费 runner 额度可控；Android 使用 `ubuntu-22.04`。
- **Dependencies**: 必须存在 `frontend/package-lock.json`、`extension/package-lock.json`、`extensions/wxt/package-lock.json` 才可使用 `cache-dependency-path`；否则回退到 `npm install`。

## Assumptions
- `Dev1`、`main`、`develop` 三个分支持续存在并受保护。
- `v*` tag 遵循语义化版本（`v0.1.0`、`v1.2.3` 等）。
- 维护者有能力在 Settings → Secrets 中维护签名凭据。

## Acceptance Criteria

### AC-1 主 CI 全绿
- **Given**: 开发者向 `Dev1` 推送 PR
- **When**: `ci.yml` 触发
- **Then**: `frontend`、`rust-lint`、`rust-check`（3 OS）、`audit` 全部 success
- **Verification**: `programmatic`

### AC-2 浏览器扩展 CI 全绿
- **Given**: 改动 `extension/**` 或 `extensions/wxt/**`
- **When**: 对应 workflow 触发
- **Then**: TypeScript 检查与 `wxt build` 通过，扩展制品上传
- **Verification**: `programmatic`

### AC-3 mypass-core 集成测试通过
- **Given**: 改动 `crates/mypass-core/**`
- **When**: `mypass-core-tests.yml` 触发
- **Then**: `cargo test --workspace --locked` 全绿
- **Verification**: `programmatic`

### AC-4 Qodana 报告生成
- **Given**: 任意 PR 打开
- **When**: `qodana_code_quality.yml` 触发
- **Then**: Qodana 检查执行，报告以 SARIF + artifact 形式保存
- **Verification**: `programmatic`

### AC-5 CodeQL 扫描通过
- **Given**: 任意 PR 或 weekly schedule
- **When**: `codeql.yml` 触发
- **Then**: TypeScript / Rust 两语言均产出 SARIF
- **Verification**: `programmatic`

### AC-6 YAML 校验
- **Given**: 开发者修改任意 `.github/workflows/*.yml`
- **When**: 本地运行 `pwsh scripts/validate-yaml.ps1`
- **Then**: 所有文件解析成功，输出 `[OK]`
- **Verification**: `programmatic`

### AC-7 桌面发布仅在 tag 上发布
- **Given**: 推送 `v*` tag
- **When**: `release.yml` 触发
- **Then**: 5 平台 + macOS Universal 产出，draft GitHub Release 创建
- **Verification**: `human-judgment`

## Open Questions
- [ ] 是否在 `qodana.yaml` 中启用 `failureConditions.severityThresholds` 把 Qodana 升级为硬门禁？
- [ ] CodeQL 是否对 `extension/`（MV2）与 `extensions/wxt/` 同时启用？
- [ ] 是否需要单独的 `docs.yml` 构建 mdbook（当前仅有 Markdown）？

## Pipeline Topology

```
push / PR (Dev1, main, develop)
        │
        ├── ci.yml                 (frontend, rust-lint, rust-check x3 OS, audit)
        ├── mypass-core-tests.yml  (cargo test --workspace incl. integration tests)
        ├── extension-ci.yml       (extension/ + extensions/wxt/ 双轨构建)
        ├── qodana_code_quality.yml (静态分析)
        └── codeql.yml             (TypeScript + Rust 安全扫描)

push tag v*
        ├── android.yml            (APK per arch + AAB on tag)
        └── release.yml            (5 平台桌面 + Universal Binary + GH Release)

schedule / dependency ecosystem
        ├── dependabot.yml         (actions / cargo / npm)
        └── stale.yml              (issue / PR 治理)
```