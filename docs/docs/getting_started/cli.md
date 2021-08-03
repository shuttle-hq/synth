---
id: command-line
title: Command-line
---

`synth` is a Unix-y command line tool wrapped around the core synthetic data engine. 

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

#### Argument

- `<namespace>` - The desired path to the imported namespace directory. Can only
  be a path relative to the current (initialised) workspace root.
  
#### Options
 
- `--from <from>` - The location from which to import. Synth supports multiple import strategies. 
  
  Importing from a file: Currently we support importing from JSON files by specifying the path to
                           the file: `/some/path/to/file.json`.
  
  Importing from standard input: Not specifying `from` will accept JSON files from stdin.

  Importing from a database (e.g.
  postgres): `synth import tpch --from postgres://user:pass@localhost:5432/tpch`

---

### Command: generate

Usage: `synth generate [OPTIONS] <namespace>`

The `synth generate` command will generate data for a given namespace. This will not mutate anything in the underlying configuration.

If there is a misconfiguration in your schema (for example referring to a field that does not exist), `synth generate` will exit with a non-zero exit code and output an error message to help you understand which part of the schema is misconfigured.

#### Argument

- `<namespace>` - The path to the namespace you wish to generate data for. Can
  only be a path relative to the current (initialised) workspace root.
  
#### Options

- `--collection <collection>` - Specify a specific collection in a namespace if you don't want to generate data from all collections.
- `--size <size>` - The number of elements which should be generated per collection. This number is not guaranteed, it serves as a lower bound.
- `--to <uri>` - The generation destination. If unspecified, generation defaults to stdout.
- `--seed <seed>` - An unsigned 64 bit integer seed to be used as a seed for generation. Defaults to 0 if unspecified.
- `--random` - A flag which toggles generation with a random seed. This cannot be used with --seed.
