#!/usr/bin/env bash

source ./run/vars.sh
source ./run/functions.sh

bold=$(tput bold)
normal=$(tput sgr0)

read -p "Enter cluster name: " CURRENT_CLUSTER_NAME
export CURRENT_CLUSTER_NAME

printf "Please select example:\n"
select EXAMPLE in ../custom-resource-examples/postgres-xl-cluster/*; do test -n "${EXAMPLE}" && break; echo ">>> Invalid Selection"; done

YAML_CUSTOM_RESOURCE=$(replace_with_env "$(cat "${EXAMPLE}")")
printf "%sCluster %s will be created with the following details: %s\n%s\n" "${bold}" "${CURRENT_CLUSTER_NAME}" "${normal}" "${YAML_CUSTOM_RESOURCE}"

read -p "Continue (y/n)?" choice
case "$choice" in
  y|Y ) echo "${YAML_CUSTOM_RESOURCE}" | kubectl apply -n "${NAMESPACE}" -f -;;
  n|N ) echo "Cancelled";;
  * ) echo "invalid";;
esac

