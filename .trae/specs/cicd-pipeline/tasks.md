# MyPass CI/CD 流水线实施计划

> **前提**: 本计划假设工程师已熟悉 GitHub Actions YAML 语法、Rust workspace 概念与 Tauri 构建流程。
> 所有 workflow 文件均放置在 `.github/workflows/` 下，YAML 语法由 `scripts/validate-yaml.ps1` 验证。

## [ ] Task 1: 本地 YAML 校验脚本扩展

- **Priority**: P0
- **Depends On**: None
- **Description**:
  - 在 `scripts/validate-yaml.ps1` 中追加对新增 workflow 文件名（`mypass-core-tests.yml`、`extension-ci.yml`、`qodana_code_quality.yml`、`codeql.yml`、`stale.yml`、`docs.yml`）的预声明，便于本地 CI 校验。
- **Files**:
  - Modify: `scripts/validate-yaml.ps1`
- **Acceptance Criteria Addressed**: AC-6
- **Verification**: `pwsh scripts/validate-yaml.ps1` 在未创建新文件时不报错（仅扫描存在的文件）。
- **Notes**: PowerShell 中 `Get-ChildItem` 默认按 glob 自动发现文件，因此本任务仅作为「未来文件创建后立即可被校验」的保障，可保留为可选提交。

## [ ] Task 2: 创建 mypass-core 集成测试 workflow

- **Priority**: P0
- **Depends On**: None
- **Description**:
  - 新增 `.github/workflows/mypass-core-tests.yml`，在 `ubuntu-latest` 上运行 workspace 级别 `cargo test`，覆盖 `crates/mypass-core/tests/`。
- **Files**:
  - Create: `.github/workflows/mypass-core-tests.yml`
- **Acceptance Criteria Addressed**: AC-3
- **Verification**: 推送到 PR 后 Actions 页面应显示 `mypass-core-tests` job success。

## [ ] Task 3: 创建浏览器扩展 CI workflow

- **Priority**: P0
- **Depends On**: None
- **Description**:
  - 新增 `.github/workflows/extension-ci.yml`，分别构建 `extension/` 与 `extensions/wxt/` 双轨产物。
- **Files**:
  - Create: `.github/workflows/extension-ci.yml`
- **Acceptance Criteria Addressed**: AC-2
- **Verification**: PR 修改任一扩展目录后，触发 `extension-ci` job，TypeScript 检查与 `wxt build` 均 success，artifact 上传。

## [ ] Task 4: 创建 Qodana 静态分析 workflow

- **Priority**: P1
- **Depends On**: None
- **Description**:
  - 新增 `.github/workflows/qodana_code_quality.yml`，调用 `jetbrains/qodana-code`（与 `qodana.yaml` 中 linter 对齐）。
- **Files**:
  - Create: `.github/workflows/qodana_code_quality.yml`
- **Acceptance Criteria Addressed**: AC-4
- **Verification**: PR 打开后 Qodana 报告以 SARIF + artifact 形式生成。

## [ ] Task 5: 创建 CodeQL 安全扫描 workflow

- **Priority**: P1
- **Depends On**: None
- **Description**:
  - 新增 `.github/workflows/codeql.yml`，矩阵覆盖 `javascript-typescript` 与 `rust`。
- **Files**:
  - Create: `.github/workflows/codeql.yml`
- **Acceptance Criteria Addressed**: AC-5
- **Verification**: SARIF 上传到 Security tab。

## [ ] Task 6: 创建 Stale 管理 workflow

- **Priority**: P2
- **Depends On**: None
- **Description**:
  - 新增 `.github/workflows/stale.yml`，使用 `actions/stale@v9`，豁免 `dependencies`、`security` 标签。
- **Files**:
  - Create: `.github/workflows/stale.yml`
- **Verification**: 手动创建 35 天前无活动的 issue，验证被自动标记与关闭。

## [ ] Task 7: 整合 SPEC 文档与 README 引用

- **Priority**: P2
- **Depends On**: Task 2,3,4,5,6
- **Description**:
  - 在 `.trae/specs/cicd-pipeline/spec.md` 中补充 "Pipeline Topology" 章节（已在初稿中包含），并在 `README.md` 的「构建/发布」小节追加指向 `.trae/specs/cicd-pipeline/` 的链接。
- **Files**:
  - Modify: `README.md`
- **Verification**: 渲染后的 `README.md` 中包含 `docs/cicd/` 链接。

## [ ] Task 8: 推送至 GitHub 前的本地校验

- **Priority**: P0
- **Depends On**: Task 2,3,4,5,6
- **Description**:
  - 运行 `pwsh scripts/validate-yaml.ps1`，确认所有 YAML 文件 `[OK]`。
  - 运行 `git diff --stat .github/workflows/` 查看变更范围。
- **Files**: 无
- **Verification**: 校验脚本输出全 `[OK]`，diff 仅包含预期的 `.github/workflows/*.yml` 与 `scripts/validate-yaml.ps1`。