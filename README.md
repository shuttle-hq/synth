<p align=center>
  <img height="128px" src="https://cdn.discordapp.com/icons/803236282088161321/fff7943ed3e3d656a4fb1fdb603d7e5d.png?size=128"/>
</p>
<p align=center>
  A declarative synthetic data engine for semi-structured data.
</p>
<br>
<p align=center>
  <a href="https://github.com/openquery-io/synth/blob/master/LICENSE"><img alt="license" src="https://img.shields.io/badge/license-Apache_2.0-green.svg"></a>
  <a href="https://github.com/openquery-io/synth/search?l=rust"><img alt="language" src="https://img.shields.io/badge/language-Rust-orange.svg"></a>
  <a href="https://github.com/openquery-io/synth/actions"><img alt="build status" src="https://img.shields.io/github/workflow/status/openquery-io/synth/docs"/></a>
  <a href="https://discord.gg/wwJVAFKKkq"><img alt="discord" src="https://img.shields.io/discord/803236282088161321?logo=discord"/></a>
</p>

------

Synth is a tool for generating realistic looking and anonymized synthetic data. Synth is database agnostic and can scale to millions of rows of data.

The key features of Synth are:
- **Synthetic Data as Code**: Data generation is described using a configuration language allowing you to specify your entire data model as code. Synthetic data as code enables you to peer review, version control and automate your synthetic data generation.
 
- **Data Inference**: Synth can ingest data from your primary data source and infer the structure of the data. Understanding the relationships, distributions and types of the underlying dataset.

- **Database Agnostic**: Synth supports semi-structured data and is database agnostic - playing nicely with SQL and NoSQL databases. 
 
- **Semantic Data Types**: Synth integrates with the (amazing) Python [Faker](https://pypi.org/project/Faker/) library, supporting generation of thousands of semantic types (e.g. credit card numbers, email addresses etc.) as well as locales.

## Installation & Getting Started

To get started quickly, check out the [docs](https://openquery-io.github.io/synth/)

## Why Rust

We decided to build Synth from the ground up in Rust. We love Rust, and given the scale of data we wanted `synth` to generate, it made sense as a first choice. The combination of memory safety, performance, expressiveness and a great community made it a no-brainer and we've never looked back!

## Get in touch

If you would like to learn more, or you would like support for your use-case, feel free to open an issue on Github.

If your query is more sensitive, you can email [opensource@getsynth.com](mailto:opensource@getsynth.com) and we'll happily chat about your usecase.

If you intend on using Synth, we would recommend joining our growing [Discord](https://discord.gg/wwJVAFKKkq) community.

## About Us

The Synth project is backed by OpenQuery. We are a [YCombinator](https://www.ycombinator.com/) backed startup based in London, England. We are passionate about data privacy, developer productivity, and building great tools for software engineers.

## License

Synth is source-available and licensed under the [Apache 2.0 License](https://github.com/openquery-io/synth/blob/master/LICENSE).

