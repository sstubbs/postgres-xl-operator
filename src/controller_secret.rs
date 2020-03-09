use super::{
    custom_resources::KubePostgresXlCluster,
    enums::ResourceAction,
    functions::{create_context, create_global_template, get_kube_config},
    structs::EmbeddedSecretTemplates,
    vars::NAMESPACE,
};
use base64::decode;
use diesel::{sql_query, Connection, PgConnection, RunQueryDsl};
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
        let context_unwrapped = context?.to_owned();
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

                            // Update root user password otherwise job won't be able to connect and alter other user passwords.
                            if &context_unwrapped.cluster.values.security.password.method
                                == "operator"
                                || &context_unwrapped.cluster.values.security.password.method
                                    == "mount"
                            {
                                if &new_resource_object_unwapped["metadata"]["name"]
                                    == &context_unwrapped
                                        .cluster
                                        .values
                                        .security
                                        .password
                                        .secret_name
                                    || &new_resource_object_unwapped["metadata"]["name"]
                                        == &format!(
                                            "{}-{}-{}",
                                            &context_unwrapped.cleaned_release_name,
                                            &context_unwrapped.cluster.cleaned_name,
                                            &context_unwrapped
                                                .cluster
                                                .values
                                                .security
                                                .password
                                                .secret_name
                                        )
                                {
                                    let password_bytes = old_resource
                                        .data
                                        .get(&context_unwrapped.cluster.values.config.postgres_user)
                                        .unwrap();
                                    let user =
                                        &context_unwrapped.cluster.values.config.postgres_user;
                                    let password =
                                        std::str::from_utf8(&password_bytes.0).unwrap().to_owned();
                                    let service_name = format!(
                                        "{}-{}-svc-crd",
                                        &context_unwrapped.cleaned_release_name,
                                        &context_unwrapped.cluster.cleaned_name
                                    );
                                    let database_url = format!(
                                        "postgres://{}:{}@{}:{}",
                                        &user,
                                        &password,
                                        &service_name,
                                        &context_unwrapped.cluster.values.config.postgres_port
                                    );
                                    let new_password_bytes = decode(
                                        &context_unwrapped
                                            .cluster
                                            .generated_passwords
                                            .iter()
                                            .filter(|&gpw| {
                                                &gpw.secret_key
                                                    == &context_unwrapped
                                                        .cluster
                                                        .values
                                                        .config
                                                        .postgres_user
                                            })
                                            .next()
                                            .unwrap()
                                            .secret_value,
                                    )
                                    .unwrap();
                                    let new_password =
                                        std::str::from_utf8(new_password_bytes.as_ref()).unwrap();
                                    let database_connection =
                                        PgConnection::establish(&database_url);

                                    if database_connection.is_ok() {
                                        let database_connection_unwrapped =
                                            database_connection.unwrap();

                                        let update_password = sql_query(format!(
                                            "ALTER USER {} WITH PASSWORD '{}';",
                                            user, new_password
                                        ))
                                        .execute(&database_connection_unwrapped);

                                        if update_password.is_ok() {
                                            info!(
                                                "Successfully updated user password for {}.",
                                                &context_unwrapped
                                                    .cluster
                                                    .values
                                                    .config
                                                    .postgres_user
                                            )
                                        } else {
                                            error!(
                                                "Could not update password for {}: {}",
                                                &context_unwrapped
                                                    .cluster
                                                    .values
                                                    .config
                                                    .postgres_user,
                                                update_password.err().unwrap().to_string()
                                            )
                                        }
                                    }
                                }
                            }
                            // End update root user password.

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
