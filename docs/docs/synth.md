---
id: synth
title: Synth
slug: /
---

Synth is a tool for generating realistic looking and anonymized synthetic data. Synth is database agnostic and can scale to millions of rows of data.

The key features of Synth are:
- **Synthetic Data as Code**: Data generation is described using a configuration language allowing you to specify your entire data model as code. Synthetic data as code enables you to peer review, version control and automate your synthetic data generation.
 
- **Data Inference**: Synth can ingest data from your primary data source and infer the structure of the data. Understanding the relationships, distributions and types of the underlying dataset.

- **Database Agnostic**: Synth supports semi-structured data and is database agnostic - playing nicely with SQL and NoSQL databases. 
 
- **Semantic Data Types**: Synth integrates with the (amazing) Python [Faker](https://pypi.org/project/Faker/) library, supporting generation of thousands of semantic types (e.g. credit card numbers, email addresses etc.) as well as locales.
  
## Concepts

This section covers the architecture of Synth, how it works under the hood, and it's features and capabilities.

Synth uses some novel ideas, so we would recommend starting with the [core concepts](core-concepts.md).


## Get Started

This section explains how to [easily get started with Synth](hello-world.md).

To see examples of Synth in action, see the [examples](examples/bank.md) section.
