#!/usr/bin/env bash

# remember to port forward a cluster before running this i.e. kubectl port-forward pgxlo-test-crd-0 5432:5432

cd ../
export DATABASE_URL="postgres://postgres@localhost/health_checks"
diesel print-schema > src/schema.rs
diesel_ext -m > src/models.rs
sed -i "" 's/Debug{trace1}/Debug/g' src/models.rs
