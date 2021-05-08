---
title: Telemetry
---

Synth collects de-identified usage data to help create a better product and user experience.

Synth telemetry is **opt-in**; the user is explicity prompted at installation, asking if they would like to participate
in de-identified data collection.

## Philosophy

Below are a set of principles that guide the telemetry decisions made in Synth:

1. It is made *completely transparent* that telemetry is going to be installed
2. It is made *completely transparent* as to what data we collect
3. No personally identifiable information is collected. (IP addresses are discarded at the Sink)
4. Nothing is collected unless the user explicitly opts-in.

## Enabling / Disabling Telemetry

Synth uses a configuration file under the user configuration directory (e.g. `~/.config/synth/config.json` on typical unix systems) as a marker for telemetry being enabled. This file
also contains the user's randomly generated unique `distinct_id`.

The user is prompted during the installation process about telemetry being enabled.

Users can check if telemetry is enabled by running `synth telemetry status`.

Users can opt-out at any time by running `synth telemetry disable`.

## Where does the data go?

All the telemetry data is collected in a [Posthog](https://posthog.com/) instance run by the maintainers of Synth.

If for whatever reason you would like the data associated with your UUID to be deleted, please contact `opensource@getsynth.com`.

## What does Synth collect?

Synth's telemetry collects 6 fields:

- `distinct_id`: A randomly generated UUID stored at `~/.config/synth/config.json`
- `command`: The command that was issued by the user. One
  of (`init | import | generate | telemetry::enabled | telemetry::disabled`)
- `success`: If the command issued succeeded or failed.
- `version`: The version of Synth being used. For example `v0.4.3`.
- `os`: The host operating system, for example 'linux'
- `timestamp`: The time at which the command was issued. For example `2021-05-06T16:13:40.084Z`.

Below is a Synth schema generates telemetry events:

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
            "generate": 10
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
            "mac os": 1
        }
    },
    "timestamp": {
      "type": "string",
      "date_time": {
        "format": "%Y-%m-%dT%H:%M:%S",
        "subtype": "naive_date_time",
        "begin": "2015-01-01T00:00:00",
        "end": "2020-01-01T12:00:00"
      }
    }
}
```