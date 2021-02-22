---
title: An hello world
---

After installing the Synth CLI, the next step is to create a *workspace*. Workspaces are directories in your filesystem
that Synth uses to read your schemas from.

To create and initialise a workspace called `synth_workspace` in your current working directory, run:

```bash
$ mkdir synth_workspace && cd synth_workspace && synth init
```

:::note Note

The command `synth init` creates a marker directory called `.synth` in your present working directory. This marker
directory acts as simply an anchor to tell Synth that this is a workspace.
:::

Next we need to create a *namespace*. Namespaces are collections of schema files which are allowed to refer to one
another. They are organized simply by creating directories in your workspace. Let's create a namespace
called `my_namespace`:

```bash
mkdir my_namespace
```

Finally, we need to add a *collection* to our namespace. Collections are JSON files which describe the "shape" of the
data we want to generate. They follow the Synth [schemas][schema] format.

To create a collection called "dummy" in our namespace, simply copy/paste the content of the following example in a file
at `synth_workspace/my_namespace/dummy.json`:

```json synth
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

The previous example snippet is an example of the Synth [schema][schema] format. All the examples of the
Synth [schema][schema] in these documentation pages are tagged with a "**Run**" button that lets you preview Synth
output data when you click it.

Finally, run
```bash
$ synth generate my_namespace/
```
and you should see an output very close to the output of the above snippet.

## Where to go from here
* Take a look at the exhaustive [generators reference](/content/null).
* Go into how Synth works by looking at the [core concepts](core-concepts) and the Synth [schema](schema) format.
* For more complex real life examples, see the [examples](/examples/bank) section.

[schema]: schema