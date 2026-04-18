use anyhow::{anyhow, Result};
use chromiumoxide::browser::{Browser, BrowserConfig};
use futures_util::StreamExt;
use std::env;
use std::path::PathBuf;
use tokio::time::{sleep, Duration};

use crate::debug_log;
use crate::flaresolverr;

pub async fn fetch_text_with_query(url: &str, query: &[(String, String)]) -> Result<String> {
    let full_url = build_url(url, query);
    fetch_text_from_url(&full_url).await
}

pub async fn fetch_text_from_url(url: &str) -> Result<String> {
    let domain = flaresolverr::extract_domain(url);

    // Tier 1: Try FlareSolverr with session cookies (headless, stealth)
    let flaresolverr_available = flaresolverr::is_available().await;
    
    if flaresolverr_available {
        // Try with existing session cookies first
        let cookies = flaresolverr::load_session_cookies(&domain);

        match flaresolverr::fetch_text_with_cookies(url, cookies).await {
            Ok((body, new_cookies)) => {
                let is_challenge = looks_like_bot_challenge(&body);
                if !is_challenge {
                    let _ = flaresolverr::save_session_cookies(&domain, &new_cookies);
                    // Extract JSON from HTML wrapper if present
                    let clean_body = extract_json_from_html(&body);
                    return Ok(clean_body);
                }
                // Challenge still present - cookies expired, try fresh
                match flaresolverr::fetch_text(url).await {
                    Ok((body, new_cookies)) => {
                        if !looks_like_bot_challenge(&body) {
                            let _ = flaresolverr::save_session_cookies(&domain, &new_cookies);
                            // Extract JSON from HTML wrapper if present
                            let clean_body = extract_json_from_html(&body);
                            return Ok(clean_body);
                        }
                    }
                    Err(_e) => {}
                }
            }
            Err(_e) => {}
        }
        
        // If we get here, FlareSolverr was tried but couldn't solve the challenge
        return Err(anyhow!(
            "FlareSolverr could not bypass Cloudflare challenge. Try:\n\
            1. Update FlareSolverr: docker pull ghcr.io/flaresolverr/flaresolverr:latest\n\
            2. Wait 5-10 minutes and retry (Cloudflare may have flagged your IP)\n\
            3. Use a VPN/proxy to change your IP address"
        ));
    }

    // Tier 2: Fallback to visible browser (chromiumoxide) - only if FlareSolverr not installed
    fetch_with_visible_browser(url).await
}

async fn fetch_with_visible_browser(url: &str) -> Result<String> {
    ensure_gui_session()?;

    let profile_dir = browser_profile_dir();
    std::fs::create_dir_all(&profile_dir)?;

    let mut builder = BrowserConfig::builder()
        .with_head()
        .disable_default_args()
        .window_size(1360, 900)
        .user_data_dir(profile_dir)
        .args([
            "--disable-gpu",
            "--disable-dev-shm-usage",
            "--disable-features=TranslateUI,BlinkGenPropertyTrees,IsolateOrigins,site-per-process",
            "--no-first-run",
            "--password-store=basic",
            "--lang=en-US",
            "--disable-blink-features=AutomationControlled",
            "--disable-web-security",
            "--disable-features=IsolateOrigins,site-per-process",
            "--user-agent=Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
        ]);

    if let Ok(bin) = env::var("NEXUS_CHROME_BIN") {
        if !bin.trim().is_empty() {
            builder = builder.chrome_executable(bin);
        }
    } else if let Some(bin) = autodetect_chromium_binary() {
        builder = builder.chrome_executable(bin);
    } else {
        return Err(anyhow!(
            "No Chromium-family browser found. Install Chrome/Chromium/Brave or set NEXUS_CHROME_BIN=/path/to/browser. Firefox is not supported by chromiumoxide."
        ));
    }

    let config = builder
        .build()
        .map_err(|e| anyhow!("Failed to build browser config: {e}"))?;
    let (mut browser, mut handler) = Browser::launch(config).await?;
    let handler_task = tokio::spawn(async move {
        while let Some(msg) = handler.next().await {
            if msg.is_err() {
                break;
            }
        }
    });

    let res = fetch_once_or_manual_retry(&browser, url).await;
    let _ = browser.close().await;
    let _ = handler_task.await;
    res
}

async fn fetch_once_or_manual_retry(browser: &Browser, url: &str) -> Result<String> {
    let page = browser.new_page("about:blank").await?;
    
    // Hide automation flags
    let _ = page.evaluate("navigator.webdriver = false; Object.defineProperty(navigator, 'webdriver', {get: () => false});").await;
    let _ = page.evaluate("window.chrome = { runtime: {} };").await;
    
    // Navigate directly to target URL (avoid double navigation)
    page.goto(url).await?;
    let _ = page.wait_for_navigation().await?;

    let mut body = current_page_text(&page).await?;
    if looks_like_bot_challenge(&body) {
        let wait_secs = std::env::var("NEXUS_BROWSER_AUTH_WAIT_SECS")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(180);
        let poll_ms = std::env::var("NEXUS_BROWSER_AUTH_POLL_MS")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(2000);

        let deadline = std::time::Instant::now() + Duration::from_secs(wait_secs);
        while std::time::Instant::now() < deadline {
            sleep(Duration::from_millis(poll_ms)).await;
            match current_page_text(&page).await {
                Ok(text) if !looks_like_bot_challenge(&text) => {
                    // Challenge cleared - use current page content without re-navigating
                    body = text;
                    break;
                }
                _ => {}
            }
        }
    }

    if looks_like_bot_challenge(&body) {
        let snippet = body.chars().take(180).collect::<String>();
        return Err(anyhow!(
            "Browser session still challenged after manual wait. Last response snippet: {snippet}"
        ));
    }

    Ok(body)
}

async fn current_page_text(page: &chromiumoxide::Page) -> Result<String> {
    Ok(page
        .evaluate("document.body ? document.body.innerText : ''")
        .await?
        .into_value()
        .unwrap_or_default())
}

fn browser_profile_dir() -> PathBuf {
    directories::ProjectDirs::from("dev", "nexus", "nexus-tui")
        .map(|d| d.data_local_dir().join("browser-profile"))
        .unwrap_or_else(|| PathBuf::from(".nexus/browser-profile"))
}

fn build_url(base: &str, query: &[(String, String)]) -> String {
    if query.is_empty() {
        return base.to_string();
    }

    let qs = query
        .iter()
        .map(|(k, v)| format!("{}={}", urlencoding::encode(k), urlencoding::encode(v)))
        .collect::<Vec<_>>()
        .join("&");
    format!("{base}?{qs}")
}

fn looks_like_bot_challenge(body: &str) -> bool {
    let low = body.to_ascii_lowercase();
    // Check for Cloudflare challenge indicators (not just <html which is in every page)
    low.contains("cf-chl")
        || low.contains("captcha")
        || low.contains("attention required")
        || low.contains("/cdn-cgi/challenge-platform")
        || low.contains("just a moment")
        || low.contains("verifying you are human")
        || low.contains("please wait")
}

/// Extract JSON from HTML wrapper that Cloudflare adds to API responses
fn extract_json_from_html(body: &str) -> String {
    // If body doesn't start with <html, it's already clean JSON
    if !body.trim_start().starts_with("<") {
        return body.to_string();
    }
    
    // Try to extract content from <pre>...</pre> tags
    if let Some(start) = body.find("<pre>") {
        let content_start = start + 5;
        if let Some(end) = body[content_start..].find("</pre>") {
            let json_content = &body[content_start..content_start + end];
            debug_log!("Extracted JSON from <pre> tags: {} bytes", json_content.len());
            return json_content.to_string();
        }
    }
    
    // If no <pre> tags found but starts with <html>, try to find JSON after body tag
    if body.contains("<body>") {
        if let Some(start) = body.find("<body>") {
            let after_body = &body[start + 6..];
            // Trim whitespace and return
            let trimmed = after_body.trim();
            if trimmed.ends_with("</body></html>") {
                let content = &trimmed[..trimmed.len() - "</body></html>".len()];
                debug_log!("Extracted JSON from <body>: {} bytes", content.len());
                return content.trim().to_string();
            }
        }
    }
    
    // Couldn't extract, return original
    body.to_string()
}

fn autodetect_chromium_binary() -> Option<PathBuf> {
    let absolute_candidates = [
        "/opt/brave.com/brave/brave-browser",
        "/usr/bin/brave-browser",
        "/usr/bin/brave",
        "/snap/bin/brave",
        "/usr/bin/google-chrome",
        "/usr/bin/google-chrome-stable",
        "/usr/bin/chromium",
        "/usr/bin/chromium-browser",
    ];
    for p in &absolute_candidates {
        let path = PathBuf::from(p);
        if path.is_file() {
            return Some(path);
        }
    }

    let path = env::var_os("PATH")?;
    let candidates = [
        "chromium",
        "chromium-browser",
        "google-chrome",
        "google-chrome-stable",
        "brave-browser",
        "brave",
    ];

    for dir in env::split_paths(&path) {
        for name in &candidates {
            let full = dir.join(name);
            if full.is_file() {
                return Some(full);
            }
        }
    }
    None
}

fn ensure_gui_session() -> Result<()> {
    #[cfg(target_os = "linux")]
    {
        let has_display = env::var_os("DISPLAY").is_some();
        let has_wayland = env::var_os("WAYLAND_DISPLAY").is_some();
        if !has_display && !has_wayland {
            return Err(anyhow!(
                "No GUI session detected (DISPLAY/WAYLAND missing). Browser auth cannot open a window in this terminal session."
            ));
        }
    }
    Ok(())
}
