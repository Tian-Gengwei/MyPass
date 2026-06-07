import { useState, useEffect, useCallback } from 'react'
import { cn } from '@/lib/utils'

const MOBILE_BREAKPOINT = 768

export function useIsMobile(): boolean {
  const [isMobile, setIsMobile] = useState(() =>
    typeof window !== 'undefined' ? window.innerWidth < MOBILE_BREAKPOINT : false
  )

  const check = useCallback(() => {
    setIsMobile(window.innerWidth < MOBILE_BREAKPOINT)
  }, [])

  useEffect(() => {
    window.addEventListener('resize', check)
    return () => window.removeEventListener('resize', check)
  }, [check])

  return isMobile
}

export function useMobileLayout() {
  const isMobile = useIsMobile()
  return { isMobile }
}

interface ResponsiveLayoutProps {
  children: React.ReactNode
  className?: string
}

export function ResponsiveLayout({ children, className }: ResponsiveLayoutProps) {
  return (
    <div className={cn('flex h-screen', className)}>
      {children}
    </div>
  )
}

interface MobileNavProps {
  children: React.ReactNode
  className?: string
}

export function MobileNav({ children, className }: MobileNavProps) {
  return (
    <nav className={cn('fixed bottom-0 left-0 right-0 border-t bg-card', className)}>
      <div className="flex h-16 items-center justify-around">
        {children}
      </div>
    </nav>
  )
}

export function MobileNavItem({
  children,
  isActive,
  onClick,
}: {
  children: React.ReactNode
  isActive?: boolean
  onClick?: () => void
}) {
  return (
    <button
      onClick={onClick}
      className={cn(
        'flex flex-col items-center justify-center space-y-1 px-4 py-2',
        isActive ? 'text-primary' : 'text-muted-foreground'
      )}
    >
      {children}
    </button>
  )
}

export function MobilePage({
  children,
  title,
  showBack,
  onBack,
}: {
  children: React.ReactNode
  title?: string
  showBack?: boolean
  onBack?: () => void
}) {
  return (
    <div className="flex flex-1 flex-col md:hidden">
      {title && (
        <header className="flex h-14 items-center border-b bg-card px-4">
          {showBack && (
            <button onClick={onBack} className="mr-2">
              <svg className="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 19l-7-7 7-7" />
              </svg>
            </button>
          )}
          <h1 className="font-semibold">{title}</h1>
        </header>
      )}
      <main className="flex-1 overflow-y-auto p-4 pb-20">{children}</main>
    </div>
  )
}

export function DesktopPage({
  children,
  className,
}: {
  children: React.ReactNode
  className?: string
}) {
  return<div className={cn('hidden flex-1 flex-col md:flex', className)}>{children}</div>
}