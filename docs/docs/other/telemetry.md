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

Synth's telemetry collects 7 fields:

- `distinct_id`: A randomly generated UUID stored
  at `~/.config/synth/config.json`
- `command`: The command that was issued by the user. This is a text field whose
  value is one of the following:
  - `init`
  - `import`
  - `generate`
  - `telemetry::enabled`
  - `telemetry::disabled`
- `success`: If the command completed in success.
- `version`: The current semver of Synth. For example `v0.4.3`.
- `os`: The target platform for which the binary was built. This is the value
  of `cargo`'s `CARGO_CFG_TARGET_OS` environment variable under which `synth`
  was built in CI/CD. Currently, this is one of:
  - `linux`
  - `windows`
  - `macos`
- `timestamp`: The time at which the command was issued. For
  example `2021-05-06T16:13:40.084Z`.
- `generators`: A list of generators that were used by the `generate` command. This includes only string fakers and number ranges. For example `string::faker::title, number::i32::range`.

Below is the [Synth schema][synth-schema] of PostHog events posted by `synth`'s
activity:

```json synth
{
    "type": "object",
    "distinct_id": {
        "type": "string",
        "uuid": {}
    },
    "command": {
        "type": "string",
        "categorical": {
            "import": 1,
            "generate": 10,
            "init": 1,
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
              "string::faker::first_name": 2,
              "string::faker::last_name": 2,
              "string::faker::title": 2,
              "string::faker::suffix": 2,
              "string::faker::name": 2,
              "string::faker::name_with_title": 2,
              "string::faker::credit_card": 2,
              "string::faker::free_email_provider": 2,
              "string::faker::domain_suffix": 2,
              "string::faker::free_email": 2,
              "string::faker::safe_email": 2,
              "string::faker::username": 2,
              "string::faker::ipv4": 2,
              "string::faker::ipv6": 2,
              "string::faker::ip": 2,
              "string::faker::mac_address": 2,
              "string::faker::color": 2,
              "string::faker::user_agent": 2,
              "string::faker::rfc_status_code": 2,
              "string::faker::valid_status_code": 2,
              "string::faker::company_suffix": 2,
              "string::faker::company_name": 2,
              "string::faker::buzzword": 2,
              "string::faker::buzzword_muddle": 2,
              "string::faker::buzzword_tail": 2,
              "string::faker::catch_phrase": 2,
              "string::faker::bs_verb": 2,
              "string::faker::bs_adj": 2,
              "string::faker::bs_noun": 2,
              "string::faker::bs": 2,
              "string::faker::profession": 2,
              "string::faker::industry": 2,
              "string::faker::city_prefix": 2,
              "string::faker::city_suffix": 2,
              "string::faker::city_name": 2,
              "string::faker::country_name": 2,
              "string::faker::country_code": 2,
              "string::faker::street_suffix": 2,
              "string::faker::street_name": 2,
              "string::faker::time_zone": 2,
              "string::faker::state_name": 2,
              "string::faker::state_abbr": 2,
              "string::faker::secondary_address_type": 2,
              "string::faker::secondary_address": 2,
              "string::faker::zip_code": 2,
              "string::faker::post_code": 2,
              "string::faker::building_number": 2,
              "string::faker::latitude": 2,
              "string::faker::longitude": 2,
              "string::faker::phone_number": 2,
              "string::faker::cell_number": 2,
              "string::faker::file_path": 2,
              "string::faker::file_name": 2,
              "string::faker::file_extension": 2,
              "string::faker::dir_path": 2,
              "number::i32::range": 10,
              "number::u32::range": 10,
              "number::f32::range": 10,
              "number::i64::range": 10,
              "number::u64::range": 10,
              "number::f64::range": 10
            }
          }
        }
      }
    }
}
```

[synth-telemetry]: https://github.com/getsynth/synth/blob/master/synth/src/cli/telemetry.rs
[synth-installer]: https://github.com/getsynth/synth/blob/master/tools/install.sh
[synth-build]: https://github.com/getsynth/synth/blob/master/.github/workflows/release.yml
[synth-schema]: ../getting_started/schema.md
