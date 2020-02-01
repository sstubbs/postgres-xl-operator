#!/usr/bin/env bash

# TODO expose ordinal from statefulset https://github.com/kubernetes/kubernetes/pull/68719 when available and stable
# For now just use hostname and remove base name https://stackoverflow.com/questions/16623835/remove-a-fixed-prefix-suffix-from-a-string-in-bash
ORDINAL=$([[ "${HOSTNAME}" =~ ^"${DATANODE_BASENAME}-"(.*)$ ]] && echo "${BASH_REMATCH[1]}")
MASTER_SERVICE="${MASTER_NAME}-dn-${ORDINAL}.${MASTER_NAME}-svc-dn"
echo "${MASTER_SERVICE}"