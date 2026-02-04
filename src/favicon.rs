use reqwest::blocking::Client;
use scraper::{Html, Selector};

pub struct FaviconResult {
    pub bytes: Vec<u8>,
    pub extension: String,
}

pub fn fetch_favicon(target_url: &str) -> Result<FaviconResult, Box<dyn std::error::Error>> {
    match attempt_fetch(target_url) {
        Ok(res) => Ok(res),
        Err(e) => {
            println!("[Favicon] Primary fetch failed for {}: {}", target_url, e);
            
            if let Ok(url_obj) = url::Url::parse(target_url) {
                if let Some(host) = url_obj.domain() {
                     let parts: Vec<&str> = host.split('.').collect();
                     // Try root domain first
                     if parts.len() >= 3 {
                         let root_domain = format!("https://{}.{}", parts[parts.len()-2], parts[parts.len()-1]);
                         println!("[Favicon] Trying fallback to root domain: {}", root_domain);
                         if let Ok(res) = attempt_fetch(&root_domain) {
                             return Ok(res);
                         }
                     }
                     
                     // Final fallback: Google Favicon Service
                     let google_url = format!("https://www.google.com/s2/favicons?domain={}&sz=128", host);
                     println!("[Favicon] Trying Google Favicon Service: {}", google_url);
                     let client = Client::builder()
                        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/121.0.0.0 Safari/537.36")
                        .build()?;
                     
                     if let Ok(icon_url) = reqwest::Url::parse(&google_url) {
                         if let Ok(res) = download_icon(&client, icon_url) {
                             return Ok(res);
                         }
                     }
                }
            }
            Err(e)
        }
    }
}

// Helper to attempt fetch
fn attempt_fetch(url: &str) -> Result<FaviconResult, Box<dyn std::error::Error>> {
    let client = Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/121.0.0.0 Safari/537.36")
        .build()?;

    let response = client.get(url)
        .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7")
        .header("Accept-Language", "en-US,en;q=0.9")
        .header("Sec-Ch-Ua", "\"Not A(Brand\";v=\"99\", \"Google Chrome\";v=\"121\", \"Chromium\";v=\"121\"")
        .header("Sec-Ch-Ua-Mobile", "?0")
        .header("Sec-Ch-Ua-Platform", "\"Windows\"")
        .header("Upgrade-Insecure-Requests", "1")
        .send()?;

    if !response.status().is_success() {
            println!("[Favicon] HTML fetch failed for {}: {}. Trying direct /favicon.ico", url, response.status());
            // Fallback to trying /favicon.ico directly
            let base_url = response.url().clone();
            if let Ok(icon_url) = base_url.join("/favicon.ico") {
                return download_icon(&client, icon_url);
            }
            return Err(format!("Status: {}", response.status()).into());
    }

    let base_url = response.url().clone();
    let html_content = response.text()?;
    
    parse_favicon(&html_content, &base_url, &client)
}

fn parse_favicon(html_content: &str, base_url: &reqwest::Url, client: &Client) -> Result<FaviconResult, Box<dyn std::error::Error>> {
    let document = Html::parse_document(html_content);
    // Select all links that might be icons
    let link_selector = Selector::parse("link").unwrap();
    
    struct IconCandidate {
        href: String,
        score: i32,
    }
    
    let mut candidates = Vec::new();
    
    for element in document.select(&link_selector) {
        if let (Some(rel), Some(href)) = (element.value().attr("rel"), element.value().attr("href")) {
            let rel_lower = rel.to_lowercase();
            if rel_lower.contains("icon") {
                let mut score = 0;
                if rel_lower.contains("apple-touch-icon") { score += 3; }
                else if rel_lower == "icon" { score += 2; }
                else if rel_lower.contains("shortcut") { score += 1; }
                
                candidates.push(IconCandidate { href: href.to_string(), score });
            }
        }
    }
    
    // Sort by score descending
    candidates.sort_by(|a, b| b.score.cmp(&a.score));
    
    let best_href = candidates.first().map(|c| c.href.clone()).unwrap_or_default();
    
    // Fallback to /favicon.ico if not found in HTML
    let icon_url = if best_href.is_empty() {
        base_url.join("/favicon.ico")?
    } else {
        base_url.join(&best_href)?
    };
    
    match download_icon(client, icon_url.clone()) {
        Ok(res) => Ok(res),
        Err(_) => {
            // Try root favicon.ico as last resort
            if !best_href.is_empty() {
                 let root_favicon = base_url.join("/favicon.ico")?;
                 if root_favicon != icon_url {
                     println!("[Favicon] Failed, trying root: {}", root_favicon);
                     return download_icon(client, root_favicon);
                 }
            }
            Err("Failed to download favicon".into())
        }
    }
}

fn download_icon(client: &Client, icon_url: reqwest::Url) -> Result<FaviconResult, Box<dyn std::error::Error>> {
    println!("[Favicon] Downloading icon from: {}", icon_url);
    let icon_response = client.get(icon_url.clone()).send()?;
    
    if !icon_response.status().is_success() {
        return Err(format!("Icon fetch failed: {}", icon_response.status()).into());
    }

    // Extract content-type before consuming body
    let content_type = icon_response.headers().get("content-type")
        .and_then(|ct| ct.to_str().ok())
        .map(|s| s.to_string());

    let bytes = icon_response.bytes()?.to_vec();
    
    // Guess extension
    let extension = if let Some(ct) = content_type {
        if ct.contains("png") { "png" }
        else if ct.contains("svg") { "svg" }
        else if ct.contains("jpeg") || ct.contains("jpg") { "jpg" }
        else if ct.contains("webp") { "webp" }
        else if ct.contains("x-icon") || ct.contains("vnd.microsoft.icon") { "ico" }
        else { 
                icon_url.path().split('.').last().unwrap_or("png") 
        }
    } else {
        icon_url.path().split('.').last().unwrap_or("ico")
    };

    Ok(FaviconResult {
        bytes,
        extension: extension.to_string(),
    })
}
