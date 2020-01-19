#!/usr/bin/env bash

source ./run/vars.sh
source ./run/functions.sh

read -p "Enter cluster name: "  CURRENT_CLUSTER_NAME
export CURRENT_CLUSTER_NAME

cd ../

YAML_CUSTOM_RESOURCE=$(replace_with_env "$(cat ./custom-resources/postgres-xl-cluster.yaml)")
echo "${YAML_CUSTOM_RESOURCE}" | kubectl apply -n "${NAMESPACE}" -f -
