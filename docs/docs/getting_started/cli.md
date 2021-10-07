---
id: command-line
title: Command-line
---

`synth` is a Unix-y command line tool wrapped around the core synthetic data engine. 

## Usage

---

### Command: import

Usage: `synth import [OPTIONS] <namespace>`

Synth can create schema files from different data sources using the `synth import` command.
Accidentally running `synth import` on an existing directory is safe - the operation will fail.

If a subdirectory for a given namespace does not exist, Synth will create it.

#### Argument

- `<namespace>` - The path to the namespace directory into which to save schema files. The directory will be created by `synth`.
  
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

The `synth generate` command will generate data from a collection of schema files.

If there is a misconfiguration in your schema (for example referring to a field that does not exist), `synth generate` will exit with a non-zero exit code and output an error message to help you understand which part of the schema is misconfigured.

#### Argument

- `<namespace>` - The path to the namespace directory from which to load schema files.
  
#### Options

- `--collection <collection>` - Specify a specific collection in a namespace if you don't want to generate data from all collections.
- `--size <size>` - The number of elements which should be generated per collection. This number is not guaranteed, it serves as a lower bound.
- `--to <uri>` - The generation destination. If unspecified, generation defaults to stdout.
- `--seed <seed>` - An unsigned 64 bit integer seed to be used as a seed for generation. Defaults to 0 if unspecified.
- `--random` - A flag which toggles generation with a random seed. This cannot be used with --seed.
