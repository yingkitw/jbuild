//! Maven Central API utilities

use anyhow::{Result, Context};

/// Fetch the latest version of a dependency from Maven Central
pub fn fetch_latest_version(group_id: &str, artifact_id: &str) -> Result<String> {
    // Use Maven Central Search API to find latest version
    // Query format: g:groupId AND a:artifactId
    let query = format!("g:{} AND a:{}", 
        urlencoding::encode(group_id),
        urlencoding::encode(artifact_id)
    );
    
    let url = format!(
        "https://search.maven.org/solrsearch/select?q={}&rows=1&wt=json&sort=version desc",
        urlencoding::encode(&query)
    );
    
    // Make HTTP request using ureq (blocking)
    let response = ureq::get(&url)
        .call()
        .context("Failed to query Maven Central")?;
    
    let body = response.into_string()
        .context("Failed to read response from Maven Central")?;
    
    // Parse JSON response
    let json: serde_json::Value = serde_json::from_str(&body)
        .context("Failed to parse Maven Central response")?;
    
    let docs = json["response"]["docs"]
        .as_array()
        .context("Invalid response format from Maven Central")?;
    
    if docs.is_empty() {
        return Err(anyhow::anyhow!(
            "No versions found for {}:{} on Maven Central",
            group_id,
            artifact_id
        ));
    }
    
    // Get the first (latest) version
    let doc = &docs[0];
    let version = doc["latestVersion"]
        .as_str()
        .or_else(|| doc["v"].as_str())
        .context("No version found in Maven Central response")?;
    
    Ok(version.to_string())
}

/// Search Maven Central and return results
pub fn search_maven_central(query: &str, limit: usize) -> Result<Vec<MavenCentralResult>> {
    let url = format!(
        "https://search.maven.org/solrsearch/select?q={}&rows={}&wt=json",
        urlencoding::encode(query),
        limit
    );
    
    let response = ureq::get(&url)
        .call()
        .context("Failed to search Maven Central")?;
    
    let body = response.into_string()
        .context("Failed to read response from Maven Central")?;
    
    let json: serde_json::Value = serde_json::from_str(&body)
        .context("Failed to parse Maven Central response")?;
    
    let docs = json["response"]["docs"]
        .as_array()
        .context("Invalid response format from Maven Central")?;
    
    let mut results = Vec::new();
    for doc in docs {
        let group = doc["g"].as_str().unwrap_or("?");
        let artifact = doc["a"].as_str().unwrap_or("?");
        let version = doc["latestVersion"].as_str().unwrap_or("?");
        let timestamp = doc["timestamp"].as_i64().unwrap_or(0);
        
        let date = if timestamp > 0 {
            let secs = timestamp / 1000;
            chrono::DateTime::from_timestamp(secs, 0)
                .map(|dt| dt.format("%Y-%m-%d").to_string())
                .unwrap_or_else(|| "?".to_string())
        } else {
            "?".to_string()
        };
        
        results.push(MavenCentralResult {
            group_id: group.to_string(),
            artifact_id: artifact.to_string(),
            version: version.to_string(),
            updated: date,
        });
    }
    
    Ok(results)
}

/// Result from Maven Central search
#[derive(Debug, Clone)]
pub struct MavenCentralResult {
    pub group_id: String,
    pub artifact_id: String,
    pub version: String,
    pub updated: String,
}

impl MavenCentralResult {
    pub fn to_coordinates(&self) -> String {
        format!("{}:{}:{}", self.group_id, self.artifact_id, self.version)
    }
}

