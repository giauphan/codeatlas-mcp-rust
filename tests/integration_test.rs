//! High-level Integration and Protocol Parity Tests

use codeatlas_application::dto::analysis_dto::AnalyzeRequest;
use codeatlas_application::use_cases::analyze_project::AnalyzeProjectUseCase;
use codeatlas_application::use_cases::query_graph::QueryGraphUseCase;
use codeatlas_application::use_cases::security_scan::SecurityScanUseCase;
use codeatlas_domain::entities::graph::AnalysisResult;
use codeatlas_domain::ports::graph_storage::GraphRepositoryPort;
use codeatlas_domain::value_objects::project_id::ProjectId;
use codeatlas_infrastructure::graph::PetgraphStore;
use codeatlas_infrastructure::parsers::TreeSitterParserEngine;
use codeatlas_infrastructure::storage::LocalFileSystemAdapter;
use std::sync::Arc;

#[tokio::test]
async fn test_analyze_and_query_flow() {
    let parser = Arc::new(TreeSitterParserEngine::new());
    let graph_repo = Arc::new(PetgraphStore::new());
    let fs = Arc::new(LocalFileSystemAdapter::new());

    let analyze_uc = AnalyzeProjectUseCase::new(parser, graph_repo.clone(), fs);
    let query_uc = QueryGraphUseCase::new(graph_repo.clone());
    let security_uc = SecurityScanUseCase::new(graph_repo.clone());

    let temp_dir = std::env::temp_dir().join(format!("codeatlas_test_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&temp_dir);
    let sample_file = temp_dir.join("sample.ts");
    let _ = std::fs::write(
        &sample_file,
        "export function add(a: number, b: number): number { return a + b; }\n",
    );

    // Analyze TypeScript source fixture
    let req = AnalyzeRequest {
        path: temp_dir.to_string_lossy().to_string(),
        max_files: Some(20),
    };

    let result = analyze_uc
        .execute(req.clone())
        .await
        .expect("Analysis should succeed");
    assert!(result.total_files_analyzed > 0);

    let project_id = ProjectId::from_path(std::path::Path::new(&req.path));
    let stored = query_uc
        .get_analysis(&project_id)
        .await
        .expect("Should query analysis");
    assert!(stored.is_some());

    // Security scan
    let findings = security_uc
        .scan_project(&project_id)
        .await
        .expect("Scan should succeed");
    assert!(
        findings.is_empty(),
        "clean fixture must produce no findings: {:?}",
        findings
    );

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[tokio::test]
async fn test_cycle_detection_tarjan() {
    let graph_repo = Arc::new(PetgraphStore::new());
    let query_uc = QueryGraphUseCase::new(graph_repo.clone());

    let project_id = ProjectId::new("test_project");
    let mut analysis = AnalysisResult::default();

    // Create A -> B -> A cycle
    analysis
        .graph
        .nodes
        .push(codeatlas_domain::entities::graph::GraphNode {
            id: "A".to_string(),
            label: "A".to_string(),
            node_type: "function".to_string(),
            val: None,
            color: None,
            file_path: None,
            line: None,
        });
    analysis
        .graph
        .nodes
        .push(codeatlas_domain::entities::graph::GraphNode {
            id: "B".to_string(),
            label: "B".to_string(),
            node_type: "function".to_string(),
            val: None,
            color: None,
            file_path: None,
            line: None,
        });
    analysis
        .graph
        .links
        .push(codeatlas_domain::entities::graph::GraphLink {
            source: "A".to_string(),
            target: "B".to_string(),
            link_type: "call".to_string(),
            label: None,
        });
    analysis
        .graph
        .links
        .push(codeatlas_domain::entities::graph::GraphLink {
            source: "B".to_string(),
            target: "A".to_string(),
            link_type: "call".to_string(),
            label: None,
        });

    graph_repo
        .store_analysis(project_id.clone(), analysis)
        .await
        .unwrap();

    let cycles = query_uc.detect_cycles(&project_id).await.unwrap();
    assert_eq!(cycles.len(), 1);
    assert_eq!(cycles[0].len(), 2);
}
