# Architecture

## Semantic shape

```text
two ends of a path, and exactly the command that joins them
    -> collection, queue, rehearsal, and transfer specifications
    -> attendance: what is running on a request's behalf, and one pass over it
    -> the additional sources a line may be claimed by, media, Youtube, and X among them
    -> derived extensions over primitive capabilities
    -> deterministic reference interpreters
    -> operating-system process, Reqwest HTTP, and filesystem production interpreters
    -> a Ratatui screen, and lines where there is no screen
    -> one executable choosing one interpreter of each
```

Transferring a path is what this application does. Every other reading of a submitted line exists
because a source claimed it: `SourceRecognitionAlg` states which lines a source retrieves, and it
is the whole of what a source adds to submission. Adding one is stating what it recognizes, what
retrieving it means, and how a run of that kind begins, is read, is held, and ends. Nothing in the
manager knows what any of them look like.

Specification crates state meaning and composition without selecting storage, network, process,
serialization, plugin, or asynchronous-runtime machinery. A concrete interpreter may choose those
mechanisms only after implementing the corresponding primitive capability.

## Dependency direction

```text
crates/rsynko
   |-> crates/rsynko-ratatui --> crates/rsynko-memory --\
   |-> crates/rsynko-process ---------------------------|
   `-> crates/rsynko-reqwest ---------------------------|
                                                        v
        rsynko-ui --> rsynko-manager --> rsynko-media --> rsynko-download
                            ^                  ^
        rsynko-rsync -------'                  `------- rsynko-yt

        rsynko-session, rsynko-x                depend on nothing here
        alux-ext, alux-sdk                      not in this workspace, taken from crates.io
```

Root packages are specifications by default. This includes specifications such as `rsynko-manager`
that compose meanings from other specifications. A package holds every specification that is used
with the others, one module per meaning: `rsynko-media` states observation, format, artifact,
extraction, selection, processing, output naming, and the program composing them. The `crates/`
directory contains only concrete interpreters, whose suffix identifies the lower-level choice they
make: memory, Reqwest, or Ratatui. No specification depends on an interpreter.

`rsynko-manager` associates identities, requests, entries, options, outputs, and collection
carriers without choosing their Rust layout. `crates/rsynko-memory` interprets those carriers as
`QueueId`, `SourceRequest`, `QueueEntry`, `TransferState`, and `ManagerState`; Ratatui consumes that
interpreter rather than importing a state representation from the specification.

`rsynko-rsync` states the transfer of one path, which is what the application is for: the two ends it joins, exactly the command that
joins them, and what each line a running transfer writes means. That path may be one file or a
whole folder, and a folder is one queue entry rather than a collection of files that happen to
travel together. Either way it is rehearsed before it is performed. `crates/rsynko-process` runs
that command as an operating-system process and decides nothing about it.

`rsynko-session` states what is *happening* rather than what is intended: which requests want work,
what beginning it is, what a run says while it runs, how it ends, and what its request wants of it
now. One derived pass reconciles the two, and the order that pass takes is its meaning. It depends
on no other package here, and `crates/rsynko-ratatui` interprets it once for each kind of work it
runs: downloads it retrieves itself, folder transfers another program performs, and source
inspections. Leaving is not a separate path: nothing is wanted once exit is requested, so the
ordinary pass ends every run.

`rsynko-ui` states the presentation of that manager: which keystroke denotes which intention, which
menu a page offers, how every page is composed, and the words and weights a reader reads. Its
`ScreenSyntax` is the vocabulary a renderer interprets, so `crates/rsynko-ratatui` supplies colors,
widgets, borders, layout, and crossterm decoding and states no page composition of its own.

## Public surfaces

Crate roots are the product surfaces. Engineering modules remain private and their intended values,
traits, programs, and extensions are re-exported explicitly. Every specification README is included
as crate documentation and its examples execute as doctests.

## Interpretation boundary

`rsynko-yt` owns URL recognition, exact request descriptions, player-response interpretation,
format ranking, and specialization of generic atomic download; `rsynko-x` owns the same for one
public tweet, down to which addresses name one and what request reads it. `crates/rsynko-reqwest`
merely executes those requests and moves their response bytes. It never infers a source's policy
from a host name or media URL. Reference downloads use in-memory resources, atomic publications, and event
traces. The runtime interprets the same meanings with Rustls HTTP and same-directory atomic rename.
Adaptive-media and ffmpeg adapters belong behind new capabilities and must not distort the existing
progressive-download program.
