import { create } from 'zustand'

interface SyncState {
  lastSync: number | null
  isSyncing: boolean
  pendingChanges: number
  syncConfig: SyncConfig | null
  conflicts: SyncConflict[]

  setLastSync: (time: number | null) => void
  setSyncing: (value: boolean) => void
  setPendingChanges: (count: number) => void
  setSyncConfig: (config: SyncConfig | null) => void
  addConflict: (conflict: SyncConflict) => void
  resolveConflict: (id: string) => void
  clearConflicts: () => void
}

export interface SyncConfig {
  syncType: 'webdav' | 's3'
  endpoint: string
  username?: string
  password?: string
}

export interface SyncConflict {
  id: string
  localHash: string
  remoteHash: string
  resolved: boolean
}

export const useSyncStore = create<SyncState>((set) => ({
  lastSync: null,
  isSyncing: false,
  pendingChanges: 0,
  syncConfig: null,
  conflicts: [],

  setLastSync: (time) => set({ lastSync: time }),
  setSyncing: (value) => set({ isSyncing: value }),
  setPendingChanges: (count) => set({ pendingChanges: count }),
  setSyncConfig: (config) => set({ syncConfig: config }),
  addConflict: (conflict) => set((state) => ({
    conflicts: [...state.conflicts, conflict]
  })),
  resolveConflict: (id) => set((state) => ({
    conflicts: state.conflicts.map(c =>
      c.id === id ? { ...c, resolved: true } : c
    )
  })),
  clearConflicts: () => set({ conflicts: [] }),
}))