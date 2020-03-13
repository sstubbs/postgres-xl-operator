use super::{
    custom_resources::KubePostgresXlCluster,
    enums::ResourceAction,
    functions::{create_context, create_global_template, get_kube_config},
    structs::EmbeddedSecretTemplates,
    vars::NAMESPACE,
};
use crate::structs::GeneratedPassword;
use base64::encode;
use kube::{
    api::{Api, DeleteParams, PostParams},
    client::APIClient,
};

pub async fn action(
    custom_resource: &KubePostgresXlCluster,
    resource_action: &ResourceAction,
    config_map_sha: String,
    generated_passwords: Vec<GeneratedPassword>,
) -> anyhow::Result<()> {
    let context = create_context(&custom_resource, config_map_sha).await;

    if context.is_ok() {
        let mut context_unwrapped = context?.to_owned();
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

                // if operator is being used and the pgpass template is being used
                match resource_action {
                    ResourceAction::Modified => {
                        if &context_unwrapped.cluster.values.security.password.method == "operator"
                        {
                            if filename == "pgpass.tpl" {
                                let secret_name = format!(
                                    "{}-{}-{}",
                                    &context_unwrapped.cleaned_release_name,
                                    &context_unwrapped.cluster.cleaned_name,
                                    &context_unwrapped
                                        .cluster
                                        .values
                                        .security
                                        .password
                                        .secret_name
                                );

                                if generated_passwords.is_empty() {
                                    // If not a password rotation use the old or current value
                                    let old_resource = resource_client.get(&secret_name).await;

                                    if old_resource.is_ok() {
                                        let old_resource_unwrapped = old_resource.unwrap();

                                        let mut updated_passwords = Vec::new();

                                        for new_password in
                                            &context_unwrapped.cluster.generated_passwords
                                        {
                                            let old_password = old_resource_unwrapped
                                                .data
                                                .get(&new_password.secret_key);
                                            if old_password.is_some() {
                                                let old_password_unwrapped = old_password.unwrap();
                                                let old_password_value =
                                                    std::str::from_utf8(&old_password_unwrapped.0)
                                                        .unwrap()
                                                        .to_owned();
                                                updated_passwords.push(GeneratedPassword {
                                                    secret_key: new_password.secret_key.to_owned(),
                                                    secret_value: encode(&old_password_value),
                                                })
                                            } else {
                                                updated_passwords.push(GeneratedPassword {
                                                    secret_key: new_password.secret_key.to_owned(),
                                                    secret_value: new_password
                                                        .secret_value
                                                        .to_owned(),
                                                })
                                            }
                                        }

                                        context_unwrapped.cluster.generated_passwords =
                                            updated_passwords;
                                    }
                                } else {
                                    // Otherwise use the rotated passwords
                                    context_unwrapped.cluster.generated_passwords =
                                        generated_passwords.to_owned();
                                }
                            }
                        }
                    }
                    _ => {}
                }

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
