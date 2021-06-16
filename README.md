<p align=center>
  <img height="150px" src="docs/static/img/getsynth_identicon.png"/>
</p>
<p align=center>
  <b>The Declarative Data Generator</b>
</p>
<br>
<p align=center>
  <a href="https://getsynth.github.io/synth"><img alt="docs" src="https://img.shields.io/badge/doc-reference-orange"></a>
  <a href="https://github.com/getsynth/synth/blob/master/LICENSE"><img alt="license" src="https://img.shields.io/badge/license-Apache_2.0-green.svg"></a>
  <a href="https://github.com/getsynth/synth/search?l=rust"><img alt="language" src="https://img.shields.io/badge/language-Rust-orange.svg"></a>
  <a href="https://github.com/getsynth/synth/actions"><img alt="build status" src="https://img.shields.io/github/workflow/status/getsynth/synth/synth%20public%20cachix"/></a>
  <a href="https://discord.gg/H33rRDTm3p"><img alt="discord" src="https://img.shields.io/discord/803236282088161321?logo=discord"/></a>
  <a href="https://ssh.cloud.google.com/cloudshell/editor?cloudshell_git_repo=https://github.com/getsynth/synth.git&cloudshell_print=tools/README-cloud-shell"><img alt="Run in Cloud Shell" src="https://img.shields.io/static/v1?label=GCP&message=Run%20in%20Cloud%20Shell&color=4394ff&logo=google-cloud&logoColor=4d9aff"></a>
</p>

------

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

## Installation & Getting Started

To get started quickly, check out the [docs](https://getsynth.github.io/synth).

## Examples

### Building a data model from scratch

To start generating data without having a source to import from, you need to first initialise a workspace using `synth init`:

```bash
$ mkdir workspace && cd workspace && synth init
```

Inside the workspace we'll create a namespace for our data model and call it `my_app`:

```bash
$ mkdir my_app
```

Next let's create a `users` collection using Synth's configuration language, and put it into `my_app/users.json`:

```json
{
    "type": "array",
    "length": {
        "type": "number",
        "constant": 1
    },
    "content": {
        "type": "object",
        "id": {
            "type": "number",
            "id": {}
        },
        "email": {
            "type": "string",
            "faker": {
                "generator": "safe_email"
            }
        },
        "joined_on": {
            "type": "string",
            "date_time": {
                "format": "%Y-%m-%d",
                "subtype": "naive_date",
                "begin": "2010-01-01",
                "end": "2020-01-01"
            }
        }
    }
}
```

Finally, generate data using the `synth generate` command:

```bash
$ synth generate my_app/ --size 2 | jq
{
  "users": [
    {
      "email": "patricia40@jordan.com",
      "id": 1,
      "joined_on": "2014-12-14"
    },
    {
      "email": "benjamin00@yahoo.com",
      "id": 2,
      "joined_on": "2013-04-06"
    }
  ]
}
```


### Building a data model from an external database

If you have an existing database, Synth can automatically generate a data model by inspecting the database. 

To get started, initialise your Synth workspace locally:

```bash
$ mkdir synth_workspace && cd synth_workspace && synth init
```

Then use the `synth import` command to build a data model from your Postgres or MongoDB database:

```bash
$ synth import tpch --from postgres://user:pass@localhost:5432/tpch
Building customer collection...
Building primary keys...
Building foreign keys...
Ingesting data for table customer...  10 rows done.
```

Finally, generate data into another instance of Postgres:

```bash
$ synth generate tpch --to postgres://user:pass@localhost:5433/tpch
```

## Why Rust

We decided to build Synth from the ground up in Rust. We love Rust, and given the scale of data we wanted `synth` to generate, it made sense as a first choice. The combination of memory safety, performance, expressiveness and a great community made it a no-brainer and we've never looked back!

## Get in touch

If you would like to learn more, or you would like support for your use-case, feel free to open an issue on Github.

If your query is more sensitive, you can email [opensource@getsynth.com](mailto:opensource@getsynth.com) and we'll happily chat about your usecase.

If you intend on using Synth, we would recommend joining our growing [Discord](https://discord.gg/H33rRDTm3p) community.

## About Us

The Synth project is backed by OpenQuery. We are a [YCombinator](https://www.ycombinator.com/) backed startup based in London, England. We are passionate about data privacy, developer productivity, and building great tools for software engineers.

## Contributing

First of all, we sincerely appreciate all contributions to Synth, large or small so thank you.

See the [contributing](./CONTRIBUTING.md) section for details.

## License

Synth is source-available and licensed under the [Apache 2.0 License](https://github.com/getsynth/synth/blob/master/LICENSE).

