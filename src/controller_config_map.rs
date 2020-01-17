use super::enums;
use super::structs;
use super::vars;
use kube::{
    api::{Api, DeleteParams, PostParams},
    client::APIClient,
    config,
};

pub async fn action(
    context_unwrapped: structs::Chart,
    global_template: String,
    resource_action: enums::ResourceAction,
) -> anyhow::Result<()> {
    let config = config::load_kube_config().await?;
    let client = APIClient::new(config);
    let namespace = std::env::var("NAMESPACE").unwrap_or(vars::NAMESPACE.into());
    let config_maps = Api::v1ConfigMap(client).within(&namespace);

    for asset in structs::EmbeddedConfigMapTemplates::iter() {
        let filename = asset.as_ref();

        // Ignore hidden files
        if !filename.starts_with(".") {
            // Create new resources
            let file_data = structs::EmbeddedConfigMapTemplates::get(&filename).unwrap();
            let file_data_string = std::str::from_utf8(file_data.as_ref())?;
            let new_resource_object = super::functions::create_resource_object(
                &context_unwrapped,
                &global_template,
                &file_data_string.to_owned(),
            )
            .await?;
            let pp = PostParams::default();

            match resource_action {
                enums::ResourceAction::Added => {
                    match config_maps
                        .create(&pp, serde_json::to_vec(&new_resource_object)?)
                        .await
                    {
                        Ok(o) => {
                            assert_eq!(new_resource_object["metadata"]["name"], o.metadata.name);
                            info!("Created {}", o.metadata.name);
                        }
                        Err(kube::Error::Api(ae)) => {
                            assert_eq!(ae.code, 409);
                            info!(
                                "{} already exists",
                                new_resource_object["metadata"]["name"].as_str().unwrap()
                            )
                        } // if you skipped delete, for instance
                        Err(e) => return Err(e.into()), // any other case is probably bad
                    }
                }
                enums::ResourceAction::Modified => {
                    let resource_name = &new_resource_object["metadata"]["name"].as_str().unwrap();
                    match config_maps
                        .replace(
                            resource_name,
                            &pp,
                            serde_json::to_vec(&new_resource_object)?,
                        )
                        .await
                    {
                        Ok(o) => {
                            assert_eq!(new_resource_object["metadata"]["name"], o.metadata.name);
                            info!("Updated {}", o.metadata.name);
                        }
                        Err(kube::Error::Api(ae)) => assert_eq!(ae.code, 409), // if you skipped delete, for instance
                        Err(e) => return Err(e.into()), // any other case is probably bad
                    }
                }
                enums::ResourceAction::Deleted => {
                    let resource_name = &new_resource_object["metadata"]["name"].as_str().unwrap();
                    match config_maps
                        .delete(resource_name, &DeleteParams::default())
                        .await
                    {
                        Ok(_o) => info!(
                            "Deleted {}",
                            new_resource_object["metadata"]["name"].as_str().unwrap()
                        ),
                        Err(kube::Error::Api(ae)) => assert_eq!(ae.code, 409), // if you skipped delete, for instance
                        Err(e) => return Err(e.into()), // any other case is probably bad
                    }
                }
            }
        }
    }
    Ok(())
}
