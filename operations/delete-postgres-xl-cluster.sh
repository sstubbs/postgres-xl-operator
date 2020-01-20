#!/usr/bin/env bash

source ./run/vars.sh
source ./run/functions.sh

CLUSTERS=$(kubectl get -n "${NAMESPACE}" -o name "${CLUSTER_RESOURCE_KIND_LOWER}.${CUSTOM_RESOURCE_GROUP}")

select CLUSTER in $CLUSTERS; do test -n "${CLUSTER}" && break; echo ">>> Invalid Selection"; done

kubectl delete -n "${NAMESPACE}" "${CLUSTER}"