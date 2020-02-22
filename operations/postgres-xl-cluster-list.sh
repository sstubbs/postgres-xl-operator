#!/usr/bin/env bash

source ./common/vars.sh
source ./common/functions.sh

kubectl get -n "${NAMESPACE}" --show-labels "${CLUSTER_RESOURCE_KIND_LOWER}.${CUSTOM_RESOURCE_GROUP}"