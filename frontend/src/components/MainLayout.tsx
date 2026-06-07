import { useCallback, useEffect, useMemo, useState } from 'react'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Dialog, DialogContent, DialogHeader, DialogTitle } from '@/components/ui/dialog'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import { useVaultStore, Entry, Group } from '@/stores/vault'
import { invoke } from '@tauri-apps/api/core'
import { open, save } from '@tauri-apps/plugin-dialog'
import {
  Search,
  FolderOpen,
  LogOut,
  Copy,
  Check,
  Plus,
  Eye,
  EyeOff,
  Edit,
  Trash2,
  FolderPlus,
  KeyRound,
  ArrowLeft,
  Shield,
  CreditCard,
  User,
  FileText,
  Settings,
  Upload,
  Download,
  Globe,
  Smartphone,
  HardDrive,
  Key,
  Zap,
  Lock,
} from 'lucide-react'
import { EntryForm } from './EntryForm'
import { TotpTimer } from './TotpTimer'
import { useUIStore } from '@/stores/ui'
import { useIsMobile } from './MobileLayout'

interface MainLayoutProps {
  onLock: () => void
}

// 分类类型
type CategoryType = 'password' | 'card' | 'identity' | 'note' | 'all'

// 菜单数据
const mainMenuItems = [
  { id: 'all' as CategoryType, label: 'All Items', icon: KeyRound, color: 'text-blue-500' },
  { id: 'password' as CategoryType, label: 'Passwords', icon: Lock, color: 'text-purple-500' },
  { id: 'card' as CategoryType, label: 'Cards', icon: CreditCard, color: 'text-green-500', disabled: true },
  { id: 'identity' as CategoryType, label: 'Identities', icon: User, color: 'text-yellow-500', disabled: true },
  { id: 'note' as CategoryType, label: 'Secure Notes', icon: FileText, color: 'text-pink-500', disabled: true },
]

const upcomingFeatures = [
  { id: 'sync', label: 'Sync', icon: Zap, disabled: true },
  { id: 'mobile', label: 'Mobile App', icon: Smartphone, disabled: true },
  { id: 'browser', label: 'Browser Extension', icon: Globe, disabled: true },
]

export function MainLayout({ onLock }: MainLayoutProps) {
  const {
    entries,
    groups,
    selectedEntryId,
    selectedGroupId,
    searchQuery,
    setEntries,
    setGroups,
    selectEntry,
    selectGroup,
    setSearchQuery,
    addEntry,
    updateEntry,
    removeEntry,
    addGroup,
  } = useVaultStore()

  const { showToast } = useUIStore()
  const isMobile = useIsMobile()

  const [copiedField, setCopiedField] = useState<string | null>(null)
  const [showPassword, setShowPassword] = useState(false)
  const [isEntryFormOpen, setIsEntryFormOpen] = useState(false)
  const [editingEntry, setEditingEntry] = useState<Entry | null>(null)
  const [isGroupDialogOpen, setIsGroupDialogOpen] = useState(false)
  const [newGroupName, setNewGroupName] = useState('')
  const [isImportDialogOpen, setIsImportDialogOpen] = useState(false)
  const [isExportDialogOpen, setIsExportDialogOpen] = useState(false)
  const [isSettingsDialogOpen, setIsSettingsDialogOpen] = useState(false)
  const [selectedCategory, setSelectedCategory] = useState<CategoryType>('all')

  const loadData = useCallback(async () => {
    try {
      const [entriesData, groupsData] = await Promise.all([
        invoke<Entry[]>('get_entries'),
        invoke<Group[]>('get_groups'),
      ])
      setEntries(entriesData)
      setGroups(groupsData)
    } catch (err) {
      showToast({ message: `Failed to load data: ${err}`, type: 'error' })
    }
  }, [setEntries, setGroups, showToast])

  useEffect(() => {
    loadData()
  }, [loadData])

  const filteredEntries = useMemo(() => {
    let result = entries
    if (selectedGroupId) {
      result = result.filter(e => e.group_id === selectedGroupId)
    }
    if (searchQuery) {
      const q = searchQuery.toLowerCase()
      result = result.filter(
        e =>
          e.name.toLowerCase().includes(q) ||
          e.username.toLowerCase().includes(q) ||
          e.url?.toLowerCase().includes(q)
      )
    }
    return result
  }, [entries, searchQuery, selectedGroupId])

  const selectedEntry = useMemo(
    () => entries.find(e => e.id === selectedEntryId) ?? null,
    [entries, selectedEntryId]
  )

  const handleLock = async () => {
    try {
      await invoke('lock_vault')
      onLock()
    } catch (err) {
      showToast({ message: `Failed to lock vault: ${err}`, type: 'error' })
    }
  }

  const handleCopy = async (text: string, field: string) => {
    try {
      await navigator.clipboard.writeText(text)
      setCopiedField(field)
      setTimeout(() => setCopiedField(null), 2000)
      showToast({ message: 'Copied to clipboard', type: 'success' })
    } catch (err) {
      showToast({ message: `Copy failed: ${err}`, type: 'error' })
    }
  }

  const handleAddEntry = () => {
    setEditingEntry(null)
    setIsEntryFormOpen(true)
  }

  const handleEditEntry = () => {
    if (!selectedEntry) return
    setEditingEntry(selectedEntry)
    setIsEntryFormOpen(true)
  }

  const handleDeleteEntry = async () => {
    if (!selectedEntry) return
    if (!confirm(`Delete "${selectedEntry.name}"?`)) return
    try {
      await invoke('delete_entry', { id: selectedEntry.id })
      removeEntry(selectedEntry.id)
      showToast({ message: 'Entry deleted', type: 'success' })
    } catch (err) {
      showToast({ message: `Failed to delete: ${err}`, type: 'error' })
    }
  }

  const handleEntrySaved = (entry: Entry) => {
    const exists = entries.find(e => e.id === entry.id)
    if (exists) {
      updateEntry(entry)
    } else {
      addEntry(entry)
    }
    setIsEntryFormOpen(false)
    setEditingEntry(null)
  }

  const handleAddGroup = async () => {
    if (!newGroupName.trim()) return
    try {
      const group = await invoke<Group>('create_group', { name: newGroupName.trim() })
      addGroup(group)
      setNewGroupName('')
      setIsGroupDialogOpen(false)
      showToast({ message: 'Group created', type: 'success' })
    } catch (err) {
      showToast({ message: `Failed to create group: ${err}`, type: 'error' })
    }
  }

  const handleImport = async (format: string) => {
    try {
      let selectedPath: string | null = null
      
      switch (format) {
        case 'bitwarden_csv':
          selectedPath = await open({
            multiple: false,
            filters: [{ name: 'Bitwarden CSV', extensions: ['csv'] }]
          }) as string | null
          break
        case 'bitwarden_json':
          selectedPath = await open({
            multiple: false,
            filters: [{ name: 'Bitwarden JSON', extensions: ['json'] }]
          }) as string | null
          break
        case 'keepass':
          selectedPath = await open({
            multiple: false,
            filters: [{ name: 'KeePass KDBX', extensions: ['kdbx'] }]
          }) as string | null
          break
        case 'chrome':
          selectedPath = await open({
            multiple: false,
            filters: [{ name: 'Chrome CSV', extensions: ['csv'] }]
          }) as string | null
          break
      }
      
      if (!selectedPath) return
      
      let result: number
      switch (format) {
        case 'bitwarden_csv':
          result = await invoke('import_bitwarden_csv', { filePath: selectedPath })
          break
        case 'bitwarden_json':
          result = await invoke('import_bitwarden', { filePath: selectedPath })
          break
        case 'keepass':
          result = await invoke('import_keepass', { filePath: selectedPath })
          break
        case 'chrome':
          result = await invoke('import_chrome_csv', { filePath: selectedPath })
          break
        default:
          throw new Error('Unsupported format')
      }
      
      showToast({ message: `Imported ${result} entries successfully!`, type: 'success' })
      await loadData()
      setIsImportDialogOpen(false)
    } catch (err) {
      showToast({ message: `Import failed: ${err}`, type: 'error' })
    }
  }

  const handleExport = async (format: string) => {
    try {
      let defaultPath = 'mypass_export'
      if (format === 'csv') defaultPath += '.csv'
      if (format === 'json') defaultPath += '.json'
      
      let selectedPath: string | null = null
      switch (format) {
        case 'csv':
          selectedPath = await save({
            defaultPath,
            filters: [{ name: 'CSV', extensions: ['csv'] }]
          }) as string | null
          break
        case 'json':
          selectedPath = await save({
            defaultPath,
            filters: [{ name: 'JSON', extensions: ['json'] }]
          }) as string | null
          break
      }
      
      if (!selectedPath) return
      
      switch (format) {
        case 'csv':
          await invoke('export_csv', { filePath: selectedPath })
          break
        case 'json':
          await invoke('export_json', { filePath: selectedPath })
          break
      }
      
      showToast({ message: `Exported successfully!`, type: 'success' })
    } catch (err) {
      showToast({ message: `Export failed: ${err}`, type: 'error' })
    }
  }

  // 渲染菜单项
  const renderMenuItem = (item: typeof mainMenuItems[0], onClick: () => void) => {
    const Icon = item.icon
    return (
      <button
        key={item.id}
        onClick={onClick}
        disabled={item.disabled}
        className={`w-full flex items-center gap-3 px-3 py-2.5 rounded-lg transition-all ${
          selectedCategory === item.id ? 'bg-accent text-accent-foreground' :
          'hover:bg-accent/50 text-muted-foreground hover:text-foreground'
        } ${item.disabled ? 'opacity-40 cursor-not-allowed' : 'cursor-pointer'}`}
      >
        <Icon className={`h-5 w-5 ${item.color}`} />
        <span className="font-medium">{item.label}</span>
        {item.disabled && (
          <span className="ml-auto text-xs bg-muted px-1.5 py-0.5 rounded text-muted-foreground">Soon</span>
        )}
      </button>
    )
  }

  const sidebar = (
    <div className={isMobile ? 'hidden' : 'w-72 border-r bg-card flex flex-col flex-shrink-0 h-full'}>
      {/* Logo 区域 */}
      <div className="p-5 border-b bg-gradient-to-b from-background to-card/50">
        <div className="flex items-center gap-3">
          <div className="w-10 h-10 rounded-xl bg-gradient-to-br from-blue-500 to-purple-600 flex items-center justify-center shadow-lg">
            <Shield className="h-6 w-6 text-white" />
          </div>
          <h1 className="text-xl font-bold bg-gradient-to-r from-blue-600 to-purple-600 bg-clip-text text-transparent">
            MyPass
          </h1>
        </div>
      </div>

      {/* 搜索和新增按钮 */}
      <div className="p-4 space-y-3 border-b">
        <div className="relative">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
          <Input
            className="pl-10 h-10 bg-background"
            placeholder="Search all items..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
          />
        </div>
        <Button
          variant="default"
          className="w-full h-10 bg-gradient-to-r from-blue-600 to-purple-600 hover:from-blue-700 hover:to-purple-700 text-white shadow-lg hover:shadow-xl transition-all"
          onClick={handleAddEntry}
        >
          <Plus className="mr-2 h-4 w-4" />
          New Item
        </Button>
      </div>

      {/* 主菜单 */}
      <div className="flex-1 overflow-y-auto p-3 space-y-1">
        <div className="space-y-1">
          <div className="px-3 py-2 text-xs font-semibold text-muted-foreground uppercase tracking-wider">
            Vault
          </div>
          {mainMenuItems.map(item => 
            renderMenuItem(item, () => !item.disabled && setSelectedCategory(item.id))
          )}
        </div>

        <div className="h-px bg-border my-3" />

        {/* 分组 */}
        <div className="space-y-1">
          <div className="px-3 py-2 text-xs font-semibold text-muted-foreground uppercase tracking-wider flex items-center justify-between">
            <span>Collections</span>
            <Button
              variant="ghost"
              size="icon"
              className="h-6 w-6"
              onClick={() => setIsGroupDialogOpen(true)}
            >
              <FolderPlus className="h-3.5 w-3.5" />
            </Button>
          </div>
          <button
            className={`w-full flex items-center gap-3 px-3 py-2.5 rounded-lg transition-all ${
              selectedGroupId === null ? 'bg-accent text-accent-foreground' :
              'hover:bg-accent/50 text-muted-foreground hover:text-foreground'
            }`}
            onClick={() => selectGroup(null)}
          >
            <FolderOpen className="h-5 w-5 text-gray-500" />
            <span className="font-medium">All Collections</span>
          </button>
          {groups.map(group => (
            <button
              key={group.id}
              className={`w-full flex items-center gap-3 px-3 py-2.5 rounded-lg transition-all ${
                selectedGroupId === group.id ? 'bg-accent text-accent-foreground' :
                'hover:bg-accent/50 text-muted-foreground hover:text-foreground'
              }`}
              onClick={() => selectGroup(group.id)}
            >
              <FolderOpen className="h-5 w-5 text-gray-500" />
              <span className="font-medium">{group.name}</span>
              <span className="ml-auto text-xs text-muted-foreground">
                {entries.filter(e => e.group_id === group.id).length}
              </span>
            </button>
          ))}
        </div>
      </div>

      {/* 底部菜单 */}
      <div className="p-3 border-t bg-muted/30">
        <div className="space-y-1">
          <div className="px-3 py-2 text-xs font-semibold text-muted-foreground uppercase tracking-wider">
            Tools
          </div>
          <button
            className="w-full flex items-center gap-3 px-3 py-2.5 rounded-lg text-muted-foreground hover:bg-accent hover:text-foreground transition-all"
            onClick={() => setIsImportDialogOpen(true)}
          >
            <Upload className="h-5 w-5" />
            <span className="font-medium">Import Data</span>
          </button>
          <button
            className="w-full flex items-center gap-3 px-3 py-2.5 rounded-lg text-muted-foreground hover:bg-accent hover:text-foreground transition-all"
            onClick={() => setIsExportDialogOpen(true)}
          >
            <Download className="h-5 w-5" />
            <span className="font-medium">Export Data</span>
          </button>
          <button
            className="w-full flex items-center gap-3 px-3 py-2.5 rounded-lg text-muted-foreground hover:bg-accent hover:text-foreground transition-all"
            onClick={() => setIsSettingsDialogOpen(true)}
          >
            <Settings className="h-5 w-5" />
            <span className="font-medium">Settings</span>
          </button>
        </div>

        <div className="h-px bg-border my-3" />

        <div className="space-y-1">
          <div className="px-3 py-2 text-xs font-semibold text-muted-foreground uppercase tracking-wider">
            Coming Soon
          </div>
          {upcomingFeatures.map(item => (
            <button
              key={item.id}
              disabled
              className="w-full flex items-center gap-3 px-3 py-2.5 rounded-lg text-muted-foreground opacity-40 cursor-not-allowed"
            >
              <item.icon className="h-5 w-5" />
              <span className="font-medium">{item.label}</span>
              <span className="ml-auto text-xs bg-muted px-1.5 py-0.5 rounded">Beta</span>
            </button>
          ))}
        </div>

        <div className="h-px bg-border my-3" />

        <button
          variant="ghost"
          className="w-full flex items-center gap-3 px-3 py-2.5 rounded-lg text-destructive hover:bg-destructive/10 transition-all"
          onClick={handleLock}
        >
          <LogOut className="h-5 w-5" />
          <span className="font-medium">Lock Vault</span>
        </button>
      </div>
    </div>
  )

  const entryList = (
    <div className={`${isMobile ? 'w-full' : 'w-96'} border-r bg-card flex flex-col flex-shrink-0 h-full`}>
      <div className="p-5 border-b space-y-4 bg-gradient-to-b from-background to-card/30">
        <div className="flex items-center justify-between">
          <h2 className="text-xl font-bold">
            {searchQuery
              ? `Search: ${searchQuery}`
              : selectedGroupId
                ? groups.find(g => g.id === selectedGroupId)?.name ?? 'Collection'
                : selectedCategory === 'all' ? 'All Items' : mainMenuItems.find(m => m.id === selectedCategory)?.label}
          </h2>
          <div className="flex items-center gap-2">
            {isMobile && (
              <DropdownMenu>
                <DropdownMenuTrigger asChild>
                  <Button variant="ghost" size="icon">
                    <Settings className="h-4 w-4" />
                  </Button>
                </DropdownMenuTrigger>
                <DropdownMenuContent align="end">
                  <DropdownMenuItem onClick={() => setIsImportDialogOpen(true)}>
                    <Upload className="mr-2 h-4 w-4" />
                    Import Data
                  </DropdownMenuItem>
                  <DropdownMenuItem onClick={() => setIsExportDialogOpen(true)}>
                    <Download className="mr-2 h-4 w-4" />
                    Export Data
                  </DropdownMenuItem>
                  <DropdownMenuItem onClick={() => setIsSettingsDialogOpen(true)}>
                    <Settings className="mr-2 h-4 w-4" />
                    Settings
                  </DropdownMenuItem>
                  <DropdownMenuSeparator />
                  <DropdownMenuItem onClick={handleLock}>
                    <LogOut className="mr-2 h-4 w-4" />
                    Lock Vault
                  </DropdownMenuItem>
                </DropdownMenuContent>
              </DropdownMenu>
            )}
          </div>
        </div>
        {isMobile && (
          <div className="relative">
            <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
            <Input
              className="pl-10"
              placeholder="Search..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
            />
          </div>
        )}
        <div className="flex items-center gap-2">
          <div className="flex items-center gap-2 text-sm text-muted-foreground">
            <KeyRound className="h-4 w-4" />
            <span>{filteredEntries.length} {filteredEntries.length === 1 ? 'item' : 'items'}</span>
          </div>
        </div>
      </div>

      <div className="overflow-y-auto flex-1">
        {filteredEntries.length === 0 ? (
          <div className="p-8 text-center text-sm text-muted-foreground">
            {searchQuery ? (
              <div className="space-y-3">
                <div className="w-16 h-16 mx-auto rounded-full bg-muted flex items-center justify-center">
                  <Search className="h-8 w-8 opacity-50" />
                </div>
                <div>
                  <p className="font-medium text-foreground">No results found</p>
                  <p className="text-sm text-muted-foreground">Try a different search term</p>
                </div>
              </div>
            ) : (
              <div className="space-y-3">
                <div className="w-16 h-16 mx-auto rounded-full bg-muted flex items-center justify-center">
                  <Key className="h-8 w-8 opacity-50" />
                </div>
                <div>
                  <p className="font-medium text-foreground mb-1">No items yet</p>
                  <p className="text-sm text-muted-foreground mb-4">Start by adding your first password</p>
                  <Button onClick={handleAddEntry} className="bg-gradient-to-r from-blue-600 to-purple-600 hover:from-blue-700 hover:to-purple-700">
                    <Plus className="mr-2 h-4 w-4" />
                    Add your first item
                  </Button>
                </div>
              </div>
            )}
          </div>
        ) : (
          <div className="divide-y">
            {filteredEntries.map(entry => (
              <button
                key={entry.id}
                onClick={() => selectEntry(entry.id)}
                className={`w-full p-4 text-left transition-all hover:bg-accent/50 ${
                  selectedEntryId === entry.id ? 'bg-accent border-l-4 border-l-blue-500' : ''
                }`}
              >
                <div className="flex items-start gap-3">
                  <div className="w-10 h-10 rounded-lg bg-gradient-to-br from-blue-100 to-purple-100 flex items-center justify-center flex-shrink-0">
                    {entry.url?.includes('github') ? (
                      <svg className="h-5 w-5" viewBox="0 0 24 24" fill="currentColor">
                        <path d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z"/>
                      </svg>
                    ) : entry.url?.includes('google') ? (
                      <svg className="h-5 w-5" viewBox="0 0 24 24">
                        <path fill="#4285F4" d="M22.56 12.25c0-.78-.07-1.53-.2-2.25H12v4.26h5.92c-.26 1.37-1.04 2.53-2.21 3.31v2.77h3.57c2.08-1.92 3.28-4.74 3.28-8.09z"/>
                        <path fill="#34A853" d="M12 23c2.97 0 5.46-.98 7.28-2.66l-3.57-2.77c-.98.66-2.23 1.06-3.71 1.06-2.86 0-5.29-1.93-6.16-4.53H2.18v2.84C3.99 20.53 7.7 23 12 23z"/>
                        <path fill="#FBBC05" d="M5.84 14.09c-.22-.66-.35-1.36-.35-2.09s.13-1.43.35-2.09V7.07H2.18C1.43 8.55 1 10.22 1 12s.43 3.45 1.18 4.93l2.85-2.22.81-.62z"/>
                        <path fill="#EA4335" d="M12 5.38c1.62 0 3.06.56 4.21 1.64l3.15-3.15C17.45 2.09 14.97 1 12 1 7.7 1 3.99 3.47 2.18 7.07l3.66 2.84c.87-2.6 3.3-4.53 6.16-4.53z"/>
                      </svg>
                    ) : (
                      <KeyRound className="h-5 w-5 text-blue-600" />
                    )}
                  </div>
                  <div className="flex-1 min-w-0">
                    <div className="font-semibold truncate">{entry.name}</div>
                    <div className="text-sm text-muted-foreground truncate mt-0.5">
                      {entry.username}
                    </div>
                    {entry.otp_auth_url && (
                      <div className="flex items-center gap-1 mt-1">
                        <Zap className="h-3 w-3 text-yellow-500" />
                        <span className="text-xs text-yellow-600">2FA</span>
                      </div>
                    )}
                  </div>
                </div>
              </button>
            ))}
          </div>
        )}
      </div>
    </div>
  )

  const entryDetail = (
    <div className="flex-1 bg-background p-6 overflow-y-auto">
      {selectedEntry ? (
        <div className="max-w-2xl mx-auto space-y-6">
          <div className="flex items-center justify-between gap-2">
            {isMobile && (
              <Button
                variant="ghost"
                size="icon"
                onClick={() => selectEntry(null)}
                aria-label="Back to entries"
              >
                <ArrowLeft className="h-5 w-5" />
              </Button>
            )}
            <div className="flex-1">
              <div className="flex items-center gap-3">
                <div className="w-12 h-12 rounded-xl bg-gradient-to-br from-blue-100 to-purple-100 flex items-center justify-center">
                  <KeyRound className="h-6 w-6 text-blue-600" />
                </div>
                <div>
                  <h2 className="text-2xl font-bold">{selectedEntry.name}</h2>
                  {selectedEntry.url && (
                    <a
                      href={selectedEntry.url}
                      target="_blank"
                      rel="noopener noreferrer"
                      className="text-sm text-muted-foreground hover:text-primary flex items-center gap-1"
                    >
                      <Globe className="h-3.5 w-3.5" />
                      {selectedEntry.url}
                    </a>
                  )}
                </div>
              </div>
            </div>
            <div className="flex gap-2">
              <Button variant="outline" size="sm" onClick={handleEditEntry}>
                <Edit className="mr-2 h-4 w-4" />
                <span className="hidden sm:inline">Edit</span>
              </Button>
              <Button variant="outline" size="sm" onClick={handleDeleteEntry} className="text-destructive hover:text-destructive">
                <Trash2 className="mr-2 h-4 w-4" />
                <span className="hidden sm:inline">Delete</span>
              </Button>
            </div>
          </div>

          <div className="space-y-4">
            {/* Username */}
            <div className="group p-4 bg-card rounded-xl border hover:border-accent transition-all">
              <div className="flex items-center justify-between">
                <div>
                  <label className="text-xs font-semibold text-muted-foreground uppercase tracking-wider">Username</label>
                  <div className="font-medium text-lg mt-1">{selectedEntry.username}</div>
                </div>
                <Button
                  variant="ghost"
                  size="icon"
                  onClick={() => handleCopy(selectedEntry.username, 'username')}
                  className="opacity-0 group-hover:opacity-100 transition-opacity"
                >
                  {copiedField === 'username' ? (
                    <Check className="h-5 w-5 text-green-500" />
                  ) : (
                    <Copy className="h-5 w-5" />
                  )}
                </Button>
              </div>
            </div>

            {/* Password */}
            <div className="group p-4 bg-card rounded-xl border hover:border-accent transition-all">
              <div className="flex items-center justify-between">
                <div className="flex-1">
                  <label className="text-xs font-semibold text-muted-foreground uppercase tracking-wider">Password</label>
                  <div className="font-mono text-lg mt-1">
                    {showPassword ? selectedEntry.password : '•'.repeat(Math.max(12, selectedEntry.password.length))}
                  </div>
                </div>
                <div className="flex items-center gap-1">
                  <Button
                    variant="ghost"
                    size="icon"
                    onClick={() => setShowPassword(s => !s)}
                    title={showPassword ? 'Hide password' : 'Show password'}
                  >
                    {showPassword ? <EyeOff className="h-5 w-5" /> : <Eye className="h-5 w-5" />}
                  </Button>
                  <Button
                    variant="ghost"
                    size="icon"
                    onClick={() => handleCopy(selectedEntry.password, 'password')}
                    className="opacity-0 group-hover:opacity-100 transition-opacity"
                  >
                    {copiedField === 'password' ? (
                      <Check className="h-5 w-5 text-green-500" />
                    ) : (
                      <Copy className="h-5 w-5" />
                    )}
                  </Button>
                </div>
              </div>
            </div>

            {/* TOTP */}
            {selectedEntry.otp_auth_url && (
              <div className="p-4 bg-gradient-to-r from-yellow-50 to-orange-50 rounded-xl border border-yellow-200">
                <label className="text-xs font-semibold text-yellow-700 uppercase tracking-wider flex items-center gap-2">
                  <Zap className="h-4 w-4" />
                  Two-Factor Authentication
                </label>
                <div className="mt-2">
                  <TotpTimer secret={selectedEntry.otp_auth_url} />
                </div>
              </div>
            )}

            {/* Notes */}
            {selectedEntry.notes && (
              <div className="p-4 bg-card rounded-xl border">
                <label className="text-xs font-semibold text-muted-foreground uppercase tracking-wider">Notes</label>
                <div className="mt-2 whitespace-pre-wrap text-muted-foreground leading-relaxed">
                  {selectedEntry.notes}
                </div>
              </div>
            )}

            {/* Metadata */}
            <div className="pt-4 border-t">
              <div className="flex flex-wrap gap-4 text-xs text-muted-foreground">
                <div>
                  Created: {new Date(selectedEntry.created_at * 1000).toLocaleDateString()}
                </div>
                <div>
                  Updated: {new Date(selectedEntry.updated_at * 1000).toLocaleDateString()}
                </div>
                {selectedEntry.group_id && groups.find(g => g.id === selectedEntry.group_id) && (
                  <div className="flex items-center gap-1">
                    <FolderOpen className="h-3 w-3" />
                    {groups.find(g => g.id === selectedEntry.group_id)?.name}
                  </div>
                )}
              </div>
            </div>
          </div>
        </div>
      ) : (
        <div className="h-full flex flex-col items-center justify-center text-center space-y-4">
          <div className="w-24 h-24 rounded-full bg-gradient-to-br from-blue-100 to-purple-100 flex items-center justify-center">
            <Shield className="h-12 w-12 text-blue-500 opacity-50" />
          </div>
          <div>
            <h3 className="text-lg font-semibold text-foreground">Welcome to MyPass</h3>
            <p className="text-muted-foreground mt-1">Select an item to view its details</p>
          </div>
        </div>
      )}
    </div>
  )

  const layout = (
    <div className="flex h-screen">
      {sidebar}
      {(!isMobile || !selectedEntry) && entryList}
      {(!isMobile || selectedEntry) && entryDetail}
    </div>
  )

  return (
    <>
      {layout}

      {/* Entry Form Dialog */}
      <Dialog open={isEntryFormOpen} onOpenChange={(open) => {
        if (!open) {
          setIsEntryFormOpen(false)
          setEditingEntry(null)
        }
      }}>
        <DialogContent className="sm:max-w-[450px] max-h-[90vh] overflow-y-auto">
          <DialogHeader>
            <DialogTitle>{editingEntry ? 'Edit Item' : 'Add New Item'}</DialogTitle>
          </DialogHeader>
          <EntryForm
            entry={editingEntry}
            groups={groups}
            defaultGroupId={selectedGroupId ?? undefined}
            onClose={() => {
              setIsEntryFormOpen(false)
              setEditingEntry(null)
            }}
            onSave={handleEntrySaved}
          />
        </DialogContent>
      </Dialog>

      {/* Import Dialog */}
      <Dialog open={isImportDialogOpen} onOpenChange={setIsImportDialogOpen}>
        <DialogContent className="sm:max-w-[450px]">
          <DialogHeader>
            <DialogTitle>Import Data</DialogTitle>
          </DialogHeader>
          <div className="space-y-3 py-4">
            <Button 
              className="w-full justify-start" 
              variant="outline"
              onClick={() => handleImport('bitwarden_csv')}
            >
              <Upload className="mr-2 h-4 w-4" />
              Import Bitwarden CSV
            </Button>
            <Button 
              className="w-full justify-start" 
              variant="outline"
              onClick={() => handleImport('bitwarden_json')}
            >
              <Upload className="mr-2 h-4 w-4" />
              Import Bitwarden JSON
            </Button>
            <Button 
              className="w-full justify-start" 
              variant="outline"
              onClick={() => handleImport('keepass')}
            >
              <HardDrive className="mr-2 h-4 w-4" />
              Import KeePass (KDBX)
            </Button>
            <Button 
              className="w-full justify-start" 
              variant="outline"
              onClick={() => handleImport('chrome')}
            >
              <Globe className="mr-2 h-4 w-4" />
              Import Chrome CSV
            </Button>
            <div className="pt-4 border-t">
              <p className="text-xs text-muted-foreground">
                Your data is encrypted locally before being stored.
              </p>
            </div>
          </div>
        </DialogContent>
      </Dialog>

      {/* Export Dialog */}
      <Dialog open={isExportDialogOpen} onOpenChange={setIsExportDialogOpen}>
        <DialogContent className="sm:max-w-[450px]">
          <DialogHeader>
            <DialogTitle>Export Data</DialogTitle>
          </DialogHeader>
          <div className="space-y-3 py-4">
            <Button 
              className="w-full justify-start" 
              variant="outline"
              onClick={() => handleExport('csv')}
            >
              <Download className="mr-2 h-4 w-4" />
              Export as CSV
            </Button>
            <Button 
              className="w-full justify-start" 
              variant="outline"
              onClick={() => handleExport('json')}
            >
              <Download className="mr-2 h-4 w-4" />
              Export as JSON
            </Button>
            <div className="pt-4 border-t">
              <p className="text-xs text-destructive">
                ⚠️ Warning: Exported data is not encrypted! Keep it secure.
              </p>
            </div>
          </div>
        </DialogContent>
      </Dialog>

      {/* Settings Dialog */}
      <Dialog open={isSettingsDialogOpen} onOpenChange={setIsSettingsDialogOpen}>
        <DialogContent className="sm:max-w-[450px]">
          <DialogHeader>
            <DialogTitle>Settings</DialogTitle>
          </DialogHeader>
          <div className="space-y-4 py-4">
            <div className="p-4 bg-muted/50 rounded-lg">
              <h3 className="font-medium mb-2">Security</h3>
              <p className="text-sm text-muted-foreground">
                Auto-lock and encryption settings coming soon.
              </p>
            </div>
            <div className="p-4 bg-muted/50 rounded-lg opacity-60">
              <h3 className="font-medium mb-2">Sync</h3>
              <p className="text-sm text-muted-foreground">
                Cloud sync feature coming soon.
              </p>
            </div>
            <div className="pt-4 border-t">
              <p className="text-xs text-muted-foreground">
                MyPass v1.0.0 · Built with ❤️
              </p>
            </div>
          </div>
        </DialogContent>
      </Dialog>

      {/* Group Dialog */}
      {isGroupDialogOpen && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div className="bg-card p-6 rounded-xl w-96 space-y-4 shadow-xl">
            <h3 className="text-lg font-semibold">New Collection</h3>
            <p className="text-sm text-muted-foreground">Organize your items into collections</p>
            <Input
              autoFocus
              placeholder="Collection name"
              value={newGroupName}
              onChange={(e) => setNewGroupName(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === 'Enter') handleAddGroup()
                if (e.key === 'Escape') setIsGroupDialogOpen(false)
              }}
            />
            <div className="flex justify-end gap-2">
              <Button variant="ghost" onClick={() => setIsGroupDialogOpen(false)}>
                Cancel
              </Button>
              <Button onClick={handleAddGroup} className="bg-gradient-to-r from-blue-600 to-purple-600">
                Create
              </Button>
            </div>
          </div>
        </div>
      )}
    </>
  )
}
