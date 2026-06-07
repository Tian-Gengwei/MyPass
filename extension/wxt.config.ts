import { defineConfig } from 'wxt'

export default defineConfig({
  entrypointDir: 'src',
  manifest: {
    name: 'MyPass',
    version: '0.1.0',
    description: 'Cross-platform password manager - auto-fill and auto-save',
    permissions: ['activeTab', 'storage', 'nativeMessaging'],
    host_permissions: ['<all_urls>'],
    action: {
      default_popup: 'popup.html',
      default_icon: {
        '16': 'icons/16.png',
        '32': 'icons/32.png',
        '48': 'icons/48.png',
      }
    },
    background: {
      service_worker: 'background.js',
      type: 'module'
    },
    content_scripts: [
      {
        matches: ['<all_urls>'],
        js: ['content.js'],
        run_at: 'document_idle'
      }
    ]
  },
  modules: ['@wxt/module-react'],
  compatibility: {
    chromium: 120,
    firefox: 120
  }
})