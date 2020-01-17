use kube::api::{Object, Void};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PostgresXlCluster {
    pub data: Option<String>,
}

pub type KubePostgresXlCluster = Object<PostgresXlCluster, Void>;
