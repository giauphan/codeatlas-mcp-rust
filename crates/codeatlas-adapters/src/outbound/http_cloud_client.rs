use async_trait::async_trait;
use codeatlas_domain::entities::brain::{DreamMemory, ImmuneGene};
use codeatlas_domain::ports::cloud_sync::{CloudSyncError, CloudSyncPort};
use reqwest::{Client, Url};
use std::time::Duration;

pub struct HttpCloudClient {
    client: Client,
    base_url: Option<String>,
    api_key: Option<String>,
}

impl HttpCloudClient {
    pub fn new(base_url: Option<String>, api_key: Option<String>) -> Self {
        let allow_http = std::env::var("CODEATLAS_ALLOW_HTTP")
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false);
        Self::with_allow_http(base_url, api_key, allow_http)
    }

    pub fn with_allow_http(
        base_url: Option<String>,
        api_key: Option<String>,
        allow_http: bool,
    ) -> Self {
        let base_url = base_url.and_then(|u| {
            let trimmed = u.trim_end_matches('/').to_string();
            let parsed = Url::parse(&trimmed).ok()?;
            let host = parsed.host_str()?;

            let is_allowed_domain = host == "opencode.ai"
                || host == "codeatlas.ai"
                || host.ends_with(".codeatlas.ai");

            let is_loopback = matches!(
                host,
                "localhost" | "127.0.0.1" | "[::1]" | "::1"
            );

            let allow_custom = std::env::var("CODEATLAS_ALLOW_CUSTOM_URL")
                .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
                .unwrap_or(false);

            let is_allowed = match parsed.scheme() {
                "https" => is_allowed_domain || is_loopback || allow_custom,
                "http" if allow_http => is_loopback || allow_custom,
                _ => false,
            };

            if is_allowed {
                Some(trimmed)
            } else {
                tracing::warn!(
                    "Insecure URL rejected for CodeAtlas Cloud: {}. Use an allowed official domain, or enable CODEATLAS_ALLOW_CUSTOM_URL=true.",
                    trimmed
                );
                None
            }
        });

        let mut builder = Client::builder().timeout(Duration::from_secs(5));
        if !allow_http {
            builder = builder.https_only(true);
        }
        let client = builder.build().unwrap_or_else(|_| Client::new());

        Self {
            client,
            base_url,
            api_key,
        }
    }

    pub fn from_env() -> Self {
        let base_url = std::env::var("CODEATLAS_API_URL").ok();
        let api_key = std::env::var("CODEATLAS_API_KEY").ok();
        Self::new(base_url, api_key)
    }

    fn auth_config(&self) -> Result<(&str, &str), CloudSyncError> {
        match (&self.base_url, &self.api_key) {
            (Some(u), Some(k)) => Ok((u.as_str(), k.as_str())),
            _ => Err(CloudSyncError::Offline),
        }
    }

    async fn post_json<T: serde::Serialize + ?Sized>(
        &self,
        endpoint_path: &str,
        payload: &T,
    ) -> Result<(), CloudSyncError> {
        let (url, key) = self.auth_config()?;
        let endpoint = format!("{url}{endpoint_path}");
        let resp = self
            .client
            .post(&endpoint)
            .header("Authorization", format!("Bearer {key}"))
            .json(payload)
            .send()
            .await
            .map_err(|e| CloudSyncError::Network(e.to_string()))?;

        if !resp.status().is_success() {
            return Err(CloudSyncError::Api {
                status: resp.status().as_u16(),
                message: resp.text().await.unwrap_or_default(),
            });
        }
        Ok(())
    }

    async fn get_json<T: serde::de::DeserializeOwned, Q: serde::Serialize + ?Sized>(
        &self,
        endpoint_path: &str,
        query: &Q,
    ) -> Result<T, CloudSyncError> {
        let (url, key) = self.auth_config()?;
        let endpoint = format!("{url}{endpoint_path}");
        let resp = self
            .client
            .get(&endpoint)
            .header("Authorization", format!("Bearer {key}"))
            .query(query)
            .send()
            .await
            .map_err(|e| CloudSyncError::Network(e.to_string()))?;

        if resp.status().is_success() {
            let body = resp
                .json()
                .await
                .map_err(|e| CloudSyncError::Network(e.to_string()))?;
            Ok(body)
        } else {
            Err(CloudSyncError::Api {
                status: resp.status().as_u16(),
                message: resp.text().await.unwrap_or_default(),
            })
        }
    }
}

impl Default for HttpCloudClient {
    fn default() -> Self {
        Self::from_env()
    }
}

#[async_trait]
impl CloudSyncPort for HttpCloudClient {
    async fn check_auth(&self) -> Result<bool, CloudSyncError> {
        let (url, key) = self.auth_config()?;

        let endpoint = format!("{url}/api/auth/verify");
        let resp = self
            .client
            .get(&endpoint)
            .header("Authorization", format!("Bearer {key}"))
            .send()
            .await
            .map_err(|e| CloudSyncError::Network(e.to_string()))?;

        let status = resp.status();
        if status.is_success() {
            Ok(true)
        } else if status == reqwest::StatusCode::UNAUTHORIZED
            || status == reqwest::StatusCode::FORBIDDEN
        {
            Ok(false)
        } else {
            Err(CloudSyncError::Api {
                status: status.as_u16(),
                message: resp.text().await.unwrap_or_default(),
            })
        }
    }

    async fn save_dream(&self, dream: &DreamMemory) -> Result<(), CloudSyncError> {
        self.post_json("/api/dreams", dream).await
    }

    async fn query_dreams(
        &self,
        query: &str,
        project: Option<&str>,
        limit: usize,
    ) -> Result<Vec<DreamMemory>, CloudSyncError> {
        let limit_str = limit.to_string();
        let mut query_params = vec![("q", query), ("limit", &limit_str)];
        if let Some(p) = project {
            query_params.push(("project", p));
        }
        self.get_json("/api/dreams/query", &query_params).await
    }

    async fn search_genome(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<ImmuneGene>, CloudSyncError> {
        let limit_str = limit.to_string();
        self.get_json("/api/genome/search", &[("q", query), ("limit", &limit_str)])
            .await
    }

    async fn scan_immune_genes(&self, problem: &str) -> Result<Vec<ImmuneGene>, CloudSyncError> {
        self.get_json("/api/immune/scan", &[("problem", problem)])
            .await
    }

    async fn save_immune_gene(&self, gene: &ImmuneGene) -> Result<(), CloudSyncError> {
        self.post_json("/api/immune/genes", gene).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_offline_when_unconfigured() {
        let client = HttpCloudClient::new(None, None);
        assert!(matches!(
            client.check_auth().await,
            Err(CloudSyncError::Offline)
        ));

        let dream = DreamMemory {
            id: "d1".into(),
            category: "mistake".into(),
            content: "test error".into(),
            project: "proj".into(),
            timestamp: "2026-11-04T00:00:00Z".into(),
            score: None,
        };
        assert!(matches!(
            client.save_dream(&dream).await,
            Err(CloudSyncError::Offline)
        ));
        assert!(matches!(
            client.query_dreams("test", None, 10).await,
            Err(CloudSyncError::Offline)
        ));
        assert!(matches!(
            client.search_genome("test", 10).await,
            Err(CloudSyncError::Offline)
        ));
        assert!(matches!(
            client.scan_immune_genes("test").await,
            Err(CloudSyncError::Offline)
        ));
    }

    #[test]
    fn test_url_sanitization() {
        // Insecure unallowed domain rejected
        let client = HttpCloudClient::with_allow_http(
            Some("http://evil.com".into()),
            Some("k".into()),
            false,
        );
        assert!(client.base_url.is_none());

        // Allowed cloud domain accepted
        let client = HttpCloudClient::with_allow_http(
            Some("https://codeatlas.ai".into()),
            Some("k".into()),
            false,
        );
        assert_eq!(client.base_url.as_deref(), Some("https://codeatlas.ai"));

        // Loopback with allow_http = true accepted
        let client = HttpCloudClient::with_allow_http(
            Some("http://localhost:8080/".into()),
            Some("k".into()),
            true,
        );
        assert_eq!(client.base_url.as_deref(), Some("http://localhost:8080"));

        // Loopback with allow_http = false rejected
        let client = HttpCloudClient::with_allow_http(
            Some("http://localhost:8080".into()),
            Some("k".into()),
            false,
        );
        assert!(client.base_url.is_none());
    }
}
