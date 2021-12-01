#!/usr/bin/env bash

date=${1}
if (( date < 1 || date > 25 )); then
  echo "Invalid date: '${date}'" >&2
  exit 1
fi
printf -v date '%02d' "$date" # add leading 0
file="src/bin/${date}.rs"
input="input/${date}.txt"

if [[ -e "$file" ]]; then
  echo "${file} already exists! Not overwriting" >&2
  exit 1
fi

sed "s/template/${date}/g" src/bin/template.rs > "$file"
touch "$input"

echo "Created ${date}.rs and ${input}:"
printf 'Run:\tcargo run --bin %s\n' "$date"
printf 'Test:\tcargo test --bin %s\n' "$date"
