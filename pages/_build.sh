#!/bin/bash

set -e

if [ $# -ne 1 ]
then
    echo >&2 "Expected exactly one argument (docs path)."
    echo >&2 "Usage:"
    echo >&2 "  $0 <docs path>"
    exit 1
fi

public="$1"

directory="$(dirname "$0")"

build_page () {
    cat "$directory/_epilogue.html" "$body" "$directory/_prologue.html" > "$out"
}

build_main () {
    body="$directory/main.html" out="$public/index.html" build_page
}

build_flavour () {
    list="<p>List of Thedes crates for $name (click to access docs):</p>"
    list="$list<ul>"
    for file_path in "$(echo "$flavour"/thedes_*)"
    do
        stem="$(basename "$file_path")"
        list="$list<li><a href=\"./$stem/\">$stem</a></li>"
    done
    list="$list</ul>"
    echo "$list" > "$directory/_flavour_tmp.html"
    body="$directory/_flavour_tmp.html" out="$flavour/index.html" build_page
    rm -f "$directory/_flavour_tmp.html"
}

build_flavours () {
    name=native flavour="$public/native" build_flavour
    name=WASM flavour="$public/wasm" build_flavour
}

build_all () {
    build_main
    build_flavours
}

build_all
