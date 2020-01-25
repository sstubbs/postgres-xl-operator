use super::{
    custom_resources::KubePostgresXlCluster,
    enums::ResourceAction,
    functions::{create_context, create_global_template, get_kube_config},
    structs::EmbeddedConfigMapTemplates,
    vars::{CLUSTER_RESOURCE_PLURAL, CUSTOM_RESOURCE_GROUP, NAMESPACE},
};
use kube::{
    api::{Api, DeleteParams, PostParams},
    client::APIClient,
};

pub async fn action_create_slave(
    custom_resource: &KubePostgresXlCluster,
    resource_action: &ResourceAction,
) -> anyhow::Result<()> {
    let context = create_context(&custom_resource).await;

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

        if context_unwrapped.to_owned().cluster.values.replication.enabled
            && context_unwrapped.to_owned().cluster.values.replication.master_name != ""
            && context_unwrapped.to_owned().cluster.values.replication.slave_name != ""
            && context_unwrapped.to_owned().cluster.values.replication.slave_name
                != context_unwrapped.to_owned().cluster.cleaned_name
        {
            match resource_action {
                ResourceAction::Added => {
                    let pp = PostParams::default();

                    let mut post_object = custom_resource.to_owned();
                    post_object.metadata.name =
                        context_unwrapped.to_owned().cluster.values.replication.slave_name;
                    post_object.metadata.resourceVersion = Some("".to_owned());

                    match resource_client
                        .create(&pp, serde_json::to_vec(&post_object)?)
                        .await
                    {
                        Ok(o) => {
                            if context_unwrapped.cluster.values.replication.slave_name
                                == o.metadata.name
                            {
                                info!("Created Slave {}", o.metadata.name);
                            }
                        }
                        Err(e) => error!("{:?}", e), // any other case is probably bad
                    }
                }
                ResourceAction::Modified => {}
                ResourceAction::Deleted => {}
            }
        }
    } else {
        error!("{}", context.err().unwrap())
    }

    Ok(())
}
