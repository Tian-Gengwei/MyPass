import { useEffect, useState } from 'react'
import { VaultGate } from '@/components/VaultGate'
import { MainLayout } from '@/components/MainLayout'
import { ToastContainer } from '@/components/ToastContainer'
import { useVaultStore } from '@/stores/vault'

function App() {
  const isUnlocked = useVaultStore((state) => state.isUnlocked)
  const unlock = useVaultStore((state) => state.unlock)
  const lock = useVaultStore((state) => state.lock)
  const [initialized, setInitialized] = useState(false)

  useEffect(() => {
    setInitialized(true)
  }, [])

  if (!initialized) {
    return (
      <div className="min-h-screen bg-background text-foreground flex items-center justify-center">
        <div className="text-muted-foreground">Loading...</div>
      </div>
    )
  }

  return (
    <div className="min-h-screen bg-background text-foreground">
      {isUnlocked ? (
        <MainLayout onLock={lock} />
      ) : (
        <VaultGate onUnlock={unlock} />
      )}
      <ToastContainer />
    </div>
  )
}

export default App
