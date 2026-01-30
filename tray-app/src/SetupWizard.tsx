import { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { Config } from './types'
import './setup-wizard.css'

interface DependencyStatus {
  ffmpeg_ok: boolean
  ffmpeg_version: string
  whisper_ok: boolean
  whisper_version: string
  claude_ok: boolean
  claude_version: string
  accessibility_ok: boolean
  accessibility_info: string
  microphone_ok: boolean
  microphone_info: string
}

type WizardStep = 1 | 2 | 3 | 4
const TOTAL_STEPS = 4

const STEP_LABELS = ['Welcome', 'Dependencies', 'Configuration', 'Complete']

const WHISPER_MODELS = ['tiny', 'base', 'small', 'medium', 'large']

function StepIndicator({ currentStep }: { currentStep: WizardStep }) {
  return (
    <div className="step-indicator">
      {STEP_LABELS.map((label, index) => {
        const stepNum = index + 1
        const isActive = stepNum === currentStep
        const isCompleted = stepNum < currentStep
        return (
          <div
            key={label}
            className={`step-item ${isActive ? 'active' : ''} ${isCompleted ? 'completed' : ''}`}
          >
            <div className="step-circle">
              {isCompleted ? (
                <span className="step-check">&#10003;</span>
              ) : (
                <span>{stepNum}</span>
              )}
            </div>
            <span className="step-label">{label}</span>
          </div>
        )
      })}
      <div className="step-line" />
    </div>
  )
}

function StatusIcon({ ok }: { ok: boolean }) {
  return (
    <span className={`status-icon ${ok ? 'ok' : 'fail'}`}>
      {ok ? '\u2713' : '\u2717'}
    </span>
  )
}

function WelcomeStep() {
  return (
    <div className="wizard-content">
      <h2>Welcome to Always Listening</h2>
      <p className="wizard-description">
        Always Listening is a voice pipeline application that captures audio,
        transcribes it with Whisper, and processes it through Claude AI. It can
        also integrate with Home Assistant for smart home voice control.
      </p>
      <div className="feature-list">
        <div className="feature-item">
          <span className="feature-icon">&#127908;</span>
          <div>
            <strong>Voice-to-Claude</strong>
            <p>Speak naturally and get AI-powered responses</p>
          </div>
        </div>
        <div className="feature-item">
          <span className="feature-icon">&#9997;</span>
          <div>
            <strong>Dictation Mode</strong>
            <p>Transcribe your speech directly into any application</p>
          </div>
        </div>
        <div className="feature-item">
          <span className="feature-icon">&#127968;</span>
          <div>
            <strong>Home Assistant</strong>
            <p>Control your smart home with voice commands</p>
          </div>
        </div>
      </div>
      <p className="wizard-subtitle">
        This setup wizard will check your system dependencies and help you
        configure the basics. It only takes a minute.
      </p>
    </div>
  )
}

function DependencyStep({
  status,
  loading,
  onRecheck,
}: {
  status: DependencyStatus | null
  loading: boolean
  onRecheck: () => void
}) {
  if (loading) {
    return (
      <div className="wizard-content">
        <h2>Checking Dependencies</h2>
        <div className="loading-spinner">
          <div className="spinner" />
          <p>Verifying system requirements...</p>
        </div>
      </div>
    )
  }

  if (!status) {
    return (
      <div className="wizard-content">
        <h2>Dependency Check</h2>
        <p>Unable to check dependencies. Please try again.</p>
        <button className="btn btn-secondary" onClick={onRecheck}>
          Retry
        </button>
      </div>
    )
  }

  const allOk =
    status.ffmpeg_ok &&
    status.whisper_ok &&
    status.claude_ok &&
    status.accessibility_ok &&
    status.microphone_ok

  return (
    <div className="wizard-content">
      <h2>System Dependencies</h2>
      <p className="wizard-description">
        The following components are needed for the voice pipeline to work correctly.
      </p>

      <div className="dependency-list">
        <div className={`dependency-item ${status.ffmpeg_ok ? 'ok' : 'fail'}`}>
          <StatusIcon ok={status.ffmpeg_ok} />
          <div className="dependency-info">
            <strong>ffmpeg</strong>
            <span className="dependency-detail">{status.ffmpeg_version}</span>
          </div>
        </div>

        <div className={`dependency-item ${status.whisper_ok ? 'ok' : 'fail'}`}>
          <StatusIcon ok={status.whisper_ok} />
          <div className="dependency-info">
            <strong>Whisper</strong>
            <span className="dependency-detail">{status.whisper_version}</span>
          </div>
        </div>

        <div className={`dependency-item ${status.claude_ok ? 'ok' : 'fail'}`}>
          <StatusIcon ok={status.claude_ok} />
          <div className="dependency-info">
            <strong>Claude CLI</strong>
            <span className="dependency-detail">{status.claude_version}</span>
          </div>
        </div>

        <div className={`dependency-item ${status.accessibility_ok ? 'ok' : 'fail'}`}>
          <StatusIcon ok={status.accessibility_ok} />
          <div className="dependency-info">
            <strong>Accessibility</strong>
            <span className="dependency-detail">{status.accessibility_info}</span>
          </div>
        </div>

        <div className={`dependency-item ${status.microphone_ok ? 'ok' : 'fail'}`}>
          <StatusIcon ok={status.microphone_ok} />
          <div className="dependency-info">
            <strong>Microphone</strong>
            <span className="dependency-detail">{status.microphone_info}</span>
          </div>
        </div>
      </div>

      {allOk ? (
        <p className="status-summary success">All dependencies are satisfied.</p>
      ) : (
        <p className="status-summary warning">
          Some dependencies are missing. You can still proceed, but some features may not work.
        </p>
      )}

      <button className="btn btn-secondary recheck-btn" onClick={onRecheck}>
        Re-check
      </button>
    </div>
  )
}

function ConfigStep({
  config,
  onChange,
}: {
  config: Config
  onChange: (updates: Partial<Config>) => void
}) {
  return (
    <div className="wizard-content">
      <h2>Quick Configuration</h2>
      <p className="wizard-description">
        Set up the essentials. You can always change these later in Preferences.
      </p>

      <div className="config-form">
        <div className="form-group">
          <label htmlFor="audio-device">Audio Device</label>
          <input
            id="audio-device"
            type="text"
            value={config.audio_device}
            onChange={(e) => onChange({ audio_device: e.target.value })}
            placeholder=":1"
          />
          <span className="form-hint">
            The audio input device identifier (e.g., ":1" for default)
          </span>
        </div>

        <div className="form-group">
          <label htmlFor="whisper-model">Whisper Model</label>
          <select
            id="whisper-model"
            value={config.whisper_model}
            onChange={(e) => onChange({ whisper_model: e.target.value })}
          >
            {WHISPER_MODELS.map((model) => (
              <option key={model} value={model}>
                {model}
              </option>
            ))}
          </select>
          <span className="form-hint">
            Larger models are more accurate but slower. "base" is a good default.
          </span>
        </div>

        <div className="form-group">
          <label className="toggle-label">
            <input
              type="checkbox"
              checked={config.ha_enabled}
              onChange={(e) => onChange({ ha_enabled: e.target.checked })}
            />
            <span>Enable Home Assistant Integration</span>
          </label>
          <span className="form-hint">
            Connect to Home Assistant for smart home voice control.
            Configure details later in Preferences.
          </span>
        </div>
      </div>
    </div>
  )
}

function CompleteStep() {
  return (
    <div className="wizard-content complete-step">
      <div className="complete-icon">&#10003;</div>
      <h2>You're All Set!</h2>
      <p className="wizard-description">
        Always Listening is configured and ready to go. You can access modes
        from the tray menu and adjust settings anytime in Preferences.
      </p>
      <div className="quick-tips">
        <h3>Quick Tips</h3>
        <ul>
          <li>Click the tray icon to select a voice mode</li>
          <li>Use Preferences to fine-tune audio and pipeline settings</li>
          <li>Check the logs if something is not working as expected</li>
        </ul>
      </div>
    </div>
  )
}

interface SetupWizardProps {
  onComplete: () => void
}

function SetupWizard({ onComplete }: SetupWizardProps) {
  const [step, setStep] = useState<WizardStep>(1)
  const [config, setConfig] = useState<Config | null>(null)
  const [depStatus, setDepStatus] = useState<DependencyStatus | null>(null)
  const [depLoading, setDepLoading] = useState(false)
  const [saving, setSaving] = useState(false)

  useEffect(() => {
    loadConfig()
  }, [])

  async function loadConfig() {
    try {
      const cfg = await invoke<Config>('get_config')
      setConfig(cfg)
    } catch (err) {
      console.error('Failed to load config:', err)
    }
  }

  async function checkDeps() {
    setDepLoading(true)
    setDepStatus(null)
    try {
      const status = await invoke<DependencyStatus>('check_dependencies')
      setDepStatus(status)
    } catch (err) {
      console.error('Failed to check dependencies:', err)
    } finally {
      setDepLoading(false)
    }
  }

  function handleConfigChange(updates: Partial<Config>) {
    if (config) {
      setConfig({ ...config, ...updates })
    }
  }

  async function handleFinish() {
    if (!config) return
    setSaving(true)
    try {
      await invoke('update_config', { newConfig: config })
      onComplete()
    } catch (err) {
      console.error('Failed to save config:', err)
    } finally {
      setSaving(false)
    }
  }

  function handleNext() {
    if (step === 1) {
      setStep(2)
      // Automatically start dependency check when entering step 2
      checkDeps()
    } else if (step === 2) {
      setStep(3)
    } else if (step === 3) {
      setStep(4)
    } else if (step === 4) {
      handleFinish()
    }
  }

  function handleBack() {
    if (step > 1) {
      setStep((step - 1) as WizardStep)
    }
  }

  function renderStep() {
    switch (step) {
      case 1:
        return <WelcomeStep />
      case 2:
        return (
          <DependencyStep
            status={depStatus}
            loading={depLoading}
            onRecheck={checkDeps}
          />
        )
      case 3:
        return config ? (
          <ConfigStep config={config} onChange={handleConfigChange} />
        ) : (
          <div className="wizard-content">
            <p>Loading configuration...</p>
          </div>
        )
      case 4:
        return <CompleteStep />
    }
  }

  return (
    <div className="setup-wizard">
      <StepIndicator currentStep={step} />

      <div className="wizard-card">{renderStep()}</div>

      <div className="wizard-nav">
        {step > 1 && (
          <button className="btn btn-secondary" onClick={handleBack}>
            Back
          </button>
        )}
        <div className="nav-spacer" />
        <button
          className="btn btn-primary"
          onClick={handleNext}
          disabled={saving}
        >
          {step === TOTAL_STEPS
            ? saving
              ? 'Saving...'
              : 'Start'
            : 'Next'}
        </button>
      </div>
    </div>
  )
}

export default SetupWizard
