Integration Tests for MySql
====================================

This is an integration test that validates the synth generate and synth import commands for MySql on a Debian flavored 
OS. The models in hospital_master are used as a known "golden" set to validate against. The *.sql scripts generate the
schema and test data within the database.

# Requirements:
- Docker
- jq

# Instructions

To run this, execute `e2e.sh test-local` script from the current directory. A non-zero return code denotes failure.
Note: run this with all dependencies installed in Vagrant with the [Vagrantfile](tools/vagrant/linux/ubuntu/Vagrantfile)

By default, the script will test MySql. To test MariaDb, set the environment variable `SCHEME=mariadb` before running.
