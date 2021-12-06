#!/usr/bin/env bash

date=${1}
if (( date < 1 || date > 25 )); then
  echo "Invalid date: '${date}'" >&2
  exit 1
fi
printf -v date '%02d' "$date" # add leading 0
dir="src/bin/${date}"

mkdir "$dir" || exit 1
cp src/bin/template/* "$dir"

echo "Created ${dir}:"
printf 'Run:\tcargo run --bin %s\n' "$date"
printf 'Test:\tcargo test --bin %s\n' "$date"
