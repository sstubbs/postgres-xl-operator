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
use std::fs;

use super::structs;

pub async fn watch() -> anyhow::Result<()> {
    let config = config::load_kube_config().await?;
    let client = APIClient::new(config);

    let namespace = std::env::var("NAMESPACE").unwrap_or("pgxl".into());
    let custom_resource_group =
        std::env::var("CUSTOM_RESOURCE_GROUP").unwrap_or("postgres-xl-operator.vanqor.com".into());

    let resource = RawApi::customResource("postgres-xl-clusters")
        .group(&custom_resource_group)
        .within(&namespace);

    let ei = Informer::raw(client, resource).init().await?;

    loop {
        let mut events = ei.poll().await?.boxed();

        while let Some(event) = events.next().await {
            let event = event?;
            handle_events(event)?;
        }
    }
}

// This is our new CustomResource struct
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CustomResource {
    pub data: String,
}

type KubeCustomResource = Object<CustomResource, Void>;

pub fn handle_events(ev: WatchEvent<KubeCustomResource>) -> anyhow::Result<()> {
    match ev {
        WatchEvent::Added(custom_resource) => {
            // Get the yaml strings
            let yaml_template =
                fs::read_to_string("./yaml_defaults/setup-postgres-xl-cluster.yaml")?;
            let yaml_added = &custom_resource.spec.data;

            // Convert them into serde values
            let mut json_template_object = serde_yaml::from_str(&yaml_template)?;
            let json_added_template = serde_yaml::from_str(&yaml_added)?;

            // Merge them
            merge(&mut json_template_object, &json_added_template);

            // Convert into struct
            let merged_yaml = serde_json::to_string(&json_template_object)?;
            let merged_object = serde_yaml::from_str(&merged_yaml);

            if merged_object.is_ok() {
                // Create cluster object
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

                let final_merged_object: structs::Values = merged_object?;

                // Main template
                let main_object = final_merged_object.clone();
                let main_context = structs::Cluster {
                    name: name.to_owned(),
                    values: main_object,
                    cleaned_name,
                };

                let main_template =
                    fs::read_to_string("./templates/main.tpl")?;

                let main_output = gtmpl::template(
                    &main_template,
                    main_context,
                );
                println!("{}", main_output.unwrap());
            } else {
                println!("aaaaaa");
            }
        }
        _ => println!("another event"),
    }
    Ok(())
}
