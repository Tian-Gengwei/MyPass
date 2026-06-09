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
  Fingerprint,
  FolderInput,
  HardDrive,
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
  const [customPath, setCustomPath] = useState('')
  const [useCustomPath, setUseCustomPath] = useState(false)
  const [isPicking, setIsPicking] = useState(false)
  const [defaultDir, setDefaultDir] = useState('')
  const [selectedVault, setSelectedVault] = useState<VaultItem | null>(null)
  const [error, setError] = useState('')
  const [isLoading, setIsLoading] = useState(false)
  const [isRefreshing, setIsRefreshing] = useState(false)
  const [hasPasskey, setHasPasskey] = useState(false)
  const [isUsingPasskey, setIsUsingPasskey] = useState(false)

  const vaults = useVaultStore((state) => state.vaults)
  const setVaults = useVaultStore((state) => state.setVaults)
  const setCurrentVaultPath = useVaultStore((state) => state.setCurrentVaultPath)
  const defaultVaultDir = useVaultStore((state) => state.defaultVaultDir)
  const setDefaultVaultDir = useVaultStore((state) => state.setDefaultVaultDir)

  const loadVaults = async () => {
    setIsRefreshing(true)
    try {
      const [result, currentDir] = await Promise.all([
        invoke('list_vaults') as Promise<VaultItem[]>,
        invoke<string>('get_default_vault_dir'),
      ])
      setVaults(result)
      setDefaultDir(currentDir)
      setDefaultVaultDir(currentDir)
    } catch (err) {
      console.error('Failed to load vaults:', err)
    } finally {
      setIsRefreshing(false)
    }
  }

  useEffect(() => {
    loadVaults()
  }, [])

  const handlePickCreateFolder = async () => {
    setIsPicking(true)
    try {
      const selected = await invoke<string | null>('pick_vault_folder')
      if (selected) {
        setCustomPath(selected)
        setUseCustomPath(true)
      }
    } catch (err) {
      console.error('Failed to pick folder:', err)
    } finally {
      setIsPicking(false)
    }
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
    if (useCustomPath && !customPath.trim()) {
      setError(t('errors.invalidPath'))
      return
    }

    setIsLoading(true)
    try {
      const path = useCustomPath && customPath.trim() ? customPath.trim() : null
      await invoke('create_vault', {
        request: { password, name: vaultName.trim(), path }
      })
      await loadVaults()
      setMode('select')
      setVaultName('')
      setPassword('')
      setConfirmPassword('')
      setCustomPath('')
      setUseCustomPath(false)
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
    setHasPasskey(false)
  }

  const handleSelectVault = (vault: VaultItem) => {
    setSelectedVault(vault)
    setMode('unlock')
    // 检查是否有通行密钥
    checkPasskeyAvailability(vault)
  }

  const checkPasskeyAvailability = async (vault: VaultItem) => {
    try {
      const vaultId = vault.path.split(/[\\/]/).pop() || vault.path
      const result = await invoke('webauthn_has_passkey', { vaultId })
      setHasPasskey(!!result)
    } catch (err) {
      console.error('Failed to check passkey availability:', err)
      setHasPasskey(false)
    }
  }

  const handleUnlockWithPasskey = async () => {
    if (!selectedVault) return
    
    setIsUsingPasskey(true)
    setError('')
    
    try {
      const vaultId = selectedVault.path.split(/[\\/]/).pop() || selectedVault.path
      
      // 获取认证选项
      const options = await invoke<any>('webauthn_get_authenticate_options', {
        vaultId
      })
      
      // 转换 base64url 到 ArrayBuffer
      const challenge = base64UrlToArrayBuffer(options.challenge)
      const allowCredentials = options.allowed_credentials?.map((cred: any) => ({
        id: base64UrlToArrayBuffer(cred.id),
        type: 'public-key',
        transports: cred.transports
      }))
      
      // 使用浏览器 WebAuthn API 进行认证
      const credential = await navigator.credentials.get({
        publicKey: {
          challenge,
          allowCredentials,
          userVerification: options.user_verification || 'preferred',
          rpId: options.relying_party_id,
          timeout: 60000,
        }
      }) as PublicKeyCredential
      
      if (credential) {
        const credentialId = arrayBufferToBase64Url(credential.rawId)
        const response = credential.response as AuthenticatorAssertionResponse
        
        // 完成认证
        const success = await invoke('webauthn_complete_authentication', {
          vaultId,
          credentialId,
          assertion: JSON.stringify({
            authenticatorData: arrayBufferToBase64Url(response.authenticatorData),
            clientDataJSON: arrayBufferToBase64Url(response.clientDataJSON),
            signature: arrayBufferToBase64Url(response.signature),
            userHandle: response.userHandle ? arrayBufferToBase64Url(response.userHandle) : null
          })
        })
        
        if (success) {
          // 使用 master password 解锁（需要先设置通行密钥时存储加密的主密码）
          // 这里为了演示，我们仍然需要输入主密码，或者需要修改后端来支持直接通行密钥解锁
          // 目前，我们先显示成功提示，但实际解锁还需要密码
          setError(t('passkey.passkeyAdded'))
          
          // 实际项目中，这里应该有完整的实现
          // 临时方案：显示提示，但仍然使用密码解锁
        }
      }
    } catch (err) {
      console.error('Passkey authentication failed:', err)
      setError(t('errors.passkeyFailed') || 'Passkey authentication failed')
    } finally {
      setIsUsingPasskey(false)
    }
  }

  // 工具函数
  const base64UrlToArrayBuffer = (base64Url: string): ArrayBuffer => {
    const base64 = base64Url.replace(/-/g, '+').replace(/_/g, '/')
    const padLength = (4 - (base64.length % 4)) % 4
    const padded = base64.padEnd(base64.length + padLength, '=')
    const binary = atob(padded)
    const bytes = new Uint8Array(binary.length)
    for (let i = 0; i < binary.length; i++) {
      bytes[i] = binary.charCodeAt(i)
    }
    return bytes.buffer
  }

  const arrayBufferToBase64Url = (buffer: ArrayBuffer): string => {
    const bytes = new Uint8Array(buffer)
    let binary = ''
    for (let i = 0; i < bytes.byteLength; i++) {
      binary += String.fromCharCode(bytes[i])
    }
    return btoa(binary).replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '')
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

              <div className="space-y-3 p-4 bg-white/5 border border-white/10 rounded-xl">
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-2 text-slate-200 text-sm font-medium">
                    <HardDrive className="h-4 w-4" />
                    {t('vault.storageLocation')}
                  </div>
                  <label className="flex items-center gap-2 text-xs text-slate-300 cursor-pointer">
                    <input
                      type="checkbox"
                      checked={useCustomPath}
                      onChange={(e) => setUseCustomPath(e.target.checked)}
                      className="rounded border-white/20"
                    />
                    {t('vault.useCustomLocation')}
                  </label>
                </div>

                {!useCustomPath ? (
                  <div className="text-xs text-slate-400 font-mono break-all p-2 bg-black/20 rounded">
                    {defaultDir || '...'}
                  </div>
                ) : (
                  <div className="flex gap-2">
                    <Input
                      value={customPath}
                      onChange={(e) => setCustomPath(e.target.value)}
                      placeholder={t('vault.customLocationPlaceholder')}
                      className="flex-1 bg-white/5 border-white/20 text-white placeholder:text-slate-500 text-sm font-mono"
                    />
                    <Button
                      type="button"
                      variant="outline"
                      size="sm"
                      onClick={handlePickCreateFolder}
                      disabled={isPicking}
                      className="border-white/20 text-white hover:bg-white/10"
                    >
                      <FolderInput className="h-4 w-4" />
                    </Button>
                  </div>
                )}
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

            {hasPasskey && (
              <Button
                type="button"
                variant="outline"
                className="w-full border-white/20 text-white hover:bg-white/10"
                onClick={handleUnlockWithPasskey}
                disabled={isUsingPasskey}
              >
                <Fingerprint className="mr-2 h-4 w-4" />
                {isUsingPasskey ? t('common.loading') || 'Loading...' : t('passkey.unlockWithPasskey')}
              </Button>
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