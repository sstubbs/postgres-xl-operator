use serde::{Deserialize, Serialize};

#[derive(RustEmbed)]
#[folder = "scripts/"]
pub struct EmbeddedScripts;

#[derive(RustEmbed)]
#[folder = "onload_scripts/"]
pub struct EmbeddedOnloadScripts;

#[derive(RustEmbed)]
#[folder = "yaml_structs/"]
pub struct EmbeddedYamlStructs;

#[derive(RustEmbed)]
#[folder = "templates/1_global/"]
pub struct EmbeddedGlobalTemplates;

#[derive(RustEmbed)]
#[folder = "templates/2_secret/"]
pub struct EmbeddedSecretTemplates;

#[derive(RustEmbed)]
#[folder = "templates/3_config_map/"]
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
#[derive(Debug, Gtmpl, Clone, PartialEq, Serialize, Deserialize)]
pub struct Chart {
    pub cluster: Cluster,
    pub name: String,
    pub cleaned_name: String,
    pub release_name: String,
    pub cleaned_release_name: String,
    pub release_service: String,
    pub version: String,
}

// Cluster
#[derive(Debug, Gtmpl, Clone, PartialEq, Serialize, Deserialize)]
pub struct Cluster {
    pub config_map_sha: String,
    pub global_labels: Vec<GlobalLabel>,
    pub generated_passwords: Vec<GeneratedPassword>,
    pub name: String,
    pub cleaned_name: String,
    pub scripts: Vec<Script>,
    pub selector_labels: Vec<SelectorLabel>,
    pub values: Values,
}

// Generated Passwords
#[derive(Debug, Gtmpl, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeneratedPassword {
    pub secret_key: String,
    pub secret_value: String,
}

// Labels
#[derive(Debug, Gtmpl, Clone, PartialEq, Serialize, Deserialize)]
pub struct GlobalLabel {
    pub name: String,
    pub content: String,
}

// Selector Labels
#[derive(Debug, Gtmpl, Clone, PartialEq, Serialize, Deserialize)]
pub struct SelectorLabel {
    pub name: String,
    pub content: String,
}

// Scripts
#[derive(Debug, Gtmpl, Clone, PartialEq, Serialize, Deserialize)]
pub struct Script {
    pub name: String,
    pub content: String,
}

// Global
#[derive(Debug, Gtmpl, Clone, PartialEq, Serialize, Deserialize)]
pub struct Values {
    pub config: Config,
    pub connection_pool: ConnectionPool,
    pub envs: Vec<Envs>,
    pub extra_labels: Vec<ExtraLabels>,
    pub homedir: String,
    pub health_check: HealthCheck,
    pub image: Image,
    pub on_load: OnLoad,
    pub override_envs: Vec<OverrideEnvs>,
    pub replication: Replication,
    pub security: Security,
    pub service: Service,
    pub gtm: Gtm,
    pub proxies: Proxy,
    pub coordinators: Coordinator,
    pub datanodes: Datanode,
}

// Config
#[derive(Debug, Gtmpl, Clone, PartialEq, Serialize, Deserialize)]
pub struct Config {
    append: ConfigAppend,
    pub database: String,
    log_level: String,
    managers_port: u16,
    pub postgres_port: u16,
    pub postgres_user: String,
}

#[derive(Debug, Gtmpl, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConfigAppend {
    coordinator: Vec<ConfigAppendCoordinator>,
    datanode: Vec<ConfigAppendDatanode>,
    gtm: Vec<ConfigAppendGtm>,
    proxy: Vec<ConfigAppendProxy>,
}

#[derive(Debug, Gtmpl, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConfigAppendCoordinator {
    name: String,
    content: String,
}

#[derive(Debug, Gtmpl, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConfigAppendDatanode {
    name: String,
    content: String,
}

#[derive(Debug, Gtmpl, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConfigAppendGtm {
    name: String,
    content: String,
}

#[derive(Debug, Gtmpl, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConfigAppendProxy {
    name: String,
    content: String,
}

// Connection Pool
#[derive(Debug, Gtmpl, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConnectionPool {
    pub enabled: bool,
    pub user: String,
}

// Envs
#[derive(Debug, Gtmpl, Clone, PartialEq, Serialize, Deserialize)]
pub struct Envs {
    pub name: String,
    pub content: String,
}

// Extra Labels
#[derive(Debug, Gtmpl, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExtraLabels {
    pub name: String,
    pub content: String,
}

// Health Check
#[derive(Debug, Gtmpl, Clone, PartialEq, Serialize, Deserialize)]
pub struct HealthCheck {
    pub enabled: bool,
    pub user: String,
}

// Image
#[derive(Debug, Gtmpl, Clone, PartialEq, Serialize, Deserialize)]
pub struct Image {
    pub name: String,
    pub version: String,
}

// On Load
#[derive(Debug, Gtmpl, Clone, PartialEq, Serialize, Deserialize)]
pub struct OnLoad {
    pub enabled: bool,
    back_off_limit: u8,
    resources: OnLoadResource,
    pub startup: Vec<OnLoadStartup>,
    init: Vec<OnLoadInit>,
    add_containers: String,
    volumes: String,
    volume_mounts: String,
    inject_main_container_yaml: String,
    inject_spec_yaml: String,
    inject_job_yaml: String,
}

#[derive(Debug, Gtmpl, Clone, PartialEq, Serialize, Deserialize)]
pub struct OnLoadResource {
    requests: OnLoadResourceRequest,
    limits: OnLoadResourceLimit,
}

#[derive(Debug, Gtmpl, Clone, PartialEq, Serialize, Deserialize)]
pub struct OnLoadResourceRequest {
    memory: String,
    cpu: f32,
}

#[derive(Debug, Gtmpl, Clone, PartialEq, Serialize, Deserialize)]
pub struct OnLoadResourceLimit {
    memory: String,
    cpu: f32,
}

#[derive(Debug, Gtmpl, Clone, PartialEq, Serialize, Deserialize)]
pub struct OnLoadStartup {
    pub name: String,
    pub content: String,
}

#[derive(Debug, Gtmpl, Clone, PartialEq, Serialize, Deserialize)]
pub struct OnLoadInit {
    name: String,
    content: String,
}

// Override Envs
#[derive(Debug, Gtmpl, Clone, PartialEq, Serialize, Deserialize)]
pub struct OverrideEnvs {
    pub name: String,
    pub content: String,
}

// Replication
#[derive(Debug, Gtmpl, Clone, PartialEq, Serialize, Deserialize)]
pub struct Replication {
    pub enabled: bool,
    pub master_name: String,
    pub standby_name: String,
}

// Security
#[derive(Debug, Gtmpl, Clone, PartialEq, Serialize, Deserialize)]
pub struct Security {
    pub password: SecurityPassword,
    pub tls: SecurityTls,
}

#[derive(Debug, Gtmpl, Clone, PartialEq, Serialize, Deserialize)]
pub struct SecurityPassword {
    pub auth_type: String,
    pub method: String,
    pub mount_path: String,
    pub secret_name: String,
    pub extra_username: Vec<String>,
}

#[derive(Debug, Gtmpl, Clone, PartialEq, Serialize, Deserialize)]
pub struct SecurityTls {
    pub method: String,
    pub mount_path: String,
    pub secret_name: String,
    pub secret_ca: String,
    pub secret_crt: String,
    pub secret_key: String,
}

// Service
#[derive(Debug, Gtmpl, Clone, PartialEq, Serialize, Deserialize)]
pub struct Service {
    enabled: bool,
    port: u16,
    service_type: String,
}

// Stateful Sets
// GTM
#[derive(Debug, Gtmpl, Clone, PartialEq, Serialize, Deserialize)]
pub struct Gtm {
    resources: GtmResource,
    pvc: GtmPvcResource,
    add_containers: String,
    volumes: String,
    volume_mounts: String,
    add_volume_claims: String,
    inject_main_container_yaml: String,
    inject_spec_yaml: String,
    inject_sts_yaml: String,
}

#[derive(Debug, Gtmpl, Clone, PartialEq, Serialize, Deserialize)]
pub struct GtmResource {
    requests: GtmResourceRequest,
    limits: GtmResourceLimit,
}

#[derive(Debug, Gtmpl, Clone, PartialEq, Serialize, Deserialize)]
pub struct GtmResourceRequest {
    memory: String,
    cpu: f32,
}

#[derive(Debug, Gtmpl, Clone, PartialEq, Serialize, Deserialize)]
pub struct GtmResourceLimit {
    memory: String,
    cpu: f32,
}

#[derive(Debug, Gtmpl, Clone, PartialEq, Serialize, Deserialize)]
pub struct GtmPvcResource {
    resources: GtmPvcResourceRequest,
}

#[derive(Debug, Gtmpl, Clone, PartialEq, Serialize, Deserialize)]
pub struct GtmPvcResourceRequest {
    requests: GtmPvcResourceRequestStorage,
}

#[derive(Debug, Gtmpl, Clone, PartialEq, Serialize, Deserialize)]
pub struct GtmPvcResourceRequestStorage {
    storage: String,
}

// GTM Proxies
#[derive(Debug, Gtmpl, Clone, PartialEq, Serialize, Deserialize)]
pub struct Proxy {
    enabled: bool,
    count: u8,
    thread_count: u8,
    resources: ProxyResource,
    add_containers: String,
    volumes: String,
    volume_mounts: String,
    add_volume_claims: String,
    inject_main_container_yaml: String,
    inject_spec_yaml: String,
    inject_sts_yaml: String,
}

#[derive(Debug, Gtmpl, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProxyResource {
    requests: ProxyResourceRequest,
    limits: ProxyResourceLimit,
}

#[derive(Debug, Gtmpl, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProxyResourceRequest {
    memory: String,
    cpu: f32,
}

#[derive(Debug, Gtmpl, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProxyResourceLimit {
    memory: String,
    cpu: f32,
}

// Coordinators
#[derive(Debug, Gtmpl, Clone, PartialEq, Serialize, Deserialize)]
pub struct Coordinator {
    count: u8,
    resources: CoordinatorResource,
    pvc: CoordinatorPvcResource,
    add_containers: String,
    volumes: String,
    volume_mounts: String,
    add_volume_claims: String,
    inject_main_container_yaml: String,
    inject_spec_yaml: String,
    inject_sts_yaml: String,
}

#[derive(Debug, Gtmpl, Clone, PartialEq, Serialize, Deserialize)]
pub struct CoordinatorResource {
    requests: CoordinatorResourceRequest,
    limits: CoordinatorResourceLimit,
}

#[derive(Debug, Gtmpl, Clone, PartialEq, Serialize, Deserialize)]
pub struct CoordinatorResourceRequest {
    memory: String,
    cpu: f32,
}

#[derive(Debug, Gtmpl, Clone, PartialEq, Serialize, Deserialize)]
pub struct CoordinatorResourceLimit {
    memory: String,
    cpu: f32,
}

#[derive(Debug, Gtmpl, Clone, PartialEq, Serialize, Deserialize)]
pub struct CoordinatorPvcResource {
    resources: CoordinatorPvcResourceRequest,
}

#[derive(Debug, Gtmpl, Clone, PartialEq, Serialize, Deserialize)]
pub struct CoordinatorPvcResourceRequest {
    requests: CoordinatorPvcResourceRequestStorage,
}

#[derive(Debug, Gtmpl, Clone, PartialEq, Serialize, Deserialize)]
pub struct CoordinatorPvcResourceRequestStorage {
    storage: String,
}

// Datanodes
#[derive(Debug, Gtmpl, Clone, PartialEq, Serialize, Deserialize)]
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

#[derive(Debug, Gtmpl, Clone, PartialEq, Serialize, Deserialize)]
pub struct DatanodeResource {
    requests: DatanodeResourceRequest,
    limits: DatanodeResourceLimit,
}

#[derive(Debug, Gtmpl, Clone, PartialEq, Serialize, Deserialize)]
pub struct DatanodeResourceRequest {
    memory: String,
    cpu: f32,
}

#[derive(Debug, Gtmpl, Clone, PartialEq, Serialize, Deserialize)]
pub struct DatanodeResourceLimit {
    memory: String,
    cpu: f32,
}

#[derive(Debug, Gtmpl, Clone, PartialEq, Serialize, Deserialize)]
pub struct DatanodePvcResource {
    resources: DatanodePvcResourceRequest,
}

#[derive(Debug, Gtmpl, Clone, PartialEq, Serialize, Deserialize)]
pub struct DatanodePvcResourceRequest {
    requests: DatanodePvcResourceRequestStorage,
}

#[derive(Debug, Gtmpl, Clone, PartialEq, Serialize, Deserialize)]
pub struct DatanodePvcResourceRequestStorage {
    storage: String,
}
// End Stateful Sets
