use super::{
    custom_resources::KubePostgresXlCluster,
    structs::{
        Chart, Cluster, EmbeddedGlobalTemplates, EmbeddedOnloadScripts, EmbeddedScripts,
        EmbeddedYamlStructs, GlobalLabel, Script, SelectorLabel, Values,
    },
    vars::{CHART_NAME, CHART_VERSION, KUBE_CONFIG_TYPE, RELEASE_NAME, RELEASE_SERVICE},
};
use crate::structs::{GeneratedPassword, OnLoadStartup};
use base64::encode;
use json_patch::merge;
use kube::config;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use sprig::SPRIG;

pub async fn get_kube_config() -> anyhow::Result<config::Configuration> {
    let kube_config_type = std::env::var("KUBE_CONFIG_TYPE").unwrap_or(KUBE_CONFIG_TYPE.into());

    if kube_config_type == "kubeconfig" {
        let config = config::load_kube_config().await?;
        return Ok(config);
    } else {
        let config = config::incluster_config()?;
        return Ok(config);
    }
}

// Create a default fully qualified kubernetes name, with max 50 chars,
// thus allowing for 13 chars of internal naming.
pub fn clean_value(value: &String) -> anyhow::Result<String> {
    let mut new_value = value.to_owned();
    new_value.truncate(45);
    let re = regex::Regex::new(r"[^a-z0-9]+").unwrap();
    let result = re.replace_all(&new_value, "-");
    return Ok(result.into());
}

pub async fn create_context(
    custom_resource: &KubePostgresXlCluster,
    config_map_sha: String,
) -> anyhow::Result<Chart> {
    // Get the yaml strings
    let yaml_struct_file = EmbeddedYamlStructs::get("postgres-xl-cluster.yaml").unwrap();
    let yaml_struct_string = std::str::from_utf8(yaml_struct_file.as_ref())?;
    // Convert template into serde values
    let mut yaml_struct_object_unwrapped = serde_yaml::from_str(&yaml_struct_string)?;

    if custom_resource.spec.data.is_some() {
        let yaml_added_string = &custom_resource.spec.data.to_owned().unwrap();

        // Convert added into serde values
        let yaml_added_object = serde_yaml::from_str(&yaml_added_string);

        if yaml_added_object.is_ok() {
            // Merge them
            let yaml_added_object_unwrapped = yaml_added_object?;
            merge(
                &mut yaml_struct_object_unwrapped,
                &yaml_added_object_unwrapped,
            );
        }
    }

    // Convert into struct
    let yaml_struct_merged_string_unwrapped = serde_json::to_string(&yaml_struct_object_unwrapped)?;
    let yaml_struct_merged_object = serde_yaml::from_str(&yaml_struct_merged_string_unwrapped);

    if yaml_struct_merged_object.is_ok() {
        // Load scripts dir
        let mut scripts = Vec::new();

        for asset in EmbeddedScripts::iter() {
            let filename = asset.as_ref();
            if !filename.starts_with(".") {
                let file_data = EmbeddedScripts::get(filename).unwrap();
                let file_data_string = std::str::from_utf8(file_data.as_ref())?;

                let script_object = Script {
                    name: filename.to_owned().replace("/", "."),
                    content: file_data_string.to_owned(),
                };
                scripts.push(script_object);
            }
        }

        let chart_name = std::env::var("CHART_NAME").unwrap_or(CHART_NAME.into());
        let chart_cleaned_name = clean_value(&chart_name)?;
        let chart_version = std::env::var("CHART_VERSION").unwrap_or(CHART_VERSION.into());
        let release_name = std::env::var("RELEASE_NAME").unwrap_or(RELEASE_NAME.into());
        let cleaned_release_name = clean_value(&release_name)?;
        let release_service = std::env::var("RELEASE_SERVICE").unwrap_or(RELEASE_SERVICE.into());
        let cluster_name = &custom_resource.metadata.name;
        let cleaned_cluster_name = clean_value(&cluster_name)?;

        let yaml_struct_merged_object_unwrapped: Values = yaml_struct_merged_object?;

        // Global context
        let mut global_context = Chart {
            name: chart_name.to_owned(),
            cleaned_name: chart_cleaned_name,
            version: chart_version.to_owned(),
            release_name,
            cleaned_release_name: cleaned_release_name.to_owned(),
            release_service: release_service.to_owned(),
            cluster: Cluster {
                config_map_sha,
                global_labels: vec![
                    GlobalLabel {
                        name: "helm.sh/chart".to_owned(),
                        content: format!("{}-{}", chart_name, chart_version),
                    },
                    GlobalLabel {
                        name: "app.kubernetes.io/managed-by".to_owned(),
                        content: release_service,
                    },
                    GlobalLabel {
                        name: "app.kubernetes.io/version".to_owned(),
                        content: yaml_struct_merged_object_unwrapped.to_owned().image.version,
                    },
                ],
                generated_passwords: vec![],
                name: cluster_name.to_owned(),
                cleaned_name: cleaned_cluster_name.to_owned(),
                scripts,
                selector_labels: vec![
                    SelectorLabel {
                        name: "app.kubernetes.io/instance".to_owned(),
                        content: cleaned_release_name.to_owned(),
                    },
                    SelectorLabel {
                        name: "app.kubernetes.io/name".to_owned(),
                        content: cleaned_cluster_name,
                    },
                ],
                values: yaml_struct_merged_object_unwrapped,
            },
        };

        if global_context.cluster.values.security.password.method == "operator"
            || global_context.cluster.values.security.password.method == "mount"
        {
            // Overrides for operator secret generation
            global_context.cluster.values.on_load.enabled = true;

            if global_context.cluster.values.security.password.method == "operator" {
                // Generate root password
                global_context
                    .cluster
                    .generated_passwords
                    .push(GeneratedPassword {
                        secret_key: global_context
                            .to_owned()
                            .cluster
                            .values
                            .config
                            .postgres_user,
                        secret_value: generate_base64_password().await?,
                    });
            }

            // Run root scripts
            let filename = "create_update_user_passwords.sh";
            let file_data = EmbeddedOnloadScripts::get(&filename).unwrap();
            let file_data_string = std::str::from_utf8(file_data.as_ref())?;
            global_context
                .cluster
                .values
                .on_load
                .startup
                .push(OnLoadStartup {
                    name: filename.to_owned(),
                    content: file_data_string.to_owned(),
                });

            if global_context.cluster.values.security.password.method == "operator" {
                // Generate extra user passwords
                for user in global_context
                    .to_owned()
                    .cluster
                    .values
                    .security
                    .password
                    .extra_username
                {
                    global_context
                        .cluster
                        .generated_passwords
                        .push(GeneratedPassword {
                            secret_key: user,
                            secret_value: generate_base64_password().await?,
                        });
                }

                if global_context.cluster.values.connection_pool.enabled {
                    // Generate connection pool password
                    global_context
                        .cluster
                        .generated_passwords
                        .push(GeneratedPassword {
                            secret_key: global_context
                                .to_owned()
                                .cluster
                                .values
                                .connection_pool
                                .user,
                            secret_value: generate_base64_password().await?,
                        });
                }
            }

            if global_context.cluster.values.health_check.enabled {
                if global_context.cluster.values.security.password.method == "operator" {
                    // Generate health check password
                    global_context
                        .cluster
                        .generated_passwords
                        .push(GeneratedPassword {
                            secret_key: global_context.to_owned().cluster.values.health_check.user,
                            secret_value: generate_base64_password().await?,
                        });
                }
                // Run health check scripts
                let filename = "set_health_check_user_perms.sh";
                let file_data = EmbeddedOnloadScripts::get(&filename).unwrap();
                let file_data_string = std::str::from_utf8(file_data.as_ref())?;
                global_context
                    .cluster
                    .values
                    .on_load
                    .startup
                    .push(OnLoadStartup {
                        name: filename.to_owned(),
                        content: file_data_string.to_owned(),
                    });
            }
        } else if global_context.cluster.values.health_check.enabled {
            // Create database for health checks
            global_context.cluster.values.on_load.enabled = true;
        }

        return Ok(global_context);
    }
    return Err(anyhow!(
        "Error in resource and struct template merge: \n {}",
        yaml_struct_merged_object.err().unwrap()
    ));
}

pub async fn create_global_template() -> anyhow::Result<String> {
    let mut global_template = "".to_owned();
    for asset in EmbeddedGlobalTemplates::iter() {
        let filename = asset.as_ref();
        if !filename.starts_with(".") {
            let file_data = EmbeddedGlobalTemplates::get(filename).unwrap();
            let file_data_string = std::str::from_utf8(file_data.as_ref())?;
            global_template.push_str(&file_data_string);
        }
    }
    return Ok(global_template);
}

pub async fn create_resource_object(
    context_unwrapped: &Chart,
    global_template: &String,
    file_data_string: &String,
) -> anyhow::Result<serde_yaml::Value> {
    let mut main_template = global_template.to_owned();
    main_template.push_str(&file_data_string);

    // Render template with gotmpl
    let mut tmpl = gtmpl::Template::default();
    tmpl.add_funcs(SPRIG as &[(&str, gtmpl::Func)]);
    tmpl.parse(&main_template).unwrap();
    let context = gtmpl::Context::from(context_unwrapped.to_owned()).unwrap();
    let new_resource_yaml = tmpl.render(&context).unwrap();

    debug!("{}", new_resource_yaml);

    return if new_resource_yaml.contains("apiVersion") {
        // Convert new template into serde object to post
        let new_resource_object: serde_yaml::Value = serde_yaml::from_str(&new_resource_yaml)?;
        Ok(new_resource_object)
    } else {
        Err(anyhow!("Missing apiVersion in template so skipping"))
    };
}

pub async fn generate_base64_password() -> anyhow::Result<String> {
    let random_string: String = thread_rng().sample_iter(&Alphanumeric).take(30).collect();
    let encoded_random_string = encode(&random_string);
    return Ok(encoded_random_string);
}
