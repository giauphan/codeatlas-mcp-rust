pub mod doctor;
pub mod router;
pub mod setup_claude;
pub mod setup_zed;
pub mod task_router;

pub use doctor::run_doctor;
pub use router::{Cli, Commands};
pub use setup_claude::run_setup_claude;
pub use setup_zed::run_setup_zed;
pub use task_router::run_task_router;
