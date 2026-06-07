import { create } from 'zustand'

export type ToastType = 'success' | 'error' | 'info' | 'warning'

export interface Toast {
  id: string
  message: string
  type: ToastType
  duration?: number
}

interface UIState {
  sidebarOpen: boolean
  showSettingsDialog: boolean
  theme: 'light' | 'dark' | 'system'
  toasts: Toast[]

  toggleSidebar: () => void
  setSidebarOpen: (open: boolean) => void
  setShowSettingsDialog: (show: boolean) => void
  setTheme: (theme: 'light' | 'dark' | 'system') => void

  showToast: (toast: Omit<Toast, 'id'>) => string
  dismissToast: (id: string) => void
}

let toastCounter = 0
const nextToastId = () => {
  toastCounter += 1
  return `t-${Date.now()}-${toastCounter}`
}

export const useUIStore = create<UIState>((set, get) => ({
  sidebarOpen: true,
  showSettingsDialog: false,
  theme: 'dark',
  toasts: [],

  toggleSidebar: () => set((state) => ({ sidebarOpen: !state.sidebarOpen })),
  setSidebarOpen: (open) => set({ sidebarOpen: open }),
  setShowSettingsDialog: (show) => set({ showSettingsDialog: show }),
  setTheme: (theme) => set({ theme }),

  showToast: (toast) => {
    const id = nextToastId()
    const duration = toast.duration ?? 3000
    set((state) => ({ toasts: [...state.toasts, { id, ...toast, duration }] }))
    if (duration > 0) {
      setTimeout(() => {
        get().dismissToast(id)
      }, duration)
    }
    return id
  },

  dismissToast: (id) => set((state) => ({
    toasts: state.toasts.filter(t => t.id !== id)
  })),
}))
