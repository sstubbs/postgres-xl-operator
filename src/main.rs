use tokio::task;
extern crate gtmpl;
extern crate gtmpl_value;
#[macro_use]
extern crate gtmpl_derive;
#[macro_use]
extern crate rust_embed;
#[macro_use]
extern crate anyhow;

mod controller_config_map;
mod controller_postgres_xl_cluster_informer;
mod functions;
mod structs;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    std::env::set_var("RUST_LOG", "info,kube=trace");
    env_logger::init();
    task::spawn(controller_postgres_xl_cluster_informer::watch()).await??;
    Ok(())
}
