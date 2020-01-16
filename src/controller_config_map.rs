use super::structs;
use kube::{
    api::{Api, PostParams},
    client::APIClient,
    config,
};

pub async fn create(
    context_unwrapped: structs::Chart,
    global_template: String,
) -> anyhow::Result<()> {
    let config = config::load_kube_config().await?;
    let client = APIClient::new(config);
    let namespace = std::env::var("NAMESPACE").unwrap_or("pgxl".into());
    let config_maps = Api::v1ConfigMap(client).within(&namespace);

    for asset in structs::EmbeddedConfigMapTemplates::iter() {
        let filename = asset.as_ref();

        // Ignore hidden files
        if !filename.starts_with(".") {
            // Create new resources
            let file_data = structs::EmbeddedConfigMapTemplates::get(&filename).unwrap();
            let file_data_string = std::str::from_utf8(file_data.as_ref())?;
            let new_resource_object = super::functions::create_resource_object(
                &context_unwrapped,
                &global_template,
                &file_data_string.to_owned(),
            )
                .await?;
            let pp = PostParams::default();

            match config_maps
                .create(&pp, serde_json::to_vec(&new_resource_object)?)
                .await
                {
                    Ok(o) => {
                        assert_eq!(new_resource_object["metadata"]["name"], o.metadata.name);
                        println!("Created {}", o.metadata.name);
                    }
                    Err(kube::Error::Api(ae)) => assert_eq!(ae.code, 409), // if you skipped delete, for instance
                    Err(e) => return Err(e.into()),                        // any other case is probably bad
                }
        }

    }
    Ok(())
}
