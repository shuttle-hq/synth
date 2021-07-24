Sanbox Dev and Testing with Vagrant
====================================

This directory contains `Vagrantfile`s that define VMs that can be used to set up a development or testing environment 
for synth on different platforms. Vagrant is very simple to spin up and great for running in a repeatable OS 
environment.

The `Vagrantfile` will install all rust dependencies to build and install `synth` from source. Docker is also installed, 
as it's convenient for running database services. Note: switch user to root after `vagrant ssh` since the rust and 
synth tools are installed under root.

Two source environments are available on the VM: 

- `/synth` is a synced folder to your local synth source and modifications there will reflect in your local source.
- `/synth-sandbox` is a copy of the synth source and modifications are not persisted after the VM is destroyed.

# Requirements

- Virtual box
- Vagrant

# Common Commands

Make sure you are in the directory with the `Vagrantfile` before executing vagrant. These command need to be run in the 
vagrant directory:

- Init/start the vagrant VM: `vagrant up`
- Stop the VM: `vagrant suspend`
- Destroy the VM completely: `vagrant destroy`
- Open a ssh session to the VM: `vagrant ssh`
- Start a remote desktop session: `vagrant rdp` (windows)
- Send a file to the VM: `vagrant upload`
