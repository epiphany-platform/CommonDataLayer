#!/usr/bin/env bash

echo ""
echo ""

shopt -s globstar nullglob

for file in **/*.new; do
  ACTUAL="$file"
  DIR=$(dirname "$file")

  echo "Accepting: $ACTUAL";
  echo "-----"

  cat "$ACTUAL" | colordiff

  echo ""
  echo ""
  echo "-----"
  read -p "[Aa]ccept, [Rr]reject or [Ss]kip: " -n 1 -r
  echo

  if [[ $REPLY =~ ^[Aa]$ ]]
  then
    cwd=$(pwd)
    cd "$DIR" || exit
    filename=$(basename -- "$ACTUAL")
    echo "FILENAME: $filename"
    patch --ignore-whitespace < "$filename" || exit
    cd "$cwd" || exit
  elif [[ $REPLY =~ ^[Rr]$ ]]
  then
    rm -- "$ACTUAL"
  elif [[ $REPLY =~ ^[Ss]$ ]]
  then
    echo "Skipping"
  fi
done
echo "All processed"

for file in ./tests/**/*.orig; do
  ACTUAL="$file"
  DIR=$(dirname "$file")

  echo "REMOVING: $ACTUAL";
  echo "-----"

  rm "$ACTUAL"
done
