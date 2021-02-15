---
id: installation
title: Installation
---

import Tabs from '@theme/Tabs';
import TabItem from '@theme/TabItem';

<Tabs
  defaultValue="linux"
  values={[
    {label: 'Linux', value: 'linux'},
    {label: 'macOS', value: 'mac'},
    {label: 'Compile from source', value: 'source'},
    {label: 'Run in the Cloud Shell', value: 'gcp_shell'},
  ]}>
  
  <TabItem value='linux'>
  
Run the following command to install the `synth` binary:


```bash
curl --proto '=https' --tlsv1.2 -sSL https://sh.getsynth.com | sh
```

## Runtime Dependencies

You'll need some shared `python` libraries. 

If you get a run-time message around not having `libpython3.6m`, you can install the dependency by running:

```bash
sudo add-apt-repository ppa:deadsnakes/ppa \
&& sudo apt update \
&& sudo apt install libpython3.6-dev
```


  </TabItem>
  
  <TabItem value='mac'>
  
Run the following command to install the `synth` binary:


```bash
curl --proto '=https' --tlsv1.2 -sSL https://sh.getsynth.com | sh
```

## Runtime Dependencies
You'll need `python3` - if you don't have it already you can `brew install python3`.

  </TabItem>
  
  <TabItem value='source'>
To get started you need the Rust package manager `cargo`. If you don't have it, you can install Rust and Cargo using (this will also make nightly the default toolchain):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh && rustup default nightly
```

Next, install Synth using `cargo`:

```bash
cargo install --locked --git https://github.com/openquery-io/synth.git synth
```
:::note
If compilation fails, there are some dependencies required at compile time which you may not have installed: `sudo apt-get install libssl-dev libsqlite3-dev libpython3-dev`
:::
    
  </TabItem>
  
  <TabItem value='gcp_shell'>

<div align="center">
<a href="https://ssh.cloud.google.com/cloudshell/editor?cloudshell_git_repo=https://github.com/openquery-io/synth.git&cloudshell_print=tools/README-cloud-shell"><img alt="Run in Cloud Shell" src="https://storage.googleapis.com/gweb-cloudblog-publish/images/run_on_google_cloud.max-300x300.png"></img></a>
</div>

The run the following to install `synth` on the Cloud Shell:
```bash
./tools/init-cloud-shell && export PATH=$HOME/.local/bin:$PATH
```

  
  </TabItem>
</Tabs>

You can run `synth --version` to make sure the CLI installed correctly.

### Python Dependencies
Synth uses the Python [Faker](https://pypi.org/project/Faker/) library to generate different flavours of dummy data. To install Faker, run:

```bash
pip3 install Faker
```