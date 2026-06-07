//! Popup 主组件
//!
//! # 概述
//! 扩展点击后的弹窗界面，提供快速搜索和自动填充功能。

import { createRoot } from 'react-dom/client';
import { useState, useEffect } from 'react';

// 获取消息响应类型
interface Response {
  entries?: Array<{
    id: string;
    name: string;
    username: string;
    url: string | null;
  }>;
  entries_?: Array<{
    id: string;
    name: string;
    username: string;
    url: string | null;
  }>;
  unlocked?: boolean;
  error?: string;
}

function App() {
  const [search, setSearch] = useState('');
  const [entries, setEntries] = useState<Array<{id: string; name: string; username: string; url: string | null}>>([]);
  const [unlocked, setUnlocked] = useState(false);
  const [loading, setLoading] = useState(true);
  
  useEffect(() => {
    checkStatus();
  }, []);
  
  useEffect(() => {
    if (unlocked && search) {
      searchEntries(search);
    } else if (unlocked) {
      loadEntries();
    }
  }, [search, unlocked]);
  
  const checkStatus = async () => {
    try {
      const response = await browser.runtime.sendMessage({ type: 'CHECK_UNLOCKED' }) as Response;
      setUnlocked(response.unlocked || false);
    } catch {
      setUnlocked(false);
    }
    setLoading(false);
  };
  
  const loadEntries = async () => {
    try {
      const response = await browser.runtime.sendMessage({ type: 'GET_ENTRIES' }) as Response;
      setEntries(response.entries_ || []);
    } catch (err) {
      console.error('Failed to load entries:', err);
    }
  };
  
  const searchEntries = async (query: string) => {
    try {
      const response = await browser.runtime.sendMessage({ type: 'SEARCH', query }) as Response;
      setEntries(response.entries || []);
    } catch (err) {
      console.error('Failed to search:', err);
    }
  };
  
  const handleFill = async (entryId: string) => {
    const entry = entries.find(e => e.id === entryId);
    if (!entry) return;
    
    const [tab] = await browser.tabs.query({ active: true, currentWindow: true });
    if (tab?.id) {
      await browser.tabs.sendMessage(tab.id, { type: 'AUTO_FILL', entry });
    }
    window.close();
  };
  
  if (loading) {
    return <div className="p-4">Loading...</div>;
  }
  
  if (!unlocked) {
    return (
      <div className="p-4 w-80">
        <h1 className="text-xl font-bold mb-4">MyPass</h1>
        <p className="text-gray-600">Please unlock MyPass desktop app first.</p>
      </div>
    );
  }
  
  return (
    <div className="p-4 w-80">
      <h1 className="text-xl font-bold mb-4">MyPass</h1>
      
      <input
        type="text"
        placeholder="Search entries..."
        value={search}
        onChange={(e) => setSearch(e.target.value)}
        className="w-full p-2 border rounded mb-4"
      />
      
      <div className="space-y-2 max-h-96 overflow-y-auto">
        {entries.map(entry => (
          <button
            key={entry.id}
            onClick={() => handleFill(entry.id)}
            className="w-full p-3 text-left border rounded hover:bg-gray-100"
          >
            <div className="font-medium">{entry.name}</div>
            <div className="text-sm text-gray-500">{entry.username}</div>
            {entry.url && (
              <div className="text-xs text-gray-400 truncate">{entry.url}</div>
            )}
          </button>
        ))}
        
        {entries.length === 0 && (
          <p className="text-gray-500 text-center py-4">No entries found</p>
        )}
      </div>
    </div>
  );
}

// 渲染
createRoot(document.getElementById('root')!).render(<App />);