#!/bin/bash

cd $1
for d in */ ; do
    cd "$d"
    for ns in */ ; do
        if synth generate $ns --size 10; then
            echo "Generated $ns"
        else
            echo "Failed generating $ns"
            exit 1
        fi
    done
done