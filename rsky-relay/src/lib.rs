#![deny(
    deprecated_safe,
    future_incompatible,
    let_underscore,
    keyword_idents,
    nonstandard_style,
    refining_impl_trait,
    rust_2018_compatibility,
    rust_2018_idioms,
    rust_2021_compatibility,
    rust_2024_compatibility,
    unused,
    warnings,
    clippy::all,
    clippy::cargo,
    clippy::dbg_macro,
    clippy::expect_used,
    clippy::iter_over_hash_type,
    clippy::nursery,
    clippy::pathbuf_init_then_push,
    clippy::pedantic,
    clippy::print_stderr,
    clippy::print_stdout,
    clippy::renamed_function_params,
    clippy::str_to_string,
    clippy::string_to_string,
    clippy::unused_result_ok,
    clippy::unwrap_used
)]
#![allow(
    clippy::cargo_common_metadata,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::missing_safety_doc,
    clippy::multiple_crate_versions
)]
#![cfg_attr(
    test,
    allow(
        clippy::expect_used,
        clippy::unwrap_used,
        clippy::redundant_pub_crate,
        clippy::min_ident_chars,
        clippy::str_to_string,
        clippy::string_to_string,
        clippy::tests_outside_test_module,
        clippy::let_underscore_must_use,
        clippy::print_stdout,
        clippy::panic,
        clippy::panic_in_result_fn,
        clippy::missing_assert_message,
        clippy::literal_string_with_formatting_args,
        clippy::unused_result_ok,
        clippy::renamed_function_params
    )
)]

mod crawler;
mod publisher;
mod server;
mod types;
mod validator;

pub mod config;
pub mod metrics;

use std::sync::atomic::AtomicBool;

use thiserror::Error;

pub static SHUTDOWN: AtomicBool = AtomicBool::new(false);

/// Set to `true` by the `PgListener` task (spawned in `main`) whenever a
/// `banned_hosts_changed` NOTIFY arrives. The crawler manager's update loop
/// checks this flag and clears it, enabling instant ban propagation instead of
/// waiting for the `BAN_REFRESH_INTERVAL` polling fallback.
pub static BAN_REFRESH_NEEDED: AtomicBool = AtomicBool::new(false);

/// Shared PostgreSQL connection pool. Initialised once in `main` and passed by reference
/// to all components that need it. Using a static avoids threading the pool through every
/// struct while keeping the pool lifetime tied to the process.
pub type PgPool = sqlx::PgPool;

pub use crawler::Manager as CrawlerManager;
pub use publisher::Manager as PublisherManager;
pub use server::Server;
pub use types::MessageRecycle;
pub use validator::Manager as ValidatorManager;

#[derive(Debug, Error)]
pub enum RelayError {
    #[error("crawler error: {0}")]
    Crawler(#[from] crawler::ManagerError),
    #[error("publisher error: {0}")]
    Publisher(#[from] publisher::ManagerError),
    #[error("validator error: {0}")]
    Validator(#[from] validator::ManagerError),
    #[error("server error: {0}")]
    Server(#[from] server::ServerError),
}
