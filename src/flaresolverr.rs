//! FlareSolverr integration - headless Cloudflare challenge solver
//!
//! FlareSolverr runs as an external service (Docker/standalone) and provides
//! an HTTP API to solve Cloudflare challenges using puppeteer-stealth.
//!
//! Environment variables:
//! - NEXUS_FLARESOLVERR_URL: FlareSolverr endpoint (default: http://localhost:8191)
//! - NEXUS_FLARESOLVERR_TIMEOUT: Max wait time in ms (default: 60000)
//! - NEXUS_DISABLE_FLARESOLVERR: Set to "1" to force direct browser fallback

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::env;
use std::path::PathBuf;
use std::time::Duration;

use crate::debug_log;

const DEFAULT_TIMEOUT_MS: u64 = 60000;
const DEFAULT_URL: &str = "http://localhost:8191";

#[derive(Debug, Serialize)]
struct Request {
    cmd: String,
    url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    cookies: Option<Vec<Cookie>>,
    #[serde(rename = "maxTimeout")]
    max_timeout: u64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Cookie {
    pub name: String,
    pub value: String,
    pub domain: String,
    pub path: String,
    #[serde(rename = "expires")]
    pub expires: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct Response {
    status: String,
    message: String,
    solution: Option<Solution>,
}

#[derive(Debug, Deserialize)]
struct Solution {
    #[allow(dead_code)]
    url: String,
    cookies: Vec<Cookie>,
    #[serde(rename = "response")]
    body: String,
}

/// Check if FlareSolverr is available and healthy
pub async fn is_available() -> bool {
    if env::var("NEXUS_DISABLE_FLARESOLVERR").unwrap_or_default() == "1" {
        debug_log!("FlareSolverr disabled via NEXUS_DISABLE_FLARESOLVERR");
        return false;
    }

    let client = match reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            debug_log!("Failed to build HTTP client for FlareSolverr check: {e}");
            return false;
        }
    };

    let url = format!("{}/health", flaresolverr_url());
    debug_log!("Checking FlareSolverr health at: {url}");
    match client.get(&url).send().await {
        Ok(resp) => {
            let success = resp.status().is_success();
            debug_log!("FlareSolverr health check result: {success}");
            success
        }
        Err(e) => {
            debug_log!("FlareSolverr health check failed: {e}");
            false
        }
    }
}

/// Fetch URL content via FlareSolverr, using session cookies if available
pub async fn fetch_text(url: &str) -> Result<(String, Vec<Cookie>)> {
    debug_log!("fetch_text (no cookies): {url}");
    fetch_text_with_cookies(url, None).await
}

/// Fetch URL with existing cookies (for session reuse)
pub async fn fetch_text_with_cookies(
    url: &str,
    cookies: Option<Vec<Cookie>>,
) -> Result<(String, Vec<Cookie>)> {
    debug_log!("fetch_text_with_cookies: {url}, cookies: {}", cookies.as_ref().map(|c| c.len()).unwrap_or(0));
    
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(120))
        .build()
        .map_err(|e| anyhow!("Failed to create HTTP client: {e}"))?;

    let req = Request {
        cmd: "request.get".to_string(),
        url: url.to_string(),
        cookies,
        max_timeout: flaresolverr_timeout(),
    };

    let endpoint = format!("{}/v1", flaresolverr_url());

    let resp = client
        .post(&endpoint)
        .json(&req)
        .send()
        .await
        .map_err(|e| anyhow!("FlareSolverr request failed: {e}"))?;

    let status = resp.status();
    let body = resp
        .text()
        .await
        .map_err(|e| anyhow!("Failed to read FlareSolverr response: {e}"))?;

    if !status.is_success() {
        return Err(anyhow!(
            "FlareSolverr returned HTTP {}: {}",
            status.as_u16(),
            body.chars().take(200).collect::<String>()
        ));
    }

    let parsed: Response = serde_json::from_str(&body)
        .map_err(|e| {
            debug_log!("Failed to parse FlareSolverr response: {e}");
            debug_log!("Response body (first 500 chars): {}", body.chars().take(500).collect::<String>());
            anyhow!("Failed to parse FlareSolverr response: {e}")
        })?;

    if parsed.status != "ok" {
        return Err(anyhow!(
            "FlareSolverr error: {} - {}",
            parsed.status,
            parsed.message
        ));
    }

    let solution = parsed
        .solution
        .ok_or_else(|| anyhow!("FlareSolverr response missing solution"))?;

    debug_log!("FlareSolverr success: {} bytes, {} cookies", solution.body.len(), solution.cookies.len());
    Ok((solution.body, solution.cookies))
}

/// Load saved session cookies for a domain
pub fn load_session_cookies(domain: &str) -> Option<Vec<Cookie>> {
    let path = session_cookie_path(domain);
    if !path.exists() {
        debug_log!("No saved cookies found for domain: {domain}");
        return None;
    }

    let content = std::fs::read_to_string(&path).ok()?;
    let cookies: Vec<Cookie> = serde_json::from_str(&content).ok()?;
    debug_log!("Loaded {} cookies for domain: {domain}", cookies.len());
    Some(cookies)
}

/// Save session cookies for a domain
pub fn save_session_cookies(domain: &str, cookies: &[Cookie]) -> Result<()> {
    let path = session_cookie_path(domain);
    debug_log!("Saving {} cookies for domain: {domain} to {:?}", cookies.len(), path);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(cookies)?;
    std::fs::write(&path, content)?;
    Ok(())
}

/// Extract domain from URL for cookie storage
pub fn extract_domain(url: &str) -> String {
    url.trim_start_matches("https://")
        .trim_start_matches("http://")
        .split('/')
        .next()
        .unwrap_or("unknown")
        .to_string()
}

fn session_cookie_path(domain: &str) -> PathBuf {
    directories::ProjectDirs::from("dev", "nexus", "nexus-tui")
        .map(|d| {
            d.data_local_dir()
                .join("sessions")
                .join(format!("{domain}.json"))
        })
        .unwrap_or_else(|| PathBuf::from(".nexus/sessions").join(format!("{domain}.json")))
}

fn flaresolverr_url() -> String {
    env::var("NEXUS_FLARESOLVERR_URL").unwrap_or_else(|_| DEFAULT_URL.to_string())
}

fn flaresolverr_timeout() -> u64 {
    env::var("NEXUS_FLARESOLVERR_TIMEOUT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(DEFAULT_TIMEOUT_MS)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_domain() {
        assert_eq!(
            extract_domain("https://api.allanime.day/graphql"),
            "api.allanime.day"
        );
        assert_eq!(extract_domain("http://example.com/path"), "example.com");
    }
}
