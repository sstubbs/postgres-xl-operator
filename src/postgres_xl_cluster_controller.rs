use futures::StreamExt;
use gtmpl::Value;
use json_patch::merge;
use kube::{
    api::{Informer, Object, Api, RawApi, Void, WatchEvent, PostParams},
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

pub async fn handle_events(ev: WatchEvent<KubeCustomResource>) -> anyhow::Result<()> {
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

                // Global template
                let global_context = structs::Chart {
                    name: std::env::var("CHART_NAME").unwrap_or("postgres-xl-operator-chart".into()),
                    cleaned_name,
                    version: std::env::var("CHART_VERSION").unwrap_or("0.0.1".into()),
                    release_name: std::env::var("RELEASE_NAME").unwrap_or("postgres-xl-operator".into()),
                    cleaned_release_name,
                    release_service: std::env::var("RELEASE_SERVICE").unwrap_or("helm".into()),
                    cluster: structs::Cluster {
                        name: name.to_owned(),
                        cleaned_name,
                        values: main_object,
                        scripts,
                    }
                };
                let mut global_template = "".to_owned();

                for asset in structs::EmbeddedGlobalTemplates::iter() {
                    let filename = asset.as_ref();
                    let file_data = structs::EmbeddedGlobalTemplates::get(filename).unwrap();
                    let file_data_string = std::str::from_utf8(file_data.as_ref())?;
                    global_template.push_str(&file_data_string);
                }

                let config = config::load_kube_config().await?;
                let client = APIClient::new(config);
                let namespace = std::env::var("NAMESPACE").unwrap_or("pgxl".into());
                let config_maps = Api::v1ConfigMap(client).within(&namespace);

                // Config map templates
                for asset in structs::EmbeddedConfigMapTemplates::iter() {
                    let main_context = global_context.clone();
                    let mut main_template = global_template.clone();
                    let filename = asset.as_ref();
                    let file_data = structs::EmbeddedConfigMapTemplates::get(&filename).unwrap();
                    let file_data_string = std::str::from_utf8(file_data.as_ref())?;
                    main_template.push_str(&file_data_string);

                    // Render template with gotmpl
                    let mut tmpl = gtmpl::Template::default();
                    tmpl.add_funcs(SPRIG as &[(&str, gtmpl::Func)]);
                    tmpl
                        .parse(&main_template)
                        .unwrap();
                    let context = gtmpl::Context::from(main_context).unwrap();
                    let new_resource_yaml = tmpl.render(&context).unwrap();

                    // Convert new template into serde object to post
                    let new_resource_object: serde_yaml::Value = serde_yaml::from_str(&new_resource_yaml)?;

                    // Create new resources
                    let pp = PostParams::default();

                    let resource_name = &new_resource_object["metadata"]["name"].as_str().unwrap();

                    match config_maps.replace(resource_name, &pp, serde_json::to_vec(&new_resource_object)?).await {
                        Ok(_o) => {
                            println!("config map created");
//                        assert_eq!(p["metadata"]["name"], o.metadata.name);
//                        info!("Created {}", o.metadata.name);
                            // wait for it..
//                        std::thread::sleep(std::time::Duration::from_millis(5_000));
                        }
                        Err(kube::Error::Api(ae)) => assert_eq!(ae.code, 409), // if you skipped delete, for instance
                        Err(e) => return Err(e.into()),                        // any other case is probably bad
                    }

                }


            } else {
                println!("There was an error rendering the template.");
            }
        }
        _ => println!("another event"),
    }
    Ok(())
}
