# NoctaliaRs

Unofficial project. It is not maintained by or affiliated with the Noctalia team.

Write Noctalia plugins in Rust. The library generates `plugin.toml`, Luau entry
stubs, the shared bridge, and translations. Plugin logic stays in Rust.

## Features

- Plain or type-state metadata construction
- `#[entry]` attribute macro for service, widget, shortcut, launcher, desktop
  widget, panel, and layer entries
- Typed ops for common host calls and a generic `HostCall` escape hatch for any
  current or future host function
- Host reads such as config, colors, clipboard, calendar, media, system stats,
  compositor, and night light are available through `Context`
- All Noctalia setting types and entry kinds
- Nix builder exported as `mkPlugin.${system}`

## Using the library

```toml
[dependencies]
noctalia = "0.1"
serde_json = "1.0.154"
```

```rust
use noctalia::{entry, run, Context, MetaBuilder, Op, Plugin, Result};

#[entry(kind = "service", id = "service")]
fn update(ctx: &Context) -> Result<Vec<Op>> {
    Ok(vec![Op::log(format!("{} tick", ctx.event))])
}

fn main() {
    let meta = MetaBuilder::new()
        .id("me/hello")
        .name("Hello")
        .version("0.1.0")
        .build();
    let plugin = Plugin::new(meta).entry(update_entry());

    std::process::exit(i32::from(run(&plugin).success()));
}
```

`run` writes the plugin when called with `emit <dir> <helper>` and dispatches
host callbacks otherwise. Generated entry scripts call the helper with
`noctalia.runAsync`.

## Building a plugin with Nix

```nix
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    noctalia-rs.url = "path:/path/to/noctalia-rs";
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

Add `./dist` as a local plugin source in Noctalia and enable `iblameyou/example`.
The example exercises all entry kinds, settings, typed ops, and generic host
calls.
