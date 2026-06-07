import { useEffect, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Dialog, DialogContent, DialogHeader, DialogTitle } from '@/components/ui/dialog'
import { TotpTimer } from './TotpTimer'
import { Entry, Group } from '@/stores/vault'
import { RefreshCw, Eye, EyeOff } from 'lucide-react'

interface EntryFormProps {
  entry?: Entry | null
  groups: Group[]
  defaultGroupId?: string
  onClose: () => void
  onSave: (entry: Entry) => void
}

const CHARSETS = {
  lower: 'abcdefghijklmnopqrstuvwxyz',
  upper: 'ABCDEFGHIJKLMNOPQRSTUVWXYZ',
  digits: '0123456789',
  symbols: '!@#$%^&*()-_=+[]{};:,.<>?',
}

function generatePassword(length: number, useSymbols: boolean): string {
  let charset = CHARSETS.lower + CHARSETS.upper + CHARSETS.digits
  if (useSymbols) charset += CHARSETS.symbols
  const bytes = new Uint8Array(length)
  crypto.getRandomValues(bytes)
  let out = ''
  for (let i = 0; i < length; i++) {
    out += charset[bytes[i] % charset.length]
  }
  return out
}

export function EntryForm({ entry, groups, defaultGroupId, onClose, onSave }: EntryFormProps) {
  const [name, setName] = useState(entry?.name ?? '')
  const [username, setUsername] = useState(entry?.username ?? '')
  const [password, setPassword] = useState(entry?.password ?? '')
  const [url, setUrl] = useState(entry?.url ?? '')
  const [notes, setNotes] = useState(entry?.notes ?? '')
  const [otpAuthUrl, setOtpAuthUrl] = useState(entry?.otp_auth_url ?? '')
  const [groupId, setGroupId] = useState<string>(entry?.group_id ?? defaultGroupId ?? '')
  const [showPassword, setShowPassword] = useState(false)
  const [isSaving, setIsSaving] = useState(false)
  const [error, setError] = useState('')

  useEffect(() => {
    setName(entry?.name ?? '')
    setUsername(entry?.username ?? '')
    setPassword(entry?.password ?? '')
    setUrl(entry?.url ?? '')
    setNotes(entry?.notes ?? '')
    setOtpAuthUrl(entry?.otp_auth_url ?? '')
    setGroupId(entry?.group_id ?? defaultGroupId ?? '')
  }, [entry, defaultGroupId])

  const handleGeneratePassword = () => {
    setPassword(generatePassword(20, true))
    setShowPassword(true)
  }

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setError('')
    setIsSaving(true)

    try {
      let savedEntry: Entry
      const request = {
        name,
        username,
        password,
        url: url || null,
        notes: notes || null,
        otp_auth_url: otpAuthUrl || null,
        group_id: groupId || null,
      }

      if (entry?.id) {
        savedEntry = await invoke('update_entry', { request: { id: entry.id, ...request } })
      } else {
        savedEntry = await invoke('create_entry', { request })
      }

      onSave(savedEntry)
    } catch (err) {
      setError(String(err))
    } finally {
      setIsSaving(false)
    }
  }

  return (
    <form onSubmit={handleSubmit} className="space-y-4">
      <div className="space-y-2">
        <Label htmlFor="name">Name</Label>
        <Input
          id="name"
          value={name}
          onChange={(e) => setName(e.target.value)}
          placeholder="e.g., GitHub"
          required
          autoFocus
        />
      </div>

      <div className="space-y-2">
        <Label htmlFor="username">Username</Label>
        <Input
          id="username"
          value={username}
          onChange={(e) => setUsername(e.target.value)}
          placeholder="username or email"
        />
      </div>

      <div className="space-y-2">
        <Label htmlFor="password">Password</Label>
        <div className="flex gap-2">
          <div className="relative flex-1">
            <Input
              id="password"
              type={showPassword ? 'text' : 'password'}
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              placeholder="password"
            />
            <button
              type="button"
              onClick={() => setShowPassword(s => !s)}
              className="absolute right-2 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground"
              aria-label={showPassword ? 'Hide password' : 'Show password'}
            >
              {showPassword ? <EyeOff className="h-4 w-4" /> : <Eye className="h-4 w-4" />}
            </button>
          </div>
          <Button type="button" variant="outline" onClick={handleGeneratePassword} title="Generate password">
            <RefreshCw className="h-4 w-4" />
          </Button>
        </div>
      </div>

      <div className="space-y-2">
        <Label htmlFor="url">URL</Label>
        <Input
          id="url"
          type="url"
          value={url}
          onChange={(e) => setUrl(e.target.value)}
          placeholder="https://example.com"
        />
      </div>

      {groups.length > 0 && (
        <div className="space-y-2">
          <Label htmlFor="group">Group</Label>
          <select
            id="group"
            value={groupId}
            onChange={(e) => setGroupId(e.target.value)}
            className="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
          >
            <option value="">No group</option>
            {groups.map(g => (
              <option key={g.id} value={g.id}>{g.name}</option>
            ))}
          </select>
        </div>
      )}

      <div className="space-y-2">
        <Label htmlFor="otp">TOTP Secret (otpauth:// URL)</Label>
        <Input
          id="otp"
          value={otpAuthUrl}
          onChange={(e) => setOtpAuthUrl(e.target.value)}
          placeholder="otpauth://totp/..."
        />
        {otpAuthUrl && (
          <div className="mt-2">
            <TotpTimer secret={otpAuthUrl} />
          </div>
        )}
      </div>

      <div className="space-y-2">
        <Label htmlFor="notes">Notes</Label>
        <textarea
          id="notes"
          value={notes}
          onChange={(e) => setNotes(e.target.value)}
          placeholder="Additional notes..."
          className="flex min-h-[80px] w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2"
        />
      </div>

      {error && (
        <p className="text-sm text-destructive">{error}</p>
      )}

      <div className="flex justify-end space-x-2">
        <Button type="button" variant="outline" onClick={onClose}>
          Cancel
        </Button>
        <Button type="submit" disabled={isSaving || !name.trim()}>
          {isSaving ? 'Saving...' : entry?.id ? 'Update' : 'Create'}
        </Button>
      </div>
    </form>
  )
}

interface AddEntryDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  groups: Group[]
  defaultGroupId?: string
  onAdd: (entry: Entry) => void
}

export function AddEntryDialog({ open, onOpenChange, groups, defaultGroupId, onAdd }: AddEntryDialogProps) {
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-[425px] max-h-[90vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle>Add New Entry</DialogTitle>
        </DialogHeader>
        {open && (
          <EntryForm
            groups={groups}
            defaultGroupId={defaultGroupId}
            onClose={() => onOpenChange(false)}
            onSave={(entry) => {
              onAdd(entry)
              onOpenChange(false)
            }}
          />
        )}
      </DialogContent>
    </Dialog>
  )
}

interface EditEntryDialogProps {
  entry: Entry | null
  open: boolean
  onOpenChange: (open: boolean) => void
  groups: Group[]
  onSave: (entry: Entry) => void
}

export function EditEntryDialog({ entry, open, onOpenChange, groups, onSave }: EditEntryDialogProps) {
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-[425px] max-h-[90vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle>Edit Entry</DialogTitle>
        </DialogHeader>
        {open && entry && (
          <EntryForm
            entry={entry}
            groups={groups}
            onClose={() => onOpenChange(false)}
            onSave={(updatedEntry) => {
              onSave(updatedEntry)
              onOpenChange(false)
            }}
          />
        )}
      </DialogContent>
    </Dialog>
  )
}
