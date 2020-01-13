#!/usr/bin/env bash

source ./run/vars.sh
source ./run/functions.sh

export NAME="cluster1"

kubectl delete -n "${NAMESPACE}" "postgresxlcluster.${CUSTOM_RESOURCE_GROUP}/${NAME}"