use super::{
    custom_resources::KubePostgresXlCluster,
    structs::{
        Chart, Cluster, ClusterScript, EmbeddedGlobalTemplates, EmbeddedScripts, EmbeddedYamlStructs,
        Values,
    },
    vars::{CHART_NAME, CHART_VERSION, RELEASE_NAME, RELEASE_SERVICE},
};
use gtmpl::Value;
use json_patch::merge;
use sprig::SPRIG;

pub async fn create_context(custom_resource: &KubePostgresXlCluster) -> anyhow::Result<Chart> {
    // Get the yaml strings
    let yaml_struct_file = EmbeddedYamlStructs::get("postgres-xl-cluster.yaml").unwrap();
    let yaml_struct_string = std::str::from_utf8(yaml_struct_file.as_ref())?;
    // Convert template into serde values
    let mut yaml_struct_object_unwrapped = serde_yaml::from_str(&yaml_struct_string)?;

    if custom_resource.spec.data.is_some() {
        let yaml_added_string = &custom_resource.spec.data.clone().unwrap();

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
        // Create cluster object
        let yaml_struct_merged_object_unwrapped: Values = yaml_struct_merged_object?;

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

        for asset in EmbeddedScripts::iter() {
            let filename = asset.as_ref();
            if !filename.starts_with(".") {
                let file_data = EmbeddedScripts::get(filename).unwrap();
                let file_data_string = std::str::from_utf8(file_data.as_ref())?;

                let script_object = ClusterScript {
                    name: filename.to_owned(),
                    content: file_data_string.to_owned(),
                };
                scripts.push(script_object);
            }
        }

        // Global context
        let global_context = Chart {
            name: std::env::var("CHART_NAME").unwrap_or(CHART_NAME.into()),
            cleaned_name,
            version: std::env::var("CHART_VERSION").unwrap_or(CHART_VERSION.into()),
            release_name: std::env::var("RELEASE_NAME").unwrap_or(RELEASE_NAME.into()),
            cleaned_release_name,
            release_service: std::env::var("RELEASE_SERVICE").unwrap_or(RELEASE_SERVICE.into()),
            cluster: Cluster {
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
    let mut main_template = global_template.clone();
    main_template.push_str(&file_data_string);

    // Render template with gotmpl
    let mut tmpl = gtmpl::Template::default();
    tmpl.add_funcs(SPRIG as &[(&str, gtmpl::Func)]);
    tmpl.parse(&main_template).unwrap();
    let context = gtmpl::Context::from(context_unwrapped.to_owned()).unwrap();
    let new_resource_yaml = tmpl.render(&context).unwrap();

    debug!("{}", new_resource_yaml);

    // Convert new template into serde object to post
    let new_resource_object: serde_yaml::Value = serde_yaml::from_str(&new_resource_yaml)?;
    return Ok(new_resource_object);
}
