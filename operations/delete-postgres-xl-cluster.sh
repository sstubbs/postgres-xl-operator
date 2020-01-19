#!/usr/bin/env bash

source ./run/vars.sh
source ./run/functions.sh

export CURRENT_CLUSTER_NAME="cluster1"

kubectl delete -n "${NAMESPACE}" "${CLUSTER_RESOURCE_KIND_LOWER}.${CUSTOM_RESOURCE_GROUP}/${CURRENT_CLUSTER_NAME}"