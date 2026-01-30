import { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { getCurrentWindow } from '@tauri-apps/api/window'
import './preferences.css'

type TabId = 'general' | 'audio' | 'pipeline'

interface TabDef {
  id: TabId
  label: string
}

const tabs: TabDef[] = [
  { id: 'general', label: 'General' },
  { id: 'audio', label: 'Audio' },
  { id: 'pipeline', label: 'Pipeline' },
]

interface Config {
  [key: string]: unknown
}

function Preferences() {
  const [activeTab, setActiveTab] = useState<TabId>('general')
  const [config, setConfig] = useState<Config>({})
  const [isDirty, setIsDirty] = useState(false)
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    loadConfig()
  }, [])

  async function loadConfig() {
    try {
      const cfg = await invoke<Config>('get_config')
      setConfig(cfg)
    } catch (err) {
      console.error('Failed to load config:', err)
    } finally {
      setLoading(false)
    }
  }

  async function handleSave() {
    try {
      await invoke('update_config', { config })
      setIsDirty(false)
      const win = getCurrentWindow()
      await win.close()
    } catch (err) {
      console.error('Failed to save config:', err)
    }
  }

  async function handleCancel() {
    const win = getCurrentWindow()
    await win.close()
  }

  function renderTabContent() {
    switch (activeTab) {
      case 'general':
        return (
          <div className="tab-content">
            <h3>General Settings</h3>
            <p className="placeholder-text">
              General preferences will be configured here.
            </p>
          </div>
        )
      case 'audio':
        return (
          <div className="tab-content">
            <h3>Audio Settings</h3>
            <p className="placeholder-text">
              Audio input and processing settings will be configured here.
            </p>
          </div>
        )
      case 'pipeline':
        return (
          <div className="tab-content">
            <h3>Pipeline Settings</h3>
            <p className="placeholder-text">
              Voice pipeline configuration will be managed here.
            </p>
          </div>
        )
    }
  }

  if (loading) {
    return (
      <div className="preferences-window">
        <p className="loading-text">Loading preferences...</p>
      </div>
    )
  }

  return (
    <div className="preferences-window">
      <div className="tab-bar">
        {tabs.map((tab) => (
          <button
            key={tab.id}
            className={`tab-button ${activeTab === tab.id ? 'active' : ''}`}
            onClick={() => setActiveTab(tab.id)}
          >
            {tab.label}
          </button>
        ))}
      </div>

      <div className="tab-panel">
        {renderTabContent()}
      </div>

      <div className="button-bar">
        <button className="btn btn-secondary" onClick={handleCancel}>
          Cancel
        </button>
        <button className="btn btn-primary" onClick={handleSave}>
          Save
        </button>
      </div>
    </div>
  )
}

export default Preferences
