use super::{
    custom_resources::KubePostgresXlCluster,
    enums::ResourceAction,
    functions::{create_context, get_kube_config},
    vars::{CLUSTER_RESOURCE_PLURAL, CUSTOM_RESOURCE_GROUP, NAMESPACE},
};
use kube::{
    api::{Api, DeleteParams, PostParams},
    client::APIClient,
};

pub async fn action_create_slave(
    custom_resource: &KubePostgresXlCluster,
    resource_action: &ResourceAction,
    config_map_sha: String,
) -> anyhow::Result<()> {
    let context = create_context(&custom_resource, config_map_sha).await;

    if context.is_ok() {
        let config = get_kube_config().await?;
        let client = APIClient::new(config);
        let namespace = std::env::var("NAMESPACE").unwrap_or(NAMESPACE.into());
        let custom_resource_group =
            std::env::var("CUSTOM_RESOURCE_GROUP").unwrap_or(CUSTOM_RESOURCE_GROUP.into());
        let resource =
            &std::env::var("CLUSTER_RESOURCE_PLURAL").unwrap_or(CLUSTER_RESOURCE_PLURAL.into());

        let resource_client: Api<KubePostgresXlCluster> = Api::customResource(client, resource)
            .group(&custom_resource_group)
            .within(&namespace);

        let context_unwrapped = context.unwrap();

        if context_unwrapped
            .to_owned()
            .cluster
            .values
            .replication
            .enabled
            && context_unwrapped
                .to_owned()
                .cluster
                .values
                .replication
                .standby_name
                != ""
        {
            let pp = PostParams::default();

            let mut post_object = custom_resource.to_owned();

            post_object.metadata.name = context_unwrapped
                .to_owned()
                .cluster
                .values
                .replication
                .standby_name;

            let mut current_data: serde_yaml::Value =
                serde_yaml::from_str(&custom_resource.to_owned().spec.data.unwrap())?;

            current_data["replication"]["master_name"] =
                serde_yaml::from_str(&custom_resource.to_owned().metadata.name)?;
            current_data["replication"]["standby_name"] = serde_yaml::from_str("\"\"")?;

            // no gtm proxies and only one coordinator are needed for standby
            current_data["proxies"]["enabled"] = serde_yaml::from_str("false")?;
            current_data["proxies"]["count"] = serde_yaml::from_str("0")?;
            current_data["coordinators"]["count"] = serde_yaml::from_str("1")?;

            post_object.spec.data = Some(serde_yaml::to_string(&current_data)?);

            match resource_action {
                ResourceAction::Added => {
                    post_object.metadata.resourceVersion = Some("".to_owned());

                    match resource_client
                        .create(&pp, serde_json::to_vec(&post_object)?)
                        .await
                    {
                        Ok(o) => {
                            if context_unwrapped.cluster.values.replication.standby_name
                                == o.metadata.name
                            {
                                info!("Created Standby {}", o.metadata.name);
                            }
                        }
                        Err(e) => error!("{:?}", e), // any other case is probably bad
                    }
                }
                ResourceAction::Modified => {
                    let old_resource = resource_client.get(&post_object.metadata.name).await?;

                    post_object.metadata.resourceVersion = old_resource.metadata.resourceVersion;
                    post_object.metadata.uid = old_resource.metadata.uid;

                    match resource_client
                        .replace(
                            &post_object.metadata.name,
                            &pp,
                            serde_json::to_vec(&post_object)?,
                        )
                        .await
                    {
                        Ok(o) => {
                            if context_unwrapped.cluster.values.replication.standby_name
                                == o.metadata.name
                            {
                                info!("Updated Standby {}", o.metadata.name);
                            }
                        }
                        Err(e) => error!("{:?}", e), // any other case is probably bad
                    }
                }
                ResourceAction::Deleted => {
                    match resource_client
                        .delete(&post_object.metadata.name, &DeleteParams::default())
                        .await
                    {
                        Ok(_o) => {
                            info!("Deleted Standby {}", &post_object.metadata.name);
                        }
                        Err(e) => error!("{:?}", e), // any other case is probably bad
                    }
                }
            }
        } else if !context_unwrapped
            .to_owned()
            .cluster
            .values
            .replication
            .enabled
            && context_unwrapped
                .to_owned()
                .cluster
                .values
                .replication
                .standby_name
                != ""
        {
            // If standby_name retained but replication disabled we will delete the standby. PVC will be retained though for promotion or recreating it.
            let mut post_object = custom_resource.to_owned();

            post_object.metadata.name = context_unwrapped
                .to_owned()
                .cluster
                .values
                .replication
                .standby_name;

            match resource_action {
                ResourceAction::Modified => {
                    match resource_client
                        .delete(&post_object.metadata.name, &DeleteParams::default())
                        .await
                    {
                        Ok(_o) => {
                            info!("Deleted Standby {}", &post_object.metadata.name);
                        }
                        Err(e) => error!("{:?}", e), // any other case is probably bad
                    }
                }
                _ => {}
            }
        }
    } else {
        error!("{}", context.err().unwrap())
    }

    Ok(())
}
