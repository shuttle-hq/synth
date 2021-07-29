#!/bin/bash

cd $1
for d in */ ; do
    cd "$d"
    if synth init; then
        for ns in */ ; do
            if synth generate $ns --size 10; then
                echo "Generated $ns"
            else
                echo "Failed generating $ns"
                exit 1
            fi
        done
    else
        echo "Failed to init $d"
        exit 1
    fi
done