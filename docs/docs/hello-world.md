---
id: hello-world
title: Hello World
---

After installing the Synth cli, the next step is to create a `workspace`. To do this create a directory and run `synth init`.

```bash
mkdir my_synth_workspace && cd my_synth_workspace && synth init
```

This will create a `.synth` subdirectory in `my_synth_workspace`. Currently, this directory acts as simply an anchor to tell Synth that this is a workspace.

Next we create a namespace with a dummy collection:

```bash
mkdir my_app 
```
And then a file, `my_app/dummy.json` with the following content:

```json
{
    "type": "array",
    "length": {
        "type": "number",
        "subtype": "u64",
        "constant": 1
    },
   "content": {
        "type": "string",
        "pattern": "Hello world!"
    }
}
```

Finally, run `synth generate my_app/` to get your first 'Hello world!'.

## Running Synth in Daemon mode

Synth comes can be run in Daemon mode using the subcommand `synth-serve`. 

Synth exposes an HTTP RESTful API on port `8182` and create an internal state which is managed by a vesion controlled index.

It is preferable to use Daemon mode in the context of a collaborating team and comes with this handy [Python client](https://openquery-io.github.io/synthpy/)

More information on Daemon mode can be found [here](cli.md). 

## Examples

For more complex examples, see the [examples section](examples/bank.md).
