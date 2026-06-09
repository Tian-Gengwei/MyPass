# MyPass 功能增强项目 - 实现计划

## \[ ] Task 1: 后端服务静默启动配置

- **Priority**: P0
- **Depends On**: None
- **Description**:
  - 修改 tauri.conf.json 配置，设置 Windows 平台静默启动
  - 确保 release 构建时不显示控制台窗口
- **Acceptance Criteria Addressed**: AC-1
- **Test Requirements**:
  - `programmatic` TR-1.1: 构建 release 版本后运行，确认无控制台窗口弹出
  - `human-judgment` TR-1.2: 检查配置文件是否正确设置
- **Notes**: 在 Tauri 中，通过 `windows_subsystem = "windows"` 属性实现静默启动

## \[ ] Task 2: Vault 选择功能实现

- **Priority**: P0
- **Depends On**: Task 4 (Vault 数据持久化)
- **Description**:
  - 创建 VaultSelector 组件，展示已有 Vault 列表
  - 实现创建新 Vault 功能
  - 实现选择/切换 Vault 功能
  - 修改 VaultGate 组件集成 VaultSelector
- **Acceptance Criteria Addressed**: AC-2
- **Test Requirements**:
  - `programmatic` TR-2.1: 验证 Vault 列表正确加载
  - `programmatic` TR-2.2: 验证创建新 Vault 后列表更新
  - `human-judgment` TR-2.3: 检查界面布局和交互体验
- **Notes**: 需要先实现 Vault 列表读取 API

## [x] Task 3: 国际化支持实现

- **Priority**: P1
- **Depends On**: None
- **Description**:
  - 集成 i18next 或类似国际化库
  - 创建中文和英文语言配置文件
  - 默认语言设置为简体中文
  - 实现语言切换功能
  - 更新所有 UI 组件使用国际化文本
- **Acceptance Criteria Addressed**: AC-3
- **Test Requirements**:
  - `human-judgment` TR-3.1: 检查默认语言是否为中文
  - `human-judgment` TR-3.2: 验证语言切换功能正常工作
  - `human-judgment` TR-3.3: 检查所有文本元素是否可翻译
- **Notes**: 需要更新前端组件，确保所有文本都通过 i18n 框架

## [x] Task 4: Vault 访问机制修复

- **Priority**: P0
- **Depends On**: None
- **Description**:
  - 实现后端 API 获取 Vault 列表
  - 修改前端状态管理，保存当前 Vault 信息
  - 确保关闭应用后重新启动仍能访问已创建的 Vault
- **Acceptance Criteria Addressed**: AC-4
- **Test Requirements**:
  - `programmatic` TR-4.1: 创建 Vault 后重启应用，验证 Vault 仍在列表中
  - `programmatic` TR-4.2: 解锁 Vault 后关闭再打开，验证可重新解锁
- **Notes**: 需要在后端添加 list\_vaults API

## \[ ] Task 5: 导入/导出功能完善

- **Priority**: P1
- **Depends On**: None
- **Description**:
  - 创建导入/导出 UI 组件
  - 实现多种格式支持（CSV、JSON、Bitwarden、KeePass）
  - 添加成功/失败反馈提示
  - 集成到菜单系统
- **Acceptance Criteria Addressed**: AC-5
- **Test Requirements**:
  - `human-judgment` TR-5.1: 检查导入/导出界面是否完善
  - `human-judgment` TR-5.2: 验证操作反馈是否明确
- **Notes**: 后端 API 已存在，需要完善前端 UI

## [x] Task 6: 菜单系统实现

- **Priority**: P1
- **Depends On**: Task 3 (国际化), Task 5 (导入导出)
- **Description**:
  - 创建顶部菜单栏组件
  - 实现文件、编辑、视图、工具、帮助菜单
  - 关联菜单操作与应用功能
  - 支持快捷键提示
- **Acceptance Criteria Addressed**: AC-6
- **Test Requirements**:
  - `human-judgment` TR-6.1: 检查菜单结构是否符合 Bitwarden 风格
  - `human-judgment` TR-6.2: 验证菜单项功能关联正确
- **Notes**: 参照 Bitwarden 界面设计，包含标准菜单选项

  <br />

