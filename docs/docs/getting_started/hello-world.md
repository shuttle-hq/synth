---
title: Hello world
---

After installing [`synth`][synth], the next step is to create a **workspace**. 

Workspaces are directories in your filesystem that [`synth`][synth] uses to read your schemas from. Currently [`synth`][synth] reads schemas written in a specialized JSON data model. You can find out everything there is to know about [`synth`][synth] schemas in the [Generators][generators] section or in the [Schema][schema] section. In this section we will show you how to set up a simple "hello world" data generator.

To create and initialise a workspace called `synth_workspace` in your current working directory, run:

```bash
mkdir synth_workspace && cd synth_workspace && synth init
```

:::note Note

The command [`synth init`][synth-init] creates a marker directory called `.synth` in the directory where it is run. This marker directory acts as simply an anchor to tell [`synth`][synth] that this is a workspace.
:::

Next we need to create a **namespace**. Namespaces are directories in an
initialized [`synth`][synth] workspace. All the schema files in a given
namespace are collated and compiled together at runtime.

Let's create a namespace called `my_namespace`:

```bash
mkdir my_namespace
```

Finally, we need to add a **collection** to our namespace. Collections describe
the "shape" of the data we want to generate. They are individual JSON files
within a namespace written according to the [`synth` schema][generators].

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

The previous example snippet is an example of
a [`synth` schema][schema]. All such examples in these pages
are tagged with a "**Run**"
button that lets you preview how [`synth`][synth] would output the corresponding
data.

Finally, run
```bash
synth generate my_namespace/
```
and you should see an output very close to the output of the snippet.

## Where to go from here
* Take a look at the exhaustive [generators reference][generators].
* Go deeper into how [`synth`][synth] works by looking at the [core concepts][core-concepts] and the specifications of the [schema][schema].
* For more complex real life examples, see the [examples][examples] section.

[synth]: cli.md
[synth-init]: cli.md#command-init
[schema]: schema.md
[generators]: /content/object.md
[core-concepts]: core-concepts.md
[examples]: /examples/bank.md
