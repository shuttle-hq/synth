---
title: How it works
---

Synth's funcionality can be broken into 3 main parts:

1. Inference Engine: The inference engine is the process by which Synth ingests a datasets and attempts to infer and build the Synth [Schema](schema.md)
2. Schema (IR): The [Schema](schema.md) intermediate representation is a compact state representing the range of data generation
3. Generator Network: Schemas are transpiled into a network of generators which actually generate the required data.

Below is a high-level diagram illustrating the process:

![How it works](img/how_it_works.png)
