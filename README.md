# Postgres-XL Operator

[Postgres-XL](https://www.postgres-xl.org/) is an all-purpose fully ACID open source multi node scalable SQL database solution, based on [PostgreSQL](https://www.postgresql.org/).

This operator allows for creating multiple multi container, multi process, distributed databases using Postgres-XL. It is based upon the chart [postgres-xl-helm](https://github.com/LamaAni/postgres-xl-helm) and docker image [postgres-xl-docker](https://github.com/pavouk-0/postgres-xl-docker) image.

For a graph description of the connections structure see [here](https://www.2ndquadrant.com/wp-content/uploads/2019/04/Postgres-XL-Display.png.webp).

#### Important Note

If using clusters created by the operator, please make sure you read the sections about persistence, backup and restore.

## Components Overview

See: [Postgres-XL documentation](https://www.postgres-xl.org/documentation/xc-overview-components.html)

1. [ Global Transaction Manager (GTM) ](https://www.postgres-xl.org/documentation/app-gtm.html) - Single pod StatefulSet - provides transaction management for the entire cluster. The data stored in the GTM is part of the database persistence and should be backed up.
2. [ Coordinator ](https://www.postgres-xl.org/documentation/runtime-config.html) - Multi-pod StatefulSet - Database external connections entry point (i.e. where I connect my client to). These pods provide transparent concurrency and integrity of transactions globally. Applications can choose any Coordinator to connect to, they work together. Any Coordinator provides the same view of the database, with the same data, as if it was one PostgreSQL database. The data stored in the coordinator is part of the DB data and should be backed up.
3. [ Datanode ](https://www.postgres-xl.org/documentation/runtime-config.html) - Multi-pod StatefulSet - All table data is stored here. A table may be replicated or distributed between datanodes. Since query work is done on the datanodes, the scale and capacity of the db will be determined by the number of datanodes. The data stored in the datanode is part of the DB data and should be backed up.
4. [ GTM Proxy (optional) ](https://www.postgres-xl.org/documentation/app-gtm-proxy.html) - A helper transaction manager. Gtm proxy groups connections and interactions between gtm and other Postgres-XL components to reduce both the number of interactions and the size of messages. Performance tests have shown greater performance with high concurrency workloads as a result.

To connect to the database, please connect to the db main service (which is the coordinator service), example:
```shell
kubectl port-forward svc/[release-name]-[cluster-name]-svc
```

## Quick Start

### Install The Operator

1. A Kubernetes cluster with helm 3 is required.
2. `kubectl create namespace <namespace name>` 
3. Clone this repo
4. `cd operations && ./crd-create.sh && cd ../`
5. `helm3 install pgxlo chart --namespace <namespace name>`

### Creating a cluster

1. `cd operations && ./postgres-xl-cluster-create-modify.sh`
2. Follow instructions

## Installation

### Helm

1. A Kubernetes cluster with helm 3 is required.
2. `kubectl create namespace <namespace name>`
3. Clone this repo
4. Change the following variables in `operations/common/vars.sh` if required

name | description | default value 
--- | --- | ---
CUSTOM_RESOURCE_GROUP | This is the group of the custom resource definitions and custom resources | postgres-xl-operator.vanqor.com
CLUSTER_RESOURCE_SINGULAR | The cluster resource singular name | postgres-xl-cluster
CLUSTER_RESOURCE_PLURAL | The cluster resource plural name | postgres-xl-clusters
CLUSTER_RESOURCE_KIND | The cluster resource kind | PostgresXlCluster
CLUSTER_RESOURCE_KIND_LOWER | The cluster resource kind lowercase | postgresxlcluster

5. `cd operations && ./crd-create.sh && cd ../`

6. change `chart/values.yaml` as required

7. `helm3 install pgxlo chart --namespace <namespace name>`

## Operations

### Create / Modify Cluster

1. `cd operations`.
2. copy one of the examples into a new file in `custom-resource-examples/postgres-xl-cluster` and edit `spec.data` as required.
3. `./postgres-xl-cluster-create-modify.sh`.
4. follow on screen instructions.

### Delete Cluster

1. `cd operations`.
2. `./postgres-xl-cluster-delete.sh`.
3. Select required cluster name to delete.

### List Clusters

1. `cd operations`.
2. `./postgres-xl-cluster-list.sh`.

### Cluster

Clusters are controlled via the `CLUSTER_RESOURCE` in `custom-resources/postgres-xl-cluster.yaml` so it can be copied, altered and applied as required.

The custom resource `metadata.name` is used for cluster names and formatted as `RELEASE_NAME`-`metadata.name`.

`spec.data` accepts the following values:

[STS] = `datanodes` or `coordinators` or `proxies` or `gtm`

Example: datanodes.count = 32

#### Global values

name | description | default value 
--- | --- | ---
image.name | The image to use | sstubbs/postgres-xl
image.version | The version of the image to use | 0.4.0
replication.enabled | If this is set to true and standby name is present a standby cluster is created/modified/deleted with the main cluster and replicated from it. If it is modified to false and the standby_name is still set the standby will be unlinked from the current master. Used for fail-over | null
replication.master_name | This is automatically set on standby clusters so does not to be set in the values | null
replication.standby_name | If this is set to the name of a new cluster and replication is true a standby cluster is created/modified/deleted with the main cluster and replicated from it | false
health_check.enabled | If this is set to true health checks will be run | false
health_check.database_name | This is the database that will be created and values will be inserted into for health checks | health_check
envs | List of `{name: "", content: ""}` to be included environment variables to add to all pods, see `./yaml_structs/postgres-xl-cluster.yaml` for an example | []
extra_labels | List of `{name: "", content: ""}` to be included as labels, see `./yaml_structs/postgres-xl-cluster.yaml` for an example | []
config.log_level | The log level to use,  accepts : ERROR, WARNING, INFO, DEBUG, DEBUG1-DEBUG5 | WARNING
config.managers_port | The port to use for transaction management (GTM or proxies) | 6666
config.postgres_port | The internal postgres port | 5432
config.postgres_user | The internal postgres user | postgres
config.append.[STS] | List of `{name: "", content: ""}` to append to the end of the postgres config file for a specific StatefulSet, see `./yaml_structs/postgres-xl-cluster.yaml` for an example | []
wal.archive.enabled | Enable wal archiving of datanodes | false
wal.archive.version | Use versions for WAL of datanodes | unversioned
wal.archive.storage_path | The storage path for WAL of datanodes | /wal_archive
wal.archive.pvc.resources.requests.storage | Enable PVC for wal archiving of datanodes | null
security.passwords_secret_name | The kubernetes secret value set to be used for passwords | null
security.pg_password | The kubernetes secret key for superuser postgres password | null
security.postgres_auth_type | The authentication type used | md5
service.enabled | If true enables the external load balancer service | true
service.port | The external service port | 5432
service.service_type | The external service type | ClusterIP
on_load.enabled | If true enables loading scripts on startup and initialisation | true
on_load.back_off_limit | The number of times the job will restart | 5
on_load.resources.requests.memory | The on load job memory request | 250Mi
on_load.resources.requests.cpu | The on load job cpu request (Must be a decimal) | 0.25
on_load.resources.limits.memory | The on load job memory limit | 250Mi
on_load.resources.limits.cpu | The on load job cpu limit (Must be a decimal) | 0.25
on_load.startup | List of `{name: "", content: ""}` to be run in this pod on startup as bash or sql, see `./yaml_structs/postgres-xl-cluster.yaml` for an example | []
on_load.init | List of `{name: "", content: ""}` to be run in this pod on initialisation as bash or sql, see `./yaml_structs/postgres-xl-cluster.yaml` for an example) | []

#### For any StatefulSet

name | description | default value 
--- | --- | ---
[STS].count | The total number of replicas, does not apply to gtm | 1
[STS].resources.requests.memory | The main pod memory request | 250Mi
[STS].resources.requests.cpu | The main pod cpu request, must be a decimal | 0.25
[STS].resources.limits.memory | The main pod memory limit | 250Mi
[STS].resources.limits.cpu | The main pod cpu limit, must be a decimal | 0.25
[STS].pvc.resources.requests.storage | The persistence volume claim for data storage. Use this value to set the internal database storage. See Persistence for recommended values. This does not apply to proxies | null
[STS].add_containers | YAML inject to add more containers
[STS].volumes | YAML inject to add more volumes
[STS].volume_mounts | YAML inject to add more volume mounts
[STS].add_volume_claims | YAML inject to add STS dependent volume claims. See [here](https://kubernetes.io/docs/tutorials/stateful-application/basic-stateful-set/) for more info about these
[STS].thread_count | Applies only to proxies, and is the proxy worker thread count | 3 

#### Advanced overriding values (use with care)

#### Global

name | description | default value 
--- | --- | ---
homedir | The image home directory | /var/lib/postgresql 
override_envs | List of `{name: "", content: ""}` to be included environment variables which are added after the chart core envs, and allows to override the chart, see `./yaml_structs/postgres-xl-cluster.yaml for an example` | []
service.inject_spec_yaml | Injects YAML into the external service | null

#### For any stateful set

name | description
--- | ---
[STS].inject_main_container_yaml | Inject YAML into the main container.
[STS].inject_spec_yaml | Inject YAML into the template spec.
[STS].inject_sts_yaml | Inject YAML into the main STS spec.

#### Persistence

The implementation in this chart relies on StatefulSets to maintain data persistence between recoveries and restarts.
To do so, one must define the `pvc` argument in the values. If you do not define the `pvc` argument, the 
data `will be lost` on any case of restart/recover/fail.

To define a persistent database you must define all three `pvc`s for each of the stateful sets,
(below are recommended test values, where x is the size of each datanode)
```yaml
datanodes.pvc:
  resources:
      requests:
        storage: [x]Gi
gtm.pvc:
  resources:
      requests:
        storage: 100Mi
coordinators.pvc:
  resources:
      requests:
        storage: 100Mi
```

Once these are defined, the DB will recover when any of datanode/coordinator/proxy/gtm are restarted.

See more about persistence in Stateful Sets [here](https://kubernetes.io/docs/tutorials/stateful-application/basic-stateful-set/)
and [here](https://kubernetes.io/docs/concepts/storage/persistent-volumes/).

#### Backup and restore

In order to keep to kubernetes principles, this helm chart allows to specify the persistent volume claim class for the workers, coordinators and gtm. This data will persist between restarts. The persistent volumes created will be prefixed by `datastore-`

Information about StorageClasses can be found [here](https://kubernetes.io/docs/concepts/storage/volumes/)

In order to make a copy of the database one must copy all the data of each and every coordinator and datanode. This means that, when relying on this type of persistence one must:

1. Create a backup solution using a specified [ persistent storage class](https://kubernetes.io/docs/concepts/storage/storage-classes/), and allow the backend to keep copies of the data between restarts. 
2. You cannot decrease the number of executing datanodes as data may be lost. Scaling up requires the redistribution of tables, information about such operations can be found [here]().
3. Coordinators must not be scaled to less than 1 as one replica is used to rebuild others when scaled up.

#### Replication and high availability

Enabling replication will create a replicated copy of the gtm, all datanodes and one coordinator. To fail-over this standby cluster can be promoted to a master cluster by running the following:
1. `cd operations && ./postgres-xl-cluster-unlink-standby-promote.sh`
2. Follow instructions

[ More about replication and high availability.](https://www.postgres-xl.org/documentation/different-replication-solutions.html)

## Health check and status

### Pods

a pod will be considered healthy if it can pass,
1. pg_isready.
2. Connect to the gtm, datanodes (all), and coordinators (all).

### Cluster

1. Periodic sql can be run by the controller to verify cluster is working correctly.

## Some other notes

[Postgres-XL FAQ](https://www.postgres-xl.org/faq/)

Benchmarks:
1. https://www.2ndquadrant.com/en/blog/postgres-xl-scalability-for-loading-data/
1. https://www.2ndquadrant.com/en/blog/benchmarking-postgres-xl/

## Caveats

The data in the DB will persist only when all datanodes, coordinators and gtm disks are safely restored. This helm 
chart does not deal with partial restores.

## Licence

It is free software, released under the MIT licence, and may be redistributed under the terms specified in `LICENSE`.

## Extending

### Operator Running Locally From Source

1. Setup a kubernetes cluster and make sure you can connect with kubectl as this uses your kube config for authorisation.
2. Install rust.
3. Clone this repo.
4. Change `operations/common/vars.sh` if required. These are the global values applied to all clusters which are:

name | description | default value 
--- | --- | ---
NAMESPACE | The namespace that this operator will run in if using helm and create clusters in | pgxl
CUSTOM_RESOURCE_GROUP | This is the group of the custom resource definitions and custom resources | postgres-xl-operator.vanqor.com
CHART_NAME | This is the chart name which will be used in helm installations | postgres-xl-operator
CHART_VERSION | This is the chart version which will be used in helm installations | 0.1.0
RELEASE_NAME | This is the installed release name which will be used in helm installations | pgxlo
RELEASE_SERVICE | This is the service used to install the operator currently only helm is planned | helm
LOG_LEVEL | This is the log_level of the operator. Current values are `info` and `debug`. Debug will show the YAML of resources that will be generated at cluster creation time. | info
CLUSTER_RESOURCE_SINGULAR | The cluster resource singular name | postgres-xl-cluster
CLUSTER_RESOURCE_PLURAL | The cluster resource plural name | postgres-xl-clusters
CLUSTER_RESOURCE_KIND | The cluster resource kind | PostgresXlCluster
CLUSTER_RESOURCE_KIND_LOWER | The cluster resource kind lowercase | postgresxlcluster

This application must be running when performing any operations by running the following:
1. `cd operations`
2. `./run.sh`

### Extending The Operator
- Adding a new parameter if required into a custom resource involves adding it to `yaml_structs/$resource_type` and mapping it to structs in `src/structs.rs`
- Templates in the `templates` directory are created from the context by [ gtmpl-rust ](https://github.com/fiji-flo/gtmpl-rust) so any other required kubernetes resources can be added to those sub folders but only one YAML document per file.

### Security
- openssl for kube-rs, config sha, and cert generation is currently being used for compliance.
- Preferably [rusttls](https://github.com/ctz/rustls) can be used in the future for performance gains.
- To run sql inside a pod get the password with `export PGPASSWORD="$(cat "${PASSWORD_SECRET_MOUNT_PATH}"/"${PGUSER}")"`