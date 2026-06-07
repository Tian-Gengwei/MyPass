// Popup 组件 - 扩展弹窗

import { useState, useEffect } from 'react'
import { createRoot } from 'react-dom/client'

interface Entry {
  id: string
  name: string
  username: string
  url?: string
}

function Popup() {
  const [isUnlocked, setIsUnlocked] = useState(false)
  const [entries, setEntries] = useState<Entry[]>([])
  const [search, setSearch] = useState('')
  const [selectedEntry, setSelectedEntry] = useState<Entry | null>(null)
  const [passwordVisible, setPasswordVisible] = useState(false)

  useEffect(() => {
    checkVaultStatus()
  }, [])

  const checkVaultStatus = async () => {
    try {
      // 检查金库状态
      const status = await browser.runtime.sendMessage({ type: 'VAULT_STATUS' })
      setIsUnlocked(status.isUnlocked)
      if (status.isUnlocked) {
        loadEntries()
      }
    } catch (err) {
      console.error('Failed to check vault status:', err)
    }
  }

  const loadEntries = async () => {
    try {
      const response = await browser.runtime.sendMessage({ type: 'GET_ENTRIES' })
      if (response.success) {
        setEntries(response.data)
      }
    } catch (err) {
      console.error('Failed to load entries:', err)
    }
  }

  const handleCopy = async (text: string) => {
    await navigator.clipboard.writeText(text)
  }

  const handleFill = async (entry: Entry) => {
    await browser.runtime.sendMessage({
      type: 'FILL_CREDENTIAL',
      data: { entryId: entry.id }
    })
    window.close()
  }

  const filteredEntries = entries.filter(entry => {
    if (!search) return true
    const q = search.toLowerCase()
    return (
      entry.name.toLowerCase().includes(q) ||
      entry.username.toLowerCase().includes(q) ||
      entry.url?.toLowerCase().includes(q)
    )
  })

  if (!isUnlocked) {
    return (
      <div className="popup-container popup-locked">
        <div className="popup-header">
          <h1>MyPass</h1>
        </div>
        <div className="popup-body">
          <p>Vault is locked</p>
          <button onClick={() => browser.runtime.sendMessage({ type: 'OPEN_DESKTOP' })}>
            Open MyPass
          </button>
        </div>
      </div>
    )
  }

  return (
    <div className="popup-container popup-unlocked">
      <div className="popup-header">
        <input
          type="text"
          placeholder="Search..."
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          className="search-input"
        />
      </div>

      <div className="popup-body">
        <div className="entries-list">
          {filteredEntries.map(entry => (
            <div
              key={entry.id}
              className={`entry-item ${selectedEntry?.id === entry.id ? 'selected' : ''}`}
              onClick={() => setSelectedEntry(entry)}
              onDoubleClick={() => handleFill(entry)}
            >
              <div className="entry-name">{entry.name}</div>
              <div className="entry-username">{entry.username}</div>
            </div>
          ))}
        </div>

        {selectedEntry && (
          <div className="entry-detail">
            <div className="detail-field">
              <label>Username</label>
              <div className="detail-value">
                <span>{selectedEntry.username}</span>
                <button onClick={() => handleCopy(selectedEntry.username)}>Copy</button>
              </div>
            </div>
            <div className="detail-field">
              <label>Password</label>
              <div className="detail-value">
                <span>{passwordVisible ? 'visible' : '••••••••'}</span>
                <button onClick={() => setPasswordVisible(!passwordVisible)}>
                  {passwordVisible ? 'Hide' : 'Show'}
                </button>
                <button onClick={() => handleCopy(selectedEntry.id)}>Copy</button>
              </div>
            </div>
            {selectedEntry.url && (
              <div className="detail-field">
                <label>URL</label>
                <div className="detail-value">
                  <span>{selectedEntry.url}</span>
                </div>
              </div>
            )}
            <button className="fill-button" onClick={() => handleFill(selectedEntry)}>
              Fill
            </button>
          </div>
        )}
      </div>
    </div>
  )
}

// 样式
const styles = `
.popup-container {
  width: 360px;
  min-height: 400px;
  background: #1a1a1a;
  color: white;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
}
.popup-header {
  padding: 12px;
  border-bottom: 1px solid #333;
}
.search-input {
  width: 100%;
  padding: 8px 12px;
  border: 1px solid #333;
  border-radius: 4px;
  background: #2a2a2a;
  color: white;
  font-size: 14px;
}
.search-input:focus {
  outline: none;
  border-color: #0066cc;
}
.popup-body {
  padding: 8px;
}
.entries-list {
  max-height: 200px;
  overflow-y: auto;
}
.entry-item {
  padding: 10px 12px;
  border-radius: 4px;
  cursor: pointer;
  margin-bottom: 4px;
}
.entry-item:hover {
  background: #2a2a2a;
}
.entry-item.selected {
  background: #333;
}
.entry-name {
  font-weight: 500;
  margin-bottom: 2px;
}
.entry-username {
  font-size: 12px;
  color: #888;
}
.entry-detail {
  margin-top: 12px;
  padding-top: 12px;
  border-top: 1px solid #333;
}
.detail-field {
  margin-bottom: 12px;
}
.detail-field label {
  display: block;
  font-size: 11px;
  color: #888;
  margin-bottom: 4px;
}
.detail-field .detail-value {
  display: flex;
  align-items: center;
  gap: 8px;
}
.detail-field button {
  padding: 4px 8px;
  font-size: 11px;
  background: #333;
  border: none;
  border-radius: 3px;
  color: white;
  cursor: pointer;
}
.detail-field button:hover {
  background: #444;
}
.fill-button {
  width: 100%;
  padding: 10px;
  background: #0066cc;
  border: none;
  border-radius: 4px;
  color: white;
  cursor: pointer;
  font-weight: 500;
}
.fill-button:hover {
  background: #0055aa;
}
`

// 渲染
const container = document.getElementById('root')
if (container) {
  const root = createRoot(container)
  root.render(<Popup />)

  // 注入样式
  const styleSheet = document.createElement('style')
  styleSheet.textContent = styles
  document.head.appendChild(styleSheet)
}