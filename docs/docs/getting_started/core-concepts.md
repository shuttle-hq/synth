---
title: Core concepts
---

This section covers the core concepts found in Synth.

## Workspaces

Workspaces are marked by the existence of a `.synth/` subdirectory. A workspace
represents a set of synthetic data namespaces managed by Synth.

A workspace can have *zero or more namespaces*. Each namespace is represented as
a subdirectory of the workspace. All information pertaining to a workspace is in
its directory, there is no external configuration.

Below is an example directory structure for a workspace with a single
namespace, `my_namepace`.

```
├── .synth
└── my_namespace
    ├── my_collection_1.json
    └── my_collection_2.json
``` 

## Namespaces

The **namespace** is the top-level abstraction in Synth. Namespaces are the
equivalent of traditional [schemas][sql-schemas] in the world of relational
databases likes PostgreSQL. [References](#field-references) can exist between
fields in a given namespace, but never across namespaces.

Namespaces are represented as sub-directories in a workspace. For example, a
workspace with single namespace `some_namespace` would have the following
structure:

```
├── .synth
└── my_namespace
``` 

You can have as many namespaces as you like within a workspace:

```
├── .synth
├── some_namespace
└── some_other_namespace 
```

## Collections

Every namespace has zero or more **collections**. Collections are addressable by
their name and correspond to [tables][sql-tables] in the world of relational
databases. Strictly speaking, collections are a super-set of tables as they are
in fact arbitrarily deep JSON document trees.

Collections are represented in a workspace as JSON files. The *name* of a
collection (the way it is referred to by [`synth`][synth]) is its filename
without the extension. For example the file `bank/transactions.json` defines a
collection named `transactions` in a namespace `bank`.

For a more comprehensive example, let's imagine our namespace `bank` has a
collection `transactions` and another collection `users`. The workspace
structure then looks like this:

```
├── .synth
└── bank
    ├── transactions.json
    └── users.json 
```

Collections inside a given namespace need to have unique names. You can however
have the same collection name in different namespaces. For
example `bank/transactions.json` and `forex/transactions.json` is a valid
workspace.

Collections must be valid instances of the [`synth` schema][schema] that
describe an array. This means at the top-level all collections must
be [array generators][array-generators].

## Field references

A field reference is a special kind of fields that is useful for declaring
relations between different parts of a collection or different collections in
the same namespace.

A field reference can be specified by using the [same_as][same-as] generator
type.

The value of the `"ref"` field should be the address of the field you want to
refer to. A field address takes the
form `<collection name>.<level_0>.<level_1>...`. For example, say we have a
collection `users.json` containing the following schema:

```json
{
  "type": "array",
  "length": {
    "type": "number",
    "subtype": "u64",
    "range": {
      "low": 1,
      "high": 4,
      "step": 1
    }
  },
  "content": {
    "type": "object",
    "username": {
      "type": "string",
      "faker": {
        "generator": "username"
      }
    },
    "credit_card": {
      "type": "string",
      "faker": {
        "generator": "credit_card"
      }
    },
    "id": {
      "type": "number",
      "subtype": "u64",
      "id": {}
    }
  }
}
```

A reference to the `username` field would have the
address `users.content.username`. If we want to add a reference to this field
from another collection we would simply add:

```json
{
  ...
  "content": {
    ...
    "username": {
      "type": "same_as",
      "ref": "users.content.username"
    },
    ...
  }
}
```

## Schema

The schema is the core data structure that you need to understand to be
productive with Synth. The schema represents your data model, it tells Synth
exactly how to generate data, which fields we need, what types and so on. This
is a little involved so there is a section devoted to just the [schema][schema].

## Importing datasets

Synth can ingest and build schemas on the fly with
the [`synth import`][synth-import] command.

## Generating data

To generate data from an existing namespace use
the [`synth generate`][synth-generate] command.

[`synth`][synth] uses a seedable pseudo-random source of entropy. By default,
the seed is set to a constant value of `0` using the
Rust-native [`rand::SeedableRng::seed_from_u64`][seedable-rng] function. This
means that, by default, the data that [`synth`][synth] generates is
deterministic: it is only a function of your schema files.

This behavior can be tuned (and the seed be changed, or randomized) using
the `--seed` or `--random` flag.

[synth]: cli.md

[sql-schemas]: https://www.postgresql.org/docs/9.1/ddl-schemas.html

[sql-tables]: https://www.postgresql.org/docs/9.1/sql-createtable.html

[same-as]: /content/same-as

[schema]: schema.md

[array-generators]: /content/array

[same-as]: /content/same-as

[synth-import]: cli.md#command-import

[synth-generate]: cli.md#command-generate

[seedable-rng]: https://docs.rs/rand/0.8.4/rand/trait.SeedableRng.html#method.seed_from_u64