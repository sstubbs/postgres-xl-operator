#!/usr/bin/env bash

source ./common/vars.sh
source ./common/functions.sh

cd ../

find ./custom-resource-definitions -type f | while read -r fname; do
  YAML_CUSTOM_RESOURCE_DEFINITION=$(replace_with_env "$(cat "${fname}")")
  echo "${YAML_CUSTOM_RESOURCE_DEFINITION}" | kubectl apply -f -
done