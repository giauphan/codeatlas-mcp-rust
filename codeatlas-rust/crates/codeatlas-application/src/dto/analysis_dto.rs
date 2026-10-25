use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct AnalyzeRequest {
    #[validate(length(min = 1, max = 4096))]
    pub path: String,
    pub max_files: Option<usize>,
}
