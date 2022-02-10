---
title: datasource
---

Synth's `datasource` generator is used to pull data from an external source. The data can be simple values like a string,
number, or booleans as well as complex values like an array or object.

The `path` option is a URI like the `--from` option for the [import command](/docs/getting_started/command-line#command-import).
Currently only JSON files are supported, and hence only the `json:` scheme is valid. For any file, the path is relative
to where `synth generate` is run from.

The `cycle` is optional and defaults to `false`. It allows you to read a datasource from the beginning once it has been
exhausted when set to `true`.

### JSON

When pulling from a JSON file, the JSON is expected to be an array with every item being the value for a single Synth
generator. The following is a valid JSON datasource:

<<<<<<< HEAD
```json
["21 Mary Street", "5 Diascia Avenue", "1062 Hill Crescent"]
=======
```json[addresses.json]
[
  "21 Mary Street",
  "5 Diascia Avenue",
  "1062 Hill Crescent"
]
>>>>>>> master
```

When generating more than 3 items `cycle` will need to be `true` for this datasource.

#### Example

```json synth
{
  "type": "datasource",
  "path": "json:addresses.json",
  "cycle": true
}
```
