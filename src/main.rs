use tokio::task;
extern crate gtmpl;
extern crate gtmpl_value;
#[macro_use]
extern crate gtmpl_derive;

mod setup_postgres_xl_cluster_controller;
mod structs;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    std::env::set_var("RUST_LOG", "info,kube=trace");
    env_logger::init();
    task::spawn(setup_postgres_xl_cluster_controller::watch()).await??;
    Ok(())
}
