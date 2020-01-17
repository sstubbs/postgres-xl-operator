use super::{
    controller_config_map, controller_service, controller_stateful_set, controller_deployment, custom_resources, enums::ResourceAction, vars,
};
use futures::StreamExt;
use kube::{
    api::{Informer, RawApi, WatchEvent},
    client::APIClient,
    config,
};

pub async fn watch() -> anyhow::Result<()> {
    let config = config::load_kube_config().await?;
    let client = APIClient::new(config);
    let namespace = std::env::var("NAMESPACE").unwrap_or(vars::NAMESPACE.into());
    let custom_resource_group =
        std::env::var("CUSTOM_RESOURCE_GROUP").unwrap_or(vars::CUSTOM_RESOURCE_GROUP.into());

    let resource = RawApi::customResource(
        &std::env::var("CLUSTER_RESOURCE_PLURAL").unwrap_or(vars::CLUSTER_RESOURCE_PLURAL.into()),
    )
    .group(&custom_resource_group)
    .within(&namespace);

    let ei = Informer::raw(client, resource).init().await?;

    loop {
        let mut events = ei.poll().await?.boxed();

        while let Some(event) = events.next().await {
            let event = event?;
            handle_events(event).await?;
        }
    }
}

pub async fn handle_events(
    ev: WatchEvent<custom_resources::KubePostgresXlCluster>,
) -> anyhow::Result<()> {
    match ev {
        WatchEvent::Added(custom_resource) => {
            controller_config_map::action(&custom_resource, &ResourceAction::Added).await?;
            controller_deployment::action(&custom_resource, &ResourceAction::Added).await?;
            controller_service::action(&custom_resource, &ResourceAction::Added).await?;
            controller_stateful_set::action(&custom_resource, &ResourceAction::Added).await?;
        }
        WatchEvent::Modified(custom_resource) => {
            controller_config_map::action(&custom_resource, &ResourceAction::Modified).await?;
            controller_deployment::action(&custom_resource, &ResourceAction::Modified).await?;
            controller_service::action(&custom_resource, &ResourceAction::Modified).await?;
            controller_stateful_set::action(&custom_resource, &ResourceAction::Modified).await?;
        }
        WatchEvent::Deleted(custom_resource) => {
            controller_config_map::action(&custom_resource, &ResourceAction::Deleted).await?;
            controller_deployment::action(&custom_resource, &ResourceAction::Deleted).await?;
            controller_service::action(&custom_resource, &ResourceAction::Deleted).await?;
            controller_stateful_set::action(&custom_resource, &ResourceAction::Deleted).await?;
        }
        _ => info!("no controller created for event."),
    }
    Ok(())
}
