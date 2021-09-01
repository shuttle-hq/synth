<p align=center>
  <img height="150px" src="docs/static/img/getsynth_identicon.png"/>
</p>
<p align=center>
  <b>The Declarative Data Generator</b>
</p>
<br>
<p align=center>
  <a href="https://getsynth.com/docs"><img alt="docs" src="https://img.shields.io/badge/doc-reference-orange"></a>
  <a href="https://github.com/getsynth/synth/blob/master/LICENSE"><img alt="license" src="https://img.shields.io/badge/license-Apache_2.0-green.svg"></a>
  <a href="https://github.com/getsynth/synth/search?l=rust"><img alt="language" src="https://img.shields.io/badge/language-Rust-orange.svg"></a>
  <a href="https://github.com/getsynth/synth/actions"><img alt="build status" src="https://img.shields.io/github/workflow/status/getsynth/synth/synth%20public%20cachix"/></a>
  <a href="https://discord.gg/H33rRDTm3p"><img alt="discord" src="https://img.shields.io/discord/803236282088161321?logo=discord"/></a>
      <img src="https://img.shields.io/github/all-contributors/getsynth/synth" alt="Synth open source contributors"/>
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

- **Semantic Data Types**: Synth has a library of semantic 'faker' types to cover PII like names, addresses, credit card numbers etc.

## Installation & Getting Started

On Linux and MacOS you can get started with the one-liner:

```bash
$ curl -sSL https://getsynth.com/install | sh
```

For more installation options, check out the [docs](https://getsynth.com/docs/getting_started/installation).

## Examples

### Building a data model from scratch

To start generating data without having a source to import from, you need to add Synth schema files to a namespace directory:

To get started we'll create a namespace directory for our data model and call it `my_app`:

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

You can use the `synth import` command to automatically generate Synth schema files from your Postgres, MySQL or MongoDB database:

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


## Contributors ✨

Thanks goes to these wonderful people ([emoji key](https://allcontributors.org/docs/en/emoji-key)):

<!-- ALL-CONTRIBUTORS-LIST:START - Do not remove or modify this section -->
<!-- prettier-ignore-start -->
<!-- markdownlint-disable -->
<table>
  <tr>
    <td align="center"><a href="https://github.com/christoshadjiaslanis"><img src="https://avatars.githubusercontent.com/u/14791384?v=4?s=100" width="100px;" alt=""/><br /><sub><b>Christos Hadjiaslanis</b></sub></a><br /><a href="#blog-christoshadjiaslanis" title="Blogposts">📝</a> <a href="#business-christoshadjiaslanis" title="Business development">💼</a> <a href="https://github.com/getsynth/synth/commits?author=christoshadjiaslanis" title="Code">💻</a> <a href="#content-christoshadjiaslanis" title="Content">🖋</a> <a href="#design-christoshadjiaslanis" title="Design">🎨</a> <a href="https://github.com/getsynth/synth/commits?author=christoshadjiaslanis" title="Documentation">📖</a> <a href="#fundingFinding-christoshadjiaslanis" title="Funding Finding">🔍</a> <a href="#ideas-christoshadjiaslanis" title="Ideas, Planning, & Feedback">🤔</a> <a href="#infra-christoshadjiaslanis" title="Infrastructure (Hosting, Build-Tools, etc)">🚇</a> <a href="#maintenance-christoshadjiaslanis" title="Maintenance">🚧</a> <a href="#platform-christoshadjiaslanis" title="Packaging/porting to new platform">📦</a> <a href="https://github.com/getsynth/synth/pulls?q=is%3Apr+reviewed-by%3Achristoshadjiaslanis" title="Reviewed Pull Requests">👀</a> <a href="#security-christoshadjiaslanis" title="Security">🛡️</a> <a href="https://github.com/getsynth/synth/commits?author=christoshadjiaslanis" title="Tests">⚠️</a> <a href="#talk-christoshadjiaslanis" title="Talks">📢</a></td>
    <td align="center"><a href="https://www.linkedin.com/in/ndaneliya/"><img src="https://avatars.githubusercontent.com/u/12720758?v=4?s=100" width="100px;" alt=""/><br /><sub><b>Nodar Daneliya</b></sub></a><br /><a href="#blog-NodarD" title="Blogposts">📝</a> <a href="#business-NodarD" title="Business development">💼</a> <a href="#content-NodarD" title="Content">🖋</a> <a href="#design-NodarD" title="Design">🎨</a> <a href="https://github.com/getsynth/synth/commits?author=NodarD" title="Documentation">📖</a> <a href="#fundingFinding-NodarD" title="Funding Finding">🔍</a> <a href="#ideas-NodarD" title="Ideas, Planning, & Feedback">🤔</a></td>
    <td align="center"><a href="https://llogiq.github.io"><img src="https://avatars.githubusercontent.com/u/4200835?v=4?s=100" width="100px;" alt=""/><br /><sub><b>llogiq</b></sub></a><br /><a href="#business-llogiq" title="Business development">💼</a> <a href="https://github.com/getsynth/synth/commits?author=llogiq" title="Code">💻</a> <a href="#content-llogiq" title="Content">🖋</a> <a href="#ideas-llogiq" title="Ideas, Planning, & Feedback">🤔</a> <a href="#infra-llogiq" title="Infrastructure (Hosting, Build-Tools, etc)">🚇</a> <a href="#maintenance-llogiq" title="Maintenance">🚧</a> <a href="#mentoring-llogiq" title="Mentoring">🧑‍🏫</a> <a href="https://github.com/getsynth/synth/pulls?q=is%3Apr+reviewed-by%3Allogiq" title="Reviewed Pull Requests">👀</a> <a href="#security-llogiq" title="Security">🛡️</a> <a href="https://github.com/getsynth/synth/commits?author=llogiq" title="Tests">⚠️</a></td>
    <td align="center"><a href="https://github.com/shkurskid"><img src="https://avatars.githubusercontent.com/u/77615792?v=4?s=100" width="100px;" alt=""/><br /><sub><b>Dmitri Shkurski</b></sub></a><br /><a href="https://github.com/getsynth/synth/commits?author=shkurskid" title="Code">💻</a></td>
    <td align="center"><a href="https://github.com/brokad"><img src="https://avatars.githubusercontent.com/u/13315034?v=4?s=100" width="100px;" alt=""/><br /><sub><b>Damien Broka</b></sub></a><br /><a href="#blog-brokad" title="Blogposts">📝</a> <a href="#business-brokad" title="Business development">💼</a> <a href="https://github.com/getsynth/synth/commits?author=brokad" title="Code">💻</a> <a href="#content-brokad" title="Content">🖋</a> <a href="#design-brokad" title="Design">🎨</a> <a href="https://github.com/getsynth/synth/commits?author=brokad" title="Documentation">📖</a> <a href="#fundingFinding-brokad" title="Funding Finding">🔍</a> <a href="#ideas-brokad" title="Ideas, Planning, & Feedback">🤔</a> <a href="#infra-brokad" title="Infrastructure (Hosting, Build-Tools, etc)">🚇</a> <a href="#maintenance-brokad" title="Maintenance">🚧</a> <a href="https://github.com/getsynth/synth/pulls?q=is%3Apr+reviewed-by%3Abrokad" title="Reviewed Pull Requests">👀</a> <a href="https://github.com/getsynth/synth/commits?author=brokad" title="Tests">⚠️</a></td>
    <td align="center"><a href="https://github.com/fretz12"><img src="https://avatars.githubusercontent.com/u/41805201?v=4?s=100" width="100px;" alt=""/><br /><sub><b>fretz12</b></sub></a><br /><a href="#ideas-fretz12" title="Ideas, Planning, & Feedback">🤔</a> <a href="https://github.com/getsynth/synth/commits?author=fretz12" title="Code">💻</a> <a href="https://github.com/getsynth/synth/commits?author=fretz12" title="Documentation">📖</a> <a href="https://github.com/getsynth/synth/commits?author=fretz12" title="Tests">⚠️</a></td>
    <td align="center"><a href="https://github.com/baile320"><img src="https://avatars.githubusercontent.com/u/26841355?v=4?s=100" width="100px;" alt=""/><br /><sub><b>Tyler Bailey</b></sub></a><br /><a href="https://github.com/getsynth/synth/commits?author=baile320" title="Code">💻</a> <a href="https://github.com/getsynth/synth/commits?author=baile320" title="Documentation">📖</a></td>
  </tr>
</table>

<!-- markdownlint-restore -->
<!-- prettier-ignore-end -->

<!-- ALL-CONTRIBUTORS-LIST:END -->

This project follows the [all-contributors](https://github.com/all-contributors/all-contributors) specification. Contributions of any kind welcome!
