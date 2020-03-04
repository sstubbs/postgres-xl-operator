use super::custom_resources::KubePostgresXlCluster;
use super::functions::{create_context, get_kube_config};
use super::schema::ping::dsl::*;
use super::vars::{
    CLUSTER_RESOURCE_PLURAL, CUSTOM_RESOURCE_GROUP, HEALTH_CHECK_INTERVAL, NAMESPACE,
};
use diesel::{insert_into, sql_query, Connection, PgConnection, RunQueryDsl};
use kube::api::{Api, ListParams, PatchParams};
use kube::client::APIClient;
use std::time::Duration;
use tokio::time;

pub async fn watch() -> anyhow::Result<()> {
    let interval: u64 = std::env::var("HEALTH_CHECK_INTERVAL")
        .unwrap_or(HEALTH_CHECK_INTERVAL.into())
        .parse::<u64>()
        .unwrap();
    let mut interval_duration = time::interval(Duration::from_secs(interval));

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
        interval_duration.tick().await;

        // get clusters
        let clusters = resource_client.list(&list_params);

        for cluster in clusters.await?.iter() {
            let context = create_context(cluster, "".to_owned()).await?;

            // Check if health check is enabled
            if context.cluster.values.health_check.enabled
                && context.cluster.values.health_check.database != ""
            {
                // If secret is being used get the password for tge database_url
                let mut password = "".to_owned();

                if &context.cluster.values.security.password.secret_name != ""
                    && &context.cluster.values.health_check.user != ""
                {
                    let secret = secret_client
                        .get(&context.cluster.values.security.password.secret_name)
                        .await;
                    if secret.is_ok() {
                        let secret_unwrapped = secret.unwrap();
                        let password_bytes = secret_unwrapped
                            .data
                            .get(&context.cluster.values.health_check.user)
                            .unwrap();
                        password = std::str::from_utf8(&password_bytes.0).unwrap().to_owned();
                    }
                }

                let service_name = format!(
                    "{}-{}-svc-crd",
                    context.cleaned_release_name, context.cluster.cleaned_name
                );

                let database_url = format!(
                    "postgres://postgres:{}@{}:{}",
                    password, service_name, context.cluster.values.config.postgres_port
                );

                let health_check_database_url = format!(
                    "{}/{}",
                    database_url, context.cluster.values.health_check.database
                );

                let patch_params = PatchParams::default();

                if !cluster.metadata.labels.contains_key("health_check") {
                    // Create health check database
                    let database_connection = PgConnection::establish(&database_url);
                    if database_connection.is_ok() {
                        let database_connection_unwrapped = database_connection.unwrap();
                        let create_health_check_database = sql_query(format!(
                            "CREATE DATABASE {}",
                            context.cluster.values.health_check.database
                        ))
                        .execute(&database_connection_unwrapped);
                        if create_health_check_database.is_ok() {
                            info!(
                                "database {} created",
                                context.cluster.values.health_check.database
                            )
                        } else {
                            error!("{}", create_health_check_database.err().unwrap())
                        }
                    } else {
                        error!("{}", database_connection.err().unwrap())
                    }

                    // Run health check database migrations
                    let health_check_database_connection =
                        PgConnection::establish(&health_check_database_url);

                    if health_check_database_connection.is_ok() {
                        let health_check_database_connection_unwrapped =
                            health_check_database_connection.unwrap();
                        embedded_migrations::run_with_output(
                            &health_check_database_connection_unwrapped,
                            &mut std::io::stdout(),
                        )?;

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
