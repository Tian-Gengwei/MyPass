// WXT 入口文件
// 扩展启动时运行

import { defineBackground } from 'wxt/background'

export default defineBackground({
  // 与 Tauri 桌面端建立长连接
  onStartup: async () => {
    console.log('MyPass extension starting')
    await setupLongConnection()
  },

  // 处理来自 content script 的消息
  onMessage: async (message, sender) => {
    switch (message.type) {
      case 'SAVE_CREDENTIAL':
        return await handleSaveCredential(message.data)
      case 'GET_CREDENTIALS':
        return await handleGetCredentials(message.data)
      case 'FILL_CREDENTIAL':
        return await handleFillCredential(message.data)
      default:
        console.warn('Unknown message type:', message.type)
    }
  },
})

// ========== 长连接管理 ==========

let wsConnection: WebSocket | null = null

async function setupLongConnection() {
  //连接到 Tauri 桌面端的 WebSocket 服务器
  // 注意：实际端口需要在 Tauri 端配置
  const wsUrl = 'ws://localhost:9312'

  try {
    wsConnection = new WebSocket(wsUrl)

    wsConnection.onopen = () => {
      console.log('Connected to MyPass desktop')
      // 心跳保活
      startHeartbeat()
    }

    wsConnection.onclose = () => {
      console.log('Disconnected from MyPass desktop')
      // 自动重连
      setTimeout(setupLongConnection, 5000)
    }

    wsConnection.onerror = (err) => {
      console.error('WebSocket error:', err)
    }

    wsConnection.onmessage = (event) => {
      handleDesktopMessage(event.data)
    }
  } catch (err) {
    console.error('Failed to connect to desktop:', err)
  }
}

function startHeartbeat() {
  setInterval(() => {
    if (wsConnection?.readyState === WebSocket.OPEN) {
      wsConnection.send(JSON.stringify({ type: 'PING' }))
    }
  }, 30000)
}

function handleDesktopMessage(data: string) {
  try {
    const message = JSON.parse(data)
    switch (message.type) {
      case 'NOTIFICATION':
        showNotification(message.title, message.body)
        break
      case 'CREDENTIAL_UPDATE':
        // 通知所有 tab 更新凭证缓存
        browser.tabs.query({}).then(tabs => {
          tabs.forEach(tab => {
            if (tab.id) {
              browser.tabs.sendMessage(tab.id, {
                type: 'CREDENTIAL_UPDATE',
                data: message.data
              })
            }
          })
        })
        break
    }
  } catch (err) {
    console.error('Failed to handle desktop message:', err)
  }
}

// ========== 消息处理 ==========

async function handleSaveCredential(data: {
  url: string
  username: string
  password: string
}) {
  if (wsConnection?.readyState === WebSocket.OPEN) {
    wsConnection.send(JSON.stringify({
      type: 'SAVE_CREDENTIAL',
      data
    }))
    return { success: true }
  }

  // 使用 native messaging 作为降级方案
  try {
    const response = await browser.runtime.sendNativeMessage(
      'com.mypass.app',
      { action: 'save_credential', ...data }
    )
    return response
  } catch (err) {
    console.error('Failed to save credential:', err)
    return { success: false, error: String(err) }
  }
}

async function handleGetCredentials(data: { url: string }) {
  if (wsConnection?.readyState === WebSocket.OPEN) {
    wsConnection.send(JSON.stringify({
      type: 'GET_CREDENTIALS',
      data
    }))

    //等待响应
    return new Promise((resolve) => {
      const timeout = setTimeout(() => {
        resolve({ success: false, error: 'Timeout' })
      }, 5000)

      wsConnection!.onmessage = (event) => {
        clearTimeout(timeout)
        const message = JSON.parse(event.data)
        if (message.type === 'CREDENTIALS') {
          resolve(message.data)
        }
      }
    })
  }

  return { success: false, error: 'Not connected' }
}

async function handleFillCredential(data: { entryId: string }) {
  // 转发给 content script
  const [tab] = await browser.tabs.query({ active: true, currentWindow: true })
  if (tab?.id) {
    return browser.tabs.sendMessage(tab.id, {
      type: 'FILL_CREDENTIAL',
      data
    })
  }
  return { success: false }
}

function showNotification(title: string, body: string) {
  // 使用 Web Extension API 显示通知
  browser.notifications.create({
    type: 'basic',
    iconUrl: 'icons/48.png',
    title,
    message: body
  })
}