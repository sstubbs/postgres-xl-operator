#!/usr/bin/env bash

function replace_with_env() {
  # replace any {{ENV_NAME}} with its respective env value.
  str="$1"

  while [[ $str =~ ('{{'([[:alnum:]_]+)'}}') ]]; do
      str=${str//${BASH_REMATCH[1]}/${!BASH_REMATCH[2]}}
  done

  echo "$str"
}