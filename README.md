# Rsynko

[![Build and Test][ga-badge]][ga-url]
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

Rsynko is a meaning-first manager for path transfers. A transfer joins two ends — a folder or a file, here or on another machine — and is rehearsed before it is performed. `rsync` performs it, and the manager never learns what `rsync` is: what it knows is the command, which it states before it runs it.

A submitted line names a path unless a source recognizes it as its own, so `/home/dev/music`, `nas.local:/srv/data`, and `rsync://nas.local/data` are all transfers.

Retrieval is the additional source composed onto that: media from Youtube and X is downloaded the same way a path is transferred, from the same list. Thanks to [yt-dlp](https://github.com/yt-dlp/yt-dlp), from which the extraction algorithm for Youtube and X downloads is taken.

```text
┌──────────────────────────────────────────────────────────────────────────────────────────────────┐
│ rsynko  0.1.0                                                                                    │
└──────────────────────────────────────────────────────────────────────────────────────────────────┘
┌ Transfers (3) ───────────────────────────────────────────────────────────────────────────────────┐
│▸ ● ██▏       28% First fetched title                Downloading  18.2 MiB / 63.0 MiB  2.0 MiB/s  │
│  ● ████████ 100% Second fetched title               Complete     —                               │
│  ○            0% A portable video title             Ready        —                               │
│                                                                                                  │
│                                                                                                  │
└──────────────────────────────────────────────────────────────────────────────────────────────────┘
┌ Actions ─────────────────────────────────────────────────────────────────────────────────────────┐
│[↑↓] Select  [Enter] Details  [a] Add  [Space] Pause  [Del] Remove  [Ctrl+Q] Quit                 │
└──────────────────────────────────────────────────────────────────────────────────────────────────┘
```

Everything the manager states — the details page, the formats a source offers, what a transfer would change, the command it runs — is in [`rsynko-manager`](rsynko-manager/README.md).

> [!IMPORTANT]
> Rsynko is an extendable specification. A source is added by stating what it means, and an
> interpreter by stating how it is carried out; neither edits the other, and neither has to be
> foreseen. That is the expression problem gone rather than managed — and once it is gone,
> compositions that were not imaginable before become ordinary. Retrieval composed onto transfers,
> without either knowing about the other, is the first one.

## ⚙️ Usage

Interactive terminals open a collection/detail manager. Command-line arguments pre-populate its
collection; press `a` to add more and Space on each source to run it. Redirected output remains
line-oriented and acts on each supplied source immediately.

`Ctrl+Q` and `Ctrl+C` both quit, and both stop what is running before they do, so nothing is left behind either way. `Ctrl+C` has to be pressed twice, quickly, so hitting it by accident quits nothing. Escape navigates back and never quits.

```sh
rsynko
rsynko /home/dev/photos/2026
rsynko 'backup@nas.local:/volume1/photos  /home/dev/photos'
rsynko 'fixture://single-video'
rsynko 'https://www.youtube.com/watch?v=jNQXAC9IVRw'
rsynko 'https://x.com/somebody/status/1234567890123456789'
```

## 📦 Installation

> [!NOTE]
> Transferring a path needs `rsync` on the `PATH`, and a remote endpoint needs `ssh` the way
> `rsync` itself does. Retrieving a web address needs nothing but the executable.

**Take the built binary.** Every push builds `rsynko-x86_64-unknown-linux-gnu.tar.gz` beside its
`.sha256`, and a tagged `v*` push attaches both to a release — the newest one is always at
[releases/latest](../../releases/latest). Unpack it and put `rsynko` wherever the `PATH` reaches:

```sh
tar -xzf rsynko-x86_64-unknown-linux-gnu.tar.gz
install -Dm755 rsynko-x86_64-unknown-linux-gnu/rsynko ~/.local/bin/rsynko
```

**Build it.** With a Rust toolchain, `just install` states the same executable on the `PATH`, built
with the release profile:

```sh
just install                          # cargo install --path crates/rsynko
cargo build --release                 # target/release/rsynko, to place by hand
cargo run -- /home/dev/photos         # without installing anything
```

## 📖 Specifications

What something means, with no interpreter in it. These sit at the top level of the repository.

| Crate | | States |
| --- | --- | --- |
| [`rsynko-manager`](rsynko-manager) | [![crates.io][v-man]][c-man] [![docs.rs][d-man]][r-man] | The collection, its pages, and its queue |
| [`rsynko-ui`](rsynko-ui) | [![crates.io][v-ui]][c-ui] [![docs.rs][d-ui]][r-ui] | Keys, menus, and pages, with nothing drawing them |
| [`rsynko-session`](rsynko-session) | [![crates.io][v-ses]][c-ses] [![docs.rs][d-ses]][r-ses] | What is running, and one pass of attention |
| [`rsynko-rsync`](rsynko-rsync) | [![crates.io][v-rs]][c-rs] [![docs.rs][d-rs]][r-rs] | Path transfers and what one would change |
| [`rsynko-media`](rsynko-media) | [![crates.io][v-med]][c-med] [![docs.rs][d-med]][r-med] | Media: described, chosen, produced, named |
| [`rsynko-download`](rsynko-download) | [![crates.io][v-dl]][c-dl] [![docs.rs][d-dl]][r-dl] | Retrieval of one resource, published atomically |
| [`rsynko-yt`](rsynko-yt) | [![crates.io][v-yt]][c-yt] [![docs.rs][d-yt]][r-yt] | Youtube as a special case of retrieval |
| [`rsynko-x`](rsynko-x) | [![crates.io][v-x]][c-x] [![docs.rs][d-x]][r-x] | What a public tweet carries, and how to ask |

## 🔌 Interpretations

What a library makes of a specification. Each lives under `crates/` and is named for the choice it makes.

| Crate | | Interprets it as |
| --- | --- | --- |
| [`rsynko`](crates/rsynko) | [![crates.io][v-bin]][c-bin] [![docs.rs][d-bin]][r-bin] | the executable, composing the rest |
| [`rsynko-memory`](crates/rsynko-memory) | [![crates.io][v-mem]][c-mem] [![docs.rs][d-mem]][r-mem] | in-memory values the laws run against |
| [`rsynko-process`](crates/rsynko-process) | [![crates.io][v-proc]][c-proc] [![docs.rs][d-proc]][r-proc] | an operating-system process running `rsync` |
| [`rsynko-reqwest`](crates/rsynko-reqwest) | [![crates.io][v-req]][c-req] [![docs.rs][d-req]][r-req] | Reqwest requests and published files |
| [`rsynko-ratatui`](crates/rsynko-ratatui) | [![crates.io][v-rat]][c-rat] [![docs.rs][d-rat]][r-rat] | a terminal screen, and keys read back |

## 🔮 Wishlist

* [ ] Better text editing in the input, output, and add-sources editors
* [ ] Scrolling in the collection and the report, so a long list is not cut to the pane
* [ ] A selectable folder filter per transfer, stated in `rsync`'s own filter-file language
* [ ] Every file a tweet carries, rather than one of them — `Take::Everything` and `Take::Images` are stated in `rsynko-x` and nothing consumes them yet
* [ ] Youtube's adaptive streams taken together: signature deciphering, proof-of-origin tokens, and a merge
* [ ] A collection remembered between runs
* [ ] Binaries beyond `x86_64-unknown-linux-gnu`

## 🧠 Design

Read [`DENOTATIONAL_DESIGN.md`](DENOTATIONAL_DESIGN.md) for semantic authority and
[`ARCHITECTURE.md`](ARCHITECTURE.md) for dependency and interpretation boundaries before adding
implementation machinery.

## 📤 Publication

Packages are licensed under MIT. Publish them in this order, allowing the crates.io index to update
between dependent packages:

1. `rsynko-download`, `rsynko-session`, and `rsynko-x` in any order
2. `rsynko-media`
3. `rsynko-manager`
4. `rsynko-rsync` and `rsynko-ui` in any order
5. `rsynko-yt`
6. `rsynko-memory`
7. `rsynko-process`, `rsynko-reqwest`, and `rsynko-ratatui` in any order
8. `rsynko`

Cargo cannot package a later step against crates.io until the preceding version is available there.

Each crate carries its own version, so a release states only the crates that changed:
`just release-crate <crate> <level>` for one, `just release <level>` for the whole workspace. Every
release tags `<crate>-v<version>`, and `rsynko-v<version>` is what builds and attaches the
executable.

## 📄 License

[MIT license](LICENSE)

[v-man]: https://img.shields.io/crates/v/rsynko-manager
[c-man]: https://crates.io/crates/rsynko-manager
[d-man]: https://docs.rs/rsynko-manager/badge.svg
[r-man]: https://docs.rs/rsynko-manager
[v-ui]: https://img.shields.io/crates/v/rsynko-ui
[c-ui]: https://crates.io/crates/rsynko-ui
[d-ui]: https://docs.rs/rsynko-ui/badge.svg
[r-ui]: https://docs.rs/rsynko-ui
[v-ses]: https://img.shields.io/crates/v/rsynko-session
[c-ses]: https://crates.io/crates/rsynko-session
[d-ses]: https://docs.rs/rsynko-session/badge.svg
[r-ses]: https://docs.rs/rsynko-session
[v-rs]: https://img.shields.io/crates/v/rsynko-rsync
[c-rs]: https://crates.io/crates/rsynko-rsync
[d-rs]: https://docs.rs/rsynko-rsync/badge.svg
[r-rs]: https://docs.rs/rsynko-rsync
[v-med]: https://img.shields.io/crates/v/rsynko-media
[c-med]: https://crates.io/crates/rsynko-media
[d-med]: https://docs.rs/rsynko-media/badge.svg
[r-med]: https://docs.rs/rsynko-media
[v-dl]: https://img.shields.io/crates/v/rsynko-download
[c-dl]: https://crates.io/crates/rsynko-download
[d-dl]: https://docs.rs/rsynko-download/badge.svg
[r-dl]: https://docs.rs/rsynko-download
[v-yt]: https://img.shields.io/crates/v/rsynko-yt
[c-yt]: https://crates.io/crates/rsynko-yt
[d-yt]: https://docs.rs/rsynko-yt/badge.svg
[r-yt]: https://docs.rs/rsynko-yt
[v-x]: https://img.shields.io/crates/v/rsynko-x
[c-x]: https://crates.io/crates/rsynko-x
[d-x]: https://docs.rs/rsynko-x/badge.svg
[r-x]: https://docs.rs/rsynko-x
[v-mem]: https://img.shields.io/crates/v/rsynko-memory
[c-mem]: https://crates.io/crates/rsynko-memory
[d-mem]: https://docs.rs/rsynko-memory/badge.svg
[r-mem]: https://docs.rs/rsynko-memory
[v-proc]: https://img.shields.io/crates/v/rsynko-process
[c-proc]: https://crates.io/crates/rsynko-process
[d-proc]: https://docs.rs/rsynko-process/badge.svg
[r-proc]: https://docs.rs/rsynko-process
[v-req]: https://img.shields.io/crates/v/rsynko-reqwest
[c-req]: https://crates.io/crates/rsynko-reqwest
[d-req]: https://docs.rs/rsynko-reqwest/badge.svg
[r-req]: https://docs.rs/rsynko-reqwest
[v-rat]: https://img.shields.io/crates/v/rsynko-ratatui
[c-rat]: https://crates.io/crates/rsynko-ratatui
[d-rat]: https://docs.rs/rsynko-ratatui/badge.svg
[r-rat]: https://docs.rs/rsynko-ratatui
[v-bin]: https://img.shields.io/crates/v/rsynko
[c-bin]: https://crates.io/crates/rsynko
[d-bin]: https://docs.rs/rsynko/badge.svg
[r-bin]: https://docs.rs/rsynko

[ga-badge]: https://github.com/tgrospic/rsynko/actions/workflows/rust.yml/badge.svg?branch=master
[ga-url]: https://github.com/tgrospic/rsynko/actions?query=branch:master
