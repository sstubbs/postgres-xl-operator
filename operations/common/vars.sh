#!/usr/bin/env bash

export NAMESPACE="pgxl"
export CUSTOM_RESOURCE_GROUP="postgres-xl-operator.vanqor.com"
export CHART_NAME="postgres-xl-operator"
export CHART_VERSION="0.1.0"
export RELEASE_NAME="pgxlo"
export RELEASE_SERVICE="Helm"
export LOG_LEVEL="info"
export KUBE_CONFIG_TYPE="kubeconfig"
export CLUSTER_RESOURCE_SINGULAR="postgres-xl-cluster"
export CLUSTER_RESOURCE_PLURAL="postgres-xl-clusters"
export CLUSTER_RESOURCE_KIND="PostgresXlCluster"
export CLUSTER_RESOURCE_KIND_LOWER="postgresxlcluster"
export HEALTH_CHECK_SCHEDULE="*/5 * * * *"
export PASSWORD_ROTATE_SCHEDULE="0 0 1 * *"