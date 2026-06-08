import { useTranslation } from 'react-i18next'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import { Button } from '@/components/ui/button'
import {
  FileText,
  Edit3,
  Eye,
  Wrench,
  HelpCircle,
  Plus,
  Upload,
  Download,
  Settings,
  Info,
  Lock,
} from 'lucide-react'

interface TopMenuProps {
  onLock: () => void
  onImport: () => void
  onExport: () => void
  onAddEntry: () => void
}

export function TopMenu({ onLock, onImport, onExport, onAddEntry }: TopMenuProps) {
  const { t } = useTranslation()

  return (
    <div className="h-12 bg-slate-800 border-b border-white/10 flex items-center px-4 gap-1">
      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <Button variant="ghost" className="h-9 px-3 text-slate-300 hover:text-white hover:bg-white/10">
            <FileText className="w-4 h-4 mr-1" />
            {t('menu.file')}
          </Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent align="start" className="w-48 bg-slate-800 border-white/10">
          <DropdownMenuItem onClick={onAddEntry} className="text-slate-200 hover:text-white hover:bg-white/10">
            <Plus className="w-4 h-4 mr-2" />
            {t('menu.newVault')}
          </DropdownMenuItem>
          <DropdownMenuSeparator className="bg-white/10" />
          <DropdownMenuItem onClick={onImport} className="text-slate-200 hover:text-white hover:bg-white/10">
            <Upload className="w-4 h-4 mr-2" />
            {t('menu.import')}
          </DropdownMenuItem>
          <DropdownMenuItem onClick={onExport} className="text-slate-200 hover:text-white hover:bg-white/10">
            <Download className="w-4 h-4 mr-2" />
            {t('menu.export')}
          </DropdownMenuItem>
          <DropdownMenuSeparator className="bg-white/10" />
          <DropdownMenuItem onClick={onLock} className="text-slate-200 hover:text-white hover:bg-white/10">
            <Lock className="w-4 h-4 mr-2" />
            {t('menu.lockVault')}
          </DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>

      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <Button variant="ghost" className="h-9 px-3 text-slate-300 hover:text-white hover:bg-white/10">
            <Edit3 className="w-4 h-4 mr-1" />
            {t('menu.edit')}
          </Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent align="start" className="w-48 bg-slate-800 border-white/10">
          <DropdownMenuItem className="text-slate-200 hover:text-white hover:bg-white/10">
            <Edit3 className="w-4 h-4 mr-2" />
            {t('common.edit')}
          </DropdownMenuItem>
          <DropdownMenuItem className="text-slate-200 hover:text-white hover:bg-white/10">
            <DeleteIcon className="w-4 h-4 mr-2" />
            {t('common.delete')}
          </DropdownMenuItem>
          <DropdownMenuSeparator className="bg-white/10" />
          <DropdownMenuItem className="text-slate-200 hover:text-white hover:bg-white/10">
            <CopyIcon className="w-4 h-4 mr-2" />
            {t('common.copy')}
          </DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>

      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <Button variant="ghost" className="h-9 px-3 text-slate-300 hover:text-white hover:bg-white/10">
            <Eye className="w-4 h-4 mr-1" />
            {t('menu.view')}
          </Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent align="start" className="w-48 bg-slate-800 border-white/10">
          <DropdownMenuItem className="text-slate-200 hover:text-white hover:bg-white/10">
            <Eye className="w-4 h-4 mr-2" />
            {t('menu.view')}
          </DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>

      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <Button variant="ghost" className="h-9 px-3 text-slate-300 hover:text-white hover:bg-white/10">
            <Wrench className="w-4 h-4 mr-1" />
            {t('menu.tools')}
          </Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent align="start" className="w-48 bg-slate-800 border-white/10">
          <DropdownMenuItem onClick={onImport} className="text-slate-200 hover:text-white hover:bg-white/10">
            <Upload className="w-4 h-4 mr-2" />
            {t('menu.import')}
          </DropdownMenuItem>
          <DropdownMenuItem onClick={onExport} className="text-slate-200 hover:text-white hover:bg-white/10">
            <Download className="w-4 h-4 mr-2" />
            {t('menu.export')}
          </DropdownMenuItem>
          <DropdownMenuSeparator className="bg-white/10" />
          <DropdownMenuItem className="text-slate-200 hover:text-white hover:bg-white/10">
            <Settings className="w-4 h-4 mr-2" />
            {t('menu.settings')}
          </DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>

      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <Button variant="ghost" className="h-9 px-3 text-slate-300 hover:text-white hover:bg-white/10">
            <HelpCircle className="w-4 h-4 mr-1" />
            {t('menu.help')}
          </Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent align="start" className="w-48 bg-slate-800 border-white/10">
          <DropdownMenuItem className="text-slate-200 hover:text-white hover:bg-white/10">
            <HelpCircle className="w-4 h-4 mr-2" />
            Help Center
          </DropdownMenuItem>
          <DropdownMenuSeparator className="bg-white/10" />
          <DropdownMenuItem className="text-slate-200 hover:text-white hover:bg-white/10">
            <Info className="w-4 h-4 mr-2" />
            {t('menu.about')}
          </DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>
    </div>
  )
}

function DeleteIcon(props: React.SVGProps<SVGSVGElement>) {
  return (
    <svg {...props} xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <path d="M3 6h18" />
      <path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6" />
      <path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2" />
    </svg>
  )
}

function CopyIcon(props: React.SVGProps<SVGSVGElement>) {
  return (
    <svg {...props} xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <rect width="14" height="14" x="8" y="8" rx="2" ry="2" />
      <path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2" />
    </svg>
  )
}