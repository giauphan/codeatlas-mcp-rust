use async_trait::async_trait;
use codeatlas_domain::entities::brain::{DreamMemory, ImmuneGene};
use codeatlas_domain::ports::cloud_sync::{CloudSyncError, CloudSyncPort};
use reqwest::Client;
use std::time::Duration;

pub struct HttpCloudClient {
    client: Client,
    base_url: Option<String>,
    api_key: Option<String>,
}

impl HttpCloudClient {
    pub fn new(base_url: Option<String>, api_key: Option<String>) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .unwrap_or_else(|_| Client::new());

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
}

impl Default for HttpCloudClient {
    fn default() -> Self {
        Self::from_env()
    }
}

#[async_trait]
impl CloudSyncPort for HttpCloudClient {
    async fn check_auth(&self) -> Result<bool, CloudSyncError> {
        let (url, key) = match (&self.base_url, &self.api_key) {
            (Some(u), Some(k)) => (u, k),
            _ => return Err(CloudSyncError::Offline),
        };

        let endpoint = format!("{}/api/auth/verify", url);
        let resp = self
            .client
            .get(&endpoint)
            .header("Authorization", format!("Bearer {}", key))
            .send()
            .await
            .map_err(|e| CloudSyncError::Network(e.to_string()))?;

        Ok(resp.status().is_success())
    }

    async fn save_dream(&self, dream: &DreamMemory) -> Result<(), CloudSyncError> {
        let (url, key) = match (&self.base_url, &self.api_key) {
            (Some(u), Some(k)) => (u, k),
            _ => return Err(CloudSyncError::Offline),
        };

        let endpoint = format!("{}/api/dreams", url);
        let resp = self
            .client
            .post(&endpoint)
            .header("Authorization", format!("Bearer {}", key))
            .json(dream)
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

    async fn query_dreams(
        &self,
        query: &str,
        project: Option<&str>,
        limit: usize,
    ) -> Result<Vec<DreamMemory>, CloudSyncError> {
        let (url, key) = match (&self.base_url, &self.api_key) {
            (Some(u), Some(k)) => (u, k),
            _ => return Err(CloudSyncError::Offline),
        };

        let limit_str = limit.to_string();
        let mut query_params = vec![("q", query), ("limit", &limit_str)];
        if let Some(p) = project {
            query_params.push(("project", p));
        }

        let endpoint = format!("{}/api/dreams/query", url);
        let resp = self
            .client
            .get(&endpoint)
            .header("Authorization", format!("Bearer {}", key))
            .query(&query_params)
            .send()
            .await
            .map_err(|e| CloudSyncError::Network(e.to_string()))?;

        if resp.status().is_success() {
            let memories: Vec<DreamMemory> = resp
                .json()
                .await
                .map_err(|e| CloudSyncError::Network(e.to_string()))?;
            Ok(memories)
        } else {
            Err(CloudSyncError::Api {
                status: resp.status().as_u16(),
                message: resp.text().await.unwrap_or_default(),
            })
        }
    }

    async fn search_genome(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<ImmuneGene>, CloudSyncError> {
        let (url, key) = match (&self.base_url, &self.api_key) {
            (Some(u), Some(k)) => (u, k),
            _ => return Err(CloudSyncError::Offline),
        };

        let limit_str = limit.to_string();
        let endpoint = format!("{}/api/genome/search", url);
        let resp = self
            .client
            .get(&endpoint)
            .header("Authorization", format!("Bearer {}", key))
            .query(&[("q", query), ("limit", &limit_str)])
            .send()
            .await
            .map_err(|e| CloudSyncError::Network(e.to_string()))?;

        if resp.status().is_success() {
            let genes = resp
                .json()
                .await
                .map_err(|e| CloudSyncError::Network(e.to_string()))?;
            Ok(genes)
        } else {
            Err(CloudSyncError::Api {
                status: resp.status().as_u16(),
                message: resp.text().await.unwrap_or_default(),
            })
        }
    }

    async fn scan_immune_genes(&self, problem: &str) -> Result<Vec<ImmuneGene>, CloudSyncError> {
        let (url, key) = match (&self.base_url, &self.api_key) {
            (Some(u), Some(k)) => (u, k),
            _ => return Err(CloudSyncError::Offline),
        };

        let endpoint = format!("{}/api/immune/scan", url);
        let resp = self
            .client
            .get(&endpoint)
            .header("Authorization", format!("Bearer {}", key))
            .query(&[("problem", problem)])
            .send()
            .await
            .map_err(|e| CloudSyncError::Network(e.to_string()))?;

        if resp.status().is_success() {
            let genes = resp
                .json()
                .await
                .map_err(|e| CloudSyncError::Network(e.to_string()))?;
            Ok(genes)
        } else {
            Err(CloudSyncError::Api {
                status: resp.status().as_u16(),
                message: resp.text().await.unwrap_or_default(),
            })
        }
    }

    async fn save_immune_gene(&self, gene: &ImmuneGene) -> Result<(), CloudSyncError> {
        let (url, key) = match (&self.base_url, &self.api_key) {
            (Some(u), Some(k)) => (u, k),
            _ => return Err(CloudSyncError::Offline),
        };

        let endpoint = format!("{}/api/immune/genes", url);
        let resp = self
            .client
            .post(&endpoint)
            .header("Authorization", format!("Bearer {}", key))
            .json(gene)
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
}
