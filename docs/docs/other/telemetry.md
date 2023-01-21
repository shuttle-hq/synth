---
title: Telemetry
---

Synth collects de-identified usage data to help create a better product and user
experience.

You can opt-out of Synth's anonymous data collection at any time by running the
command:

```bash
$ synth telemetry disable
```

or deleting the file at `~/.config/synth/config.json`.

Synth is completely free and open-source. This means all the code that provide
us with telemetry information is accessible by anyone. You can find
[the `telemetry.rs` submodule of Synth in our public repository][synth-telemetry].

The binary file that is downloaded by the [installer script][synth-installer] is
built transparently by [GitHub's CI/CD pipeline][synth-build] directly from
public releases.

## Philosophy

Below are a set of principles that guide the telemetry decisions made in Synth:

1. It is made *completely transparent* that telemetry is going to be installed
2. It is made *completely transparent* as to what data we collect
3. No personally identifiable information is collected. (IP addresses are
   discarded at the sink)
4. Nothing is collected unless it is explicitly and clearly documented here.

## Enabling / Disabling Telemetry

Synth uses a configuration file under the user configuration directory (
e.g. `~/.config/synth/config.json` on typical unix systems) as a marker for
telemetry being enabled. This file also contains a randomly generated
identifier. We use this identifier to better understand how we can improve the
user experience for Synth.

You can check if telemetry is enabled by running `synth telemetry status`.

You can opt-out at any time by running `synth telemetry disable`.

## Where does the data go?

All the telemetry data is collected in a [Posthog](https://posthog.com/)
instance run exclusively by the maintainers of Synth.

If you would like the data associated with your UUID to be deleted, please
contact `opensource@getsynth.com`.

## What does Synth collect?

Synth's telemetry collects 8 fields:

- `distinct_id`: A randomly generated UUID stored
  at `~/.config/synth/config.json`
- `command`: The command that was issued by the user. This is a text field whose
  value is one of the following:
  - `import`
  - `generate`
  - `telemetry::enabled`
  - `telemetry::disabled`
- `success`: If the command completed in success.
- `version`: The current semver of Synth. For example `v0.4.3`.
- `os`: The target platform for which the binary was built. This value is [std::env::consts::OS](https://doc.rust-lang.org/std/env/consts/constant.OS.html), a constant set at compile time. For example `windows`.
- `timestamp`: The time at which the command was issued. For
  example `2021-05-06T16:13:40.084Z`.
- `generators`: A list of generators that were used by the `generate` command. For example `string::title, number::I32::Range`.
- `integration`: The integration used in a `generate` or `import` command. For example `json`, `postgres`, etc.

Below is the [Synth schema][synth-schema] of PostHog events posted by `synth`'s
activity:

```json synth
{
  "type": "object",
  "distinct_id": {
      "type": "uuid"
  },
  "command": {
      "type": "string",
      "categorical": {
          "import": 1,
          "generate": 10,
          "telemetry::enabled": 10,
          "telemetry::disabled": 1
      }
  },
  "version": {
      "type": "string",
      "pattern": "v0\\.4\\.3"
  },
  "os": {
      "type": "string",
      "categorical": {
          "linux": 10,
          "macos": 10,
          "windows": 10
      }
  },
  "timestamp": {
    "type": "date_time",
    "format": "%Y-%m-%dT%H:%M:%S",
    "subtype": "naive_date_time",
    "begin": "2015-01-01T00:00:00",
    "end": "2020-01-01T12:00:00"
  },
  "generators": {
    "type": "string",
    "serialized": {
      "serializer": "json",
      "content": {
        "type": "array",
        "length": {
          "type": "number",
          "subtype": "u64",
          "range": {
            "low": 2,
            "high": 20
          }
        },
        "content": {
          "type": "string",
          "categorical": {
            "string::first_name": 2,
            "string::last_name": 2,
            "string::title": 2,
            "string::suffix": 2,
            "string::name": 2,
            "string::name_with_title": 2,
            "string::credit_card": 2,
            "string::free_email_provider": 2,
            "string::domain_suffix": 2,
            "string::free_email": 2,
            "string::safe_email": 2,
            "string::username": 2,
            "string::ipv4": 2,
            "string::ipv6": 2,
            "string::ip": 2,
            "string::mac_address": 2,
            "string::color": 2,
            "string::user_agent": 2,
            "string::rfc_status_code": 2,
            "string::valid_status_code": 2,
            "string::company_suffix": 2,
            "string::company_name": 2,
            "string::buzzword": 2,
            "string::buzzword_muddle": 2,
            "string::buzzword_tail": 2,
            "string::catch_phrase": 2,
            "string::bs_verb": 2,
            "string::bs_adj": 2,
            "string::bs_noun": 2,
            "string::bs": 2,
            "string::profession": 2,
            "string::industry": 2,
            "string::city_prefix": 2,
            "string::city_suffix": 2,
            "string::city_name": 2,
            "string::country_name": 2,
            "string::country_code": 2,
            "string::street_suffix": 2,
            "string::street_name": 2,
            "string::time_zone": 2,
            "string::state_name": 2,
            "string::state_abbr": 2,
            "string::secondary_address_type": 2,
            "string::secondary_address": 2,
            "string::zip_code": 2,
            "string::post_code": 2,
            "string::building_number": 2,
            "string::latitude": 2,
            "string::longitude": 2,
            "string::phone_number": 2,
            "string::cell_number": 2,
            "string::file_path": 2,
            "string::file_name": 2,
            "string::file_extension": 2,
            "string::dir_path": 2,
            "string::pattern": 2,
            "string::categorical": 2,
            "string::serialized": 2,
            "string::uuid": 2,
            "string::trancated": 2,
            "string::format": 2,
            "number::I32::Range": 10,
            "number::U32::Range": 10,
            "number::F32::Range": 10,
            "number::I64::Range": 10,
            "number::U64::Range": 10,
            "number::F64::Range": 10,
            "bool::constant": 3,
            "bool::frequency": 3,
            "bool::categorical": 3,
            "series::cyclical": 1,
            "series::poisson": 1,
            "series::incrementing": 1,
            "series::zip": 1,
            "null": 1,
            "date_time": 1,
            "array": 1,
            "object": 1,
            "same_as": 1,
            "one_of": 1,
            "unique": 1,
            "hidden": 1,
            "datasource": 1
          }
        }
      }
    }
  },
  "integration": {
    "type": "one_of",
    "variants": [
      {
        "weight": 0.5,
        "type": "string",
        "categorical": {
          "json": 1,
          "jsonl": 1,
          "csv": 1,
          "postgres": 1,
          "postgresql": 1,
          "mongodb": 1,
          "mysql": 1,
          "mariadb": 1
        }
      },
      {
        "weight": 0.5,
        "type": "null"
      }
    ]
  }
}
```

[synth-telemetry]: https://github.com/getsynth/synth/blob/master/synth/src/cli/telemetry.rs
[synth-installer]: https://github.com/getsynth/synth/blob/master/tools/install.sh
[synth-build]: https://github.com/getsynth/synth/blob/master/.github/workflows/release.yml
[synth-schema]: ../getting_started/schema.md
