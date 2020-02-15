#!/usr/bin/env bash

source ./common/vars.sh
source ./common/functions.sh

CLUSTERS=$(kubectl get -n "${NAMESPACE}" -o name "${CLUSTER_RESOURCE_KIND_LOWER}.${CUSTOM_RESOURCE_GROUP}")

printf "Please select master cluster that you want to unlink and promote slave from:\n"
select MASTER_CLUSTER in $CLUSTERS; do test -n "${MASTER_CLUSTER}" && break; echo ">>> Invalid Selection"; done

# Unlink master from standby
UPDATED_YAML=$(kubectl get -n "${NAMESPACE}" "${MASTER_CLUSTER}" -o yaml | sed '/replication:/{
 N
 s/enabled: true/enabled: false/
}')
echo "${UPDATED_YAML}" | kubectl apply -n "${NAMESPACE}" -f -

sleep 30
STANDBY_NAME=$(kubectl get -n "${NAMESPACE}" "${MASTER_CLUSTER}" -o yaml | sed -n '/standby_name/p' | tail -n 1 | cut -d ":" -f 2 | xargs)
# TODO wait for pod to be terminated
#kubectl wait --for=condition=terminated pod -l "app.kubernetes.io/instance=pgxlo,app.kubernetes.io/name=${STANDBY_NAME}" --timeout=180s

# Promote standby
MASTER_CLUSTER_NAME=$(echo "${MASTER_CLUSTER}" | cut -d "/" -f 2)
REPLACE_STRING="s/name: ${MASTER_CLUSTER_NAME}/name: ${STANDBY_NAME}/g"
PROMOTED_YAML=$(echo "${UPDATED_YAML}" | sed "${REPLACE_STRING}")
echo "${PROMOTED_YAML}" | kubectl apply -n "${NAMESPACE}" -f -