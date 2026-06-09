import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Dialog, DialogContent, DialogHeader, DialogTitle } from '@/components/ui/dialog'
import { Button } from '@/components/ui/button'
import { Lock, Fingerprint, Key, Smartphone, Globe, Shield, HardDrive } from 'lucide-react'
import { PasskeyManager } from './PasskeyManager'
import { StorageSettings } from './StorageSettings'

interface SettingsDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
}

type SettingsTab = 'security' | 'storage' | 'appearance' | 'sync' | 'about'

export function SettingsDialog({ open, onOpenChange }: SettingsDialogProps) {
  const { t } = useTranslation()
  const [activeTab, setActiveTab] = useState<SettingsTab>('security')

  const tabs = [
    { id: 'security' as SettingsTab, label: t('settings.security'), icon: Shield },
    { id: 'storage' as SettingsTab, label: t('settings.storage'), icon: HardDrive },
    { id: 'appearance' as SettingsTab, label: t('settings.appearance'), icon: Smartphone },
    { id: 'sync' as SettingsTab, label: t('settings.sync'), icon: Globe },
    { id: 'about' as SettingsTab, label: t('settings.about'), icon: Lock },
  ]

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-3xl h-[80vh] flex flex-col p-0 overflow-hidden">
        <div className="flex h-full">
          {/* Sidebar */}
          <div className="w-48 border-r bg-muted/30 p-4">
            <DialogHeader className="mb-6">
              <DialogTitle>{t('settings.title')}</DialogTitle>
            </DialogHeader>
            <div className="space-y-1">
              {tabs.map((tab) => {
                const Icon = tab.icon
                return (
                  <Button
                    key={tab.id}
                    variant={activeTab === tab.id ? 'default' : 'ghost'}
                    className="w-full justify-start"
                    onClick={() => setActiveTab(tab.id)}
                  >
                    <Icon className="mr-2 h-4 w-4" />
                    {tab.label}
                  </Button>
                )
              })}
            </div>
          </div>

          {/* Content */}
          <div className="flex-1 p-6 overflow-y-auto">
            {activeTab === 'security' && <SecuritySettings />}
            {activeTab === 'storage' && <StorageSettings />}
            {activeTab === 'appearance' && <AppearanceSettings />}
            {activeTab === 'sync' && <SyncSettings />}
            {activeTab === 'about' && <AboutSettings />}
          </div>
        </div>
      </DialogContent>
    </Dialog>
  )
}

function SecuritySettings() {
  const { t } = useTranslation()

  return (
    <div className="space-y-6">
      <div>
        <h3 className="text-lg font-semibold mb-4">{t('settings.security')}</h3>
        
        {/* Passkeys */}
        <PasskeyManager />

        {/* PIN */}
        <div className="mt-6 p-4 bg-muted/50 rounded-lg">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-3">
              <Key className="h-5 w-5 text-muted-foreground" />
              <div>
                <div className="font-medium">{t('security.pin')}</div>
                <div className="text-sm text-muted-foreground">{t('security.pinEnabled')}</div>
              </div>
            </div>
            <Button variant="outline">{t('security.setPin')}</Button>
          </div>
        </div>

        {/* Biometrics */}
        <div className="mt-4 p-4 bg-muted/50 rounded-lg">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-3">
              <Fingerprint className="h-5 w-5 text-muted-foreground" />
              <div>
                <div className="font-medium">{t('security.biometrics')}</div>
                <div className="text-sm text-muted-foreground">{t('security.enableBiometrics')}</div>
              </div>
            </div>
            <Button variant="outline">{t('common.add')}</Button>
          </div>
        </div>

        {/* Auto-lock */}
        <div className="mt-4 p-4 bg-muted/50 rounded-lg">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-3">
              <Lock className="h-5 w-5 text-muted-foreground" />
              <div>
                <div className="font-medium">{t('security.autoLock')}</div>
                <div className="text-sm text-muted-foreground">{t('security.autoLockAfter')}</div>
              </div>
            </div>
            <div className="flex items-center gap-2">
              <input
                type="number"
                defaultValue={5}
                className="w-16 px-2 py-1 border rounded text-center"
              />
              <span className="text-muted-foreground">{t('security.minutes')}</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}

function AppearanceSettings() {
  const { t } = useTranslation()
  return (
    <div className="space-y-6">
      <h3 className="text-lg font-semibold">{t('settings.appearance')}</h3>
      <div className="p-4 bg-muted/50 rounded-lg text-muted-foreground">
        Appearance settings coming soon...
      </div>
    </div>
  )
}

function SyncSettings() {
  const { t } = useTranslation()
  return (
    <div className="space-y-6">
      <h3 className="text-lg font-semibold">{t('settings.sync')}</h3>
      <div className="p-4 bg-muted/50 rounded-lg text-muted-foreground">
        Sync settings coming soon...
      </div>
    </div>
  )
}

function AboutSettings() {
  const { t } = useTranslation()
  return (
    <div className="space-y-6">
      <h3 className="text-lg font-semibold">{t('settings.about')}</h3>
      <div className="p-4 bg-muted/50 rounded-lg">
        <div className="font-medium">MyPass</div>
        <div className="text-sm text-muted-foreground">Version 1.0.0</div>
        <div className="text-sm text-muted-foreground mt-2">
          Local-first, cross-platform, end-to-end encrypted password manager
        </div>
      </div>
    </div>
  )
}
