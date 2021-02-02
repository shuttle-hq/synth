---
id: core-concepts
title: Core Concepts
---

This section covers the core concepts found in Synth.

## Workspaces

A workspace, marked by a `.synth/` subdirectory. A workspace represents a set of synthetic data namespaces managed by Synth.

A workspace can have *zero or more namespaces*, where the namespaces are just represented as subdirectories as well as some internal state in `.synth/`.

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

The **namespace** is the top-level abstraction in Synth. Namespaces are the equivalent of *Schemas* in SQL-land. Fields in a namespace can refer to other fields in a namespace - but you cannot reference data across namespaces.

## Collections

Every namespace has zero or more **collections**. Collections are addressable by their name (for example `my_collection`) and correspond to tables in SQL-land. Strictly speaking, Collections are a super-set of tables as they are in fact arbitrarily deep document trees.

## Schema

The schema is the core data structure that you need to understand to be productive with Synth. The schema represents your data model, it tells Synth exactly how to generate data, which fields we need, what types and so on. This is a little involved so there is a section devoted to just the [Schema](schema.md)

## Field References

Field References are a way to reference fields in the Schema. It's pretty intuitive.

To reference the `age` field in `users` we simply write `users.age`. This can go arbitrarily deep, so we can do `users.address.postcode` and we can even go through array variants like this, for example `users.friends.0`.

Field references can also be used inside a schema to specify things like Foreign Keys (more on this later).

## Importing Datasets

Synth can ingest and build data models (aka Synth Schemas) on the fly - assuming it is fed a syntactically correct JSON object.

You can use the `synth import` command to import data into a namespace.

Not only will Synth automatically *derive* the Schema for you, inferring the types and topology of the content graph. Synth will also automatically adjust the Schema as new information is ingested. For more on this, refer to the [inference](inference.md) page.

