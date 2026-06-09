import { useState, useEffect } from 'react'
import { useTranslation } from 'react-i18next'
import { invoke } from '@tauri-apps/api/core'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import {
  FolderOpen,
  FolderInput,
  RotateCcw,
  Check,
  AlertCircle,
  HardDrive,
} from 'lucide-react'
import { useUIStore } from '@/stores/ui'
import { useVaultStore } from '@/stores/vault'

type LocationMode = 'default' | 'custom'

export function StorageSettings() {
  const { t } = useTranslation()
  const showToast = useUIStore((s) => s.showToast)
  const defaultVaultDir = useVaultStore((s) => s.defaultVaultDir)
  const setDefaultVaultDir = useVaultStore((s) => s.setDefaultVaultDir)

  const [builtinDir, setBuiltinDir] = useState<string>('')
  const [currentDir, setCurrentDir] = useState<string>('')
  const [customPath, setCustomPath] = useState<string>('')
  const [isLoading, setIsLoading] = useState(true)
  const [isSaving, setIsSaving] = useState(false)
  const [isPicking, setIsPicking] = useState(false)
  const [mode, setMode] = useState<LocationMode>('default')

  const refresh = async () => {
    setIsLoading(true)
    try {
      const [builtin, current] = await Promise.all([
        invoke<string>('get_builtin_vault_dir'),
        invoke<string>('get_default_vault_dir'),
      ])
      setBuiltinDir(builtin)
      setCurrentDir(current)
      setMode(builtin === current ? 'default' : 'custom')
      setDefaultVaultDir(current)
      if (builtin !== current) {
        setCustomPath(current)
      }
    } catch (err) {
      console.error('Failed to load vault settings:', err)
      showToast({
        message: `${t('errors.unknown')}: ${err}`,
        type: 'error',
      })
    } finally {
      setIsLoading(false)
    }
  }

  useEffect(() => {
    refresh()
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])

  const handlePickFolder = async () => {
    setIsPicking(true)
    try {
      const selected = await invoke<string | null>('pick_vault_folder')
      if (selected) {
        setCustomPath(selected)
        setMode('custom')
      }
    } catch (err) {
      console.error('Failed to pick folder:', err)
      showToast({
        message: `${t('errors.unknown')}: ${err}`,
        type: 'error',
      })
    } finally {
      setIsPicking(false)
    }
  }

  const handleApplyCustom = async () => {
    const trimmed = customPath.trim()
    if (!trimmed) {
      showToast({ message: t('errors.invalidPath'), type: 'error' })
      return
    }
    setIsSaving(true)
    try {
      const result = await invoke<string>('set_default_vault_dir', { path: trimmed })
      setCurrentDir(result)
      setDefaultVaultDir(result)
      setMode('custom')
      showToast({ message: t('vault.locationUpdated'), type: 'success' })
    } catch (err) {
      console.error('Failed to set vault dir:', err)
      showToast({
        message: `${t('errors.storageLocationFailed')}: ${err}`,
        type: 'error',
      })
    } finally {
      setIsSaving(false)
    }
  }

  const handleReset = async () => {
    setIsSaving(true)
    try {
      const result = await invoke<string>('reset_default_vault_dir')
      setCurrentDir(result)
      setDefaultVaultDir(result)
      setCustomPath('')
      setMode('default')
      showToast({ message: t('vault.locationResetSuccess'), type: 'success' })
    } catch (err) {
      console.error('Failed to reset vault dir:', err)
      showToast({
        message: `${t('errors.storageLocationFailed')}: ${err}`,
        type: 'error',
      })
    } finally {
      setIsSaving(false)
    }
  }

  const handleUseDefault = () => {
    setMode('default')
    setCustomPath('')
  }

  if (isLoading) {
    return (
      <div className="flex items-center justify-center py-12 text-muted-foreground">
        {t('common.loading')}
      </div>
    )
  }

  return (
    <div className="space-y-6">
      <div>
        <h3 className="text-lg font-semibold mb-1">{t('settings.storage')}</h3>
        <p className="text-sm text-muted-foreground mb-4">
          {t('settings.storageDesc')}
        </p>
      </div>

      <div className="p-4 bg-muted/50 rounded-lg space-y-3">
        <div className="flex items-center gap-3">
          <HardDrive className="h-5 w-5 text-muted-foreground" />
          <div className="flex-1 min-w-0">
            <div className="font-medium">
              {t('vault.storageLocation')}
            </div>
            <div className="text-xs text-muted-foreground break-all font-mono">
              {currentDir || builtinDir}
            </div>
          </div>
        </div>

        <div className="flex items-center gap-2 text-xs">
          <span
            className={`px-2 py-0.5 rounded-full ${
              mode === 'default'
                ? 'bg-blue-500/20 text-blue-300'
                : 'bg-purple-500/20 text-purple-300'
            }`}
          >
            {mode === 'default'
              ? t('vault.defaultLocationLabel')
              : t('vault.customLocationLabel')}
          </span>
        </div>
      </div>

      <div className="space-y-4">
        <div className="flex items-center justify-between p-4 border rounded-lg">
          <div className="flex items-center gap-3">
            <FolderOpen className="h-5 w-5 text-muted-foreground" />
            <div>
              <div className="font-medium">{t('vault.defaultLocationLabel')}</div>
              <div className="text-xs text-muted-foreground font-mono break-all">
                {builtinDir}
              </div>
            </div>
          </div>
          <Button
            variant={mode === 'default' ? 'default' : 'outline'}
            size="sm"
            onClick={handleUseDefault}
            disabled={mode === 'default' || isSaving}
          >
            {mode === 'default' ? (
              <>
                <Check className="mr-1 h-3 w-3" />
                {t('common.current')}
              </>
            ) : (
              t('vault.useDefaultLocation')
            )}
          </Button>
        </div>

        <div className="p-4 border rounded-lg space-y-3">
          <div className="flex items-center gap-3">
            <FolderInput className="h-5 w-5 text-muted-foreground" />
            <div>
              <div className="font-medium">{t('vault.customLocation')}</div>
              <div className="text-xs text-muted-foreground">
                {t('vault.useCustomLocation')}
              </div>
            </div>
          </div>

          <div className="space-y-2">
            <Label htmlFor="custom-path" className="text-xs text-muted-foreground">
              {t('vault.customLocation')}
            </Label>
            <div className="flex gap-2">
              <Input
                id="custom-path"
                value={customPath}
                onChange={(e) => setCustomPath(e.target.value)}
                placeholder={builtinDir}
                className="flex-1 font-mono text-sm"
              />
              <Button
                variant="outline"
                onClick={handlePickFolder}
                disabled={isPicking}
              >
                <FolderOpen className="mr-2 h-4 w-4" />
                {isPicking ? t('common.loading') : t('vault.browse')}
              </Button>
            </div>
          </div>

          <div className="flex justify-end gap-2">
            <Button
              variant="outline"
              onClick={handleReset}
              disabled={isSaving}
            >
              <RotateCcw className="mr-2 h-4 w-4" />
              {t('vault.locationReset')}
            </Button>
            <Button
              onClick={handleApplyCustom}
              disabled={isSaving || !customPath.trim() || customPath.trim() === currentDir}
            >
              {isSaving ? t('common.loading') : t('common.apply')}
            </Button>
          </div>
        </div>
      </div>

      <div className="flex items-start gap-2 p-3 bg-blue-500/10 border border-blue-500/20 rounded-lg text-xs text-muted-foreground">
        <AlertCircle className="h-4 w-4 text-blue-400 mt-0.5 flex-shrink-0" />
        <div>
          {t('vault.locationHelp')}
        </div>
      </div>
    </div>
  )
}
