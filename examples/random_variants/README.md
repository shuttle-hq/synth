# Bank Example

A small example demonstrating weighted variants.

## Demonstration

The number 1 has a weight of 99 while the number 2 has a weight of 1.
Therefore, we should expect a ratio of 99:1,

```bash
hbina@akarin:~/git/synth$ ./target/debug/synth generate examples/random_variants/random/ --size 1 --random  | jq | rg '1' | wc -l
9912
hbina@akarin:~/git/synth$ ./target/debug/synth generate examples/random_variants/random/ --size 1 --random  | jq | rg '2' | wc -l
118
```
