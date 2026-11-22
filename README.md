# NoctaliaRs

[![Stars](https://img.shields.io/gitea/stars/DontBlameMe/noctalia_rs?gitea_url=https%3A%2F%2Fcodeberg.org&style=for-the-badge&labelColor=%231e1e2e&color=%23f9e2af)](https://codeberg.org/DontBlameMe/noctalia_rs/stars)
[![Forks](https://img.shields.io/gitea/forks/DontBlameMe/noctalia_rs?gitea_url=https%3A%2F%2Fcodeberg.org&style=for-the-badge&labelColor=%231e1e2e&color=%2394e2d5)](https://codeberg.org/DontBlameMe/noctalia_rs/forks)
[![Latest Update](https://img.shields.io/gitea/last-commit/DontBlameMe/noctalia_rs?gitea_url=https%3A%2F%2Fcodeberg.org&include_prereleases&display_name=tag&style=for-the-badge&label=Latest+Update&labelColor=%231e1e2e&color=%23cba6f7)](https://codeberg.org/DontBlameMe/noctalia_rs)
[![NoGithub](https://img.shields.io/badge/Github-No-static?style=for-the-badge&labelColor=%231e1e2e&color=%23f38ba8)](https://nogithub.codeberg.page)
Unofficial project. It is not maintained by or affiliated with the Noctalia team.

Write Noctalia plugins in Rust. The crate generates `plugin.toml`, the Luau
entry stubs, the shared bridge, and translations. Plugin logic stays in Rust.

## Using the library

Add the crate to a plugin project:

```toml
[dependencies]
noctalia = "0.1"
serde_json = "1.0.154"
```

Declare entries and hand `main` to the library:

```rust
use noctalia::{run, Context, Meta, Op, Plugin, Result};

fn update(ctx: &Context) -> Result<Vec<Op>> {
    Ok(vec![Op::log(format!("{} tick", ctx.event))])
}

fn main() {
    let plugin = Plugin::new(Meta::new("me/hello", "Hello", "0.1.0"))
        .service("service", update);

    std::process::exit(i32::from(run(&plugin).success()));
}
```

`run` writes the plugin when called with `emit <dir> <helper>` and dispatches
host callbacks otherwise. Generated entry scripts call the helper with
`noctalia.runAsync`.

## Building a plugin with Nix

Import the flake and call `mkPlugin`:

```nix
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    noctalia-rs.url = "git+https://codeberg.org/DontBlameMe/noctalia_rs";
  };

  outputs = { self, nixpkgs, noctalia-rs }:
    let system = builtins.currentSystem;
    in {
      packages.${system}.default = noctalia-rs.mkPlugin.${system} {
        pname = "my-plugin";
        version = "0.1.0";
        src = ./.;
        cargoLock.lockFile = ./Cargo.lock;
        package = "my-plugin";
        helperName = "my-plugin-helper";
      };
    };
}
```

Run `nix build --out-link dist`. The output is a Noctalia local plugin source.

## Building the example

```sh
nix build --out-link dist
```

Add `./dist` as a local plugin source in Noctalia and enable
`dontblameme/timed`. The example covers a service, bar widget, launcher
provider, and panel.
