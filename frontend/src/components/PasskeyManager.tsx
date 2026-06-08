import { useState, useEffect } from 'react'
import { useTranslation } from 'react-i18next'
import { invoke } from '@tauri-apps/api/core'
import { Button } from '@/components/ui/button'
import { useVaultStore } from '@/stores/vault'
import { Key, Usb, Trash2, Plus } from 'lucide-react'

interface PasskeyInfo {
  credential_id: string
  user_display_name: string
  rp_name: string
  authenticator: {
    id: string
    name: string
    authenticator_type: 'platform' | 'cross-platform'
    supports_user_verification: boolean
    transports: string[]
  }
  created_at: number
  last_used_at: number | null
}

export function PasskeyManager() {
  const { t } = useTranslation()
  const [passkeys, setPasskeys] = useState<PasskeyInfo[]>([])
  const [loading, setLoading] = useState(false)
  const currentVaultPath = useVaultStore((state) =>
    state.currentVaultPath)

  // 简化的vault id提取方法
  const getVaultId = () => {
    if (currentVaultPath) {
      return currentVaultPath.split('/').pop() || currentVaultPath.split('\\').pop() || 'default'
    }
    return 'default'
  }

  const loadPasskeys = async () => {
    try {
      setLoading(true)
      const vaultId = getVaultId()
      const result = await invoke<PasskeyInfo[]>('webauthn_list_passkeys', {
        vaultId
      })
      setPasskeys(result)
    } catch (error) {
      console.error('Failed to load passkeys:', error)
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => {
    loadPasskeys()
  }, [])

  const addPasskey = async (type: 'platform' | 'cross-platform') => {
    try {
      const vaultId = getVaultId()
      const options = await invoke<any>('webauthn_get_register_options', {
        vaultId,
        username: 'MyPass User'
      })

      // 使用浏览器 WebAuthn API
      const credential = await navigator.credentials.create({
        publicKey: {
          challenge: base64UrlToArrayBuffer(options.challenge),
          rp: { id: options.relying_party_id, name: options.relying_party_name },
          user: {
            id: base64UrlToArrayBuffer(options.user_id),
            name: options.username,
            displayName: options.user_display_name,
          },
          pubKeyCredParams: [
            { type: 'public-key', alg: -7 },
            { type: 'public-key', alg: -257 },
          ],
          authenticatorSelection: {
            authenticatorAttachment: type === 'platform' ? 'platform' : 'cross-platform',
            userVerification: 'required',
          },
          timeout: 60000,
        },
      }) as PublicKeyCredential

      if (credential) {
        const response = credential.response as AuthenticatorAttestationResponse
        const credentialId = arrayBufferToBase64Url(credential.rawId)
        
        await invoke('webauthn_complete_registration', {
          vaultId,
          credentialId,
          authenticatorData: JSON.stringify({
            authenticator_name: type === 'platform' 
              ? t('passkey.authenticatorPlatform') 
              : t('passkey.authenticatorCrossPlatform')
          })
        })
        
        await loadPasskeys()
      }
    } catch (error) {
      console.error('Failed to add passkey:', error)
    }
  }

  const removePasskey = async (credentialId: string) => {
    if (!confirm(t('passkey.removeConfirm'))) return

    try {
      const vaultId = getVaultId()
      await invoke('webauthn_remove_passkey', {
        vaultId,
        credentialId
      })
      await loadPasskeys()
    } catch (error) {
      console.error('Failed to remove passkey:', error)
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

  const formatDate = (timestamp: number) => {
    return new Date(timestamp * 1000).toLocaleDateString()
  }

  return (
    <div className="space-y-4">
      <div>
        <div className="flex items-center justify-between">
          <div>
            <h4 className="font-medium">{t('passkey.title')}</h4>
            <p className="text-sm text-muted-foreground">{t('passkey.description')}</p>
          </div>
        </div>
      </div>

      {loading ? (
        <div className="text-center py-8 text-muted-foreground">
          Loading...
        </div>
      ) : passkeys.length === 0 ? (
        <div className="text-center py-8 text-muted-foreground">
          <Key className="mx-auto h-12 w-12 mb-4 opacity-50" />
          <p>{t('passkey.noPasskeys')}</p>
        </div>
      ) : (
        <div className="space-y-2">
          {passkeys.map((passkey) => (
            <div
              key={passkey.credential_id}
              className="flex items-center justify-between p-4 bg-muted/50 rounded-lg"
            >
              <div className="flex items-center gap-3">
                {passkey.authenticator.authenticator_type === 'platform' ? (
                  <Key className="h-5 w-5 text-muted-foreground" />
                ) : (
                  <Usb className="h-5 w-5 text-muted-foreground" />
                )}
                <div>
                  <div className="font-medium">{passkey.authenticator.name}</div>
                  <div className="text-sm text-muted-foreground">
                    {t('passkey.created')}: {formatDate(passkey.created_at)}
                    {passkey.last_used_at && (
                      <> • {t('passkey.lastUsed')}: {formatDate(passkey.last_used_at)}
                    </>
                    )}
                  </div>
                </div>
              </div>
              <Button
                variant="ghost"
                size="icon"
                onClick={() => removePasskey(passkey.credential_id)}
              >
                <Trash2 className="h-4 w-4" />
              </Button>
            </div>
          ))}
        </div>
      )}

      <div className="flex gap-3 pt-2">
        <Button
        onClick={() => addPasskey('platform')}
        variant="outline"
      >
        <Key className="mr-2 h-4 w-4" />
        {t('passkey.addPasskey')}
      </Button>
      <Button
        onClick={() => addPasskey('cross-platform')}
        variant="outline"
      >
        <Usb className="mr-2 h-4 w-4" />
        {t('passkey.addHardwareKey')}
      </Button>
    </div>
  )
}
