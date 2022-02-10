---
title: date_time
---

A `date_time` is a generator type that generates values from keys that specify a date/time format, subtype, start time,
and end time. This allows one to, for example, generate valid days of the year for an `updated_at` column, generate a
valid [RFC 2822](https://tools.ietf.org/html/rfc2822) timestamp for an email header field, etc.

#### Example

```json synth
{
  "type": "date_time",
  "format": "%Y-%m-%d",
  "subtype": "naive_date",
  "begin": "2020-01-01",
  "end": "2025-01-01"
}
```

Accepted keys and values that can be contained in a `date_time` generator are as follows:

- `"format"`: a [strftime](https://docs.rs/chrono/0.4.19/chrono/format/strftime/index.html)-style parameter specifying
  the string formatting of the underlying `date_time` value.
- `"subtype"`: one of the following
  - `"naive_date"`: indicates the `date_time` value should be a simple `date` without timezone specification,
  - `"naive_time"`: indicates the `date_time` value should be a simple `time` without timezone specification,
  - `"naive_date_time"`: indicates the `date_time` value should be a combined `date` and `time` without timezone
    specification,
  - `"date_time"`: indicates the `date_time` value should be a combined `date` and `time` _with_ timezone specification.
- `"begin"` and `"end"`: the lower and upper bounds of the `date_time` value to generate. The formatting of these values
  must adhere to the `strftime`-string specified in the `"format"` field.

Not specifying `begin` or `end` will result in these defaulting to the current time.

```json synth
{
  "type": "date_time",
  "format": "%Y-%m-%d",
  "subtype": "naive_date",
  "end": "2030-01-01"
}
```

Or optionally both, will result in a constant time:

```json synth
{
  "type": "date_time",
  "format": "%Y-%m-%d",
  "subtype": "naive_date"
}
```

#### Example

```json synth
{
  "type": "date_time",
  "format": "%Y-%m-%dT%H:%M:%S",
  "subtype": "naive_date_time",
  "begin": "2015-01-01T00:00:00",
  "end": "2020-01-01T12:00:00"
}
```
