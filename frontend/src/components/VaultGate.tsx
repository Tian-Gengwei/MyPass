import { useState } from 'react'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { invoke } from '@tauri-apps/api/core'

interface VaultGateProps {
  onUnlock: () => void
}

export function VaultGate({ onUnlock }: VaultGateProps) {
  const [mode, setMode] = useState<'unlock' | 'create'>('unlock')
  const [password, setPassword] = useState('')
  const [confirmPassword, setConfirmPassword] = useState('')
  const [vaultName, setVaultName] = useState('')
  const [error, setError] = useState('')
  const [isLoading, setIsLoading] = useState(false)

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setError('')

    if (mode === 'create') {
      if (password !== confirmPassword) {
        setError('Passwords do not match')
        return
      }
      if (password.length < 8) {
        setError('Password must be at least 8 characters')
        return
      }
    }

    setIsLoading(true)
    try {
      if (mode === 'create') {
        await invoke('create_vault', {
          request: { password, name: vaultName }
        })
      } else {
        await invoke('unlock_vault', {
          request: { password }
        })
      }
      onUnlock()
    } catch (err) {
      setError(String(err))
    } finally {
      setIsLoading(false)
    }
  }

  return (
    <div className="flex min-h-screen items-center justify-center">
      <div className="w-full max-w-md space-y-8 rounded-lg border bg-card p-8">
        <div className="text-center">
          <h1 className="text-3xl font-bold">MyPass</h1>
          <p className="mt-2 text-muted-foreground">
            {mode === 'unlock' ? 'Unlock your vault' : 'Create a new vault'}
          </p>
        </div>

        <form onSubmit={handleSubmit} className="space-y-4">
          {mode === 'create' && (
            <div className="space-y-2">
              <label className="text-sm font-medium">Vault Name</label>
              <Input
                value={vaultName}
                onChange={(e) => setVaultName(e.target.value)}
                placeholder="My Vault"
                required
              />
            </div>
          )}

          <div className="space-y-2">
            <label className="text-sm font-medium">Master Password</label>
            <Input
              type="password"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              placeholder="Enter your master password"
              required
            />
          </div>

          {mode === 'create' && (
            <div className="space-y-2">
              <label className="text-sm font-medium">Confirm Password</label>
              <Input
                type="password"
                value={confirmPassword}
                onChange={(e) => setConfirmPassword(e.target.value)}
                placeholder="Confirm your password"
                required
              />
            </div>
          )}

          {error && (
            <p className="text-sm text-destructive">{error}</p>
          )}

          <Button type="submit" className="w-full" disabled={isLoading}>
            {isLoading ? 'Please wait...' : mode === 'unlock' ? 'Unlock' : 'Create Vault'}
          </Button>
        </form>

        <div className="text-center">
          <button
            type="button"
            onClick={() => setMode(mode === 'unlock' ? 'create' : 'unlock')}
            className="text-sm text-primary hover:underline"
          >
            {mode === 'unlock'
              ? "Don't have a vault? Create one"
              : 'Already have a vault? Unlock it'}
          </button>
        </div>
      </div>
    </div>
  )
}