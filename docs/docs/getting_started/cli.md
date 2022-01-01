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

- `--from <uri>` - The location from which to import. Synth uses Uniform Resource Identifiers (URIs) to define interactions with databases and the filesystem. `<uri>` must therefore be a valid RFC 3986 URI.

  Importing from a database is possible using the standard URI format of the respective database. For example: `postgres://user:pass@localhost:5432/tpch`

  It is possible to import from a file using the URI schemes `json:`, `jsonl:`, `csv:` depending on whether the data specified is encoded as JSON, JSON Lines or CSV respectively. For example, one could import data from a file `data.jsonl` in the current working directory by specifying `jsonl:data.json`. Note the lack of `//` that you may be used to seeing - this can be omitted as this URI will never have an authority component (unlike, for example, a database URI).

  Data can be imported from standard input by simply not specifying a path in the URI (e.g. `jsonl:` will read JSON Lines data directly from standard input). If no `--from` argument is specified, JSON data will read from standard input by default.

  When dealing with JSON Lines and not specifying a single collection with the `--collection` argument, each generated object is tagged with the name of the collection it was generated from. By default, this is done by adding a property `type` to the object (e.g. `"type": "collection_name"`). The name of this property can be changed using an additional parameter `collection_field_name` added at the end of the URI like so: `jsonl:file.jsonl?collection_field_name=foobar` - with this URI used with `--from`, generate objects will instead have a property like `"foobar": "collection_name"`.

  With regards to CSV importing/exporting, it is important to note that the URI path should specify a directory and not an individual file. This is because, unlike JSON and JSON Lines, a single CSV file cannot easily represent data from multiple collections so each collection's data is stored in a separate `.csv` file. Also, when importing CSV, Synth by default assumes that the input data will contain a header row, unless a `?header_row=false` argument is present at the end of the URI.

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
- `--to <uri>` - The generation destination specified using a URI (see `import --from` explanation above). If unspecified, generation defaults to stdout using JSON.
- `--seed <seed>` - An unsigned 64 bit integer seed to be used as a seed for generation. Defaults to 0 if unspecified.
- `--random` - A flag which toggles generation with a random seed. This cannot be used with --seed.
