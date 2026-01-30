import { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { Config } from './types'
import GeneralTab from './tabs/GeneralTab'
import AudioTab from './tabs/AudioTab'
import PipelineTab from './tabs/PipelineTab'
import HomeAssistantTab from './tabs/HomeAssistantTab'
import DockerSetupTab from './tabs/DockerSetupTab'
import './preferences.css'

type TabId = 'general' | 'audio' | 'pipeline' | 'homeassistant' | 'docker'

interface TabDef {
  id: TabId
  label: string
}

const tabs: TabDef[] = [
  { id: 'general', label: 'General' },
  { id: 'audio', label: 'Audio' },
  { id: 'pipeline', label: 'Pipeline' },
  { id: 'homeassistant', label: 'Home Assistant' },
  { id: 'docker', label: 'Docker Setup' },
]

function Preferences() {
  const [activeTab, setActiveTab] = useState<TabId>('general')
  const [config, setConfig] = useState<Config | null>(null)
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

  function handleConfigChange(updates: Partial<Config>) {
    if (config) {
      setConfig({ ...config, ...updates })
    }
  }

  async function handleSave() {
    try {
      await invoke('update_config', { newConfig: config })
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
    if (!config) return null
    switch (activeTab) {
      case 'general':
        return <GeneralTab config={config} onChange={handleConfigChange} />
      case 'audio':
        return <AudioTab config={config} onChange={handleConfigChange} />
      case 'pipeline':
        return <PipelineTab config={config} onChange={handleConfigChange} />
      case 'homeassistant':
        return <HomeAssistantTab config={config} onChange={handleConfigChange} />
      case 'docker':
        return <DockerSetupTab config={config} onChange={handleConfigChange} />
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
