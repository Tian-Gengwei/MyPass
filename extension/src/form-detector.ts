// 表单检测模块
// 智能识别登录、注册、改密表单

export interface FormInputs {
  username: HTMLInputElement | null
  password: HTMLInputElement | null
  confirmPassword: HTMLInputElement | null
}

export type FormType = 'login' | 'register' | 'change-password' | 'unknown'

/**
 * 检测表单类型
 *
 * 策略：
 * - 登录：1 个 password + 附近有 email/text input
 * - 注册：2 个 password（确认框）
 * - 改密：2 个 password + 确认框
 */
export function detectFormType(form: HTMLFormElement): FormType {
  const inputs = findFormInputs(form)

  // 统计 password 数量
  const passwordCount = form.querySelectorAll('input[type="password"]').length

  if (passwordCount === 1) {
    if (inputs.username) {
      return 'login'
    }
  } else if (passwordCount === 2) {
    return 'change-password'
  } else if (passwordCount >= 3) {
    // 新注册通常有：原始密码、新密码、确认密码
    return 'register'
  }

  return 'unknown'
}

/**
 * 查找表单中的关键输入框
 */
export function findFormInputs(form: HTMLFormElement): FormInputs {
  const result: FormInputs = {
    username: null,
    password: null,
    confirmPassword: null,
  }

  const allInputs = form.querySelectorAll('input')

  allInputs.forEach(input => {
    // 跳过隐藏输入框
    if (input.offsetParent === null && input.type !== 'hidden') {
      return
    }

    // 用户名检测
    if (!result.username) {
      if (isUsernameInput(input)) {
        result.username = input
      }
    }

    // 密码检测
    if (input.type === 'password') {
      if (!result.password) {
        result.password = input
      } else if (!result.confirmPassword) {
        result.confirmPassword = input
      }
    }
  })

  // 递归查找 Shadow DOM
  if (!result.username || !result.password) {
    const shadowInputs = findShadowInputs(form)
    shadowInputs.forEach(input => {
      if (input.type === 'password') {
        if (!result.password) {
          result.password = input
        } else if (!result.confirmPassword) {
          result.confirmPassword = input
        }
      } else if (!result.username && isUsernameInput(input)) {
        result.username = input
      }
    })
  }

  return result
}

/**
 * 判断是否是用户名输入框
 */
function isUsernameInput(input: HTMLInputElement): boolean {
  const type = input.type?.toLowerCase()
  const autocomplete = input.getAttribute('autocomplete')?.toLowerCase()
  const name = input.name?.toLowerCase()
  const id = input.id?.toLowerCase()
  const placeholder = input.placeholder?.toLowerCase()

  // 类型检测
  if (type === 'email' || type === 'tel') {
    return true
  }

  // autocomplete 属性检测
  if (autocomplete) {
    if (autocomplete.includes('username') ||
        autocomplete.includes('email') ||
        autocomplete.includes('current-password')) {
      return true
    }
  }

  // name/id 检测
  if (name || id) {
    if (name?.includes('username') ||
        name?.includes('email') ||
        name?.includes('user') ||
        id?.includes('username') ||
        id?.includes('email') ||
        id?.includes('user')) {
      return true
    }
  }

  // placeholder 检测
  if (placeholder) {
    if (placeholder.includes('username') ||
        placeholder.includes('email') ||
        placeholder.includes('user')) {
      return true
    }
  }

  // 位置启发式：如果 password 前面有 input，可能是 username
  const passwordIndex = getElementIndex(input)
  if (type === 'password') {
    const previousInputs = Array.from(form.querySelectorAll('input'))
      .filter((el, i) => i < passwordIndex && el.offsetParent !== null)
    if (previousInputs.length === 1 && previousInputs[0].type === 'text') {
      return true
    }
  }

  return false
}

/**
 * 获取元素在兄弟节点中的索引
 */
function getElementIndex(el: Element): number {
  let index = 0
  let sibling = el.previousElementSibling
  while (sibling) {
    index++
    sibling = sibling.previousElementSibling
  }
  return index
}

/**
 * 递归查找 Shadow DOM 中的 input
 */
function findShadowInputs(root: Element | ShadowRoot): HTMLInputElement[] {
  const inputs: HTMLInputElement[] = []

  const allElements = root.querySelectorAll('*')
  allElements.forEach(el => {
    if (el.shadowRoot) {
      inputs.push(...findShadowInputs(el.shadowRoot))
    }
  })

  return inputs
}

/**
 * 查找页面上的所有表单
 */
export function findAllForms(root: Document | ShadowRoot): HTMLFormElement[] {
  const forms: HTMLFormElement[] = []

  root.querySelectorAll('form').forEach(form => {
    forms.push(form)
  })

  // 递归查找 Shadow DOM
  const allElements = root.querySelectorAll('*')
  allElements.forEach(el => {
    if (el.shadowRoot) {
      forms.push(...findAllForms(el.shadowRoot))
    }
  })

  return forms
}