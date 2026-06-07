//! 后台脚本
//!
//! # 概述
//! 管理扩展生命周期，处理后台任务，维护与 Tauri 的长连接。

import { tauriBridge } from './utils/tauri';

// 监听来自 content script 的消息
browser.runtime.onMessage.addListener((message, sender, sendResponse) => {
  handleMessage(message, sender)
    .then(sendResponse)
    .catch(err => sendResponse({ error: err.message }));
  return true; // 异步响应
});

/** 处理消息 */
async function handleMessage(message: Message, sender: browser.runtime.MessageSender): Promise<Response> {
  switch (message.type) {
    case 'GET_ENTRIES':
      return { entries: await tauriBridge.getEntries() };
      
    case 'SEARCH':
      return { entries: await tauriBridge.searchEntries(message.query) };
      
    case 'AUTO_FILL':
      return { entries: await tauriBridge.getAutoFillSuggestions(message.url) };
      
    case 'SAVE_CREDENTIAL':
      return { entry: await tauriBridge.saveNewCredential(message.data) };
      
    case 'CHECK_UNLOCKED':
      return { unlocked: await tauriBridge.checkUnlocked() };
      
    default:
      throw new Error(`Unknown message type: ${message.type}`);
  }
}

/** 消息类型 */
type Message = {
  type: 'GET_ENTRIES' | 'SEARCH' | 'AUTO_FILL' | 'SAVE_CREDENTIAL' | 'CHECK_UNLOCKED';
  query?: string;
  url?: string;
  data?: {
    name: string;
    username: string;
    password: string;
    url?: string;
    notes?: string;
  };
};

/** 响应类型 */
type Response = { entries?: any[]; entry?: any; unlocked?: boolean; error?: string };

// 扩展安装时初始化
browser.runtime.onInstalled.addListener(() => {
  console.log('MyPass extension installed');
});