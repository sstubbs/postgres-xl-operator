use super::{
    controller_config_map, controller_deployment, controller_job, controller_postgres_xl_cluster,
    controller_secret, controller_service, controller_stateful_set, custom_resources,
    enums::ResourceAction,
    functions::get_kube_config,
    vars::{CLUSTER_RESOURCE_PLURAL, CUSTOM_RESOURCE_GROUP, NAMESPACE},
};
use futures::StreamExt;
use kube::{
    api::{Informer, RawApi, WatchEvent},
    client::APIClient,
};

pub async fn watch() -> anyhow::Result<()> {
    let config = get_kube_config().await?;
    let client = APIClient::new(config);
    let namespace = std::env::var("NAMESPACE").unwrap_or(NAMESPACE.into());
    let custom_resource_group =
        std::env::var("CUSTOM_RESOURCE_GROUP").unwrap_or(CUSTOM_RESOURCE_GROUP.into());

    let resource = RawApi::customResource(
        &std::env::var("CLUSTER_RESOURCE_PLURAL").unwrap_or(CLUSTER_RESOURCE_PLURAL.into()),
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
            controller_secret::action(&custom_resource, &ResourceAction::Added, "".to_owned())
                .await?;
            let config_map_sha = controller_config_map::action(
                &custom_resource,
                &ResourceAction::Added,
                "".to_owned(),
            )
            .await?;
            controller_deployment::action(
                &custom_resource,
                &ResourceAction::Added,
                config_map_sha.to_owned(),
            )
            .await?;
            controller_stateful_set::action(
                &custom_resource,
                &ResourceAction::Added,
                config_map_sha.to_owned(),
            )
            .await?;
            controller_job::action(
                &custom_resource,
                &ResourceAction::Added,
                config_map_sha.to_owned(),
            )
            .await?;
            controller_service::action(
                &custom_resource,
                &ResourceAction::Added,
                config_map_sha.to_owned(),
            )
            .await?;
            controller_postgres_xl_cluster::action_create_slave(
                &custom_resource,
                &ResourceAction::Added,
                config_map_sha.to_owned(),
            )
            .await?;
        }
        WatchEvent::Modified(custom_resource) => {
            controller_secret::action(&custom_resource, &ResourceAction::Modified, "".to_owned())
                .await?;
            let config_map_sha = controller_config_map::action(
                &custom_resource,
                &ResourceAction::Modified,
                "".to_owned(),
            )
            .await?;
            controller_deployment::action(
                &custom_resource,
                &ResourceAction::Modified,
                config_map_sha.to_owned(),
            )
            .await?;
            controller_stateful_set::action(
                &custom_resource,
                &ResourceAction::Modified,
                config_map_sha.to_owned(),
            )
            .await?;
            controller_job::action(
                &custom_resource,
                &ResourceAction::Modified,
                config_map_sha.to_owned(),
            )
            .await?;
            controller_service::action(
                &custom_resource,
                &ResourceAction::Modified,
                config_map_sha.to_owned(),
            )
            .await?;
            controller_postgres_xl_cluster::action_create_slave(
                &custom_resource,
                &ResourceAction::Modified,
                config_map_sha.to_owned(),
            )
            .await?;
        }
        WatchEvent::Deleted(custom_resource) => {
            controller_secret::action(&custom_resource, &ResourceAction::Deleted, "".to_owned())
                .await?;
            let config_map_sha = controller_config_map::action(
                &custom_resource,
                &ResourceAction::Deleted,
                "".to_owned(),
            )
            .await?;
            controller_deployment::action(
                &custom_resource,
                &ResourceAction::Deleted,
                config_map_sha.to_owned(),
            )
            .await?;
            controller_stateful_set::action(
                &custom_resource,
                &ResourceAction::Deleted,
                config_map_sha.to_owned(),
            )
            .await?;
            controller_job::action(
                &custom_resource,
                &ResourceAction::Deleted,
                config_map_sha.to_owned(),
            )
            .await?;
            controller_service::action(
                &custom_resource,
                &ResourceAction::Deleted,
                config_map_sha.to_owned(),
            )
            .await?;
            controller_postgres_xl_cluster::action_create_slave(
                &custom_resource,
                &ResourceAction::Deleted,
                config_map_sha.to_owned(),
            )
            .await?;
        }
        _ => info!("no controller created for event."),
    }
    Ok(())
}
