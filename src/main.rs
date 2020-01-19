use tokio::task;
extern crate gtmpl;
extern crate gtmpl_value;
#[macro_use]
extern crate gtmpl_derive;
#[macro_use]
extern crate rust_embed;
#[macro_use]
extern crate anyhow;
#[macro_use]
extern crate log;

mod controller_config_map;
mod controller_deployment;
mod controller_job;
mod controller_postgres_xl_cluster_informer;
mod controller_service;
mod controller_stateful_set;
mod custom_resources;
mod enums;
mod functions;
mod structs;
mod vars;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    std::env::set_var(
        "RUST_LOG",
        std::env::var("LOG_LEVEL").unwrap_or(vars::LOG_LEVEL.into()),
    );
    env_logger::init();
    task::spawn(controller_postgres_xl_cluster_informer::watch()).await??;
    Ok(())
}
