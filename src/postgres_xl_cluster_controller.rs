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
use sprig::SPRIG;

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
            let yaml_struct_data = structs::EmbeddedYamlStructs::get("postgres-xl-cluster.yaml").unwrap();
            let yaml_struct_string = std::str::from_utf8(yaml_struct_data.as_ref())?;

            let yaml_added = &custom_resource.spec.data;

            // Convert them into serde values
            let mut json_template_object = serde_yaml::from_str(&yaml_struct_string)?;
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

                // Load scripts dir
                let mut scripts = Vec::new();

                for asset in structs::EmbeddedScripts::iter() {
                    let filename = asset.as_ref();
                    let file_data = structs::EmbeddedScripts::get(filename).unwrap();
                    let file_data_string = std::str::from_utf8(file_data.as_ref())?;

                    let script_object = structs::ClusterScript {name: filename.to_owned(), script: file_data_string.to_owned()};
                    scripts.push(script_object);
                }

                // Main context
                let main_object = final_merged_object.clone();
                let main_context = structs::Chart {
                    name: std::env::var("CHART_NAME").unwrap_or("postgres-xl-operator-chart".into()),
                    cleaned_name,
                    version: std::env::var("CHART_VERSION").unwrap_or("0.0.1".into()),
                    cluster: structs::Cluster {
                        name: name.to_owned(),
                        cleaned_name,
                        values: main_object,
                        scripts,
                    }
                };

                // Main template
                let mut main_template = "".to_owned();

                for asset in structs::EmbeddedTemplates::iter() {
                    let filename = asset.as_ref();
                    println!("{}", &filename);
                    let file_data = structs::EmbeddedTemplates::get(filename).unwrap();
                    let file_data_string = std::str::from_utf8(file_data.as_ref())?;
                    main_template.push_str(&file_data_string);
                }

                // Render template with gotmpl
                let mut tmpl = gtmpl::Template::default();
                tmpl.add_funcs(SPRIG as &[(&str, gtmpl::Func)]);
                tmpl
                    .parse(&main_template)
                    .unwrap();
                let context = gtmpl::Context::from(main_context).unwrap();
                let main_output = tmpl.render(&context);
                println!("{}", main_output.unwrap());
            } else {
                println!("There was an error rendering the template.");
            }
        }
        _ => println!("another event"),
    }
    Ok(())
}
