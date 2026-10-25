use crate::dto::analysis_dto::AnalyzeRequest;
use codeatlas_domain::entities::graph::AnalysisResult;
use codeatlas_domain::ports::analyzer::AstParserPort;
use codeatlas_domain::ports::file_system::FileSystemPort;
use codeatlas_domain::ports::graph_storage::GraphRepositoryPort;
use codeatlas_domain::value_objects::project_id::ProjectId;
use std::path::Path;
use std::sync::Arc;
use thiserror::Error;
use validator::Validate;

#[derive(Error, Debug)]
pub enum AnalyzeUseCaseError {
    #[error("Validation failed: {0}")]
    Validation(#[from] validator::ValidationErrors),
    #[error("Invalid path: {0}")]
    InvalidPath(String),
    #[error("Filesystem error: {0}")]
    FileSystem(String),
    #[error("Parser error: {0}")]
    Parser(String),
    #[error("Storage error: {0}")]
    Storage(String),
}

pub struct AnalyzeProjectUseCase {
    parser: Arc<dyn AstParserPort>,
    graph_repo: Arc<dyn GraphRepositoryPort>,
    fs: Arc<dyn FileSystemPort>,
}

impl AnalyzeProjectUseCase {
    pub fn new(
        parser: Arc<dyn AstParserPort>,
        graph_repo: Arc<dyn GraphRepositoryPort>,
        fs: Arc<dyn FileSystemPort>,
    ) -> Self {
        Self {
            parser,
            graph_repo,
            fs,
        }
    }

    pub async fn execute(
        &self,
        req: AnalyzeRequest,
    ) -> Result<AnalysisResult, AnalyzeUseCaseError> {
        req.validate()?;
        let root = Path::new(&req.path);
        if !self.fs.file_exists(root).await {
            return Err(AnalyzeUseCaseError::InvalidPath(format!(
                "Directory does not exist: {}",
                req.path
            )));
        }

        let max_files = req.max_files.unwrap_or(5000);
        let discovered = self
            .fs
            .discover_files(root, max_files)
            .await
            .map_err(|e| AnalyzeUseCaseError::FileSystem(e.to_string()))?;

        let total_discovered = discovered.len();
        let files: Vec<_> = discovered
            .into_iter()
            .filter(|f| self.parser.supports_language(f.as_path()))
            .collect();

        let graph_data = self
            .parser
            .parse_batch(&files)
            .await
            .map_err(|e| AnalyzeUseCaseError::Parser(e.to_string()))?;

        let project_id = ProjectId::from_path(root);
        let result = AnalysisResult {
            graph: graph_data,
            insights: Vec::new(),
            entity_counts: Default::default(),
            total_files_analyzed: files.len(),
            total_files_skipped: total_discovered.saturating_sub(files.len()),
        };

        self.graph_repo
            .store_analysis(project_id, result.clone())
            .await
            .map_err(|e| AnalyzeUseCaseError::Storage(e.to_string()))?;

        Ok(result)
    }
}
