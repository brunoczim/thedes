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

epilogue="$directory/_epilogue.html"
list_tmp="$directory/_list_tmp.html"
prologue="$directory/_prologue.html"
output="$public/index.html"

make_list_items () {
    list=""
    for file_path in "$(echo thedes_*)"
    do
        stem="$(basename "$file_path")"
        list="$list<li><a href=\"./$stem/\">$stem</a></li>"
    done
    echo "$list" > "$list_tmp"
}

build () {
    cat "$epilogue" "$list_tmp" "$prologue" > "$output"
}

cleanup () {
    rm -f "$list_tmp"
}

make_list_items
build
cleanup
