use super::{
    custom_resources::KubePostgresXlCluster,
    enums::ResourceAction,
    functions::{create_context, create_global_template, get_kube_config},
    structs::EmbeddedConfigMapTemplates,
    vars::NAMESPACE,
};
use base64::encode;
use kube::{
    api::{Api, DeleteParams, PostParams},
    client::APIClient,
};
use ring::digest;

pub async fn action(
    custom_resource: &KubePostgresXlCluster,
    resource_action: &ResourceAction,
    config_map_sha: String,
) -> anyhow::Result<String> {
    let context = create_context(&custom_resource, config_map_sha).await;
    let mut config_map_concatenated = "".to_owned();

    if context.is_ok() {
        let context_unwrapped = context?;
        let global_template = create_global_template().await?;

        let config = get_kube_config().await?;
        let client = APIClient::new(config);
        let namespace = std::env::var("NAMESPACE").unwrap_or(NAMESPACE.into());
        let resource_client = Api::v1ConfigMap(client).within(&namespace);

        for asset in EmbeddedConfigMapTemplates::iter() {
            let filename = asset.as_ref();

            // Ignore hidden files
            if !filename.starts_with(".") {
                // Create new resources
                let file_data = EmbeddedConfigMapTemplates::get(&filename).unwrap();
                let file_data_string = std::str::from_utf8(file_data.as_ref())?;
                let new_resource_object = super::functions::create_resource_object(
                    &context_unwrapped,
                    &global_template,
                    &file_data_string.to_owned(),
                )
                .await;

                if new_resource_object.is_ok() {
                    let new_resource_object_unwapped = new_resource_object.unwrap();

                    // append to config string to generate hash
                    let config_string = serde_yaml::to_string(&new_resource_object_unwapped)?;
                    config_map_concatenated =
                        format!("{}{}", &config_map_concatenated, config_string);

                    // Generate and post data
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
    let config_map_sha = digest::digest(&digest::SHA256, config_map_concatenated.as_bytes());
    Ok(encode(config_map_sha.as_ref()))
}
