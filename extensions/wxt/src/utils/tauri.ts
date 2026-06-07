//! Tauri 通信桥接模块
//!
//! # 概述
//! 提供浏览器扩展与 Tauri 后端的通信接口。
//! 使用 Tauri's native messaging 进行通信。

import { invoke } from '@tauri-apps/api/core';

/** 条目接口 */
export interface Entry {
  id: string;
  name: string;
  username: string;
  password: string;
  url: string | null;
  notes: string | null;
  otp_auth_url: string | null;
  group_id: string | null;
  created_at: number;
  updated_at: number;
  version: number;
}

/** 新凭证数据 */
export interface CredentialData {
  name: string;
  username: string;
  password: string;
  url?: string;
  notes?: string;
}

/** Tauri 桥接器 */
export class TauriBridge {
  /** 检查金库是否已解锁 */
  async checkUnlocked(): Promise<boolean> {
    try {
      const result = await invoke<{ is_unlocked: boolean }>('vault_status');
      return result.is_unlocked;
    } catch {
      return false;
    }
  }

  /** 搜索条目 */
  async searchEntries(query: string): Promise<Entry[]> {
    return invoke<Entry[]>('search_entries', { query });
  }

  /** 获取自动填充建议 */
  async getAutoFillSuggestions(url: string): Promise<Entry[]> {
    const entries = await invoke<Entry[]>('get_entries');
    // 过滤匹配 URL 的条目
    return entries.filter(entry => 
      entry.url && (
        entry.url.includes(url) || 
        url.includes(new URL(entry.url).hostname || '')
      )
    );
  }

  /** 保存新凭证 */
  async saveNewCredential(data: CredentialData): Promise<Entry> {
    return invoke<Entry>('create_entry', {
      request: {
        name: data.name,
        username: data.username,
        password: data.password,
        url: data.url || null,
        notes: data.notes || null,
      }
    });
  }

  /** 获取所有条目 */
  async getEntries(): Promise<Entry[]> {
    return invoke<Entry[]>('get_entries');
  }

  /** 解锁金库 */
  async unlockVault(password: string): Promise<boolean> {
    try {
      await invoke('unlock_vault', { 
        request: { password } 
      });
      return true;
    } catch {
      return false;
    }
  }
}

// 导出单例
export const tauriBridge = new TauriBridge();