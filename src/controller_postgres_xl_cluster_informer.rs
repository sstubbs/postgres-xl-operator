use super::structs;
use super::vars;
use futures::StreamExt;
use gtmpl::Value;
use json_patch::merge;
use kube::{
    api::{Informer, Object, RawApi, Void, WatchEvent},
    client::APIClient,
    config,
};
use serde::{Deserialize, Serialize};
use serde_yaml;

pub async fn watch() -> anyhow::Result<()> {
    let config = config::load_kube_config().await?;
    let client = APIClient::new(config);
    let namespace = std::env::var("NAMESPACE").unwrap_or(vars::NAMESPACE.into());
    let custom_resource_group =
        std::env::var("CUSTOM_RESOURCE_GROUP").unwrap_or(vars::CUSTOM_RESOURCE_GROUP.into());

    let resource = RawApi::customResource("postgres-xl-clusters")
        .group(&custom_resource_group)
        .within(&namespace);

    let ei = Informer::raw(client, resource).init().await?;

    loop {
        let mut events = ei.poll().await?.boxed();

        while let Some(event) = events.next().await {
            let event = event?;
            handle_events(event).await?;
        }
    }
}

// This is our new CustomResource struct
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CustomResource {
    pub data: String,
}

type KubeCustomResource = Object<CustomResource, Void>;

async fn create_context(
    custom_resource: &Object<CustomResource, Void>,
) -> anyhow::Result<structs::Chart> {
    // Get the yaml strings
    let yaml_struct_file = structs::EmbeddedYamlStructs::get("postgres-xl-cluster.yaml").unwrap();
    let yaml_struct_string = std::str::from_utf8(yaml_struct_file.as_ref())?;
    let yaml_added_string = &custom_resource.spec.data;

    // Convert them into serde values
    let mut yaml_struct_object_unwrapped = serde_yaml::from_str(&yaml_struct_string)?;
    let yaml_added_object = serde_yaml::from_str(&yaml_added_string);

    if yaml_added_object.is_ok() {
        // Merge them
        let yaml_added_object_unwrapped = yaml_added_object?;
        merge(
            &mut yaml_struct_object_unwrapped,
            &yaml_added_object_unwrapped,
        );

        // Convert into struct
        let yaml_struct_merged_string_unwrapped =
            serde_json::to_string(&yaml_struct_object_unwrapped)?;
        let yaml_struct_merged_object = serde_yaml::from_str(&yaml_struct_merged_string_unwrapped);

        if yaml_struct_merged_object.is_ok() {
            // Create cluster object
            let yaml_struct_merged_object_unwrapped: structs::Values = yaml_struct_merged_object?;

            let name = &custom_resource.metadata.name;
            // Create a default fully qualified kubernetes name, with max 50 chars,
            // thus allowing for 13 chars of internal naming.
            fn cleaned_name(args: &[Value]) -> Result<Value, String> {
                if let Value::Object(ref o) = &args[0] {
                    if let Some(Value::String(ref n)) = o.get("name") {
                        let mut name = n.to_owned();
                        name.truncate(45);
                        let re = regex::Regex::new(r"[^a-z0-9]+").unwrap();
                        let result = re.replace_all(&name, "-");
                        return Ok(result.into());
                    }
                }
                Err("Failed cleaning name".to_owned())
            }

            // Create a default fully qualified kubernetes name, with max 50 chars,
            // thus allowing for 13 chars of internal naming.
            fn cleaned_release_name(args: &[Value]) -> Result<Value, String> {
                if let Value::Object(ref o) = &args[0] {
                    if let Some(Value::String(ref n)) = o.get("release_name") {
                        let mut name = n.to_owned();
                        name.truncate(45);
                        let re = regex::Regex::new(r"[^a-z0-9]+").unwrap();
                        let result = re.replace_all(&name, "-");
                        return Ok(result.into());
                    }
                }
                Err("Failed cleaning name".to_owned())
            }

            // Load scripts dir
            let mut scripts = Vec::new();

            for asset in structs::EmbeddedScripts::iter() {
                let filename = asset.as_ref();
                if !filename.starts_with(".") {
                    let file_data = structs::EmbeddedScripts::get(filename).unwrap();
                    let file_data_string = std::str::from_utf8(file_data.as_ref())?;

                    let script_object = structs::ClusterScript {
                        name: filename.to_owned(),
                        script: file_data_string.to_owned(),
                    };
                    scripts.push(script_object);
                }
            }

            // Global context
            let global_context = structs::Chart {
                name: std::env::var("CHART_NAME").unwrap_or(vars::CHART_NAME.into()),
                cleaned_name,
                version: std::env::var("CHART_VERSION").unwrap_or(vars::CHART_VERSION.into()),
                release_name: std::env::var("RELEASE_NAME").unwrap_or(vars::RELEASE_NAME.into()),
                cleaned_release_name,
                release_service: std::env::var("RELEASE_SERVICE")
                    .unwrap_or(vars::RELEASE_SERVICE.into()),
                cluster: structs::Cluster {
                    name: name.to_owned(),
                    cleaned_name,
                    values: yaml_struct_merged_object_unwrapped,
                    scripts,
                },
            };
            return Ok(global_context);
        }
        return Err(anyhow!(
            "Error in resource and struct template merge: \n {}",
            yaml_struct_merged_object.err().unwrap()
        ));
    }
    return Err(anyhow!(
        "Error in resource data: \n {}",
        yaml_added_object.err().unwrap()
    ));
}

async fn create_global_template() -> anyhow::Result<String> {
    let mut global_template = "".to_owned();
    for asset in structs::EmbeddedGlobalTemplates::iter() {
        let filename = asset.as_ref();
        if !filename.starts_with(".") {
            let file_data = structs::EmbeddedGlobalTemplates::get(filename).unwrap();
            let file_data_string = std::str::from_utf8(file_data.as_ref())?;
            global_template.push_str(&file_data_string);
        }
    }
    return Ok(global_template);
}

pub async fn handle_events(ev: WatchEvent<KubeCustomResource>) -> anyhow::Result<()> {
    match ev {
        WatchEvent::Added(custom_resource) => {
            let context = create_context(&custom_resource).await;

            if context.is_ok() {
                let context_unwrapped = context?;
                let global_template = create_global_template().await?;

                super::controller_config_map::create(context_unwrapped, global_template).await?;
            } else {
                error!("{}", context.err().unwrap())
            }
        }
        _ => info!("no controller created for event."),
    }
    Ok(())
}
