//! CodeAtlas Binary Entrypoint & Composition Root

#[cfg(not(target_env = "msvc"))]
use tikv_jemallocator::Jemalloc;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

use clap::Parser;
use codeatlas_adapters::cli::{
    run_doctor, run_setup_claude, run_setup_zed, run_task_router, Cli, Commands,
};
use codeatlas_adapters::mcp::protocol::ToolRegistry;
use codeatlas_adapters::mcp::stdio_transport::StdioTransport;
use codeatlas_adapters::mcp::tools::register_all_tools;
use codeatlas_adapters::outbound::{AdrFileSystemAdapter, HttpCloudClient};
use codeatlas_application::use_cases::{
    AnalyzeProjectUseCase, ManageAdrUseCase, QueryGraphUseCase, SearchCodeUseCase,
    SecondBrainUseCase, SecurityScanUseCase,
};
use codeatlas_infrastructure::graph::PetgraphStore;
use codeatlas_infrastructure::parsers::TreeSitterParserEngine;
use codeatlas_infrastructure::storage::LocalFileSystemAdapter;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // 1. Initialize Infrastructure Singletons
    let parser = Arc::new(TreeSitterParserEngine::new());
    let graph_repo = Arc::new(PetgraphStore::new());
    let fs = Arc::new(LocalFileSystemAdapter::new());
    let adr_repo = Arc::new(AdrFileSystemAdapter::new());
    let cloud_sync = Arc::new(HttpCloudClient::from_env());

    // 2. Compose Application Layer Use Cases
    let brain_uc = Arc::new(SecondBrainUseCase::new(cloud_sync.clone()));

    if let Some(cmd) = cli.command {
        match cmd {
            Commands::Doctor => run_doctor()?,
            Commands::Init | Commands::Setup => {
                run_doctor()?;
                println!();
                run_setup_claude(false)?;
            }
            Commands::SetupZed { force } => run_setup_zed(force)?,
            Commands::SetupClaude { force } => run_setup_claude(force)?,
            Commands::RouteTask { task } => run_task_router(&task)?,
            Commands::BrainContext { task } => {
                let ctx = brain_uc
                    .load_brain_context(&task, None)
                    .await
                    .map_err(|e| anyhow::anyhow!("{}", e))?;
                println!("{}", serde_json::to_string_pretty(&ctx)?);
            }
            Commands::BrainSave { content, category } => {
                let now_nanos = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos();
                let project_name = cli
                    .project_dir
                    .as_ref()
                    .and_then(|p| p.file_name())
                    .and_then(|n| n.to_str())
                    .unwrap_or("default");
                let dream = codeatlas_domain::entities::brain::DreamMemory {
                    id: format!("cli_dream_{:x}", now_nanos),
                    category,
                    content,
                    project: project_name.to_string(),
                    timestamp: codeatlas_adapters::mcp::tools::current_rfc3339(),
                    score: None,
                };
                brain_uc
                    .save_dream(&dream)
                    .await
                    .map_err(|e| anyhow::anyhow!("{}", e))?;
                println!("✓ Dream memory saved successfully.");
            }
        }
        return Ok(());
    }

    if cli.sync_dreams {
        let project_name = cli
            .project_dir
            .as_ref()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("default");
        match brain_uc.query_dreams("", Some(project_name), 50).await {
            Ok(dreams) => {
                println!(
                    "{{\"success\": true, \"message\": \"Dream sync complete\", \"count\": {}}}",
                    dreams.len()
                );
                return Ok(());
            }
            Err(e) => {
                eprintln!("Dream sync failed: {}", e);
                std::process::exit(1);
            }
        }
    }

    let analyze_uc = Arc::new(AnalyzeProjectUseCase::new(
        parser.clone(),
        graph_repo.clone(),
        fs.clone(),
    ));
    let query_uc = Arc::new(QueryGraphUseCase::new(graph_repo.clone()));
    let search_uc = Arc::new(SearchCodeUseCase::new(graph_repo.clone(), fs.clone()));
    let security_uc = Arc::new(SecurityScanUseCase::new(graph_repo.clone()));
    let adr_uc = Arc::new(ManageAdrUseCase::new(adr_repo.clone()));

    // 3. Register All Tools in MCP Tool Registry
    let mut registry = ToolRegistry::new();
    register_all_tools(
        &mut registry,
        analyze_uc,
        query_uc,
        search_uc,
        security_uc,
        brain_uc,
        adr_uc,
    );

    // 4. Start Stdio MCP Server
    let transport = StdioTransport::new(Arc::new(registry));
    transport.run().await?;

    Ok(())
}
