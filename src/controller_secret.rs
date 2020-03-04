use super::{
    custom_resources::KubePostgresXlCluster,
    enums::ResourceAction,
    functions::{
        create_context, create_global_template, generate_base64_password, get_kube_config,
    },
    structs::EmbeddedSecretTemplates,
    vars::NAMESPACE,
};
use crate::structs::GeneratedPassword;
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
        let mut context_unwrapped = context?.to_owned();

        if context_unwrapped.cluster.values.security.password.method == "operator" || context_unwrapped.cluster.values.security.password.method == "mount" {
            // Generate passwords
            // Root
            context_unwrapped
                .cluster
                .generated_passwords
                .push(GeneratedPassword {
                    secret_key: context_unwrapped
                        .to_owned()
                        .cluster
                        .values
                        .config
                        .postgres_user,
                    secret_value: generate_base64_password().await?,
                });

            if context_unwrapped.cluster.values.connection_pool.enabled {
                // Connection Pool
                context_unwrapped
                    .cluster
                    .generated_passwords
                    .push(GeneratedPassword {
                        secret_key: context_unwrapped
                            .to_owned()
                            .cluster
                            .values
                            .connection_pool
                            .user,
                        secret_value: generate_base64_password().await?,
                    });
            }

            if context_unwrapped.cluster.values.health_check.enabled {
                // Health check
                context_unwrapped
                    .cluster
                    .generated_passwords
                    .push(GeneratedPassword {
                        secret_key: context_unwrapped
                            .to_owned()
                            .cluster
                            .values
                            .health_check
                            .user,
                        secret_value: generate_base64_password().await?,
                    });
            }

            for user in context_unwrapped
                .to_owned()
                .cluster
                .values
                .security
                .password
                .extra_username
            {
                // Extra users
                context_unwrapped
                    .cluster
                    .generated_passwords
                    .push(GeneratedPassword {
                        secret_key: user,
                        secret_value: generate_base64_password().await?,
                    });
            }
        }

        let global_template = create_global_template().await?;

        let config = get_kube_config().await?;
        let client = APIClient::new(config);
        let namespace = std::env::var("NAMESPACE").unwrap_or(NAMESPACE.into());
        let resource_client = Api::v1Secret(client).within(&namespace);

        for asset in EmbeddedSecretTemplates::iter() {
            let filename = asset.as_ref();

            // Ignore hidden files
            if !filename.starts_with(".") {
                // Create new resources
                let file_data = EmbeddedSecretTemplates::get(&filename).unwrap();
                let file_data_string = std::str::from_utf8(file_data.as_ref())?;
                let new_resource_object = super::functions::create_resource_object(
                    &context_unwrapped.to_owned(),
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
                            let resource_name = &new_resource_object_unwapped["metadata"]["name"]
                                .as_str()
                                .unwrap();

                            let old_resource = resource_client
                                .get(
                                    &new_resource_object_unwapped["metadata"]["name"]
                                        .as_str()
                                        .unwrap(),
                                )
                                .await?;

                            let mut mut_new_resource_object_unwapped =
                                new_resource_object_unwapped.to_owned();
                            mut_new_resource_object_unwapped["metadata"]["resourceVersion"] =
                                serde_yaml::from_str(&format!(
                                    "\"{}\"",
                                    &old_resource.metadata.resourceVersion.unwrap().as_str()
                                ))?;

                            match resource_client
                                .replace(
                                    resource_name,
                                    &pp,
                                    serde_json::to_vec(&mut_new_resource_object_unwapped)?,
                                )
                                .await
                            {
                                Ok(o) => {
                                    if new_resource_object_unwapped["metadata"]["name"]
                                        == o.metadata.name
                                    {
                                        info!("Updated {}", o.metadata.name);
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
