#!/bin/bash

for d in ${@:1} ; do
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
