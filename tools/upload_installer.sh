#!/bin/sh

gsutil -h "Content-Type:application/x-shellscript" \
       -h "Cache-Control:no-store, max-age=0" \
        cp install.sh \
        gs://artifacts.getsynth.appspot.com/install/installer.sh