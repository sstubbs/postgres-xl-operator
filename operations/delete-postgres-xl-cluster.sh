#!/usr/bin/env bash

source ./run/vars.sh
source ./run/functions.sh

read -p "Enter cluster name: " CURRENT_CLUSTER_NAME
export CURRENT_CLUSTER_NAME

kubectl delete -n "${NAMESPACE}" "${CLUSTER_RESOURCE_KIND_LOWER}.${CUSTOM_RESOURCE_GROUP}/${CURRENT_CLUSTER_NAME}"