import { useEffect, useRef, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { AlertCircle, Check, Copy } from 'lucide-react'
import { cn } from '@/lib/utils'

interface TotpCode {
  code: string
  remaining_secs: number
}

const TOTP_PERIOD = 30
let sharedInterval: number | null = null
let listeners: Set<(code: TotpCode | null, error: string | null) => void> = new Set()
let currentSecret: string | null = null
let lastError: string | null = null
let lastCode: TotpCode | null = null
let lastFetchedPeriod = -1

const notify = () => {
  for (const fn of listeners) {
    fn(lastCode, lastError)
  }
}

const fetchOnce = async () => {
  if (!currentSecret) return
  try {
    const result = await invoke<TotpCode>('generate_totp', { secret: currentSecret })
    lastCode = result
    lastError = null
  } catch (err) {
    lastCode = null
    lastError = err instanceof Error ? err.message : String(err)
  }
  notify()
}

const ensureLoop = () => {
  if (sharedInterval !== null) return
  const now = Math.floor(Date.now() / 1000)
  const msToNext = (TOTP_PERIOD - (now % TOTP_PERIOD)) * 1000
  setTimeout(() => {
    fetchOnce()
    sharedInterval = window.setInterval(() => {
      const period = Math.floor(Date.now() / 1000) / TOTP_PERIOD
      if (Math.floor(period) !== lastFetchedPeriod) {
        lastFetchedPeriod = Math.floor(period)
        fetchOnce()
      }
      const code = lastCode
      if (code) {
        const newRemaining = TOTP_PERIOD - (Math.floor(Date.now() / 1000) % TOTP_PERIOD)
        if (newRemaining !== code.remaining_secs) {
          lastCode = { ...code, remaining_secs: newRemaining }
          notify()
        }
      }
    }, 1000) as unknown as number
  }, msToNext) as unknown as number
}

const stopLoopIfUnused = () => {
  if (listeners.size === 0 && sharedInterval !== null) {
    window.clearInterval(sharedInterval)
    sharedInterval = null
    currentSecret = null
    lastCode = null
    lastError = null
    lastFetchedPeriod = -1
  }
}

const subscribe = (
  secret: string,
  fn: (code: TotpCode | null, error: string | null) => void
) => {
  if (currentSecret !== secret) {
    currentSecret = secret
    lastCode = null
    lastError = null
    lastFetchedPeriod = -1
  }
  listeners.add(fn)
  ensureLoop()
  if (lastCode || lastError) {
    fn(lastCode, lastError)
  } else {
    fetchOnce()
  }
  return () => {
    listeners.delete(fn)
    stopLoopIfUnused()
  }
}

export function useTotp(secret: string): { code: TotpCode | null; error: string | null } {
  const [state, setState] = useState<{ code: TotpCode | null; error: string | null }>({
    code: lastCode,
    error: lastError,
  })
  const secretRef = useRef(secret)
  secretRef.current = secret

  useEffect(() => {
    if (!secret) {
      setState({ code: null, error: null })
      return
    }
    return subscribe(secret, (code, error) => {
      setState({ code, error })
    })
  }, [secret])

  return state
}

interface TotpTimerProps {
  secret: string
  className?: string
}

export function TotpTimer({ secret, className }: TotpTimerProps) {
  const { code, error } = useTotp(secret)
  const [copied, setCopied] = useState(false)

  if (error) {
    return (
      <div className={cn('flex items-center gap-2 text-destructive text-sm', className)}>
        <AlertCircle className="h-4 w-4" />
        <span>Invalid TOTP secret</span>
      </div>
    )
  }

  if (!code) {
    return (
      <div className={cn('flex items-center space-x-2', className)}>
        <div className="h-8 w-20 animate-pulse rounded bg-muted" />
      </div>
    )
  }

  const progress = (code.remaining_secs / TOTP_PERIOD) * 100

  const handleCopy = async () => {
    try {
      await navigator.clipboard.writeText(code.code)
      setCopied(true)
      setTimeout(() => setCopied(false), 2000)
    } catch {
      // ignore
    }
  }

  return (
    <div className={cn('flex items-center space-x-3', className)}>
      <button
        type="button"
        onClick={handleCopy}
        className="flex items-center space-x-1 font-mono text-2xl font-bold tracking-wider hover:text-primary transition-colors"
        title="Click to copy"
      >
        {code.code.split('').map((digit, i) => (
          <span key={i}>{digit}</span>
        ))}
        {copied && <Check className="ml-2 h-4 w-4 text-green-500" />}
      </button>
      <div className="flex flex-col items-center">
        <div className="h-8 w-8">
          <svg viewBox="0 0 36 36" className="transform -rotate-90">
            <circle
              cx="18"
              cy="18"
              r="16"
              fill="none"
              stroke="currentColor"
              strokeWidth="3"
              className="text-muted"
            />
            <circle
              cx="18"
              cy="18"
              r="16"
              fill="none"
              stroke="currentColor"
              strokeWidth="3"
              strokeDasharray="100"
              strokeDashoffset={100 - progress}
              strokeLinecap="round"
              className={code.remaining_secs <= 5 ? 'text-destructive' : 'text-primary'}
            />
          </svg>
        </div>
        <span className="text-xs text-muted-foreground">{code.remaining_secs}s</span>
      </div>
    </div>
  )
}

export function TotpInput({
  secret,
  value,
  onChange,
  className,
}: {
  secret: string
  value: string
  onChange: (value: string) => void
  className?: string
}) {
  const { code, error } = useTotp(secret)
  const [isCopied, setIsCopied] = useState(false)

  useEffect(() => {
    if (!value && code) {
      onChange(code.code)
    }
  }, [code, value, onChange])

  if (error) {
    return (
      <div className={cn('text-sm text-destructive', className)}>
        Invalid TOTP secret
      </div>
    )
  }

  const handleCopy = async () => {
    if (code) {
      try {
        await navigator.clipboard.writeText(code.code)
        setIsCopied(true)
        setTimeout(() => setIsCopied(false), 2000)
      } catch {
        // ignore
      }
    }
  }

  return (
    <div className={cn('flex items-center space-x-2', className)}>
      <input
        type="text"
        value={value}
        onChange={(e) => onChange(e.target.value)}
        placeholder="000000"
        maxLength={6}
        className="w-24 font-mono text-center text-lg tracking-widest rounded-md border border-input bg-background px-3 py-2"
      />
      {code && (
        <button
          type="button"
          onClick={handleCopy}
          className="flex items-center gap-1 text-sm text-muted-foreground hover:text-foreground"
        >
          {isCopied ? (
            <>
              <Check className="h-3 w-3" />
              Copied
            </>
          ) : (
            <>
              <Copy className="h-3 w-3" />
              Copy
            </>
          )}
        </button>
      )}
    </div>
  )
}
