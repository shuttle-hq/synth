#!/bin/sh

gsutil -h "Content-Type:application/x-shellscript" \
       -h "Cache-Control:no-store, max-age=0" \
        cp install.sh \
        gs://getsynth-public/install/install.sh