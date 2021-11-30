#!/usr/bin/env bash

date=${1}
if (( date < 1 || date > 25 )); then
  echo "Invalid date: '${date}'"
  exit 1
fi
printf -v date '%02d' "$date" # add leading 0
sed "s/template/${date}/g" src/bin/template.rs > "src/bin/${date}.rs"
