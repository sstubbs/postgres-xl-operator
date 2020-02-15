#!/usr/bin/env bash

source ./common/vars.sh
source ./common/functions.sh

./crd-create.sh

cargo clean --package postgres-xl-operator

cargo run
