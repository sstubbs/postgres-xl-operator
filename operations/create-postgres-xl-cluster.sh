#!/usr/bin/env bash

source ./run/vars.sh
source ./run/functions.sh

export CURRENT_CLUSTER_NAME="cluster1"

cd ../

YAML_CUSTOM_RESOURCE=$(replace_with_env "$(cat ./custom-resources/postgres-xl-cluster.yaml)")
echo "${YAML_CUSTOM_RESOURCE}" | kubectl apply -n "${NAMESPACE}" -f -
