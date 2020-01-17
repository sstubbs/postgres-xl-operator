#!/usr/bin/env bash

source ./run/vars.sh
source ./run/functions.sh

kubectl get -n "${NAMESPACE}" "${CLUSTER_RESOURCE_KIND_LOWER}.${CUSTOM_RESOURCE_GROUP}"