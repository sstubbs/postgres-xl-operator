use super::custom_resources::KubePostgresXlCluster;
use super::functions::{create_context, get_kube_config};
use super::vars::{
    CLUSTER_RESOURCE_PLURAL, CUSTOM_RESOURCE_GROUP, NAMESPACE, PASSWORD_ROTATE_SCHEDULE,
};
use crate::controller_secret;
use crate::enums::ResourceAction;
use crate::functions::generate_password;
use crate::structs::GeneratedPassword;
use base64::encode;
use chrono::Utc;
use cron_parser::parse;
use diesel::{sql_query, Connection, PgConnection, RunQueryDsl};
use kube::api::{Api, ListParams};
use kube::client::APIClient;
use std::time::Duration;
use tokio::time;

pub async fn watch() -> anyhow::Result<()> {
    let schedule =
        std::env::var("PASSWORD_ROTATE_SCHEDULE").unwrap_or(PASSWORD_ROTATE_SCHEDULE.into());

    // Cluster and secret clients
    let config = get_kube_config().await?;
    let client = APIClient::new(config);
    let namespace = std::env::var("NAMESPACE").unwrap_or(NAMESPACE.into());
    let custom_resource_group =
        std::env::var("CUSTOM_RESOURCE_GROUP").unwrap_or(CUSTOM_RESOURCE_GROUP.into());
    let resource =
        &std::env::var("CLUSTER_RESOURCE_PLURAL").unwrap_or(CLUSTER_RESOURCE_PLURAL.into());
    let resource_client: Api<KubePostgresXlCluster> =
        Api::customResource(client.to_owned(), resource)
            .group(&custom_resource_group)
            .within(&namespace);
    let secret_client = Api::v1Secret(client).within(&namespace);

    let list_params = ListParams::default();

    loop {
        let next = parse(&schedule, &Utc::now());
        if next.is_ok() {
            let wait_seconds = &next.unwrap().timestamp() - &Utc::now().timestamp();
            time::delay_for(Duration::from_secs(wait_seconds as u64)).await;
        } else {
            time::delay_for(Duration::from_secs(300)).await;
            error!(
                "{}: error with health check schedule, using default of 5 minutes.",
                next.err().unwrap().to_string()
            )
        }

        // get clusters
        let clusters = resource_client.list(&list_params);

        for cluster in clusters.await?.iter() {
            let context = create_context(cluster, "".to_owned()).await?;
            // Check if operator is used for rotation
            if &context.cluster.values.security.password.method == "operator" {
                let mut user = &context.cluster.values.config.postgres_user.to_owned();
                let mut password = "".to_owned();

                let secret_name = format!(
                    "{}-{}-{}",
                    &context.cleaned_release_name,
                    &context.cluster.cleaned_name,
                    &context.cluster.values.security.password.secret_name
                );

                let secret = secret_client.get(&secret_name).await;

                if secret.is_ok() {
                    let secret_unwrapped = secret.unwrap();
                    let password_bytes = secret_unwrapped
                        .data
                        .get(&context.cluster.values.config.postgres_user)
                        .unwrap();
                    user = &context.cluster.values.config.postgres_user;
                    password = std::str::from_utf8(&password_bytes.0).unwrap().to_owned();
                }

                let service_name = format!(
                    "{}-{}-svc-crd",
                    context.cleaned_release_name, context.cluster.cleaned_name
                );

                let database_url = format!(
                    "postgres://{}:{}@{}:{}",
                    user, password, service_name, context.cluster.values.config.postgres_port
                );

                let database_connection = PgConnection::establish(&database_url);

                if database_connection.is_ok() {
                    let database_connection_unwrapped = database_connection.unwrap();
                    let mut generated_passwords: Vec<GeneratedPassword> = Vec::new();
                    for generated_password in context.cluster.generated_passwords {
                        let new_password = generate_password().await?;
                        // Update password in the database

                        let update_password = sql_query(format!(
                            "ALTER USER {} WITH PASSWORD '{}';",
                            generated_password.secret_key, new_password
                        ))
                        .execute(&database_connection_unwrapped);

                        if update_password.is_ok() {
                            info!(
                                "Successfully updated user password for {}.",
                                &generated_password.secret_key
                            )
                        } else {
                            error!(
                                "Could not update password for {}: {}",
                                &generated_password.secret_key,
                                update_password.err().unwrap().to_string()
                            )
                        }

                        generated_passwords.push(GeneratedPassword {
                            secret_key: generated_password.secret_key,
                            secret_value: encode(&new_password),
                        });
                    }

                    // Update password secrets
                    controller_secret::action(
                        &cluster,
                        &ResourceAction::Modified,
                        "".to_owned(),
                        generated_passwords,
                    )
                    .await?;
                }
            }
        }
    }
}
