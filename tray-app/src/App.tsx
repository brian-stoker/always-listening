import { useState, useEffect } from 'react'
import { listen } from '@tauri-apps/api/event'
import Preferences from './Preferences'

type View = 'status' | 'preferences'

function App() {
  const [status, setStatus] = useState<'idle' | 'listening' | 'recording'>('idle')
  const [view, setView] = useState<View>('status')

  useEffect(() => {
    // Check if this window was opened as the preferences window
    const label = window.__TAURI_INTERNALS__?.metadata?.currentWindow?.label
    if (label === 'preferences') {
      setView('preferences')
      return
    }

    // Listen for the open-preferences event from the tray menu
    const unlisten = listen('open-preferences', () => {
      setView('preferences')
    })

    return () => {
      unlisten.then((fn) => fn())
    }
  }, [])

  if (view === 'preferences') {
    return <Preferences />
  }

  return (
    <div className="container">
      <h1>Always Listening</h1>
      <p>Voice Pipeline Tray Application</p>

      <div className="status-indicator">
        <span className={`status-dot ${status}`}></span>
        <span>Status: {status}</span>
      </div>

      <div className="button-group">
        <button onClick={() => setStatus('idle')}>Idle</button>
        <button onClick={() => setStatus('listening')}>Listening</button>
        <button onClick={() => setStatus('recording')}>Recording</button>
      </div>
    </div>
  )
}

export default App
