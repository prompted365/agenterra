//! HAR parsing utilities for deriving API endpoint information.
//!
//! This module provides a simple parser for HAR (HTTP Archive) files so
//! that recorded network traffic can be transformed into high level
//! endpoint metadata. The intent is to support automated generation of
//! MCP tools by observing existing API calls in the browser or other
//! clients.
//!
//! This is intentionally minimal and focused on extracting request method
//! and path information. The resulting data can then be mapped into an
//! OpenAPI context or other structures for further processing.

use serde::Deserialize;
use std::path::Path;
use tokio::fs;
use url::Url;

use crate::Error;

/// Top level structure for a HAR file.
#[derive(Debug, Deserialize)]
struct HarFile {
    log: HarLog,
}

#[derive(Debug, Deserialize)]
struct HarLog {
    entries: Vec<HarEntry>,
}

#[derive(Debug, Deserialize)]
struct HarEntry {
    request: HarRequest,
}

#[derive(Debug, Deserialize)]
struct HarRequest {
    method: String,
    url: String,
}

/// Parsed representation of a HAR file.
pub struct HarContext {
    entries: Vec<HarEntry>,
}

impl HarContext {
    /// Load a HAR file from disk.
    pub async fn from_file<P: AsRef<Path>>(path: P) -> crate::Result<Self> {
        let content = fs::read_to_string(&path).await?;
        let har: HarFile = serde_json::from_str(&content).map_err(|e| {
            Error::config(format!("Failed to parse HAR {}: {}", path.as_ref().display(), e))
        })?;
        Ok(Self {
            entries: har.log.entries,
        })
    }

    /// Return unique endpoint operations discovered in the HAR.
    pub fn unique_operations(&self) -> Vec<HarOperation> {
        use std::collections::HashSet;
        let mut ops = Vec::new();
        let mut seen = HashSet::new();

        for entry in &self.entries {
            if let Ok(url) = Url::parse(&entry.request.url) {
                let method = entry.request.method.to_uppercase();
                let path = url.path().to_string();
                if seen.insert((method.clone(), path.clone())) {
                    ops.push(HarOperation { method, path });
                }
            }
        }
        ops
    }
}

/// Simplified representation of an API call extracted from a HAR file.
#[derive(Debug, PartialEq, Eq)]
pub struct HarOperation {
    pub method: String,
    pub path: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_unique_operations() -> crate::Result<()> {
        let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let base = manifest.parent().unwrap().parent().unwrap();
        let har_path = base.join("tests/fixtures/har/sample.har");
        let ctx = HarContext::from_file(&har_path).await?;
        let ops = ctx.unique_operations();
        assert_eq!(ops.len(), 2);
        assert!(ops.contains(&HarOperation { method: "GET".into(), path: "/api/items".into() }));
        assert!(ops.contains(&HarOperation { method: "POST".into(), path: "/api/items".into() }));
        Ok(())
    }
}

