import { useTranslation } from 'react-i18next'
import { Button } from '@/components/ui/button'
import { Globe } from 'lucide-react'

export function LanguageSelector() {
  const { i18n, t } = useTranslation()

  const changeLanguage = (lng: string) => {
    i18n.changeLanguage(lng)
  }

  const currentLanguage = i18n.language

  return (
    <div className="flex items-center gap-2">
      <Globe className="w-4 h-4 text-slate-400" />
      <Button
        variant="ghost"
        size="sm"
        onClick={() => changeLanguage(currentLanguage === 'zh' ? 'en' : 'zh')}
        className="text-slate-300 hover:text-white hover:bg-white/10"
      >
        {currentLanguage === 'zh' ? t('language.chinese') : t('language.english')}
      </Button>
    </div>
  )
}