#!/usr/bin/env bash

source ./common/vars.sh
source ./common/functions.sh

./create-crd.sh

cargo clean --package postgres-xl-operator

cargo run
