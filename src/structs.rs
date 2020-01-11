use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Global
#[derive(Gtmpl, Debug, PartialEq, Serialize, Deserialize)]
pub struct Value {
    pub image: String,
    pub envs: String,
    pub extra_labels: String,
    pub homedir: String,
    pub override_envs: String,
    pub config: Config,
    pub security: Security,
    pub service: Service,
    pub on_load: OnLoad,
    pub gtm: Gtm,
    pub proxies: Proxy,
    pub coordinators: Coordinator,
}

// Config
#[derive(Gtmpl, Debug, PartialEq, Serialize, Deserialize)]
pub struct Config {
    log_level: String,
    managers_port: u16,
    postgres_port: u16,
    postgres_user: String,
    append: ConfigAppend,
}

#[derive(Gtmpl, Debug, PartialEq, Serialize, Deserialize)]
pub struct ConfigAppend {
    gtm: String,
    proxy: String,
    datanode: String,
    coordinator: String,
}

// WAL
#[derive(Gtmpl, Debug, PartialEq, Serialize, Deserialize)]
pub struct Wal {
    archive: WalArchive,
}

#[derive(Gtmpl, Debug, PartialEq, Serialize, Deserialize)]
pub struct WalArchive {
    enable: bool,
    version: String,
    storage_path: String,
    pvc: String,
}

// Security
#[derive(Gtmpl, Debug, PartialEq, Serialize, Deserialize)]
pub struct Security {
    passwords_secret_name: String,
    pg_password: String,
    postgres_auth_type: String,
}

// Service
#[derive(Gtmpl, Debug, PartialEq, Serialize, Deserialize)]
pub struct Service {
    enabled: bool,
    port: u16,
    service_type: String,
}

// On Load
#[derive(Gtmpl, Debug, PartialEq, Serialize, Deserialize)]
pub struct OnLoad {
    enabled: bool,
    back_off_limit: u8,
    resources: OnLoadResource,
    startup: HashMap<String, String>,
    init: HashMap<String, String>,
}

#[derive(Gtmpl, Debug, PartialEq, Serialize, Deserialize)]
pub struct OnLoadResource {
    limits: OnLoadResourceLimit,
}

#[derive(Gtmpl, Debug, PartialEq, Serialize, Deserialize)]
pub struct OnLoadResourceLimit {
    memory: String,
    cpu: String,
}

// GTM
#[derive(Gtmpl, Debug, PartialEq, Serialize, Deserialize)]
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

#[derive(Gtmpl, Debug, PartialEq, Serialize, Deserialize)]
pub struct GtmResource {
    limits: GtmResourceRequestLimit,
}

#[derive(Gtmpl, Debug, PartialEq, Serialize, Deserialize)]
pub struct GtmResourceRequestLimit {
    memory: String,
    cpu: String,
}

#[derive(Gtmpl, Debug, PartialEq, Serialize, Deserialize)]
pub struct GtmPvcResource {
    resources: GtmPvcResourceResourceRequest,
}

#[derive(Gtmpl, Debug, PartialEq, Serialize, Deserialize)]
pub struct GtmPvcResourceResourceRequest {
    requests: GtmPvcResourceResourceRequestStorage,
}

#[derive(Gtmpl, Debug, PartialEq, Serialize, Deserialize)]
pub struct GtmPvcResourceResourceRequestStorage {
    storage: String,
}

// GTM Proxies
#[derive(Gtmpl, Debug, PartialEq, Serialize, Deserialize)]
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

#[derive(Gtmpl, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProxyResource {
    limits: ProxyResourceLimit,
}

#[derive(Gtmpl, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProxyResourceLimit {
    memory: String,
    cpu: String,
}

// Coordinators
#[derive(Gtmpl, Debug, PartialEq, Serialize, Deserialize)]
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

#[derive(Gtmpl, Debug, PartialEq, Serialize, Deserialize)]
pub struct CoordinatorResource {
    limits: CoordinatorResourceLimit,
}

#[derive(Gtmpl, Debug, PartialEq, Serialize, Deserialize)]
pub struct CoordinatorResourceLimit {
    memory: String,
    cpu: String,
}

#[derive(Gtmpl, Debug, PartialEq, Serialize, Deserialize)]
pub struct CoordinatorPvcResource {
    resources: CoordinatorPvcResourceResourceRequest,
}

#[derive(Gtmpl, Debug, PartialEq, Serialize, Deserialize)]
pub struct CoordinatorPvcResourceResourceRequest {
    requests: CoordinatorPvcResourceResourceRequestStorage,
}

#[derive(Gtmpl, Debug, PartialEq, Serialize, Deserialize)]
pub struct CoordinatorPvcResourceResourceRequestStorage {
    storage: String,
}

// Datanodes
#[derive(Gtmpl, Debug, PartialEq, Serialize, Deserialize)]
pub struct Datanode {
    count: u8,
    resources: DatanodeResource,
    pvc: DatanodeResource,
    add_containers: String,
    volumes: String,
    volume_mounts: String,
    add_volume_claims: String,
    inject_main_container_yaml: String,
    inject_spec_yaml: String,
    inject_sts_yaml: String,
}

#[derive(Gtmpl, Debug, PartialEq, Serialize, Deserialize)]
pub struct DatanodeResource {
    limits: DatanodeResourceLimit,
}

#[derive(Gtmpl, Debug, PartialEq, Serialize, Deserialize)]
pub struct DatanodeResourceLimit {
    memory: String,
    cpu: String,
}

#[derive(Gtmpl, Debug, PartialEq, Serialize, Deserialize)]
pub struct DatanodePvcResource {
    resources: DatanodePvcResourceResourceRequest,
}

#[derive(Gtmpl, Debug, PartialEq, Serialize, Deserialize)]
pub struct DatanodePvcResourceResourceRequest {
    requests: DatanodePvcResourceResourceRequestStorage,
}

#[derive(Gtmpl, Debug, PartialEq, Serialize, Deserialize)]
pub struct DatanodePvcResourceResourceRequestStorage {
    storage: String,
}
