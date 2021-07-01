---
title: Installation
---

import Tabs from '@theme/Tabs'; import TabItem from '@theme/TabItem';

<Tabs defaultValue="linux"
values={[
{label: 'Linux', value: 'linux'}, {label: 'macOS', value: 'mac'}, {label: 'Windows', value: 'windows'}, {label: 'Nix', value: 'nix'}, {label: 'Compile from source', value: 'cargo'},
]}>

<TabItem value='linux'>

Run the following command to install the `synth` binary:

```bash
curl --proto '=https' --tlsv1.2 -sSL https://sh.getsynth.com | sh
```

:::note
To skip the telemetry prompt (if you are installing Synth in CI for example) you can use the `--ci` flag.
:::

</TabItem>

<TabItem value='windows'>

To install on Windows, [download](https://github.com/getsynth/synth/releases/latest/download/synth-windows-latest-x86_64.exe) the Synth executable and run it from your `cmd` or `Git BASH` or `Windows PowerShell`.

Then copy the downloaded executable to a suitable folder (e.g. C:\synth\synth.exe).

Finally - [add Synth to your PATH](https://www.architectryan.com/2018/03/17/add-to-the-path-on-windows-10/) via your environment variables.

You should now be able to use synth:

```
PS C:\Users\user\workspace> synth --version
```

</TabItem>

<TabItem value='nix'>

If you happen to be running the [Nix](https://nixos.org/download.html#nix-quick-install) package manager or if you're on [NixOS](https://nixos.org/), you can use our automated Nix packaging that will set everything up for you.

:::note
We recommend you add [getsynth.cachix.org](https://app.cachix.org/cache/getsynth) to your list of binary caches. This will speed up your installation considerably by downloading [GitHub Actions build artifacts](https://github.com/getsynth/synth/actions/workflows/cachix.yml) instead of compiling everything locally.
:::

To install the latest released version of `synth` with `nix >= 2.4`, run:

```bash
nix-env -i -f https://github.com/getsynth/synth/releases/latest/download/install-nix
```

For versions of `nix < 2.4`, run:

```bash
SYNTH_TMP=$(mktemp); \
	curl -L --output - https://github.com/getsynth/synth/releases/latest/download/install-nix |\
	tar -xO > $SYNTH_TMP; \
	nix-env -i -f $SYNTH_TMP
```

</TabItem>

<TabItem value='cargo'>

To get started, make sure you have a recent version of the [Rust nightly toolchain](https://www.rust-lang.org/tools/install). Then run:

```bash
cargo +nightly install --locked --git https://github.com/getsynth/synth.git synth
```

:::note

If compilation fails, it may be because some required dependencies are not installed. On Ubuntu, you can try:

```
sudo apt-get install libssl-dev libsqlite3-dev
```
:::

</TabItem>
  
<TabItem value='mac'>

Run the following command to install the `synth` binary:

```bash
curl --proto '=https' --tlsv1.2 -sSL https://sh.getsynth.com | sh
```

:::note
To skip the telemetry prompt (if you are installing Synth in CI for example) you can use the `--ci` flag.
:::

</TabItem>

</Tabs>
