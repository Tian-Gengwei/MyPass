import { create } from 'zustand'

export interface Entry {
  id: string
  name: string
  username: string
  password: string
  url?: string
  notes?: string
  otp_auth_url?: string
  group_id?: string
  created_at: number
  updated_at: number
  version: number
}

export interface Group {
  id: string
  name: string
  parent_id?: string
  created_at: number
  updated_at: number
  version: number
}

interface VaultState {
  isUnlocked: boolean
  entries: Entry[]
  groups: Group[]
  selectedEntryId: string | null
  selectedGroupId: string | null
  searchQuery: string

  unlock: () => void
  lock: () => void
  setEntries: (entries: Entry[]) => void
  addEntry: (entry: Entry) => void
  updateEntry: (entry: Entry) => void
  removeEntry: (id: string) => void
  setGroups: (groups: Group[]) => void
  addGroup: (group: Group) => void
  updateGroup: (group: Group) => void
  removeGroup: (id: string) => void
  selectEntry: (id: string | null) => void
  selectGroup: (id: string | null) => void
  setSearchQuery: (query: string) => void
}

export const useVaultStore = create<VaultState>((set) => ({
  isUnlocked: false,
  entries: [],
  groups: [],
  selectedEntryId: null,
  selectedGroupId: null,
  searchQuery: '',

  unlock: () => set({ isUnlocked: true }),
  lock: () => set({
    isUnlocked: false,
    entries: [],
    groups: [],
    selectedEntryId: null,
    selectedGroupId: null,
    searchQuery: ''
  }),

  setEntries: (entries) => set({ entries }),
  addEntry: (entry) => set((state) => ({
    entries: [...state.entries, entry]
  })),
  updateEntry: (entry) => set((state) => ({
    entries: state.entries.map(e => e.id === entry.id ? entry : e)
  })),
  removeEntry: (id) => set((state) => ({
    entries: state.entries.filter(e => e.id !== id),
    selectedEntryId: state.selectedEntryId === id ? null : state.selectedEntryId
  })),

  setGroups: (groups) => set({ groups }),
  addGroup: (group) => set((state) => ({
    groups: [...state.groups, group]
  })),
  updateGroup: (group) => set((state) => ({
    groups: state.groups.map(g => g.id === group.id ? group : g)
  })),
  removeGroup: (id) => set((state) => ({
    groups: state.groups.filter(g => g.id !== id),
    selectedGroupId: state.selectedGroupId === id ? null : state.selectedGroupId
  })),

  selectEntry: (id) => set({ selectedEntryId: id }),
  selectGroup: (id) => set({ selectedGroupId: id }),
  setSearchQuery: (query) => set({ searchQuery: query }),
}))

export const useSelectedGroup = () => useVaultStore((state) => state.selectedGroupId)
