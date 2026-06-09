# MyPass CI/CD 推送指引

本目录汇总新增的 CI/CD 规范与 workflow 文件，并提供推送至 GitHub 前的所有检查清单与命令。

## 1. 本次新增 / 涉及文件

### 规范文档
- `.trae/specs/cicd-pipeline/spec.md` — CI/CD 规范
- `.trae/specs/cicd-pipeline/tasks.md` — 实施计划
- `.trae/specs/cicd-pipeline/checklist.md` — 验证清单
- `.trae/specs/cicd-pipeline/PUSH.md` — 本文档

### 新增 Workflow
- `.github/workflows/mypass-core-tests.yml` — mypass-core 集成测试
- `.github/workflows/extension-ci.yml` — 双轨浏览器扩展 CI
- `.github/workflows/qodana_code_quality.yml` — Qodana 静态分析
- `.github/workflows/codeql.yml` — CodeQL 安全扫描
- `.github/workflows/stale.yml` — Stale 管理
- `.github/workflows/docs.yml` — 文档 Markdown 检查

### 既有但未提交到 Git 的 Workflow
> ⚠️ `git status` 显示整个 `.github/` 目录在当前仓库中尚未被追踪，
> 推送时必须一并加入 stage。

- `.github/workflows/ci.yml`
- `.github/workflows/android.yml`
- `.github/workflows/release.yml`
- `.github/dependabot.yml`
- `.github/ISSUE_TEMPLATE/bug_report.md`
- `.github/ISSUE_TEMPLATE/feature_request.md`
- `.github/PULL_REQUEST_TEMPLATE.md`

## 2. 推送前本地校验

### 2.1 YAML 语法校验
```powershell
# 需要先安装 powershell-yaml 模块：
# Install-Module -Name powershell-yaml -Scope CurrentUser
pwsh -File scripts/validate-yaml.ps1
```

或者使用 Python（无需额外安装）：

```bash
python -c "
import yaml, sys, glob
files = glob.glob('.github/workflows/*.yml') + ['.github/dependabot.yml']
ok = 0
for f in files:
    try:
        with open(f, encoding='utf-8') as fp:
            yaml.safe_load(fp)
        print(f'[OK]   {f}')
        ok += 1
    except yaml.YAMLError as e:
        print(f'[FAIL] {f}: {e}')
        sys.exit(1)
print(f'Validated {ok}/{len(files)} files')
"
```

✅ 当前所有 10 个 YAML 文件均通过校验。

### 2.2 快速预览新增 workflow
```powershell
code .github/workflows/mypass-core-tests.yml
code .github/workflows/extension-ci.yml
code .github/workflows/qodana_code_quality.yml
code .github/workflows/codeql.yml
code .github/workflows/stale.yml
code .github/workflows/docs.yml
```

## 3. Git 提交与推送

### 3.1 单次提交（推荐）

```powershell
# 1. 暂存所有 CI/CD 变更
git add .github/
git add .trae/specs/cicd-pipeline/

# 2. 预览变更
git diff --cached --stat

# 3. 提交
git commit -m "ci: add mypass-core tests, extension CI, Qodana, CodeQL, stale, docs workflows

- mypass-core-tests.yml: workspace-level cargo test incl. vault_lifecycle_test
- extension-ci.yml: dual-track (extension/ + extensions/wxt/) build
- qodana_code_quality.yml: invoke qodana-jvm:2026.1 per qodana.yaml
- codeql.yml: TypeScript + Rust security scanning
- stale.yml: 30d issue / 45d PR mark stale, exemptions for deps/security
- docs.yml: markdownlint-cli2 over tools/docs and top-level docs

Ref: .trae/specs/cicd-pipeline/spec.md"

# 4. 推送（按需选择分支）
git push origin Dev1
```

### 3.2 拆分为多个提交（可选）

```powershell
# 第一批：CI 主体（保留既有内容并加固）
git add .github/workflows/ci.yml .github/workflows/android.yml .github/workflows/release.yml
git add .github/dependabot.yml
git commit -m "ci: import existing workflows into git tracking"

# 第二批：核心测试与扩展
git add .github/workflows/mypass-core-tests.yml .github/workflows/extension-ci.yml
git commit -m "ci: add mypass-core integration tests and browser extension CI"

# 第三批：代码质量与安全
git add .github/workflows/qodana_code_quality.yml .github/workflows/codeql.yml
git commit -m "ci: add Qodana static analysis and CodeQL security scanning"

# 第四批：仓库治理与文档
git add .github/workflows/stale.yml .github/workflows/docs.yml
git commit -m "ci: add stale management and docs lint workflows"

# 第五批：规范文档
git add .trae/specs/cicd-pipeline/
git commit -m "docs: add CI/CD pipeline specification and implementation plan"
```

## 4. GitHub 端配置（合并后）

| 项 | 说明 |
|----|------|
| **Secrets** | 在 Settings → Secrets and variables → Actions 维护 `TAURI_SIGNING_PRIVATE_KEY`、`TAURI_SIGNING_PRIVATE_KEY_PASSWORD`、`APPLE_SIGNING_IDENTITY`、`APPLE_ID`、`APPLE_PASSWORD`、`APPLE_TEAM_ID`、`WINDOWS_CERT_FILE`、`WINDOWS_CERT_PASSWORD`。可选：增加 `QODANA_TOKEN`（社区版免费 token）以启用 Qodana 上传。 |
| **Branch protection** | `Dev1`、`main`、`develop` 三分支建议设为 Required: `CI / rust-lint`、`CI / frontend`、`MyPass Core Tests / integration-tests`、`Extension CI / native-extension`、`Extension CI / wxt-extension` 通过后才允许合并。 |
| **GitHub Pages** | 不需要，文档为 Markdown 直读。 |
| **Code scanning** | CodeQL 首次运行后会自动启用 Security tab，可在 Settings → Code security 中配置默认告警阈值。 |
| **Dependabot** | 保持现有配置即可，CI 中 `audit` 容忍 transient 失败。 |

## 5. 上线后回滚预案

如发现某个 workflow 误报或失败导致 main 分支无法合并：

```powershell
# 临时禁用某个 workflow：在文件顶端添加
# if: false
# 然后重新提交、推送

# 或在 GitHub UI: Actions → 选中 workflow → "..." → Disable workflow
```

如需整体回退本次新增 workflow：

```powershell
git revert <commit-sha>
git push origin Dev1
```

## 6. 监控要点

合并后请观察 1-2 个 PR cycle：

1. **CI 总时长** — 若 `mypass-core-tests` + `extension-ci` + `ci.yml` 总计 > 15 分钟，考虑启用 GitHub Actions cache 调优或拆分 `paths` 过滤。
2. **CodeQL 首次跑** — 会下载查询包，可能 5-10 分钟，之后走缓存。
3. **Qodana 首次跑** — 需要拉 jetbrains/qodana-jvm 镜像，约 2 GB。
4. **Stale 行为** — schedule 默认 UTC 02:00，注意在白天不要立即标记活跃 issue。

---

**责任人**: MyPass 维护者
**目标分支**: `Dev1`（建议先合并到此观察 1-2 个 PR 后再 cherry-pick 到 `main`）
**预计总 PR 数**: 1（聚合）或 5（拆分），见 3.1 / 3.2