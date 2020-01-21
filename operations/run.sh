#!/usr/bin/env bash

source ./common/vars.sh
source ./common/functions.sh

./create-namespace-crd.sh

cargo run
