use gtmpl::Func;
use serde::{Deserialize, Serialize};

#[derive(RustEmbed)]
#[folder = "scripts/"]
pub struct EmbeddedScripts;

#[derive(RustEmbed)]
#[folder = "yaml_structs/"]
pub struct EmbeddedYamlStructs;

#[derive(RustEmbed)]
#[folder = "templates/_global/"]
pub struct EmbeddedGlobalTemplates;

#[derive(RustEmbed)]
#[folder = "templates/config_map/"]
pub struct EmbeddedConfigMapTemplates;

#[derive(RustEmbed)]
#[folder = "templates/deployment/"]
pub struct EmbeddedDeploymentTemplates;

#[derive(RustEmbed)]
#[folder = "templates/job/"]
pub struct EmbeddedJobTemplates;

#[derive(RustEmbed)]
#[folder = "templates/service/"]
pub struct EmbeddedServiceTemplates;

#[derive(RustEmbed)]
#[folder = "templates/stateful_set/"]
pub struct EmbeddedStatefulSetTemplates;

// Chart
#[derive(Gtmpl, Clone)]
pub struct Chart {
    pub name: String,
    pub cleaned_name: Func,
    pub version: String,
    pub release_name: String,
    pub cleaned_release_name: Func,
    pub release_service: String,
    pub cluster: Cluster,
}

// Cluster
#[derive(Gtmpl, Clone)]
pub struct Cluster {
    pub name: String,
    pub cleaned_name: Func,
    pub values: Values,
    pub scripts: Vec<ClusterScript>,
}

// Scripts
#[derive(Gtmpl, Clone)]
pub struct ClusterScript {
    pub name: String,
    pub content: String,
}

// Global
#[derive(Gtmpl, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Values {
    pub image: Image,
    pub envs: String,
    pub extra_labels: Vec<ExtraLabels>,
    pub homedir: String,
    pub override_envs: String,
    pub config: Config,
    pub wal: Wal,
    pub security: Security,
    pub service: Service,
    pub on_load: OnLoad,
    pub gtm: Gtm,
    pub proxies: Proxy,
    pub coordinators: Coordinator,
    pub datanodes: Datanode,
}

// Image
#[derive(Gtmpl, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Image {
    pub name: String,
    pub version: String,
}

// Extra Labels
#[derive(Gtmpl, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ExtraLabels {
    pub name: String,
    pub content: String,
}

// Config
#[derive(Gtmpl, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Config {
    log_level: String,
    managers_port: u16,
    postgres_port: u16,
    postgres_user: String,
    append: ConfigAppend,
}

#[derive(Gtmpl, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ConfigAppend {
    gtm: String,
    proxy: String,
    datanode: String,
    coordinator: String,
}

// WAL
#[derive(Gtmpl, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Wal {
    archive: WalArchive,
}

#[derive(Gtmpl, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WalArchive {
    enable: bool,
    version: String,
    storage_path: String,
    pvc: String,
}

// Security
#[derive(Gtmpl, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Security {
    passwords_secret_name: String,
    pg_password: String,
    postgres_auth_type: String,
}

// Service
#[derive(Gtmpl, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Service {
    enabled: bool,
    port: u16,
    service_type: String,
}

// On Load
#[derive(Gtmpl, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OnLoad {
    enabled: bool,
    back_off_limit: u8,
    resources: OnLoadResource,
    startup: Vec<OnLoadStartup>,
    init: Vec<OnLoadInit>,
}

#[derive(Gtmpl, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OnLoadResource {
    limits: OnLoadResourceLimit,
}

#[derive(Gtmpl, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OnLoadResourceLimit {
    memory: String,
    cpu: String,
}

#[derive(Gtmpl, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OnLoadStartup {
    name: String,
    content: String,
}

#[derive(Gtmpl, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OnLoadInit {
    name: String,
    content: String,
}

// GTM
#[derive(Gtmpl, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Gtm {
    resources: GtmResource,
    pvc: GtmPvcResource,
    add_containers: String,
    volumes: String,
    volume_mounts: String,
    inject_main_container_yaml: String,
    inject_dep_yaml: String,
    inject_sts_yaml: String,
}

#[derive(Gtmpl, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GtmResource {
    limits: GtmResourceRequestLimit,
}

#[derive(Gtmpl, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GtmResourceRequestLimit {
    memory: String,
    cpu: String,
}

#[derive(Gtmpl, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GtmPvcResource {
    resources: GtmPvcResourceResourceRequest,
}

#[derive(Gtmpl, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GtmPvcResourceResourceRequest {
    requests: GtmPvcResourceResourceRequestStorage,
}

#[derive(Gtmpl, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GtmPvcResourceResourceRequestStorage {
    storage: String,
}

// GTM Proxies
#[derive(Gtmpl, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Proxy {
    enabled: bool,
    count: u8,
    thread_count: u8,
    resources: ProxyResource,
    add_containers: String,
    volumes: String,
    volume_mounts: String,
    inject_main_container_yaml: String,
    inject_spec_yaml: String,
    inject_sts_yaml: String,
}

#[derive(Gtmpl, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProxyResource {
    limits: ProxyResourceLimit,
}

#[derive(Gtmpl, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProxyResourceLimit {
    memory: String,
    cpu: String,
}

// Coordinators
#[derive(Gtmpl, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Coordinator {
    count: u8,
    resources: CoordinatorResource,
    pvc: CoordinatorPvcResource,
    add_containers: String,
    volumes: String,
    volume_mounts: String,
    inject_main_container_yaml: String,
    inject_spec_yaml: String,
    inject_sts_yaml: String,
}

#[derive(Gtmpl, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CoordinatorResource {
    limits: CoordinatorResourceLimit,
}

#[derive(Gtmpl, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CoordinatorResourceLimit {
    memory: String,
    cpu: String,
}

#[derive(Gtmpl, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CoordinatorPvcResource {
    resources: CoordinatorPvcResourceResourceRequest,
}

#[derive(Gtmpl, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CoordinatorPvcResourceResourceRequest {
    requests: CoordinatorPvcResourceResourceRequestStorage,
}

#[derive(Gtmpl, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CoordinatorPvcResourceResourceRequestStorage {
    storage: String,
}

// Datanodes
#[derive(Gtmpl, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Datanode {
    count: u8,
    resources: DatanodeResource,
    pvc: DatanodePvcResource,
    add_containers: String,
    volumes: String,
    volume_mounts: String,
    add_volume_claims: String,
    inject_main_container_yaml: String,
    inject_spec_yaml: String,
    inject_sts_yaml: String,
}

#[derive(Gtmpl, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DatanodeResource {
    limits: DatanodeResourceLimit,
}

#[derive(Gtmpl, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DatanodeResourceLimit {
    memory: String,
    cpu: String,
}

#[derive(Gtmpl, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DatanodePvcResource {
    resources: DatanodePvcResourceResourceRequest,
}

#[derive(Gtmpl, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DatanodePvcResourceResourceRequest {
    requests: DatanodePvcResourceResourceRequestStorage,
}

#[derive(Gtmpl, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DatanodePvcResourceResourceRequestStorage {
    storage: String,
}
