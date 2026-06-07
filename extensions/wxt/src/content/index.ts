//! Content Script 入口
//!
//! # 概述
//! 运行在每个网页中的脚本，负责表单检测和自动填充。

import { tryAutoFill, setupFormListener } from './fill';
import { getBestForm, detectForms } from './detect';

// 监听来自 popup/background 的消息
browser.runtime.onMessage.addListener((message, sender, sendResponse) => {
  if (message.type === 'AUTO_FILL') {
    tryAutoFill().then(sendResponse);
    return true;
  }
  
  if (message.type === 'DETECT_FORMS') {
    sendResponse({ forms: detectForms() });
    return false;
  }
  
  if (message.type === 'GET_BEST_FORM') {
    sendResponse({ form: getBestForm() });
    return false;
  }
});

// 页面加载完成后初始化
if (document.readyState === 'loading') {
  document.addEventListener('DOMContentLoaded', init);
} else {
  init();
}

function init(): void {
  // 监听表单提交
  setupFormListener();
  
  console.log('MyPass content script loaded');
}