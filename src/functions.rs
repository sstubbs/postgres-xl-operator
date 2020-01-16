use super::structs;
use sprig::SPRIG;

pub async fn create_resource_object(
    context_unwrapped: &structs::Chart,
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

    // Convert new template into serde object to post
    let new_resource_object: serde_yaml::Value = serde_yaml::from_str(&new_resource_yaml)?;

    return Ok(new_resource_object);
}
