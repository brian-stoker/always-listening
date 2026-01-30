import { useState } from 'react'

function App() {
  const [status, setStatus] = useState<'idle' | 'listening' | 'recording'>('idle')

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
