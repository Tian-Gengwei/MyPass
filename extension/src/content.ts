// Content Script - 运行在网页上下文中
// 负责表单检测和自动填存

import { detectFormType, findFormInputs } from './form-detector'

// 监听来自 background script 的消息
browser.runtime.onMessage.addListener((message, sender, sendResponse) => {
  switch (message.type) {
    case 'FILL_CREDENTIAL':
      fillCredential(message.data)
      sendResponse({ success: true })
      break
    case 'CREDENTIAL_UPDATE':
      // 可以选择重新检测表单
      break
  }
  return true
})

// 页面加载时检测表单
detectPageForms()

function detectPageForms() {
  // 递归查找所有表单（包括 Shadow DOM）
  const forms = findAllForms(document)
  forms.forEach(form => {
    const formType = detectFormType(form)
    const inputs = findFormInputs(form)

    if (inputs.password) {
      // 标记这是一个密码相关的表单
      form.dataset.mypassForm = formType
      form.dataset.mypassEntryId = inputs.entryId || ''

      // 监听表单提交
      form.addEventListener('submit', handleFormSubmit)
    }
  })
}

function findAllForms(root: Document | ShadowRoot): HTMLFormElement[] {
  const forms: HTMLFormElement[] = []
  const elements = root.querySelectorAll('form')

  elements.forEach(form => forms.push(form))

  // 递归查找 Shadow DOM
  const allElements = root.querySelectorAll('*')
  allElements.forEach(el => {
    if (el.shadowRoot) {
      forms.push(...findAllForms(el.shadowRoot))
    }
  })

  return forms
}

async function handleFormSubmit(event: Event) {
  const form = event.target as HTMLFormElement
  event.preventDefault()

  const formType = form.dataset.mypassForm
  const inputs = findFormInputs(form)

  if (!inputs.username || !inputs.password) {
    return
  }

  const url = window.location.origin
  const username = (inputs.username as HTMLInputElement).value
  const password = (inputs.password as HTMLInputElement).value

  // 检查是否已有此网站的凭证
  const existingCredentials = await checkExistingCredentials(url, username)

  if (existingCredentials) {
    // 更新现有凭证
    if (formType === 'change-password') {
      // 改密场景：只更新密码
      await browser.runtime.sendMessage({
        type: 'SAVE_CREDENTIAL',
        data: { url, username, password, action: 'update-password' }
      })
    } else {
      // 普通登录：更新时间戳
      await browser.runtime.sendMessage({
        type: 'SAVE_CREDENTIAL',
        data: { url, username, password, action: 'update-login' }
      })
    }
  } else {
    // 新凭证：显示保存提示
    showSavePrompt(url, username, password)
  }
}

async function checkExistingCredentials(url: string, username: string) {
  try {
    const response = await browser.runtime.sendMessage({
      type: 'GET_CREDENTIALS',
      data: { url, username }
    })
    return response.success ? response.data : null
  } catch {
    return null
  }
}

function showSavePrompt(url: string, username: string, password: string) {
  // 创建保存提示弹窗
  const popup = document.createElement('div')
  popup.id = 'mypass-save-popup'
  popup.innerHTML = `
    <div class="mypass-popup-content">
      <div class="mypass-popup-header">Save login for ${new URL(url).hostname}?</div>
      <div class="mypass-popup-body">
        <div class="mypass-popup-field">
          <label>Username</label>
          <span>${username}</span>
        </div>
        <div class="mypass-popup-field">
          <label>Password</label>
          <span>${'*'.repeat(8)}</span>
        </div>
      </div>
      <div class="mypass-popup-actions">
        <button id="mypass-save-btn">Save</button>
        <button id="mypass-dismiss-btn">Dismiss</button>
      </div>
    </div>
  `

  // 添加样式
  const style = document.createElement('style')
  style.textContent = `
    #mypass-save-popup {
      position: fixed;
      top: 20px;
      right: 20px;
      z-index: 999999;
      background: white;
      border: 1px solid #e0e0e0;
      border-radius: 8px;
      box-shadow: 0 4px 12px rgba(0,0,0,0.15);
      font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
      max-width: 320px;
    }
    .mypass-popup-content { padding: 16px; }
    .mypass-popup-header { font-weight: 600; margin-bottom: 12px; }
    .mypass-popup-field { display: flex; justify-content: space-between; margin-bottom: 8px; font-size: 14px; }
    .mypass-popup-field label { color: #666; }
    .mypass-popup-actions { display: flex; gap: 8px; margin-top: 16px; }
    .mypass-popup-actions button { flex: 1; padding: 8px 12px; border-radius: 4px; border: none; cursor: pointer; }
    #mypass-save-btn { background: #0066cc; color: white; }
    #mypass-dismiss-btn { background: #f0f0f0; }
  `
  popup.appendChild(style)

  document.body.appendChild(popup)

  //绑定按钮事件
  popup.querySelector('#mypass-save-btn')?.addEventListener('click', async () => {
    await browser.runtime.sendMessage({
      type: 'SAVE_CREDENTIAL',
      data: { url, username, password }
    })
    popup.remove()
  })

  popup.querySelector('#mypass-dismiss-btn')?.addEventListener('click', () => {
    popup.remove()
  })

  //5 秒后自动消失
  setTimeout(() => popup.remove(), 5000)
}

function fillCredential(data: { username: string; password: string }) {
  // 找到当前聚焦的表单
  const activeForm = document.activeElement?.closest('form') as HTMLFormElement | null
  if (!activeForm) return

  const inputs = findFormInputs(activeForm)

  if (inputs.username) {
    (inputs.username as HTMLInputElement).value = data.username
    inputs.username.dispatchEvent(new Event('input', { bubbles: true }))
  }

  if (inputs.password) {
    (inputs.password as HTMLInputElement).value = data.password
    inputs.password.dispatchEvent(new Event('input', { bubbles: true }))
  }
}

// 导出给测试用
export { findAllForms, findFormInputs }