#!/bin/sh
bucket=${bucket:-openquery-synth-prod-public}
gsutil -h "Content-Type:application/x-shellscript" \
       -h "Cache-Control:no-store, max-age=0" \
        cp install.sh \
        gs://$bucket/install
