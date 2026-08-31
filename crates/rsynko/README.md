# rsynko

`rsynko` is the executable: the entry point where everything this workspace states separately is put together and starts running.

A specification states what a transfer is, what a collection of requests denotes, what a retrieval reads and chooses between — without knowing what performs any of it. An interpreter states how a process is started, how bytes are read and written, how a screen is drawn — without knowing what any of it is for. They never name each other, which is why neither is an application on its own.

This is where they meet. One interpreter is chosen for each specification — `rsynko-memory` for the manager's state, `rsynko-process` for transfers, `rsynko-reqwest` for the web and the filesystem, `rsynko-ratatui` for the screen — wired together, handed what was asked for, and set going. Before this file, everything is meaning; after it, something is happening.

Two decisions belong to it and to nowhere else: what was asked for on the command line, read once into a list of requests, and which interpreter can state it — a terminal that can be drawn on gets [`rsynko_ratatui::terminal`], and anything else states the same work as lines, one source at a time.

## Command line

```sh
rsynko [OPTIONS] [SOURCE]...
```

| Argument | Meaning |
| --- | --- |
| `SOURCE...` | Names the paths to transfer, or the web addresses a source retrieves, to add initially. A line naming two ends is one transfer, and a line nobody claims is a path. |
| `-o`, `--output <OUTPUT>` | Names the final output path instead of deriving it from the media title. It applies to one source, so naming several is refused rather than guessed at. |
| `--no-tui` | States the work as lines instead of a screen, which is also what a pipe or a redirect states. |
| `-h`, `--help` | Print help. |
| `-V`, `--version` | Print version. |

Named sources are what the collection starts with, not everything it can hold: more are added from the screen, which states what each one is doing while it does it. Naming none opens an empty collection.
