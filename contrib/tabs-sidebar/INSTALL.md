# Installing the herdr-tabs helper

The helper runs the sidebar tabs branch next to an installed stable Herdr.
The test instance keeps its own config, server socket, sessions, saved
machines, and sidebar preferences under `~/.herdr-tabs`, so the stable
install is never touched.

## Prerequisites

- Rust toolchain via [rustup](https://rustup.rs).
- Zig 0.16.0 on your PATH. Check with `zig version`. On macOS,
  `brew install zig` works when Homebrew ships that exact version. Otherwise
  download 0.16.0 from https://ziglang.org/download/ and either add it to
  your PATH or set `ZIG=/path/to/zig` before building.
- A checkout of this repository. The script defaults to `~/src/herdr` and
  clones the fork there when the directory does not exist. Set
  `HERDR_TABS_REPO` to use another location.

## Install the script

macOS ships the BSD `install` utility, so this works as written. It copies the
file and sets the executable bit in one step. Create the target directory
first, because `install` does not create it.

```bash
mkdir -p ~/.local/bin
install -m 755 contrib/tabs-sidebar/herdr-tabs ~/.local/bin/herdr-tabs
```

Plain commands do the same:

```bash
mkdir -p ~/.local/bin
cp contrib/tabs-sidebar/herdr-tabs ~/.local/bin/herdr-tabs
chmod +x ~/.local/bin/herdr-tabs
```

Make sure `~/.local/bin` is on your PATH. The official Herdr installer already
adds it. Otherwise add this line to `~/.zshrc` and open a new terminal:

```bash
export PATH="$HOME/.local/bin:$PATH"
```

## First run

```bash
herdr-tabs build     # fetches the branch and builds target/release/herdr
herdr-tabs run       # starts the test UI with show_tabs enabled
```

In another terminal, create a named tab or connect a remote machine against
the test instance:

```bash
herdr-tabs cli tab create --label "#482 oauth callback"
herdr-tabs cli machine add workbox --label "Build machine"
```

The config file lives at `~/.herdr-tabs/config/herdr/config.toml`. Open it
with `herdr-tabs config`. Stop the test server with `herdr-tabs stop`, and
remove everything with `herdr-tabs clean`.

## Commands

| Command | Effect |
| --- | --- |
| `herdr-tabs build` | Fetch the branch and build a release binary |
| `herdr-tabs run [args]` | Start the test UI, or pass CLI arguments through |
| `herdr-tabs cli <args>` | Run any Herdr command against the test server |
| `herdr-tabs stop` | Stop the test server; this ends its pane processes |
| `herdr-tabs status` | Show test client and server status |
| `herdr-tabs config` | Open the test instance config in `$EDITOR` |
| `herdr-tabs clean` | Stop the server and delete `~/.herdr-tabs` |

Environment overrides: `HERDR_TABS_REPO`, `HERDR_TABS_HOME`, `HERDR_TABS_BRANCH`.
