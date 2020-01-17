#!/usr/bin/env bash

export NAMESPACE="pgxl"
export CUSTOM_RESOURCE_GROUP="postgres-xl-operator.vanqor.com"
export CHART_NAME="postgres-xl-operator-chart"
export CHART_VERSION="0.0.1"
export RELEASE_NAME="postgres-xl-operator"
export RELEASE_SERVICE="helm"
export RUST_LOG="info"
export CLUSTER_RESOURCE_SINGULAR="postgres-xl-cluster"
export CLUSTER_RESOURCE_PLURAL="postgres-xl-clusters"
export CLUSTER_RESOURCE_KIND="PostgresXlCluster"