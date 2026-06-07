//! 自动填存模块
//!
//! # 概述
//! 处理自动填充用户名和密码，以及捕获新凭证保存。

import { tauriBridge, type Entry } from '../utils/tauri';
import { getBestForm, type FormInfo } from './detect';

/**
 * 自动填充凭证
 */
export async function autoFill(entry: Entry): Promise<boolean> {
  const form = getBestForm();
  if (!form) return false;
  
  if (form.usernameInput) {
    form.usernameInput.value = entry.username;
    form.usernameInput.dispatchEvent(new Event('input', { bubbles: true }));
  }
  
  if (form.passwordInputs.length > 0) {
    form.passwordInputs[0].value = entry.password;
    form.passwordInputs[0].dispatchEvent(new Event('input', { bubbles: true }));
  }
  
  return true;
}

/**
 * 尝试自动填充当前页面
 */
export async function tryAutoFill(): Promise<boolean> {
  const url = window.location.hostname;
  const entries = await tauriBridge.getAutoFillSuggestions(url);
  
  if (entries.length === 0) return false;
  
  return autoFill(entries[0]);
}

/**
 * 监听表单提交，捕获新凭证
 */
export function setupFormListener(): void {
  // 监听所有表单提交
  document.addEventListener('submit', async (event) => {
    const form = event.target as HTMLFormElement;
    if (!form) return;
    
    const formInfo = analyzeFormSubmit(form);
    if (!formInfo) return;
    
    // 发送保存请求
    try {
      await tauriBridge.saveNewCredential(formInfo);
      console.log('Credential saved:', formInfo.name);
    } catch (err) {
      console.error('Failed to save credential:', err);
    }
  });
}

/**
 * 分析提交的表单数据
 */
function analyzeFormSubmit(form: HTMLFormElement): { name: string; username: string; password: string; url: string } | null {
  const formData = new FormData(form);
  
  // 获取用户名
  const usernameField = Array.from(form.querySelectorAll('input')).find(i => 
    i.type === 'text' || i.type === 'email' || i.type === 'tel'
  );
  
  // 获取密码
  const passwordField = form.querySelector('input[type="password"]');
  
  if (!usernameField || !passwordField) return null;
  
  const username = formData.get(usernameField.name || 'username')?.toString() || usernameField.value;
  const password = formData.get(passwordField.name || 'password')?.toString() || passwordField.value;
  
  if (!username || !password) return null;
  
  return {
    name: document.title || window.location.hostname,
    username,
    password,
    url: window.location.href,
  };
}