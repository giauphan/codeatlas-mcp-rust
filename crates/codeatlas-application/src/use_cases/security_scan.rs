use codeatlas_domain::entities::graph::AnalysisResult;
use codeatlas_domain::entities::security::SecurityFinding;
use codeatlas_domain::ports::graph_storage::GraphRepositoryPort;
use codeatlas_domain::value_objects::project_id::ProjectId;
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SecurityScanError {
    #[error("Storage error: {0}")]
    Storage(String),
}

pub struct SecurityScanUseCase {
    graph_repo: Arc<dyn GraphRepositoryPort>,
}

impl SecurityScanUseCase {
    pub fn new(graph_repo: Arc<dyn GraphRepositoryPort>) -> Self {
        Self { graph_repo }
    }

    pub async fn scan_project(
        &self,
        project: &ProjectId,
    ) -> Result<Vec<SecurityFinding>, SecurityScanError> {
        let analysis_opt = self
            .graph_repo
            .get_analysis(project)
            .await
            .map_err(|e| SecurityScanError::Storage(e.to_string()))?;

        let analysis = match analysis_opt {
            Some(a) => a,
            None => return Ok(Vec::new()),
        };

        Ok(Self::evaluate_security_rules(&analysis))
    }

    fn is_test_file(file_path: &str) -> bool {
        let p = std::path::Path::new(file_path);
        let in_test_dir = p.components().any(|c| {
            matches!(
                c.as_os_str().to_string_lossy().as_ref(),
                "test" | "tests" | "__tests__" | "__mocks__" | "mocks" | "spec"
            )
        });
        let file_name = p
            .file_name()
            .map(|n| n.to_string_lossy().to_lowercase())
            .unwrap_or_default();

        in_test_dir
            || file_name.contains(".test.")
            || file_name.contains(".spec.")
            || file_name.ends_with("_test.py")
            || file_name.starts_with("test_")
    }

    pub fn evaluate_security_rules(analysis: &AnalysisResult) -> Vec<SecurityFinding> {
        let mut findings = Vec::new();
        let secret_keywords = [
            "api_key",
            "secret",
            "password",
            "token",
            "private_key",
            "access_key",
        ];
        let unsafe_functions = [
            "eval",
            "exec",
            "system",
            "child_process",
            "spawn",
            "shell_exec",
        ];

        let mut node_to_filepath: HashMap<String, String> = HashMap::new();

        // 1. Scan nodes for variables with hardcoded secrets or unsafe function definitions
        for node in &analysis.graph.nodes {
            let file_path = node
                .file_path
                .clone()
                .unwrap_or_else(|| "unknown".to_string());
            node_to_filepath.insert(node.id.clone(), file_path.clone());
            node_to_filepath.insert(node.label.clone(), file_path.clone());

            if Self::is_test_file(&file_path) {
                continue;
            }

            let label_lower = node.label.to_lowercase();

            if node.node_type == "variable" {
                for secret in &secret_keywords {
                    if label_lower.contains(secret) {
                        findings.push(SecurityFinding {
                            severity: "HIGH".to_string(),
                            finding_type: "HARDCODED_SECRET".to_string(),
                            message: format!(
                                "Potential hardcoded secret found in variable: {}",
                                node.label
                            ),
                            file_path: file_path.clone(),
                            line: node.line,
                            snippet: None,
                        });
                        break;
                    }
                }
            } else if node.node_type == "function" {
                for dangerous in &unsafe_functions {
                    if label_lower == *dangerous {
                        findings.push(SecurityFinding {
                            severity: "CRITICAL".to_string(),
                            finding_type: "UNSAFE_FUNCTION".to_string(),
                            message: format!(
                                "Use of potentially dangerous function: {}",
                                node.label
                            ),
                            file_path: file_path.clone(),
                            line: node.line,
                            snippet: None,
                        });
                        break;
                    }
                }
            }
        }

        // 2. Scan call links to identify invocations of dangerous functions
        for link in &analysis.graph.links {
            if link.link_type == "call" {
                let source_path = node_to_filepath
                    .get(&link.source)
                    .map(|s| s.as_str())
                    .unwrap_or(link.source.as_str());

                if Self::is_test_file(source_path) {
                    continue;
                }

                let target_lower = link.target.to_lowercase();
                for dangerous in &unsafe_functions {
                    if target_lower == *dangerous
                        || target_lower.ends_with(&format!("::{}", dangerous))
                    {
                        findings.push(SecurityFinding {
                            severity: "CRITICAL".to_string(),
                            finding_type: "UNSAFE_FUNCTION_CALL".to_string(),
                            message: format!(
                                "Dangerous function call to '{}' detected from '{}'",
                                link.target, link.source
                            ),
                            file_path: source_path.to_string(),
                            line: None,
                            snippet: None,
                        });
                        break;
                    }
                }
            }
        }

        findings
    }
}
