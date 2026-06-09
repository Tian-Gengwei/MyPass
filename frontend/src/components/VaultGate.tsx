import { useState } from 'react'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { invoke } from '@tauri-apps/api/core'
import { Shield, Lock, Key, User } from 'lucide-react'

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
      if (!vaultName.trim()) {
        setError('Vault name is required')
        return
      }
    }

    setIsLoading(true)
    try {
      if (mode === 'create') {
        await invoke('create_vault', {
          request: { password, name: vaultName.trim() }
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
    <div className="flex min-h-screen items-center justify-center bg-gradient-to-br from-slate-900 via-purple-900 to-slate-900">
      <div className="w-full max-w-md">
        <div className="text-center mb-8">
          <div className="inline-flex items-center justify-center w-20 h-20 bg-gradient-to-br from-blue-500 to-purple-600 rounded-2xl shadow-2xl mb-4">
            <Shield className="w-10 h-10 text-white" />
          </div>
          <h1 className="text-4xl font-bold text-white mb-2">MyPass</h1>
          <p className="text-slate-300">
            Secure password manager
          </p>
        </div>

        <div className="bg-white/10 backdrop-blur-lg rounded-2xl p-8 shadow-2xl border border-white/20">
          <div className="text-center mb-6">
            <h2 className="text-2xl font-semibold text-white mb-1">
              {mode === 'unlock' ? 'Unlock Vault' : 'Create Vault'}
            </h2>
            <p className="text-slate-300">
              {mode === 'unlock' 
                ? 'Enter your master password to unlock' 
                : 'Create your master password to get started'
              }
            </p>
          </div>

          <form onSubmit={handleSubmit} className="space-y-5">
            {mode === 'create' && (
              <div className="space-y-2">
                <Label htmlFor="vaultName" className="text-slate-200 text-sm font-medium">
                  <div className="flex items-center gap-2">
                    <User className="h-4 w-4" />
                    Vault Name
                  </div>
                </Label>
                <Input
                  id="vaultName"
                  value={vaultName}
                  onChange={(e) => setVaultName(e.target.value)}
                  placeholder="My Personal Vault"
                  className="bg-white/5 border-white/20 text-white placeholder:text-slate-400 h-12 focus:ring-blue-500 focus:border-blue-500"
                  required
                />
              </div>
            )}

            <div className="space-y-2">
              <Label htmlFor="password" className="text-slate-200 text-sm font-medium">
                <div className="flex items-center gap-2">
                  <Lock className="h-4 w-4" />
                  Master Password
                </div>
              </Label>
              <Input
                id="password"
                type="password"
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                placeholder="••••••••"
                className="bg-white/5 border-white/20 text-white placeholder:text-slate-400 h-12 focus:ring-blue-500 focus:border-blue-500"
                required
              />
              {mode === 'create' && (
                <p className="text-xs text-slate-400">
                  At least 8 characters
                </p>
              )}
            </div>

            {mode === 'create' && (
              <div className="space-y-2">
                <Label htmlFor="confirmPassword" className="text-slate-200 text-sm font-medium">
                  <div className="flex items-center gap-2">
                    <Key className="h-4 w-4" />
                    Confirm Password
                  </div>
                </Label>
                <Input
                  id="confirmPassword"
                  type="password"
                  value={confirmPassword}
                  onChange={(e) => setConfirmPassword(e.target.value)}
                  placeholder="••••••••"
                  className="bg-white/5 border-white/20 text-white placeholder:text-slate-400 h-12 focus:ring-blue-500 focus:border-blue-500"
                  required
                />
              </div>
            )}

            {error && (
              <div className="p-3 bg-red-500/20 border border-red-500/30 rounded-lg text-red-300 text-sm">
                {error}
              </div>
            )}

            <Button 
              type="submit" 
              className="w-full h-12 bg-gradient-to-r from-blue-600 to-purple-600 hover:from-blue-700 hover:to-purple-700 text-white font-semibold text-lg shadow-lg hover:shadow-xl transition-all"
              disabled={isLoading}
            >
              {isLoading ? (
                <div className="flex items-center gap-2">
                  <div className="w-5 h-5 border-2 border-white/30 border-t-white rounded-full animate-spin" />
                  {mode === 'unlock' ? 'Unlocking...' : 'Creating...'}
                </div>
              ) : (
                mode === 'unlock' ? 'Unlock Vault' : 'Create Vault'
              )}
            </Button>
          </form>

          <div className="mt-6 pt-6 border-t border-white/20">
            <button
              type="button"
              onClick={() => setMode(mode === 'unlock' ? 'create' : 'unlock')}
              className="w-full text-center text-sm text-slate-300 hover:text-white transition-colors"
            >
              {mode === 'unlock' 
                ? "Don't have a vault? Create one" 
                : 'Already have a vault? Unlock it'
              }
            </button>
          </div>
        </div>
      </div>
    </div>
  )
}
