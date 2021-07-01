---
id: command-line
title: Synth CLI
---

The Synth CLI (`synth`) is a Unix-y command line tool wrapped around the core synthetic data engine. 

## Usage

---

### Command: init

Usage: `synth init`


This is the first command that should be run for any new or existing when starting out with Synth. 
This initialises the workspace and  sets up all the local data necessary to run Synth.
A `.synth/` subdirectory is created that is typically not committed to version control.

This command is always safe to run multiple times though subsequent runs
may give errors. This command will never erase your workspace.

---

### Command: import

Usage: `synth import [OPTIONS] <namespace>`

Synth can create namespaces from different data sources using the `synth import` command.
Accidentally running `synth import` on an existing namespace is safe - the operation will fail.

If a subdirectory for a given namespace does not exist in your workspace, Synth will create it.
    

#### Options

- `--from <from>` - The location from which to import. Synth supports multiple import strategies. 
                  Importing directly from a database will be supported in future versions.
  
  Importing from a file: Currently we support importing from JSON files by specifying the path to
                           the file: `/some/path/to/file.json`.
  
  Importing from standard input: Not specifying `from` will accept JSON files from stdin.

---

### Command: generate

Usage: `synth generate [OPTIONS] <namespace>`

The `synth generate` command will generate data for a given namespace. This will not mutate anything in the underlying configuration.

If there is a misconfiguration in your schema (for example referring to a field that does not exist), `synth generate` will exit with a non-zero exit code and hopefully some error message helping you understand which part of the schema is misconfigured.

#### Options

- `--collection <collection>` - Specify a specific collection in a namespace if you don't want to generate data from all collections.  
- `--size <size>` - The number of elements which should be generated per collection. This number is not guaranteed, it serves as a lower bound.
- `--to <uri>` - The generation destination. If unspecified, generation defaults to stdout.
- `--seed <seed>` - An unsigned 64 bit integer seed to be used as a seed for generation. Defaults to 0 if unspecified.
- `--random` - A flag which toggles generation with a random seed. This cannot be used with --seed.
---

### Command: serve

Usage: `synth serve [OPTIONS]`

Run Synth in Daemon mode. The Daemon exposes an HTTP RESTful API on port `8182` and creates an internal state which is managed by a version controlled index.
                            
Daemon mode is used when `synth` is used in the context of a collaborating team and comes with a very handy [Python client](https://getsynth.github.io/synthpy/)

#### Options

- `-b, --bind <bind> [default: 0.0.0.0:8182]` - The endpoint on which the HTTP server should be exposed.  
- `-d, --data-directory <data-directory> [default: <workspace>/.synth/]` - The directory which should host the index. (Default is fine in the context of a workspace)
