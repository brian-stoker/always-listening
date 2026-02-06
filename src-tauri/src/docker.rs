use serde::Serialize;
use tokio::process::Command;

/// Result of a Docker status check
#[derive(Debug, Clone, Serialize)]
pub struct DockerStatus {
    pub installed: bool,
    pub version: String,
}

/// Result of an HA container status check
#[derive(Debug, Clone, Serialize)]
pub struct HaContainerStatus {
    pub exists: bool,
    pub running: bool,
    pub status: String,
}

/// Check if Docker is installed by running `docker --version`
pub async fn check_docker_installed() -> DockerStatus {
    match Command::new("docker")
        .arg("--version")
        .output()
        .await
    {
        Ok(output) if output.status.success() => {
            let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
            DockerStatus {
                installed: true,
                version,
            }
        }
        _ => DockerStatus {
            installed: false,
            version: String::new(),
        },
    }
}

/// Check if the Home Assistant container exists and its status
pub async fn check_ha_container() -> HaContainerStatus {
    // Check running containers first
    let running_result = Command::new("docker")
        .args(["ps", "--filter", "name=homeassistant", "--format", "{{.Status}}"])
        .output()
        .await;

    if let Ok(output) = &running_result {
        let status_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !status_str.is_empty() {
            return HaContainerStatus {
                exists: true,
                running: true,
                status: status_str,
            };
        }
    }

    // Check all containers (including stopped) if not running
    let all_result = Command::new("docker")
        .args(["ps", "-a", "--filter", "name=homeassistant", "--format", "{{.Status}}"])
        .output()
        .await;

    if let Ok(output) = &all_result {
        let status_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !status_str.is_empty() {
            return HaContainerStatus {
                exists: true,
                running: false,
                status: status_str,
            };
        }
    }

    HaContainerStatus {
        exists: false,
        running: false,
        status: "Not found".to_string(),
    }
}

/// Start an existing Home Assistant container
pub async fn start_ha_container() -> Result<String, String> {
    let output = Command::new("docker")
        .args(["start", "homeassistant"])
        .output()
        .await
        .map_err(|e| format!("Failed to execute docker start: {}", e))?;

    if output.status.success() {
        Ok("Home Assistant container started successfully".to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        Err(format!("Failed to start container: {}", stderr))
    }
}

/// Create and start a new Home Assistant container with the given config path
pub async fn setup_ha_container(config_path: &str) -> Result<String, String> {
    // Expand ~ to actual home directory
    let expanded_path = if config_path.starts_with("~/") {
        let home = dirs::home_dir().ok_or("Could not determine home directory")?;
        home.join(&config_path[2..]).to_string_lossy().to_string()
    } else {
        config_path.to_string()
    };

    // Create the config directory if it doesn't exist
    tokio::fs::create_dir_all(&expanded_path)
        .await
        .map_err(|e| format!("Failed to create config directory: {}", e))?;

    let output = Command::new("docker")
        .args([
            "create",
            "--name", "homeassistant",
            "--restart", "unless-stopped",
            "--privileged",
            "--network", "host",
            "-v", &format!("{}:/config", expanded_path),
            "-v", "/etc/localtime:/etc/localtime:ro",
            "-e", "TZ=America/New_York",
            "ghcr.io/home-assistant/home-assistant:stable",
        ])
        .output()
        .await
        .map_err(|e| format!("Failed to execute docker create: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(format!("Failed to create container: {}", stderr));
    }

    // Now start the container
    let start_output = Command::new("docker")
        .args(["start", "homeassistant"])
        .output()
        .await
        .map_err(|e| format!("Container created but failed to start: {}", e))?;

    if start_output.status.success() {
        Ok("Home Assistant container created and started successfully".to_string())
    } else {
        let stderr = String::from_utf8_lossy(&start_output.stderr).to_string();
        Err(format!("Container created but failed to start: {}", stderr))
    }
}

// --- Tauri Commands ---

#[tauri::command]
pub async fn check_docker() -> Result<DockerStatus, String> {
    Ok(check_docker_installed().await)
}

#[tauri::command]
pub async fn check_ha_container_status() -> Result<HaContainerStatus, String> {
    Ok(check_ha_container().await)
}

#[tauri::command]
pub async fn start_ha() -> Result<String, String> {
    start_ha_container().await
}

#[tauri::command]
pub async fn setup_ha(config_path: String) -> Result<String, String> {
    setup_ha_container(&config_path).await
}
