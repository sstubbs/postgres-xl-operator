#!/usr/bin/env bash

source ./run/vars.sh
source ./run/functions.sh

cd ../

kubectl create namespace "${NAMESPACE}"

find ./custom-resource-definitions -type f | while read -r fname; do
  YAML_CUSTOM_RESOURCE_DEFINITION=$(replace_with_env "$(cat "${fname}")")
  echo "${YAML_CUSTOM_RESOURCE_DEFINITION}" | kubectl apply -n "${NAMESPACE}" -f -
done

cargo run
