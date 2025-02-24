# <h1 align="center"> An EigenLayer AVS 🌐 </h1>

**A simple Hello World AVS for EigenLayer with the BLS-based Contract Configuration**

## 📚 Overview

This project is about creating a simple Hello World AVS for EigenLayer.
An AVS (Actively Validated Service) is an off-chain service that runs arbitrary computations for a user-specified period of time.

## 📚 Prerequisites

Before you can run this project, you will need to have the following software installed on your machine:

- [Rust](https://www.rust-lang.org/tools/install)
- [Forge](https://getfoundry.sh)

You will also need to install [cargo-tangle](https://crates.io/crates/cargo-tangle), our CLI tool for creating and
deploying Blueprints:

To install the Tangle CLI, run the following command:

> Supported on Linux, MacOS, and Windows (WSL2)

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/tangle-network/gadget/releases/download/cargo-tangle-v0.1.2/cargo-tangle-installer.sh | sh
```

Or, if you prefer to install the CLI from crates.io:

```bash
cargo install cargo-tangle --force # to get the latest version.
```

## 🚀 Getting Started

Once `cargo-tangle` is installed, you can create a new project with the following command:

```sh
cargo tangle blueprint create --name <project-name> --eigenlayer <type>
```
where `<project-name>` is the name of the project that will be generated, and `<type>` is BLS or ECDSA. If you aren't sure which type to use, you likely want the default: BLS. After all, this is the template for BLS. If you don't specify a type, it will default to BLS.

Upon running the above command, you will be prompted with questions regarding the setup for your generated project. If you aren't sure for any of them, you can just hit enter to select the default for that questions. 

### Note
If Soldeer fails to update/install the necessary dependencies, you may need to run it manually with the following command:

```bash
forge soldeer update -d
```

## 📜 License

Licensed under either of

* Apache License, Version 2.0
  ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license
  ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## 📬 Feedback and Contributions

We welcome feedback and contributions to improve this blueprint.
Please open an issue or submit a pull request on
our [GitHub repository](https://github.com/tangle-network/blueprint-template/issues).

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
