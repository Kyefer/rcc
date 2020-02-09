#!/bin/bash

cargo build;

dir="test/stage_$1/valid"

if [ -d $dir ]; then
    for filename in $dir/*.c; do
        echo "-------------------------------------------"
        echo "Running $filename:"
        echo "-------------------------------------------"
        ./target/debug/rcc $filename;
    done
else
    echo "$dir is not a valid directory"
    exit 1
fi