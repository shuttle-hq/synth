# Bank Example

A small example demonstrating weighted variants.

## Demonstration

The number 1 have a weight of 99 while the number 2 have a weight of 1.

```bash
synth/examples/random_variants on  hbina-ISSUE-149-utilize-weights-in-oneof [?] on ☁️  (ap-southeast-1)
❯ cargo run --bin synth -- generate random --size 1 | jq | rg '1' | wc -l
warning: /home/hbina/git/synth/synth/Cargo.toml: unused manifest key: target.i686-pc-windows-msvc.rustflags
warning: /home/hbina/git/synth/synth/Cargo.toml: unused manifest key: target.x86_64-pc-windows-msvc.rustflags
    Finished dev [unoptimized + debuginfo] target(s) in 0.11s
     Running `/home/hbina/git/synth/target/debug/synth generate random --size 1`
9930

synth/examples/random_variants on  hbina-ISSUE-149-utilize-weights-in-oneof [?] on ☁️  (ap-southeast-1)
❯ cargo run --bin synth -- generate random --size 1 | jq | rg '2' | wc -l
warning: /home/hbina/git/synth/synth/Cargo.toml: unused manifest key: target.i686-pc-windows-msvc.rustflags
warning: /home/hbina/git/synth/synth/Cargo.toml: unused manifest key: target.x86_64-pc-windows-msvc.rustflags
    Finished dev [unoptimized + debuginfo] target(s) in 0.12s
     Running `/home/hbina/git/synth/target/debug/synth generate random --size 1`
70
```
