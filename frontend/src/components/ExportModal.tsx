import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { invoke } from '@tauri-apps/api/core'
import { save } from '@tauri-apps/api/dialog'
import { Button } from '@/components/ui/button'
import { Label } from '@/components/ui/label'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { X, Download, FileText, CheckCircle, AlertCircle } from 'lucide-react'
import { useVaultStore } from '@/stores/vault'
import { toast } from '@/hooks/use-toast'

interface ExportModalProps {
  isOpen: boolean
  onClose: () => void
}

export function ExportModal({ isOpen, onClose }: ExportModalProps) {
  const { t } = useTranslation()
  const [format, setFormat] = useState('')
  const [isExporting, setIsExporting] = useState(false)
  const [result, setResult] = useState<'success' | 'error' | null>(null)
  const [resultMessage, setResultMessage] = useState('')
  
  const entries = useVaultStore((state) => state.entries)

  const handleExport = async () => {
    if (!format) {
      toast({
        title: t('common.error'),
        description: t('export.selectFormat'),
        variant: 'destructive',
      })
      return
    }

    const extension = format === 'csv' ? 'csv' : 'json'
    const defaultName = `mypass_export.${extension}`
    
    const filePath = await save({
      defaultPath: defaultName,
      filters: [
        { name: format.toUpperCase(), extensions: [extension] },
        { name: 'All Files', extensions: ['*'] },
      ],
    })

    if (!filePath) return

    setIsExporting(true)
    setResult(null)

    try {
      if (format === 'csv') {
        await invoke('export_csv', { file_path: filePath })
      } else {
        await invoke('export_json', { file_path: filePath })
      }

      setResult('success')
      setResultMessage(t('export.success'))
      toast({
        title: t('common.success'),
        description: t('export.success'),
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
      setIsExporting(false)
    }
  }

  const handleClose = () => {
    onClose()
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
            <div className="w-10 h-10 bg-gradient-to-br from-green-500/20 to-emerald-500/20 rounded-xl flex items-center justify-center">
              <Download className="w-5 h-5 text-green-400" />
            </div>
            <div>
              <h2 className="text-xl font-semibold text-white">{t('export.title')}</h2>
              <p className="text-sm text-slate-400">{t('export.selectFormat')}</p>
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
                {t('export.selectFormat')}
              </div>
            </Label>
            <Select value={format} onValueChange={setFormat}>
              <SelectTrigger className="bg-white/5 border-white/20 text-white">
                <SelectValue placeholder={t('export.selectFormat')} />
              </SelectTrigger>
              <SelectContent className="bg-slate-800 border-white/10">
                <SelectItem value="csv">CSV</SelectItem>
                <SelectItem value="json">JSON</SelectItem>
              </SelectContent>
            </Select>
          </div>

          <div className="p-4 bg-white/5 rounded-xl">
            <p className="text-sm text-slate-400">
              {t('common.info')}: {t('export.entries', { count: entries.length })}
            </p>
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
            className="flex-1 bg-gradient-to-r from-green-600 to-emerald-600" 
            onClick={handleExport}
            disabled={isExporting || !format}
          >
            {isExporting ? (
              <div className="flex items-center gap-2">
                <div className="w-5 h-5 border-2 border-white/30 border-t-white rounded-full animate-spin" />
                {t('export.exporting')}
              </div>
            ) : (
              t('common.export')
            )}
          </Button>
        </div>
      </div>
    </div>
  )
}