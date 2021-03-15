---
title: Core concepts
---

This section covers the core concepts found in Synth.

## Workspaces

Workspaces are marked by `.synth/` subdirectory. A workspace represents a set of synthetic data namespaces managed by
Synth.

A workspace can have *zero or more namespaces*, where the namespaces are just represented as subdirectories (as well as
some hidden state in `.synth/` when using Synth in `daemon` mode). All information pertaining to a workspace is in its
directory.

Below is an example directory structure for a workspace with a single namespace, `my_namepace`.

```
├── .synth
│   ├── config.toml
│   └── db.sqlite
└── my_namespace
    ├── my_collection_1.json
    └── my_collection_2.json
``` 

## Namespaces

The **namespace** is the top-level abstraction in Synth. Namespaces are the equivalent of *Schemas* in SQL-land. Fields
in a namespace can refer to other fields in a namespace - but you cannot reference data across namespaces.

Namespaces are represented as sub-directories in your workspace. For example, a workspace with single namespace `some_namespace` would have the following structure:

```
├── .synth
│   ├── config.toml
│   └── db.sqlite
└── my_namespace
``` 

You can have as many namespaces as you like within a workspace, however they must have unique names:

```
├── .synth
│   ├── config.toml
│   └── db.sqlite
├── some_namespace
└── some_other_namespace 
```


## Collections

Every namespace has zero or more **collections**. Collections are addressable by their name and correspond to tables in SQL-land. Strictly speaking, Collections are a super-set of tables
as they are in fact arbitrarily deep document trees.

Collections are represented in a workspace as `json` files. The *name* of a collection is its filename (without the extension). For example a file `bank/transactions.json` defines a collection `transactions` in the namespace `bank`.
 
For a more comprehensive example, let's imagine our namespace `bank` has a collection `transactions` and another collection `users`. The workspace structure then looks like this:

```
├── .synth
│   ├── config.toml
│   └── db.sqlite
└── bank
    ├── transactions.json
    └── users.json 
```

Collections inside a namespace need to have unique names (you *can* however the same collection name spanning different namespaces, for example `bank/transactions.json` and `forex/transactions.json`)

## Field References

Field References are a way to reference fields in the Schema. It's pretty intuitive.

Field References take the form `<collection>.<field>.<field>...` (since Field References are confined to a namespace, the namespace is not specified in the reference.). Since collections in Synth are recursive and can be arbitrarily deep, so can field references be arbitrarily long.

For a concrete example from the `bank` namespace above; let's assume that our `users` collection has a field `id`. This field can then be referenced from anywhere inside the namespace using the reference `users.content.id`.

## Schema

The schema is the core data structure that you need to understand to be productive with Synth. The schema represents
your data model, it tells Synth exactly how to generate data, which fields we need, what types and so on. This is a
little involved so there is a section devoted to just the [Schema](schema.md).

## Importing Datasets

Synth can ingest and build data models (aka Synth Schemas) on the fly - assuming it is fed a syntactically correct JSON
object.

You can use the `synth import` command to import data into a namespace.

Not only will Synth automatically *derive* the Schema for you, inferring the types and topology of the content graph.
Synth will also automatically adjust the Schema as new information is ingested.

