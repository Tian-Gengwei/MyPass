//! 表单识别模块
//!
//! # 概述
//! 检测网页中的登录、注册、密码修改等表单，
//! 识别用户名和密码输入框。

export type FormType = 'login' | 'register' | 'change_password' | 'unknown';

/** 表单信息 */
export interface FormInfo {
  type: FormType;
  usernameInput: HTMLInputElement | null;
  passwordInputs: HTMLInputElement[];
  form: HTMLFormElement | null;
  url: string;
}

/**
 * 递归遍历 DOM（包括 Shadow DOM）
 */
function traverseDOM(root: Element | ShadowRoot, callback: (el: Element) => void): void {
  callback(root);
  const shadowRoot = (root as Element).shadowRoot;
  if (shadowRoot) {
    traverseDOM(shadowRoot, callback);
  }
  for (const child of root.children) {
    traverseDOM(child, callback);
  }
}

/**
 * 检测所有表单
 */
export function detectForms(): FormInfo[] {
  const forms: FormInfo[] = [];
  
  traverseDOM(document, (el) => {
    if (el.tagName === 'FORM') {
      const formInfo = analyzeForm(el as HTMLFormElement);
      if (formInfo) {
        forms.push(formInfo);
      }
    }
  });
  
  return forms;
}

/**
 * 分析单个表单
 */
function analyzeForm(form: HTMLFormElement): FormInfo | null {
  const inputs = Array.from(form.querySelectorAll('input'));
  const passwordInputs = inputs.filter(i => i.type === 'password');
  const textInputs = inputs.filter(i => i.type === 'text' || i.type === 'email' || i.type === 'tel');
  const usernameInput = textInputs.find(i => {
    const name = i.name?.toLowerCase() || '';
    const id = i.id?.toLowerCase() || '';
    const placeholder = i.placeholder?.toLowerCase() || '';
    return name.includes('user') || name.includes('email') || 
           id.includes('user') || id.includes('email') ||
           placeholder.includes('user') || placeholder.includes('email');
  }) || textInputs[0];
  
  const passwordCount = passwordInputs.length;
  
  let type: FormType;
  if (passwordCount === 1 && usernameInput) {
    type = 'login';
  } else if (passwordCount === 2) {
    type = 'register';
  } else if (passwordCount >= 2 && form.innerHTML.includes('change')) {
    type = 'change_password';
  } else {
    type = 'unknown';
  }
  
  return {
    type,
    usernameInput: type === 'login' ? usernameInput || null : null,
    passwordInputs,
    form,
    url: window.location.origin,
  };
}

/**
 * 获取当前页面最佳表单
 */
export function getBestForm(): FormInfo | null {
  const forms = detectForms();
  // 优先返回登录表单
  return forms.find(f => f.type === 'login') || forms[0] || null;
}