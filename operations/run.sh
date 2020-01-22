#!/usr/bin/env bash

source ./common/vars.sh
source ./common/functions.sh

./create-crd.sh

cargo run
