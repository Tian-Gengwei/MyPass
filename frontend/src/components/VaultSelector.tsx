import { useState, useEffect } from 'react'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { invoke } from '@tauri-apps/api/core'
import { 
  Shield, 
  Lock, 
  Key, 
  User, 
  FolderOpen, 
  Plus, 
  ChevronRight,
  RefreshCw,
  Trash2
} from 'lucide-react'
import { useVaultStore, type VaultItem } from '@/stores/vault'

interface VaultSelectorProps {
  onUnlock: () => void
}

export function VaultSelector({ onUnlock }: VaultSelectorProps) {
  const { t } = useTranslation()
  const [mode, setMode] = useState<'select' | 'unlock' | 'create'>('select')
  const [password, setPassword] = useState('')
  const [confirmPassword, setConfirmPassword] = useState('')
  const [vaultName, setVaultName] = useState('')
  const [selectedVault, setSelectedVault] = useState<VaultItem | null>(null)
  const [error, setError] = useState('')
  const [isLoading, setIsLoading] = useState(false)
  const [isRefreshing, setIsRefreshing] = useState(false)
  
  const vaults = useVaultStore((state) => state.vaults)
  const setVaults = useVaultStore((state) => state.setVaults)
  const setCurrentVaultPath = useVaultStore((state) => state.setCurrentVaultPath)

  const loadVaults = async () => {
    setIsRefreshing(true)
    try {
      const result = await invoke('list_vaults') as VaultItem[]
      setVaults(result)
    } catch (err) {
      console.error('Failed to load vaults:', err)
    } finally {
      setIsRefreshing(false)
    }
  }

  useEffect(() => {
    loadVaults()
  }, [])

  const handleSelectVault = (vault: VaultItem) => {
    setSelectedVault(vault)
    setMode('unlock')
  }

  const handleCreateVault = async (e: React.FormEvent) => {
    e.preventDefault()
    setError('')

    if (password !== confirmPassword) {
      setError(t('errors.passwordMismatch'))
      return
    }
    if (password.length < 8) {
      setError(t('errors.passwordLength'))
      return
    }
    if (!vaultName.trim()) {
      setError(t('errors.vaultNameRequired'))
      return
    }

    setIsLoading(true)
    try {
      await invoke('create_vault', {
        request: { password, name: vaultName.trim() }
      })
      await loadVaults()
      setMode('select')
      setVaultName('')
      setPassword('')
      setConfirmPassword('')
    } catch (err) {
      setError(String(err))
    } finally {
      setIsLoading(false)
    }
  }

  const handleUnlockVault = async (e: React.FormEvent) => {
    e.preventDefault()
    setError('')

    if (!password) {
      setError(t('errors.enterPassword'))
      return
    }

    setIsLoading(true)
    try {
      const path = selectedVault?.path
      await invoke('unlock_vault', {
        request: { password, path }
      })
      if (path) {
        setCurrentVaultPath(path)
      }
      onUnlock()
    } catch (err) {
      setError(String(err))
    } finally {
      setIsLoading(false)
    }
  }

  const goBack = () => {
    setMode('select')
    setSelectedVault(null)
    setPassword('')
    setError('')
  }

  // Select mode - show vault list
  if (mode === 'select') {
    return (
      <div className="flex min-h-screen items-center justify-center bg-gradient-to-br from-slate-900 via-purple-900 to-slate-900">
        <div className="w-full max-w-2xl">
          <div className="text-center mb-8">
            <div className="inline-flex items-center justify-center w-20 h-20 bg-gradient-to-br from-blue-500 to-purple-600 rounded-2xl shadow-2xl mb-4">
              <Shield className="w-10 h-10 text-white" />
            </div>
            <h1 className="text-4xl font-bold text-white mb-2">{t('app.title')}</h1>
            <p className="text-slate-300">
              {t('app.subtitle')}
            </p>
          </div>

          <div className="bg-white/10 backdrop-blur-lg rounded-2xl p-8 shadow-2xl border border-white/20">
            <div className="flex items-center justify-between mb-6">
              <h2 className="text-2xl font-semibold text-white">{t('vault.select')}</h2>
              <div className="flex items-center gap-2">
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={loadVaults}
                  disabled={isRefreshing}
                  className="text-slate-300 hover:text-white hover:bg-white/10"
                >
                  <RefreshCw className={`w-4 h-4 ${isRefreshing ? 'animate-spin' : ''}`} />
                </Button>
                <Button
                  onClick={() => setMode('create')}
                  className="bg-gradient-to-r from-blue-600 to-purple-600 hover:from-blue-700 hover:to-purple-700"
                >
                  <Plus className="w-4 h-4 mr-2" />
                  {t('vault.newVault')}
                </Button>
              </div>
            </div>

            {vaults.length === 0 ? (
              <div className="text-center py-12">
                <FolderOpen className="w-16 h-16 text-slate-500 mx-auto mb-4" />
                <p className="text-slate-400 text-lg">{t('vault.noVaults')}</p>
                <p className="text-slate-500 text-sm mt-2">{t('vault.createFirst')}</p>
              </div>
            ) : (
              <div className="space-y-3 max-h-96 overflow-y-auto pr-2">
                {vaults.map((vault) => (
                  <div
                    key={vault.path}
                    onClick={() => handleSelectVault(vault)}
                    className="flex items-center justify-between p-4 bg-white/5 hover:bg-white/10 rounded-xl cursor-pointer transition-all border border-transparent hover:border-white/20 group"
                  >
                    <div className="flex items-center gap-4">
                      <div className="w-12 h-12 bg-gradient-to-br from-blue-500/20 to-purple-500/20 rounded-xl flex items-center justify-center">
                        <FolderOpen className="w-6 h-6 text-blue-400" />
                      </div>
                      <div>
                        <h3 className="text-white font-medium">{vault.name}</h3>
                        <p className="text-slate-400 text-sm">
                          {t('vault.entries', { count: vault.entry_count })} · {t('vault.groups', { count: vault.group_count })}
                        </p>
                      </div>
                    </div>
                    <ChevronRight className="w-5 h-5 text-slate-500 group-hover:text-white transition-colors" />
                  </div>
                ))}
              </div>
            )}
          </div>
        </div>
      </div>
    )
  }

  // Create mode - create new vault
  if (mode === 'create') {
    return (
      <div className="flex min-h-screen items-center justify-center bg-gradient-to-br from-slate-900 via-purple-900 to-slate-900">
        <div className="w-full max-w-md">
          <div className="text-center mb-8">
            <div className="inline-flex items-center justify-center w-20 h-20 bg-gradient-to-br from-blue-500 to-purple-600 rounded-2xl shadow-2xl mb-4">
              <Shield className="w-10 h-10 text-white" />
            </div>
            <h1 className="text-4xl font-bold text-white mb-2">{t('app.title')}</h1>
            <p className="text-slate-300">{t('app.subtitle')}</p>
          </div>

          <div className="bg-white/10 backdrop-blur-lg rounded-2xl p-8 shadow-2xl border border-white/20">
            <div className="text-center mb-6">
              <h2 className="text-2xl font-semibold text-white mb-1">{t('vault.create')}</h2>
              <p className="text-slate-300">{t('vault.createFirst')}</p>
            </div>

            <form onSubmit={handleCreateVault} className="space-y-5">
              <div className="space-y-2">
                <Label htmlFor="vaultName" className="text-slate-200 text-sm font-medium">
                <div className="flex items-center gap-2">
                  <User className="h-4 w-4" />
                  {t('vault.name')}
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

              <div className="space-y-2">
                <Label htmlFor="password" className="text-slate-200 text-sm font-medium">
                  <div className="flex items-center gap-2">
                    <Lock className="h-4 w-4" />
                    {t('vault.masterPassword')}
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
                <p className="text-xs text-slate-400">At least 8 characters</p>
              </div>

              <div className="space-y-2">
                <Label htmlFor="confirmPassword" className="text-slate-200 text-sm font-medium">
                  <div className="flex items-center gap-2">
                    <Key className="h-4 w-4" />
                    {t('vault.confirmPassword')}
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

              {error && (
                <div className="p-3 bg-red-500/20 border border-red-500/30 rounded-lg text-red-300 text-sm">
                  {error}
                </div>
              )}

              <div className="flex gap-3">
                <Button
                  variant="outline"
                  className="flex-1 border-white/20 text-white hover:bg-white/10"
                  onClick={goBack}
                >
                  {t('vault.cancel')}
                </Button>
                <Button
                  type="submit"
                  className="flex-1 bg-gradient-to-r from-blue-600 to-purple-600 hover:from-blue-700 hover:to-purple-700"
                  disabled={isLoading}
                >
                  {isLoading ? (
                    <div className="flex items-center gap-2">
                      <div className="w-5 h-5 border-2 border-white/30 border-t-white rounded-full animate-spin" />
                      {t('vault.creating')}
                    </div>
                  ) : (
                    t('vault.create')
                  )}
                </Button>
              </div>
            </form>
          </div>
        </div>
      </div>
    )
  }

  // Unlock mode - unlock selected vault
  return (
    <div className="flex min-h-screen items-center justify-center bg-gradient-to-br from-slate-900 via-purple-900 to-slate-900">
      <div className="w-full max-w-md">
        <div className="text-center mb-8">
          <div className="inline-flex items-center justify-center w-20 h-20 bg-gradient-to-br from-blue-500 to-purple-600 rounded-2xl shadow-2xl mb-4">
            <Shield className="w-10 h-10 text-white" />
          </div>
          <h1 className="text-4xl font-bold text-white mb-2">{t('app.title')}</h1>
          <p className="text-slate-300">{t('app.subtitle')}</p>
        </div>

        <div className="bg-white/10 backdrop-blur-lg rounded-2xl p-8 shadow-2xl border border-white/20">
          <div className="text-center mb-6">
            <h2 className="text-2xl font-semibold text-white mb-1">{t('vault.unlock')}</h2>
            <p className="text-slate-300">
              {selectedVault?.name && (
                <span className="text-blue-400">{selectedVault.name}</span>
              )}
            </p>
          </div>

          <form onSubmit={handleUnlockVault} className="space-y-5">
            <div className="space-y-2">
              <Label htmlFor="password" className="text-slate-200 text-sm font-medium">
                <div className="flex items-center gap-2">
                  <Lock className="h-4 w-4" />
                  {t('vault.masterPassword')}
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
            </div>

            {error && (
              <div className="p-3 bg-red-500/20 border border-red-500/30 rounded-lg text-red-300 text-sm">
                {error}
              </div>
            )}

            <div className="flex gap-3">
              <Button
                variant="outline"
                className="flex-1 border-white/20 text-white hover:bg-white/10"
                onClick={goBack}
              >
                {t('vault.back')}
              </Button>
              <Button
                type="submit"
                className="flex-1 bg-gradient-to-r from-blue-600 to-purple-600 hover:from-blue-700 hover:to-purple-700"
                disabled={isLoading}
              >
                {isLoading ? (
                  <div className="flex items-center gap-2">
                    <div className="w-5 h-5 border-2 border-white/30 border-t-white rounded-full animate-spin" />
                    {t('vault.unlocking')}
                  </div>
                ) : (
                  t('vault.unlock')
                )}
              </Button>
            </div>
          </form>
        </div>
      </div>
    </div>
  )
}