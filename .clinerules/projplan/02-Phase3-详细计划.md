# Phase 3: 浏览器扩展详细计划

> **目标时间**: Week 6-7
> **核心价值**: 实现核心差异化体验，打通浏览器边界

---

## 3.1 WXT 项目初始化 (P0)

### 项目结构
```
extensions/wxt/
├── src/
│   ├── main.ts              # 扩展入口
│   ├── background.ts        # 后台脚本（长连接）
│   ├── popup/App.tsx        # Popup UI
│   ├── content/
│   │   ├── detect.ts        # 表单识别
│   │   └── fill.ts          # 自动填存
│   └── utils/tauri.ts        # Tauri 通信
├── package.json
└── wxt.config.ts
```

### 初始化命令
```bash
cd E:/MyPass
npm create wxt@latest extensions/wxt
cd extensions/wxt
npm install @tauri-apps/api
```

---

## 3.2 Tauri 通信 (P0)

### 通信架构
```
┌─────────────────┐     WebSocket      ┌─────────────────┐
│   Browser       │ ◄──────────────►  │   Tauri Backend │
│   Extension     │                    │                 │
│   (Content+     │                    │  - 查重 API      │
│    Popup)       │                    │  - 填存 API      │
└─────────────────┘                    └─────────────────┘
```

### 核心接口
```typescript
export class TauriBridge {
  async searchEntries(query: string): Promise<Entry[]>
  async getAutoFillSuggestions(url: string): Promise<Entry[]>
  async saveNewCredential(data: CredentialData): Promise<void>
  async checkUnlocked(): Promise<boolean>
}
```

---

## 3.3 表单识别算法 (P1)

### 识别规则
| 表单类型 | 特征 | 识别逻辑 |
|----------|------|----------|
| 登录表单 | 1 个 password + 1+ 个 email/text | `input[type="password"]` 数量 === 1 |
| 注册表单 | 2 个 password | `input[type="password"]` 数量 === 2 |
| 改密表单 | 2-3 个 password | 包含 `change-password` 意图 |

### 算法
- 递归遍历 Shadow DOM
- 启发式规则识别表单类型
- 返回 username/password input 引用

---

## 3.4 秒填秒存流 (P1)

### 流程
1. 页面加载 → detectFormType()
2. 调用 TauriBridge.getAutoFillSuggestions(url)
3. 自动填充 username/password
4. 监听 form.submit → 捕获数据 → saveNewCredential()

---

**最后更新**: 2026-05-11