import { defineConfig } from 'wxt';

export default defineConfig({
  title: 'MyPass',
  description: 'MyPass Browser Extension',
  
  // 权限配置
  permissions: [
    'storage',
    'tabs',
    'nativeMessaging',
  ],
  
  // Tauri 应用 ID
  tauri: {
    devtools: true,
  },
  
  // 入口点
  entrypoints: {
    popup: './src/popup/main.tsx',
    background: './src/background.ts',
    content: './src/content/index.ts',
  },
});