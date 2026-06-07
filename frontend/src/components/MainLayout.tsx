import { useCallback, useEffect, useMemo, useState } from 'react'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Dialog, DialogContent, DialogHeader, DialogTitle } from '@/components/ui/dialog'
import { useVaultStore, Entry, Group } from '@/stores/vault'
import { invoke } from '@tauri-apps/api/core'
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
} from 'lucide-react'
import { EntryForm } from './EntryForm'
import { TotpTimer } from './TotpTimer'
import { useUIStore } from '@/stores/ui'
import { useIsMobile } from './MobileLayout'

interface MainLayoutProps {
  onLock: () => void
}

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

  const sidebar = (
    <div className={isMobile ? 'hidden' : 'w-64 border-r bg-card flex flex-col flex-shrink-0'}>
      <div className="p-4 border-b">
        <h1 className="text-xl font-bold">MyPass</h1>
      </div>

      <div className="p-3 space-y-2">
        <div className="relative">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
          <Input
            className="pl-9"
            placeholder="Search..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
          />
        </div>
        <Button
          variant="default"
          className="w-full"
          onClick={handleAddEntry}
        >
          <Plus className="mr-2 h-4 w-4" />
          New Entry
        </Button>
      </div>

      <div className="flex-1 overflow-y-auto p-2">
        <div className="space-y-1">
          <Button
            variant={selectedGroupId === null ? 'secondary' : 'ghost'}
            className="w-full justify-start"
            onClick={() => selectGroup(null)}
          >
            <FolderOpen className="mr-2 h-4 w-4" />
            All Items
          </Button>
          {groups.map(group => (
            <Button
              key={group.id}
              variant={selectedGroupId === group.id ? 'secondary' : 'ghost'}
              className="w-full justify-start"
              onClick={() => selectGroup(group.id)}
            >
              <FolderOpen className="mr-2 h-4 w-4" />
              {group.name}
            </Button>
          ))}
          {groups.length === 0 && (
            <div className="px-3 py-2 text-xs text-muted-foreground">
              No groups yet
            </div>
          )}
        </div>
      </div>

      <div className="p-3 border-t space-y-2">
        <Button
          variant="ghost"
          className="w-full"
          onClick={() => setIsGroupDialogOpen(true)}
        >
          <FolderPlus className="mr-2 h-4 w-4" />
          New Group
        </Button>
        <Button variant="ghost" className="w-full" onClick={handleLock}>
          <LogOut className="mr-2 h-4 w-4" />
          Lock
        </Button>
      </div>
    </div>
  )

  const entryList = (
    <div className={`${isMobile ? 'w-full' : 'w-80'} border-r bg-card flex flex-col flex-shrink-0`}>
      <div className="p-4 border-b space-y-3">
        <div className="flex items-center justify-between">
          <h2 className="text-lg font-semibold">
            {searchQuery
              ? `Search: ${searchQuery}`
              : selectedGroupId
                ? groups.find(g => g.id === selectedGroupId)?.name ?? 'Group'
                : 'All Items'}
          </h2>
          {isMobile && (
            <Button variant="ghost" size="icon" onClick={handleLock} title="Lock">
              <LogOut className="h-4 w-4" />
            </Button>
          )}
        </div>
        {isMobile && (
          <div className="relative">
            <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
            <Input
              className="pl-9"
              placeholder="Search..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
            />
          </div>
        )}
        <p className="text-sm text-muted-foreground">
          {filteredEntries.length} {filteredEntries.length === 1 ? 'item' : 'items'}
        </p>
      </div>

      <div className="overflow-y-auto flex-1">
        {filteredEntries.length === 0 ? (
          <div className="p-8 text-center text-sm text-muted-foreground">
            {searchQuery ? (
              <div>
                <Search className="mx-auto h-8 w-8 mb-2 opacity-50" />
                <p>No entries match your search</p>
              </div>
            ) : (
              <div>
                <KeyRound className="mx-auto h-8 w-8 mb-2 opacity-50" />
                <p className="mb-3">No entries yet</p>
                <Button size="sm" onClick={handleAddEntry}>
                  <Plus className="mr-2 h-4 w-4" />
                  Add your first entry
                </Button>
              </div>
            )}
          </div>
        ) : (
          filteredEntries.map(entry => (
            <button
              key={entry.id}
              onClick={() => selectEntry(entry.id)}
              className={`w-full p-4 text-left border-b hover:bg-accent transition-colors ${
                selectedEntryId === entry.id ? 'bg-accent' : ''
              }`}
            >
              <div className="font-medium truncate">{entry.name}</div>
              <div className="text-sm text-muted-foreground truncate">
                {entry.username}
              </div>
            </button>
          ))
        )}
      </div>
    </div>
  )

  const entryDetail = (
    <div className="flex-1 bg-background p-4 sm:p-6 overflow-y-auto">
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
            <h2 className="text-xl sm:text-2xl font-bold flex-1 truncate">
              {selectedEntry.name}
            </h2>
            <div className="flex gap-1 sm:gap-2">
              <Button variant="outline" size="sm" onClick={handleEditEntry}>
                <Edit className="sm:mr-2 h-4 w-4" />
                <span className="hidden sm:inline">Edit</span>
              </Button>
              <Button variant="outline" size="sm" onClick={handleDeleteEntry}>
                <Trash2 className="sm:mr-2 h-4 w-4 text-destructive" />
                <span className="hidden sm:inline">Delete</span>
              </Button>
            </div>
          </div>

          <div className="space-y-4">
            <div className="flex items-center justify-between p-3 bg-card rounded-lg">
              <div>
                <label className="text-sm text-muted-foreground">Username</label>
                <div className="font-medium">{selectedEntry.username}</div>
              </div>
              <Button
                variant="ghost"
                size="icon"
                onClick={() => handleCopy(selectedEntry.username, 'username')}
              >
                {copiedField === 'username' ? (
                  <Check className="h-4 w-4" />
                ) : (
                  <Copy className="h-4 w-4" />
                )}
              </Button>
            </div>

            <div className="flex items-center justify-between p-3 bg-card rounded-lg">
              <div>
                <label className="text-sm text-muted-foreground">Password</label>
                <div className="font-medium font-mono">
                  {showPassword ? selectedEntry.password : '•'.repeat(Math.max(12, selectedEntry.password.length))}
                </div>
              </div>
              <div className="flex">
                <Button
                  variant="ghost"
                  size="icon"
                  onClick={() => setShowPassword(s => !s)}
                  title={showPassword ? 'Hide password' : 'Show password'}
                >
                  {showPassword ? <EyeOff className="h-4 w-4" /> : <Eye className="h-4 w-4" />}
                </Button>
                <Button
                  variant="ghost"
                  size="icon"
                  onClick={() => handleCopy(selectedEntry.password, 'password')}
                >
                  {copiedField === 'password' ? (
                    <Check className="h-4 w-4" />
                  ) : (
                    <Copy className="h-4 w-4" />
                  )}
                </Button>
              </div>
            </div>

            {selectedEntry.otp_auth_url && (
              <div className="p-3 bg-card rounded-lg">
                <label className="text-sm text-muted-foreground">TOTP Code</label>
                <div className="mt-1">
                  <TotpTimer secret={selectedEntry.otp_auth_url} />
                </div>
              </div>
            )}

            {selectedEntry.url && (
              <div className="p-3 bg-card rounded-lg">
                <label className="text-sm text-muted-foreground">URL</label>
                <div className="font-medium break-all">
                  <a
                    href={selectedEntry.url}
                    target="_blank"
                    rel="noopener noreferrer"
                    className="text-primary hover:underline"
                  >
                    {selectedEntry.url}
                  </a>
                </div>
              </div>
            )}

            {selectedEntry.notes && (
              <div className="p-3 bg-card rounded-lg">
                <label className="text-sm text-muted-foreground">Notes</label>
                <div className="whitespace-pre-wrap">{selectedEntry.notes}</div>
              </div>
            )}
          </div>
        </div>
      ) : (
        <div className="h-full flex flex-col items-center justify-center text-muted-foreground">
          <KeyRound className="h-12 w-12 mb-3 opacity-30" />
          <p>Select an entry to view details</p>
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

      <Dialog open={isEntryFormOpen} onOpenChange={(open) => {
        if (!open) {
          setIsEntryFormOpen(false)
          setEditingEntry(null)
        }
      }}>
        <DialogContent className="sm:max-w-[425px] max-h-[90vh] overflow-y-auto">
          <DialogHeader>
            <DialogTitle>{editingEntry ? 'Edit Entry' : 'Add New Entry'}</DialogTitle>
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

      {isGroupDialogOpen && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div className="bg-card p-6 rounded-lg w-96 space-y-4">
            <h3 className="text-lg font-semibold">New Group</h3>
            <Input
              autoFocus
              placeholder="Group name"
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
              <Button onClick={handleAddGroup}>Create</Button>
            </div>
          </div>
        </div>
      )}
    </>
  )
}
