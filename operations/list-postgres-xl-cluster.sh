#!/usr/bin/env bash

source ./run/vars.sh
source ./run/functions.sh

kubectl get -n "${NAMESPACE}" "postgresxlcluster.${CUSTOM_RESOURCE_GROUP}"