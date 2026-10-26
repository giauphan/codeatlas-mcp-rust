pub mod analysis_tools;
pub mod brain_tools;
pub mod dependency_tools;
pub mod genome_tools;
pub mod search_tools;
pub mod security_tools;
pub mod system_memory_tools;
pub mod workspace_tools;

pub use analysis_tools::register_analysis_tools;
pub use brain_tools::{current_rfc3339, register_brain_tools};
pub use dependency_tools::register_dependency_tools;
pub use genome_tools::register_genome_tools;
pub use search_tools::register_search_tools;
pub use security_tools::register_security_tools;
pub use system_memory_tools::register_system_memory_tools;
pub use workspace_tools::register_workspace_tools;

use super::protocol::ToolRegistry;
use codeatlas_application::use_cases::{
    AnalyzeProjectUseCase, ManageAdrUseCase, QueryGraphUseCase, SearchCodeUseCase,
    SecondBrainUseCase, SecurityScanUseCase,
};
use std::sync::Arc;

pub fn register_all_tools(
    registry: &mut ToolRegistry,
    analyze_uc: Arc<AnalyzeProjectUseCase>,
    query_uc: Arc<QueryGraphUseCase>,
    search_uc: Arc<SearchCodeUseCase>,
    security_uc: Arc<SecurityScanUseCase>,
    brain_uc: Arc<SecondBrainUseCase>,
    adr_uc: Arc<ManageAdrUseCase>,
) {
    register_analysis_tools(registry, analyze_uc, query_uc.clone());
    register_dependency_tools(registry, query_uc.clone());
    register_search_tools(registry, search_uc, query_uc.clone());
    register_security_tools(registry, security_uc, query_uc.clone());
    register_brain_tools(registry, brain_uc.clone());
    register_genome_tools(registry, brain_uc);
    register_system_memory_tools(registry);
    register_workspace_tools(registry, adr_uc, query_uc);
}
