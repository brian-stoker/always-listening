use std::collections::HashMap;
use std::process::Stdio;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::process::{Command, Child};
use tokio::io::{AsyncBufReadExt, BufReader};
use tracing;

/// Tracks a spawned process
struct TrackedProcess {
    name: String,
    child: Child,
}

/// Manages all spawned subprocess lifecycles
pub struct ProcessManager {
    processes: Arc<Mutex<HashMap<String, TrackedProcess>>>,
}

impl ProcessManager {
    pub fn new() -> Self {
        Self {
            processes: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Spawn a new subprocess with the given name, command, and arguments.
    /// Stdout/stderr are captured and logged with the process name prefix.
    pub async fn spawn(&self, name: &str, command: &str, args: &[&str]) -> Result<u32, String> {
        let mut cmd = Command::new(command);
        cmd.args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        // On Unix, create a new process group
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            unsafe {
                cmd.pre_exec(|| {
                    libc::setsid();
                    Ok(())
                });
            }
        }

        let mut child = cmd.spawn().map_err(|e| format!("Failed to spawn {}: {}", name, e))?;

        let pid = child.id().unwrap_or(0);
        tracing::info!("[{}] Process spawned (PID: {})", name, pid);

        // Capture stdout
        if let Some(stdout) = child.stdout.take() {
            let name_clone = name.to_string();
            tokio::spawn(async move {
                let reader = BufReader::new(stdout);
                let mut lines = reader.lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    tracing::debug!("[{}] {}", name_clone, line);
                }
            });
        }

        // Capture stderr
        if let Some(stderr) = child.stderr.take() {
            let name_clone = name.to_string();
            tokio::spawn(async move {
                let reader = BufReader::new(stderr);
                let mut lines = reader.lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    tracing::debug!("[{}:err] {}", name_clone, line);
                }
            });
        }

        let tracked = TrackedProcess {
            name: name.to_string(),
            child,
        };

        self.processes.lock().await.insert(name.to_string(), tracked);
        Ok(pid)
    }

    /// Stop a specific process by name. Sends SIGTERM, waits 3s, then SIGKILL.
    pub async fn stop(&self, name: &str) -> Result<(), String> {
        let mut processes = self.processes.lock().await;
        if let Some(mut tracked) = processes.remove(name) {
            tracing::info!("[{}] Stopping process...", name);

            #[cfg(unix)]
            {
                // Send SIGTERM to process group
                if let Some(pid) = tracked.child.id() {
                    unsafe {
                        libc::kill(-(pid as i32), libc::SIGTERM);
                    }
                }
            }

            #[cfg(windows)]
            {
                let _ = tracked.child.kill().await;
            }

            // Wait up to 3 seconds for graceful shutdown
            match tokio::time::timeout(
                std::time::Duration::from_secs(3),
                tracked.child.wait()
            ).await {
                Ok(Ok(status)) => {
                    tracing::info!("[{}] Process exited with: {}", name, status);
                }
                Ok(Err(e)) => {
                    tracing::warn!("[{}] Error waiting for process: {}", name, e);
                }
                Err(_) => {
                    // Timeout - force kill
                    tracing::warn!("[{}] Process didn't exit gracefully, force killing...", name);
                    #[cfg(unix)]
                    {
                        if let Some(pid) = tracked.child.id() {
                            unsafe {
                                libc::kill(-(pid as i32), libc::SIGKILL);
                            }
                        }
                    }
                    let _ = tracked.child.kill().await;
                    let _ = tracked.child.wait().await;
                    tracing::info!("[{}] Process force killed", name);
                }
            }
            Ok(())
        } else {
            Err(format!("Process '{}' not found", name))
        }
    }

    /// Stop all tracked processes
    pub async fn stop_all(&self) {
        let names: Vec<String> = {
            self.processes.lock().await.keys().cloned().collect()
        };

        for name in names {
            if let Err(e) = self.stop(&name).await {
                tracing::error!("Failed to stop {}: {}", name, e);
            }
        }
        tracing::info!("All processes stopped");
    }

    /// Check if a process is still running
    pub async fn is_running(&self, name: &str) -> bool {
        let mut processes = self.processes.lock().await;
        if let Some(tracked) = processes.get_mut(name) {
            // Try to check if process has exited
            match tracked.child.try_wait() {
                Ok(Some(_)) => {
                    // Process has exited, remove it
                    processes.remove(name);
                    false
                }
                Ok(None) => true,  // Still running
                Err(_) => {
                    processes.remove(name);
                    false
                }
            }
        } else {
            false
        }
    }

    /// Get list of running process names
    pub async fn list_running(&self) -> Vec<String> {
        self.processes.lock().await.keys().cloned().collect()
    }
}
