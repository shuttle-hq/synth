---
title: What is Synth?
---

Synth is a tool for generating realistic data using a declarative data model. Synth is database agnostic and can scale to millions of rows of data.

## Why Synth

Synth answers a simple question. There are so many ways to consume data, why are there no frameworks for *generating* data?

Synth provides a robust, declarative framework for specifying constraint based data generation, solving the following problems developers face on the regular:

1. You're creating an App from scratch and have no way to populate your fresh schema with correct, realistic data.
2. You're doing integration testing / QA on **production** data, but you know it is bad practice, and you really should not be doing that.
3. You want to see how your system will scale if your database suddenly has 10x the amount of data.

Synth solves exactly these problems with a flexible declarative data model which you can version control in git, peer review, and automate.

## Key Features

The key features of Synth are:

- **Data as Code**: Data generation is described using a declarative configuration language allowing you to specify your entire data model as code.

- **Import from Existing Sources**: Synth can import data from existing sources and automatically create data models. Synth currently has Alpha support for Postgres!
 
- **Data Inference**: While ingesting data, Synth automatically infers the relations, distributions and types of the dataset.

- **Database Agnostic**: Synth supports semi-structured data and is database agnostic - playing nicely with SQL and NoSQL databases.  
 
- **Semantic Data Types**: Synth integrates with the (amazing) Python [Faker](https://pypi.org/project/Faker/) library, supporting generation of thousands of semantic types (e.g. credit card numbers, email addresses etc.) as well as locales.