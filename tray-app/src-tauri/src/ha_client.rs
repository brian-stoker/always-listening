use reqwest::Client;

/// Test the connection to a Home Assistant instance.
/// Makes a GET request to `{url}/api/` with a Bearer token.
/// Returns Ok(true) if the server responds with 200.
pub async fn test_connection(url: &str, token: &str) -> Result<bool, String> {
    let client = Client::new();
    let api_url = format!("{}/api/", url.trim_end_matches('/'));

    let response = client
        .get(&api_url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| format!("Failed to connect: {}", e))?;

    if response.status().is_success() {
        Ok(true)
    } else {
        Err(format!(
            "Home Assistant returned status {}",
            response.status()
        ))
    }
}

/// Tauri command to test the Home Assistant connection.
#[tauri::command]
pub async fn test_ha_connection(url: String, token: String) -> Result<bool, String> {
    test_connection(&url, &token).await
}
