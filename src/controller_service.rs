use super::{
    custom_resources::KubePostgresXlCluster,
    enums::ResourceAction,
    functions::{create_context, create_global_template, get_kube_config},
    structs::EmbeddedServiceTemplates,
    vars::NAMESPACE,
};
use kube::{
    api::{Api, DeleteParams, PostParams},
    client::APIClient,
};

pub async fn action(
    custom_resource: &KubePostgresXlCluster,
    resource_action: &ResourceAction,
    config_map_sha: String,
) -> anyhow::Result<()> {
    let context = create_context(&custom_resource, config_map_sha).await;

    if context.is_ok() {
        let context_unwrapped = context?;
        let global_template = create_global_template().await?;

        let config = get_kube_config().await?;
        let client = APIClient::new(config);
        let namespace = std::env::var("NAMESPACE").unwrap_or(NAMESPACE.into());
        let resource_client = Api::v1Service(client).within(&namespace);

        for asset in EmbeddedServiceTemplates::iter() {
            let filename = asset.as_ref();

            // Ignore hidden files
            if !filename.starts_with(".") {
                // Create new resources
                let file_data = EmbeddedServiceTemplates::get(&filename).unwrap();
                let file_data_string = std::str::from_utf8(file_data.as_ref())?;
                let new_resource_object = super::functions::create_resource_object(
                    &context_unwrapped,
                    &global_template,
                    &file_data_string.to_owned(),
                )
                .await;

                if new_resource_object.is_ok() {
                    let new_resource_object_unwapped = new_resource_object.unwrap();

                    let pp = PostParams::default();

                    match resource_action {
                        ResourceAction::Added => {
                            match resource_client
                                .create(&pp, serde_json::to_vec(&new_resource_object_unwapped)?)
                                .await
                            {
                                Ok(o) => {
                                    if new_resource_object_unwapped["metadata"]["name"]
                                        == o.metadata.name
                                    {
                                        info!("Created {}", o.metadata.name);
                                    }
                                }
                                Err(e) => error!("{:?}", e), // any other case is probably bad
                            }
                        }
                        ResourceAction::Modified => {
                            // Services have immutable fields so rather delete and recreate on modify than modify the resource in place like other types
                            // Delete service
                            let resource_name = &new_resource_object_unwapped["metadata"]["name"]
                                .as_str()
                                .unwrap();
                            match resource_client
                                .delete(resource_name, &DeleteParams::default())
                                .await
                            {
                                Ok(_o) => {
                                    info!(
                                        "Deleted {}",
                                        new_resource_object_unwapped["metadata"]["name"]
                                            .as_str()
                                            .unwrap()
                                    );

                                    // Recreate it
                                    match resource_client
                                        .create(
                                            &pp,
                                            serde_json::to_vec(&new_resource_object_unwapped)?,
                                        )
                                        .await
                                    {
                                        Ok(o) => {
                                            if new_resource_object_unwapped["metadata"]["name"]
                                                == o.metadata.name
                                            {
                                                info!("Created {}", o.metadata.name);
                                            }
                                        }
                                        Err(e) => error!("{:?}", e), // any other case is probably bad
                                    }
                                }
                                Err(e) => error!("{:?}", e), // any other case is probably bad
                            }
                        }
                        ResourceAction::Deleted => {
                            let resource_name = &new_resource_object_unwapped["metadata"]["name"]
                                .as_str()
                                .unwrap();
                            match resource_client
                                .delete(resource_name, &DeleteParams::default())
                                .await
                            {
                                Ok(_o) => info!(
                                    "Deleted {}",
                                    new_resource_object_unwapped["metadata"]["name"]
                                        .as_str()
                                        .unwrap()
                                ),
                                Err(e) => error!("{:?}", e), // any other case is probably bad
                            }
                        }
                    }
                }
            }
        }
    } else {
        error!("{}", context.err().unwrap())
    }

    Ok(())
}
