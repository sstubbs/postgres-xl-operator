use super::custom_resources::KubePostgresXlCluster;
use super::functions::{create_context, get_kube_config};
use super::schema::ping::dsl::*;
use super::vars::{
    CLUSTER_RESOURCE_PLURAL, CUSTOM_RESOURCE_GROUP, HEALTH_CHECK_SCHEDULE, NAMESPACE,
    PASSWORD_ROTATE_SCHEDULE,
};
use chrono::{DateTime, NaiveDateTime, Utc};
use cron_parser::parse;
use diesel::{insert_into, Connection, PgConnection, RunQueryDsl};
use kube::api::{Api, ListParams, PatchParams};
use kube::client::APIClient;
use std::time::Duration;
use tokio::time;

pub async fn watch() -> anyhow::Result<()> {
    let schedule = std::env::var("HEALTH_CHECK_SCHEDULE").unwrap_or(HEALTH_CHECK_SCHEDULE.into());

    let rotate_schedule =
        std::env::var("PASSWORD_ROTATE_SCHEDULE").unwrap_or(PASSWORD_ROTATE_SCHEDULE.into());

    embed_migrations!("migrations");

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
        let next_health_check = parse(&schedule, &Utc::now());
        let next_rotation = parse(&rotate_schedule, &Utc::now());
        if next_health_check.is_ok() && next_rotation.is_ok() {
            let next_health_check_timestamp = &next_health_check.unwrap().timestamp();
            let next_rotation_timestamp = &next_rotation.unwrap().timestamp();
            if &next_health_check_timestamp == &next_rotation_timestamp {
                // Increment by one so we can find the next interval as we don't want it to run at the same time as rotation
                let incremented_health_check_timestamp = next_health_check_timestamp + 1;
                // Create a NaiveDateTime from the timestamp
                let naive = NaiveDateTime::from_timestamp(incremented_health_check_timestamp, 0);
                // Create a normal DateTime from the NaiveDateTime
                let datetime: DateTime<Utc> = DateTime::from_utc(naive, Utc);
                // crete a new next interval object
                let incremented_health_check = parse(&schedule, &datetime);
                if incremented_health_check.is_ok() {
                    info!("Skipping next health check to allow password rotation.");
                    let wait_seconds =
                        &incremented_health_check.unwrap().timestamp() - &Utc::now().timestamp();
                    time::delay_for(Duration::from_secs(wait_seconds as u64)).await;
                } else {
                    error!(
                        "{}: error with health check schedule, using default of 5 minutes.",
                        incremented_health_check.err().unwrap().to_string()
                    );
                    time::delay_for(Duration::from_secs(300)).await;
                }
            } else {
                let wait_seconds =
                    &next_health_check_timestamp.to_owned() - &Utc::now().timestamp().to_owned();
                time::delay_for(Duration::from_secs(wait_seconds as u64)).await;
            }
        } else {
            error!(
                "{}: error with health check schedule, using default of 5 minutes.",
                next_health_check.err().unwrap().to_string()
            );
            time::delay_for(Duration::from_secs(300)).await;
        }

        // get clusters
        let clusters = resource_client.list(&list_params);

        for cluster in clusters.await?.iter() {
            let context = create_context(cluster, "".to_owned()).await?;

            // Check if health check is enabled
            if context.cluster.values.health_check.enabled
                && context.cluster.values.config.database != ""
            {
                // If secret is being used get the password for the database_url
                let mut user = &context.cluster.values.config.postgres_user.to_owned();
                let mut password = "".to_owned();

                if &context.cluster.values.security.password.method == "operator"
                    || &context.cluster.values.security.password.method == "mount"
                {
                    let mut secret_name = "".to_owned();
                    if &context.cluster.values.security.password.method == "operator" {
                        secret_name = format!(
                            "{}-{}-{}",
                            &context.cleaned_release_name,
                            &context.cluster.cleaned_name,
                            &context.cluster.values.security.password.secret_name
                        )
                    } else if &context.cluster.values.security.password.method == "mount" {
                        secret_name = context.cluster.values.security.password.secret_name;
                    }

                    let secret = secret_client.get(&secret_name).await;
                    if secret.is_ok() {
                        let secret_unwrapped = secret.unwrap();
                        let password_bytes = secret_unwrapped
                            .data
                            .get(&context.cluster.values.health_check.user)
                            .unwrap();
                        user = &context.cluster.values.health_check.user;
                        password = std::str::from_utf8(&password_bytes.0).unwrap().to_owned();
                    }
                }

                let service_name = format!(
                    "{}-{}-svc-crd",
                    context.cleaned_release_name, context.cluster.cleaned_name
                );

                let database_url = format!(
                    "postgres://{}:{}@{}:{}",
                    user, password, service_name, context.cluster.values.config.postgres_port
                );

                let health_check_database_url = format!(
                    "{}/{}",
                    database_url, context.cluster.values.config.database
                );

                let patch_params = PatchParams::default();

                if !cluster.metadata.labels.contains_key("health_check") {
                    // Run health check database migrations
                    let health_check_database_connection =
                        PgConnection::establish(&health_check_database_url);

                    if health_check_database_connection.is_ok() {
                        let health_check_database_connection_unwrapped =
                            health_check_database_connection.unwrap();
                        embedded_migrations::run(&health_check_database_connection_unwrapped)?;

                        // set health_check label to initialized
                        let patch = json!({
                            "metadata": {
                                "labels": {
                                    "health_check": "initialized",
                                },
                            },
                        });

                        let _p_patched = resource_client
                            .patch(
                                &cluster.metadata.name,
                                &patch_params,
                                serde_json::to_vec(&patch)?,
                            )
                            .await?;
                    } else {
                        error!("{}", health_check_database_connection.err().unwrap())
                    }
                } else {
                    // Do the health check
                    let health_check_database_connection =
                        PgConnection::establish(&health_check_database_url);
                    let patch;
                    if health_check_database_connection.is_ok() {
                        let health_check_database_connection_unwrapped =
                            health_check_database_connection.unwrap();
                        let health_check = insert_into(ping)
                            .default_values()
                            .execute(&health_check_database_connection_unwrapped);

                        // patch the label depending on the result
                        if health_check.is_ok() {
                            // set health_check label to healthy
                            patch = json!({
                                "metadata": {
                                    "labels": {
                                        "health_check": "healthy",
                                    },
                                },
                            });
                        } else {
                            error!("{}", health_check.err().unwrap());
                            // set health_check label to unhealthy
                            patch = json!({
                                "metadata": {
                                    "labels": {
                                        "health_check": "unhealthy",
                                    },
                                },
                            });
                        }
                    } else {
                        error!("{}", health_check_database_connection.err().unwrap());
                        // set health_check label to unhealthy
                        patch = json!({
                            "metadata": {
                                "labels": {
                                    "health_check": "unhealthy",
                                },
                            },
                        });
                    }
                    resource_client
                        .patch(
                            &cluster.metadata.name,
                            &patch_params,
                            serde_json::to_vec(&patch)?,
                        )
                        .await?;
                }
            }
        }
    }
}
