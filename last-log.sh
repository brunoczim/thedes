#!/bin/bash

set -e

cd "$HOME/.cache/thedes"

file="$(ls -At  | head -n 1)"

less -R "$file"
