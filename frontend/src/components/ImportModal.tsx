import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/api/dialog'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { X, Upload, FileText, CheckCircle, AlertCircle } from 'lucide-react'
import { useVaultStore } from '@/stores/vault'
import { toast } from '@/hooks/use-toast'

interface ImportModalProps {
  isOpen: boolean
  onClose: () => void
}

export function ImportModal({ isOpen, onClose }: ImportModalProps) {
  const { t } = useTranslation()
  const [format, setFormat] = useState('')
  const [filePath, setFilePath] = useState('')
  const [isImporting, setIsImporting] = useState(false)
  const [result, setResult] = useState<'success' | 'error' | null>(null)
  const [resultMessage, setResultMessage] = useState('')
  
  const setEntries = useVaultStore((state) => state.setEntries)
  const setGroups = useVaultStore((state) => state.setGroups)
  const entries = useVaultStore((state) => state.entries)

  const handleFileSelect = async () => {
    const selected = await open({
      multiple: false,
      filters: [
        { name: 'KeePass', extensions: ['kdbx'] },
        { name: 'JSON', extensions: ['json'] },
        { name: 'CSV', extensions: ['csv'] },
        { name: 'All Files', extensions: ['*'] },
      ],
    })
    
    if (selected && typeof selected === 'string') {
      setFilePath(selected)
    }
  }

  const handleImport = async () => {
    if (!filePath || !format) {
      toast({
        title: t('common.error'),
        description: format ? t('import.selectFile') : t('import.selectFormat'),
        variant: 'destructive',
      })
      return
    }

    setIsImporting(true)
    setResult(null)

    try {
      let count: number
      
      switch (format) {
        case 'keepass':
          count = await invoke('import_keepass', { file_path: filePath }) as number
          break
        case 'bitwarden':
          count = await invoke('import_bitwarden', { file_path: filePath }) as number
          break
        case 'bitwarden_csv':
          count = await invoke('import_bitwarden_csv', { file_path: filePath }) as number
          break
        case 'chrome_csv':
          count = await invoke('import_chrome_csv', { file_path: filePath }) as number
          break
        default:
          throw new Error(t('import.selectFormat'))
      }

      setResult('success')
      setResultMessage(t('import.success', { count }))
      toast({
        title: t('common.success'),
        description: t('import.success', { count }),
      })
      
    } catch (error) {
      setResult('error')
      setResultMessage(String(error))
      toast({
        title: t('common.error'),
        description: String(error),
        variant: 'destructive',
      })
    } finally {
      setIsImporting(false)
    }
  }

  const handleClose = () => {
    onClose()
    setFilePath('')
    setFormat('')
    setResult(null)
    setResultMessage('')
  }

  if (!isOpen) return null

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center">
      <div className="absolute inset-0 bg-black/60 backdrop-blur-sm" onClick={handleClose} />
      
      <div className="relative w-full max-w-md bg-slate-900 rounded-2xl shadow-2xl border border-white/20 overflow-hidden">
        <div className="flex items-center justify-between p-6 border-b border-white/10">
          <div className="flex items-center gap-3">
            <div className="w-10 h-10 bg-gradient-to-br from-blue-500/20 to-purple-500/20 rounded-xl flex items-center justify-center">
              <Upload className="w-5 h-5 text-blue-400" />
            </div>
            <div>
              <h2 className="text-xl font-semibold text-white">{t('import.title')}</h2>
              <p className="text-sm text-slate-400">{t('import.selectFormat')}</p>
            </div>
          </div>
          <Button variant="ghost" size="sm" onClick={handleClose} className="text-slate-400 hover:text-white">
            <X className="w-5 h-5" />
          </Button>
        </div>

        <div className="p-6 space-y-5">
          <div className="space-y-2">
            <Label className="text-slate-200 text-sm font-medium">
              <div className="flex items-center gap-2">
                <FileText className="w-4 h-4" />
                {t('import.selectFormat')}
              </div>
            </Label>
            <Select value={format} onValueChange={setFormat}>
              <SelectTrigger className="bg-white/5 border-white/20 text-white">
                <SelectValue placeholder={t('import.selectFormat')} />
              </SelectTrigger>
              <SelectContent className="bg-slate-800 border-white/10">
                <SelectItem value="keepass">KeePass KDBX</SelectItem>
                <SelectItem value="bitwarden">Bitwarden JSON</SelectItem>
                <SelectItem value="bitwarden_csv">Bitwarden CSV</SelectItem>
                <SelectItem value="chrome_csv">Chrome CSV</SelectItem>
              </SelectContent>
            </Select>
          </div>

          <div className="space-y-2">
            <Label className="text-slate-200 text-sm font-medium">{t('import.selectFile')}</Label>
            <div className="flex items-center gap-3">
              <Input
                value={filePath}
                onChange={(e) => setFilePath(e.target.value)}
                placeholder={t('import.selectFile')}
                className="flex-1 bg-white/5 border-white/20 text-white placeholder:text-slate-500"
                disabled
              />
              <Button onClick={handleFileSelect} className="bg-gradient-to-r from-blue-600 to-purple-600">
                {t('common.select')}
              </Button>
            </div>
          </div>

          {result && (
            <div className={`p-4 rounded-xl border ${result === 'success' ? 'bg-green-500/20 border-green-500/30' : 'bg-red-500/20 border-red-500/30'}`}>
              <div className="flex items-center gap-3">
                {result === 'success' ? (
                  <CheckCircle className="w-5 h-5 text-green-400" />
                ) : (
                  <AlertCircle className="w-5 h-5 text-red-400" />
                )}
                <span className={result === 'success' ? 'text-green-300' : 'text-red-300'}>
                  {resultMessage}
                </span>
              </div>
            </div>
          )}
        </div>

        <div className="flex gap-3 p-6 border-t border-white/10">
          <Button variant="outline" className="flex-1 border-white/20 text-white hover:bg-white/10" onClick={handleClose}>
            {t('common.cancel')}
          </Button>
          <Button 
            className="flex-1 bg-gradient-to-r from-blue-600 to-purple-600" 
            onClick={handleImport}
            disabled={isImporting || !filePath || !format}
          >
            {isImporting ? (
              <div className="flex items-center gap-2">
                <div className="w-5 h-5 border-2 border-white/30 border-t-white rounded-full animate-spin" />
                {t('import.importing')}
              </div>
            ) : (
              t('common.import')
            )}
          </Button>
        </div>
      </div>
    </div>
  )
}