# rsynko-manager

`rsynko-manager` defines a renderer-neutral transfer manager. It owns the source collection, stable queue identities, list selection, details navigation, breadcrumbs, add-source editing, scheduling intentions, keyed transfer state, the record of what was observed about each request, diagnostics, and safe-exit meaning. It also owns the pre-start format choice, source-format discovery, first-start immutability, and fresh-request duplication.

How that state is presented — the key map, the menu, the composition of every page, and the vocabulary a reader reads — is a separate specification in `rsynko-ui`, and Ratatui is the first interpreter of both. Future web, desktop, mobile, and accessibility interfaces can interpret the same manager state without redefining application behavior.

## Specification surface

The specification begins with associated meanings rather than a storage layout:

- [`ManagerSorts::Downloads`] represents a downloads collection chosen by an interpreter;
- [`ManagerSorts::Source`] represents a source request chosen by an interpreter;
- [`ManagerSorts::Options`] and [`ManagerSorts::Output`] represent choices without fixing their storage;
- [`DownloadsAlg::empty_downloads`] defines the empty collection;
- [`SourceRequestAlg::source`] defines one request;
- [`DownloadCollectionAlg::add_sources`] denotes ordered collection composition.

[`DownloadsExt::downloads`] is the derived construction. Its bounds are the specification:

```rust ignore
This: DownloadsAlg + SourceRequestAlg
This::Downloads: DownloadCollectionAlg<Source = This::Source>
```

The associated-type equality states that the requests constructed by the ambient vocabulary are exactly the requests accepted by its collection carrier. A specification is therefore an ordinary generic function composed through the extension; it does not allocate or mutate a concrete manager record:

```rust
use alux_ext::ext;
use rsynko_manager::{
    DownloadCollectionAlg, DownloadsAlg, MediaOptionsAlg, OutputChoiceAlg,
    ProgressiveDownloadsExt, SourceRequestAlg,
};

#[ext(name = ExampleDownloadsExt)]
impl<This, Options, Output> This
where
    This: DownloadsAlg
        + SourceRequestAlg<Options = Options, Output = Output>
        + MediaOptionsAlg<Options = Options>
        + OutputChoiceAlg<Output = Output>,
    This::Downloads: DownloadCollectionAlg<Source = This::Source>,
{
    fn example_downloads(&self) -> This::Downloads {
        self.progressive_downloads([
            "fixture://single-video",
            "https://www.youtube.com/watch?v=VIDEO_ID",
        ])
    }
}
```

Different interpreters may represent that result as an in-memory state machine, a persistent command value, a remote application request, or a test trace. `ManagerIntent` is separately reified first-order syntax because interactive interpreters must inspect and dispatch intentions received over time. [`ManagerIntentExt`] is only the event fold. Collection navigation, expanded-row navigation, choice editing, output editing, draft submission, and queue lifecycle are separate extensions with independent primitive bounds; renderers and tests can reuse any one meaning without acquiring the entire manager vocabulary.

## Application map

```text
Transfers                         collection page
├── Add sources                   editor page
└── <selected source>             expanded collection row
    ├── Transfer | Formats        what the request chooses between
    ├── Input                     input editor, until the transfer is performed
    ├── Output | File name        where what is retrieved comes to rest
    ├── Report                    what a transfer would do
    ├── Command                   the command a transfer states
    └── Log                       the record of what was observed
```

Every page states what is running it and which version of it, then the pages it rests under. The collection is left out of that: every page rests under it and it names itself, so stating it in the header would say the same thing twice on every screen. Which application it is belongs to whoever runs the manager, so it is stated to the composition rather than known by it.

Breadcrumbs derive from the current page, stable identity, and entry label observations. An entry states itself by its title where it has one, and by its source where it has not:

```text
Transfers
Transfers  ›  Add sources
Transfers  ›  backup@nas.local:/volume1/photos/2026/
Transfers  ›  backup@nas.local:/volume1/photos/2026/  ›  Transfer
Transfers  ›  backup@nas.local:/volume1/photos/2026/  ›  Input
Transfers  ›  backup@nas.local:/volume1/photos/2026/  ›  Output
Transfers  ›  backup@nas.local:/volume1/photos/2026/  ›  Report
Transfers  ›  backup@nas.local:/volume1/photos/2026/  ›  Command
Transfers  ›  backup@nas.local:/volume1/photos/2026/  ›  Log
Transfers  ›  First fetched title  ›  Formats
Transfers  ›  First fetched title  ›  File name
```

Two of those segments state what the entry is rather than which page it is. What a request chooses between is `Transfer` where it moves paths and `Formats` where it retrieves media; where it comes to rest is `Output` where somebody stated a path and `File name` where this application derives one from the title.

## Collection and list selector

The main page is a classic collection selector. Selection is a stable [`ManagerSorts::Id`], never a vector index, so removing or updating other entries does not change what the details page denotes.

<details open>
<summary>Ratatui rendering: the collection page</summary>

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

</details>

Selection wraps at both ends. Adding a non-empty source batch focuses its first new entry. An empty addition preserves focus. Removing the selected entry selects the nearest remaining neighbor. Opening details preserves the selected identity.

Before source inspection succeeds, each row and breadcrumb displays exactly the source text entered or pasted by the user. After inspection, the extracted media title replaces that provisional label. Every compact row and expanded-row heading begins with the same short progress bar and percentage; percentage is therefore visible before the title rather than repeated after byte counts.

`○` denotes a source that has not started or an active transfer that is paused. `●` denotes a source that has entered the execution queue, including waiting, running, and terminal entries. On a ready source, Space fixes its options and appends it to that queue. On an active transfer whose interpreter advertises cooperative pause, Space switches between Pause and Resume. Waiting, publishing, and terminal entries have no Space action.

Starting multiple downloads requires no batch mode: select each ready source and press Space. Each becomes `Waiting` immediately, and the interpreter consumes waiting entries in collection order.

## Menu availability

The manager derives every reusable menu action as [`ActionAvailability::Enabled`] or [`ActionAvailability::Disabled`]. Availability is application meaning, not a color chosen by the terminal renderer. An interpreter uses the same observation twice: enabled actions accept input; disabled actions remain visible for orientation but cannot be selected or dispatched.

An empty collection has no selection to move, inspect, start, or remove, so every action naming one is disabled. Once an inspected ready entry is selected, Details, Space Start, and Remove become enabled. Cursor movement remains disabled until the page has more than one position. During an active transfer, `Space pause` is enabled only after the runtime advertises cooperative pause support.

<details open>
<summary>Ratatui rendering: a disabled menu</summary>

```text
┌──────────────────────────────────────────────────────────────────────────────────────────────────┐
│ rsynko  0.1.0                                                                                    │
└──────────────────────────────────────────────────────────────────────────────────────────────────┘
┌ Transfers (0) ───────────────────────────────────────────────────────────────────────────────────┐
│  No sources. Press [a] to add or paste sources.                                                  │
│                                                                                                  │
│                                                                                                  │
│                                                                                                  │
│                                                                                                  │
└──────────────────────────────────────────────────────────────────────────────────────────────────┘
┌ Actions ─────────────────────────────────────────────────────────────────────────────────────────┐
│[↑↓] Select  [Enter] Details  [a] Add  [Space] —  [Del] Remove  [Ctrl+Q] Quit                     │
└──────────────────────────────────────────────────────────────────────────────────────────────────┘
```

Ratatui renders those disabled items in dim gray. `[a] Add` and `[Ctrl+Q] Quit` remain normally styled and usable.

</details>

A renderer composes availability without knowing the manager representation, and without materializing a menu: the actions it offers are a sequence in and a sequence out.

```rust
use alux_ext::ext;
use rsynko_manager::{
    DetailSelectionAlg, DraftStateAlg, InputDraftAlg, ManagerAction, ManagerMenuExt,
    NavigationStateAlg, OutputDraftAlg, QueueCatalogAlg, QueueEntryAlg, RequestOptionsAlg,
    TextEditorStateAlg,
};

#[ext(name = MenuViewExt)]
impl<This, Id> This
where
    This: NavigationStateAlg<Id = Id>
        + QueueCatalogAlg<Id = Id>
        + DraftStateAlg
        + OutputDraftAlg
        + InputDraftAlg
        + TextEditorStateAlg
        + DetailSelectionAlg,
    This::Entry: QueueEntryAlg + RequestOptionsAlg,
    Id: Copy + Eq,
{
    /// Observes which of the stated actions may be dispatched now, in the order stated.
    fn dispatchable<'a>(
        &'a self,
        actions: impl IntoIterator<Item = ManagerAction> + 'a,
    ) -> impl Iterator<Item = ManagerAction> + 'a {
        actions
            .into_iter()
            .filter(move |action| self.action_availability(*action).is_enabled())
    }
}
```

## Add-sources page

The editor accepts one source per line. Submitting an empty draft preserves the page and reports a message. Successful submission appends every non-empty trimmed line in order and returns to the collection.

<details open>
<summary>Ratatui rendering: the add-sources editor</summary>

```text
┌──────────────────────────────────────────────────────────────────────────────────────────────────┐
│ rsynko  0.1.0  Add sources                                                                       │
└──────────────────────────────────────────────────────────────────────────────────────────────────┘
┌ Sources — editing ───────────────────────────────────────────────────────────────────────────────┐
│/home/dev/photos/2026                                                                             │
│backup@nas.local:/volume1/photos  /home/dev/photos                                                │
│https://www.youtube.com/watch?v=VIDEO_ID                                                          │
│                                                                                                  │
└──────────────────────────────────────────────────────────────────────────────────────────────────┘
┌ Examples ────────────────────────────────────────────────────────────────────────────────────────┐
│ /home/dev/photos/2026                               a folder here, into a folder of the same name│
│ backup@nas.local:/volume1/photos  /home/dev/photos  a folder on another machine, into a folder he│
│ /home/dev/photos  backup@nas.local:/volume1/photos  the same transfer, the other way around      │
│ rsync -a nas.local:/srv/data /mnt/data              a whole command, read as the two ends it name│
│ https://www.youtube.com/watch?v=VIDEO_ID            a web address a source recognizes, fetched in│
│ https://x.com/user/status/1234567890                a tweet, taking the media it carries         │
└──────────────────────────────────────────────────────────────────────────────────────────────────┘
┌ Actions ─────────────────────────────────────────────────────────────────────────────────────────┐
│[←] Move  [→] Move  [Enter] Add sources  [Esc] Transfers  [Ctrl+Q] Quit                           │
└──────────────────────────────────────────────────────────────────────────────────────────────────┘
```

</details>

## Expanded details

Details expand the selected row in place without moving focus away from that collection row. Enter therefore expands and then collapses the same row, so two consecutive Enter presses preserve the original collection state. Up and Down move from the row into visible fields and actions; Enter then opens or executes the selected control. The collection remains visible, so progress on neighboring downloads is never hidden. Escape also collapses the row. Breadcrumbs continue to identify the expanded stable [`ManagerSorts::Id`].

The expanded content is derived from one [`ManagerSorts::Entry`] and its transfer observations. It shows every known transfer observation when space permits. Concise failure summaries and detailed diagnostics are distinct values, so collection rows never leak enormous signed URLs.

<details open>
<summary>Ratatui rendering: expanded details</summary>

```text
┌──────────────────────────────────────────────────────────────────────────────────────────────────┐
│ rsynko  0.1.0  A portable video title                                                            │
└──────────────────────────────────────────────────────────────────────────────────────────────────┘
┌ Transfers (3) ───────────────────────────────────────────────────────────────────────────────────┐
│  ● ██▏       28% First fetched title                Downloading  18.2 MiB / 63.0 MiB  2.0 MiB/s  │
│  ● ████████ 100% Second fetched title               Complete     —                               │
│▾ ○            0% A portable video title  Ready                                                   │
│    Source     https://www.youtube.com/watch?v=third                                              │
│    File name  A portable video title.mp4                                                         │
│    Format     Best matching audio + video                                                        │
│    State      Ready                                                                              │
│    Downloaded —                                                                                  │
│    Speed      —                                                                                  │
│    Elapsed    00:00                                                                              │
│    Estimated  —                                                                                  │
│    Log        137  mp4   video only    1080p                                                     │
│    [Duplicate]                                                                                   │
│    [Delete]                                                                                      │
│                                                                                                  │
│                                                                                                  │
└──────────────────────────────────────────────────────────────────────────────────────────────────┘
┌ Actions ─────────────────────────────────────────────────────────────────────────────────────────┐
│[↑↓] Select field or action  [Enter] Close details  [Space] Start  [Esc] Transfers  [Ctrl+Q] Quit │
└──────────────────────────────────────────────────────────────────────────────────────────────────┘
```

</details>

`Speed` is the average byte rate `downloaded / elapsed`. `Estimated` is the remaining duration derived from that same observed rate and appears only when downloaded bytes, elapsed time, and total size make an estimate meaningful. These are specification-level derived values, not Ratatui calculations. Details contain no empty error placeholders. A failure shows one red `Error` row. `Note` appears only for non-error information, never as a duplicate error diagnostic.

## Output file name

Successful source inspection prefills the editable `File name` field from the media title and the selected format extension. When no title exists, the extractor media identity is used. Select the field in expanded details and press Enter to edit that same field inline.

<details open>
<summary>Ratatui rendering: the output-name editor</summary>

```text
┌──────────────────────────────────────────────────────────────────────────────────────────────────┐
│ rsynko  0.1.0  A portable video title  ›  File name                                              │
└──────────────────────────────────────────────────────────────────────────────────────────────────┘
┌ Transfers (3) ───────────────────────────────────────────────────────────────────────────────────┐
│  ● ██▏       28% First fetched title                Downloading  18.2 MiB / 63.0 MiB  2.0 MiB/s  │
│  ● ████████ 100% Second fetched title               Complete     —                               │
│▾ ○            0% A portable video title  Ready                                                   │
│    Source     https://www.youtube.com/watch?v=third                                              │
│  ▸ File name  A portable video title.mp4                                                         │
│    Format     Best matching audio + video                                                        │
│    State      Ready                                                                              │
│    Downloaded —                                                                                  │
│    Speed      —                                                                                  │
│    Elapsed    00:00                                                                              │
│    Estimated  —                                                                                  │
│    Log        137  mp4   video only    1080p                                                     │
│    [Duplicate]                                                                                   │
│    [Delete]                                                                                      │
│                                                                                                  │
│                                                                                                  │
└──────────────────────────────────────────────────────────────────────────────────────────────────┘
┌ Actions ─────────────────────────────────────────────────────────────────────────────────────────┐
│[←] Move  [→] Move  [Enter] Save  [Esc] Details  [Ctrl+Q] Quit                                    │
└──────────────────────────────────────────────────────────────────────────────────────────────────┘
```

</details>

The editor displays a terminal cursor at the exact insertion position. Left and Right move by one Unicode scalar, Home and End move to logical line boundaries, Backspace removes the preceding scalar, Delete removes the following scalar, and paste inserts at the cursor. Cursor position and UTF-8-safe editing are manager-spec meaning shared with the add-sources editor; terminal column width, horizontal clipping, and caret placement are renderer interpretations. Enter commits the normalized name. Escape cancels the draft and returns to the same selected `File name` details row; the collection and neighboring download activity remain visible throughout editing.

The portable-name specification replaces `/`, `\\`, Windows-forbidden punctuation and control characters, removes trailing spaces and dots, avoids Windows device names such as `CON` and `LPT1`, and bounds the UTF-8 component length. It therefore denotes one ordinary file component on both Linux and Windows. A valid extension typed by the user is preserved. Until first Start, changing the format choice also refreshes the suggested extension unless the user has explicitly saved a file name.

## Format options

One selector states the whole choice. It begins with the three preferred stream roles — `Best audio + video`, `Best video only`, `Best audio only` — and continues with every format extraction discovered, in extractor preference order. Preferring a role means "the most preferred format carrying those streams, whatever they turn out to be"; choosing an identity fixes exactly that format, and the roles then add nothing to it.

On expanded details, select the visible `Format` field with Up and Down and press Enter. The page behaves like a collection list: the current value is highlighted, Up and Down select an available value, and Enter or Escape returns to details. Format discovery stores identifiers and stream descriptions, never transient signed media URLs.

<details open>
<summary>Ratatui rendering: the format selector</summary>

```text
┌──────────────────────────────────────────────────────────────────────────────────────────────────┐
│ rsynko  0.1.0  A portable video title  ›  Formats                                                │
└──────────────────────────────────────────────────────────────────────────────────────────────────┘
┌ Format (6) ──────────────────────────────────────────────────────────────────────────────────────┐
│▸ Best audio + video                                                                              │
│  Best video only                                                                                 │
│  Best audio only                                                                                 │
│  18   mp4   audio + video 360p                                                                   │
│  140  m4a   audio only    tiny                                                                   │
│  137  mp4   video only    1080p                                                                  │
└──────────────────────────────────────────────────────────────────────────────────────────────────┘
┌ Actions ─────────────────────────────────────────────────────────────────────────────────────────┐
│[↑↓] Choose format  [Enter] Accept  [Esc] Details  [Ctrl+Q] Quit                                  │
└──────────────────────────────────────────────────────────────────────────────────────────────────┘
```

</details>

Adding a source and opening Format therefore shows exactly what was extracted, with the three preferences in front of it.

## First-start immutability and duplication

The transition from `Ready` to `Waiting` is the first start and permanently fixes the request's media role, format, and output policy. Failure does not make it editable again, but a failed request exposes `Restart`, which schedules the same fixed request again. This keeps a queue item an honest record of what was requested and executed.

A value that can no longer be changed is marked by nothing at all. The cursor does not stop on it, because [`QueueEntryAlg::detail_controls`] no longer offers it, and a field the cursor walks past has already said so; a mark beside it would be a second way of saying the same thing, and one a reader would have to learn. After first start, details expose state-appropriate actions as selectable rows. Duplicate creates a new stable identity for the same source, preserves its media and format choices, clears explicit output and transfer state, and makes the new entry editable. A successfully discovered format catalog is reusable by the duplicate; incomplete or failed discovery is restarted when needed.

<details open>
<summary>Ratatui rendering: a request whose options are fixed</summary>

```text
┌──────────────────────────────────────────────────────────────────────────────────────────────────┐
│ rsynko  0.1.0  First fetched title                                                               │
└──────────────────────────────────────────────────────────────────────────────────────────────────┘
┌ Transfers (3) ───────────────────────────────────────────────────────────────────────────────────┐
│▾ ● ██▏       28% First fetched title  Downloading                                                │
│    Source     https://www.youtube.com/watch?v=first                                              │
│    File name  first.mp4                                                                          │
│    Format     Best matching audio + video                                                        │
│    State      Downloading                                                                        │
│    Downloaded 18.2 MiB / 63.0 MiB                                                                │
│    Speed      2.0 MiB/s                                                                          │
│    Elapsed    00:09                                                                              │
│    Estimated  00:22                                                                              │
│    Log        retrieval started                                                                  │
│    [Duplicate]                                                                                   │
│    [Delete]                                                                                      │
│  ● ████████ 100% Second fetched title               Complete     —                               │
│  ○            0% A portable video title             Ready        —                               │
│                                                                                                  │
│                                                                                                  │
└──────────────────────────────────────────────────────────────────────────────────────────────────┘
┌ Actions ─────────────────────────────────────────────────────────────────────────────────────────┐
│[↑↓] Select field or action  [Enter] Close details  [Space] Pause  [Esc] Transfers  [Ctrl+Q] Quit │
└──────────────────────────────────────────────────────────────────────────────────────────────────┘
```

</details>

## Transfers

This application transfers paths, so a submitted line names one unless a source recognizes it as its own: `/home/dev/music`, `nas.local:/srv/data`, and `rsync://nas.local/data` are all transfers, and a web address is not because retrieval claims it. Which lines a source claims is [`SourceRecognitionAlg`], and it is the whole of what a source adds to submission — nothing here knows what any of them look like. What is transferred may be one file or a whole folder — rsync makes no distinction and neither does this. A folder is one queue entry and is never expanded into an entry per file: what is transferred, what is reported, and what is recorded is the path itself.

Both ends are stated by whoever asked, so they are named `Input` and `Output` rather than a source and a file name, and both may be edited until the transfer is first performed — a rehearsal is not a performance, so rehearsing, correcting an end, and rehearsing again is an ordinary thing to do. A media item offers no such thing: its source is what extraction already read, and changing it would make the title, the formats, and the name derived from them a lie. An edited `Output` is taken exactly as written — [`OutputNaming`] is the distinction, and it is the reason a path is not run through the portable-file-name rules that would replace its separators. Each end is also passed on exactly as written, because a trailing separator is what says *the contents of* rather than *this, into*, and both are wanted.

A transfer chooses no format, because a media role says nothing about a path, so expanded details state no `Format` row for it and the format selector is not offered.

One line names one request, and a line naming two ends names a transfer between them — which is how a whole transfer command reads as well, so one can be pasted. The add-sources page states the shapes it accepts beside the draft.

A transfer naming only where it comes from comes to rest under the name that end already has, beside wherever the application was started. Naming a folder is safer than naming the one somebody happens to be standing in, which is what a transfer that removes would empty.

A transfer performed by another program is held still by signalling it, so Space pauses and resumes one exactly as it does a download, and time spent held is not counted as time spent transferring. A transfer whose request is removed, or which is still running at exit, is ended rather than left behind: a program that has not started yet cannot be told to stop, so it is told again until it is over.

A transfer is rehearsed before it is performed. A fresh transfer request states a *rehearsal mode*, which is on, so Space means `Dry run` rather than `Start`: it asks what the transfer would do and changes nothing.

Expanded details are therefore a builder for one command, and state the command they build. [`QueueEntryAlg::stated_command`] observes exactly what an interpreter would run to perform the request, so the fields above it read as the parts that make it and Space reads as running it. A request retrieved by fetching states nothing there — there is no command to read.

<details open>
<summary>Ratatui rendering: a folder that has not been rehearsed</summary>

```text
┌──────────────────────────────────────────────────────────────────────────────────────────────────┐
│ rsynko  0.1.0  backup@nas.local:/volume1/photos/2026/                                            │
└──────────────────────────────────────────────────────────────────────────────────────────────────┘
┌ Transfers (1) ───────────────────────────────────────────────────────────────────────────────────┐
│▾ ○            0% backup@nas.local:/volume1/photos/2026/  Ready                                   │
│    Input      backup@nas.local:/volume1/photos/2026/                                             │
│    Output     /home/dev/photos/2026                                                              │
│    Transfer   copy adds and replaces, removes nothing, good for topping a folder up              │
│    Command    rsync --archive --partial --itemize-changes --itemize-changes --out-format=%i|%l|%…│
│    State      Ready                                                                              │
│    Downloaded —                                                                                  │
│    Speed      —                                                                                  │
│    Elapsed    00:00                                                                              │
│    Estimated  —                                                                                  │
│    Log        added backup@nas.local:/volume1/photos/2026/                                       │
│    [Dry run — changes nothing]                                                                   │
│    [Duplicate]                                                                                   │
│    [Delete]                                                                                      │
│                                                                                                  │
│                                                                                                  │
│                                                                                                  │
│                                                                                                  │
└──────────────────────────────────────────────────────────────────────────────────────────────────┘
┌ Actions ─────────────────────────────────────────────────────────────────────────────────────────┐
│[↑↓] Select field or action  [Enter] Close details  [Space] Dry run  [Esc] Transfers  [Ctrl+Q] Qu…│
└──────────────────────────────────────────────────────────────────────────────────────────────────┘
```

</details>

A rehearsal is not a start. It runs in [`TransferPhase::Rehearsing`] and returns the request to `Ready` exactly as editable as it found it, so the first-start locking that fixes a media request's options never triggers. Rehearsing, adjusting, and rehearsing again is therefore an ordinary thing to do. A rehearsal that *fails* is not a failed request either: it returns to `Ready` with its reason recorded, and Space still means something afterward.

What the rehearsal said appears as a selectable `Report` field naming what would happen, counted by kind.

<details open>
<summary>Ratatui rendering: a folder that has been rehearsed</summary>

```text
┌──────────────────────────────────────────────────────────────────────────────────────────────────┐
│ rsynko  0.1.0  backup@nas.local:/volume1/photos/2026/                                            │
└──────────────────────────────────────────────────────────────────────────────────────────────────┘
┌ Transfers (1) ───────────────────────────────────────────────────────────────────────────────────┐
│▾ ○            0% backup@nas.local:/volume1/photos/2026/  Ready                                   │
│    Input      backup@nas.local:/volume1/photos/2026/                                             │
│    Output     /home/dev/photos/2026                                                              │
│    Transfer   copy adds and replaces, removes nothing, good for topping a folder up              │
│    Command    rsync --archive --partial --itemize-changes --itemize-changes --out-format=%i|%l|%…│
│    State      Ready                                                                              │
│    Downloaded —                                                                                  │
│    Speed      —                                                                                  │
│    Elapsed    00:00                                                                              │
│    Estimated  —                                                                                  │
│    Report     2 new, 1 updated, 1 deleted, 2 unchanged                                           │
│    Log        the rehearsal stated 4 of 6 paths would change                                     │
│    [Dry run — changes nothing]                                                                   │
│    [Duplicate]                                                                                   │
│    [Delete]                                                                                      │
│                                                                                                  │
│                                                                                                  │
│                                                                                                  │
└──────────────────────────────────────────────────────────────────────────────────────────────────┘
┌ Actions ─────────────────────────────────────────────────────────────────────────────────────────┐
│[↑↓] Select field or action  [Enter] Close details  [Space] Dry run  [Esc] Transfers  [Ctrl+Q] Qu…│
└──────────────────────────────────────────────────────────────────────────────────────────────────┘
```

</details>

A transfer chooses instead one of the ways a path may be transferred. Each way is named by what it *does* rather than by what it is for — `mirror`, `mirror-keeping`, `skip-newer` — and states what it does before what that is usually wanted for, so a reader chooses by the first half and confirms by the second. The `Transfer` field names the chosen way; activating it opens every way on offer, the same page a media item chooses a representation on, because choosing how a request runs is one meaning and not two.

<details open>
<summary>Ratatui rendering: the ways a folder may be transferred</summary>

```text
┌──────────────────────────────────────────────────────────────────────────────────────────────────┐
│ rsynko  0.1.0  backup@nas.local:/volume1/photos/2026/  ›  Transfer                               │
└──────────────────────────────────────────────────────────────────────────────────────────────────┘
┌ Transfer (13) ───────────────────────────────────────────────────────────────────────────────────┐
│▸ copy            adds and replaces, removes nothing, good for topping a folder up                │
│  mirror          makes the destination match exactly, good for keeping a copy current            │
│  mirror-keeping  mirrors, keeps what it replaces, compresses, good for remote backups            │
│  mirror-whole    mirrors by sending whole files, good for local disks and SSDs                   │
│  mirror-readable mirrors and makes everything readable, good for publishing a web root           │
│  skip-newer      replaces nothing newer at the destination, good for merging two copies          │
│  compare-content compares what files hold, not when they changed, good after clock drift         │
│  resume          continues large files where they left off, good for flaky links                 │
│  move            moves, removing what arrived safely, good for clearing a staging area           │
│  limit-rate      leaves room on the line, good for a shared connection                           │
│  keep-marks      keeps links, ownership, and every file mark, good for system backups            │
│  one-disk        stays on the disk it started on, good for roots with mounts under               │
│  skip-junk       leaves behind what no one meant to keep, good for editor droppings              │
└──────────────────────────────────────────────────────────────────────────────────────────────────┘
┌ Actions ─────────────────────────────────────────────────────────────────────────────────────────┐
│[↑↓] Choose transfer  [Enter] Accept  [Esc] Details  [Ctrl+Q] Quit                                │
└──────────────────────────────────────────────────────────────────────────────────────────────────┘
```

</details>

Choosing one rewrites the command, so what a way *is* can be read rather than remembered. It also forgets the report: a report describes one command, and changing either end or the way of running it leaves the old report describing something nobody is going to do.

<details open>
<summary>Ratatui rendering: a transfer that keeps what it replaces</summary>

```text
┌──────────────────────────────────────────────────────────────────────────────────────────────────┐
│ rsynko  0.1.0  backup@nas.local:/volume1/photos/2026/                                            │
└──────────────────────────────────────────────────────────────────────────────────────────────────┘
┌ Transfers (1) ───────────────────────────────────────────────────────────────────────────────────┐
│▾ ○            0% backup@nas.local:/volume1/photos/2026/  Ready                                   │
│    Input      backup@nas.local:/volume1/photos/2026/                                             │
│    Output     /home/dev/photos/2026                                                              │
│    Transfer   mirror-keeping mirrors, keeps what it replaces, compresses, good for remote backup │
│    Command    rsync --archive --partial --itemize-changes --itemize-changes --out-format=%i|%l|%…│
│    State      Ready                                                                              │
│    Downloaded —                                                                                  │
│    Speed      —                                                                                  │
│    Elapsed    00:00                                                                              │
│    Estimated  —                                                                                  │
│    Log        the report was forgotten: it no longer states what would happen                    │
│  ▸ [⚠️ Real run — writes files]                                                                  │
│    [Duplicate]                                                                                   │
│    [Delete]                                                                                      │
│                                                                                                  │
│                                                                                                  │
│                                                                                                  │
│                                                                                                  │
└──────────────────────────────────────────────────────────────────────────────────────────────────┘
┌ Actions ─────────────────────────────────────────────────────────────────────────────────────────┐
│[↑↓] Select field or action  [Enter] Activate  [Space] Start  [Esc] Transfers  [Ctrl+Q] Quit      │
└──────────────────────────────────────────────────────────────────────────────────────────────────┘
```

</details>

Activating `Report` states every path the rehearsal named, and not only the ones that would change: what the transfer would leave exactly as it is stands under the changes, quietly, so the report is the whole picture rather than a list of alarms. [`ChangeKind`] distinguishes the four, and an interpreter states each with its own weight.

Every field above builds one command, and `Command` states as much of it as the column holds. Activating it states the whole command on its own, with nothing drawn around it: no border, no label, and no break the command did not already have. A reader selects it the way they select anything else in a terminal, and what they take away is exactly the command — which is the point, since a command wrapped inside a bordered pane cannot be copied out of one.

<details open>
<summary>Ratatui rendering: the command, stated so it can be taken away</summary>

```text
┌──────────────────────────────────────────────────────────────────────────────────────────────────┐
│ rsynko  0.1.0  backup@nas.local:/volume1/photos/2026/  ›  Command                                │
└──────────────────────────────────────────────────────────────────────────────────────────────────┘
rsync --archive --partial --itemize-changes --itemize-changes --out-format=%i|%l|%n --info=progress2
--dry-run backup@nas.local:/volume1/photos/2026/ /home/dev/photos/2026



┌ Actions ─────────────────────────────────────────────────────────────────────────────────────────┐
│[Esc] Details  [Ctrl+Q] Quit                                                                      │
└──────────────────────────────────────────────────────────────────────────────────────────────────┘
```

</details>

<details open>
<summary>Ratatui rendering: the report</summary>

```text
┌──────────────────────────────────────────────────────────────────────────────────────────────────┐
│ rsynko  0.1.0  backup@nas.local:/volume1/photos/2026/  ›  Report                                 │
└──────────────────────────────────────────────────────────────────────────────────────────────────┘
┌ Report (4 of 6 would change) ────────────────────────────────────────────────────────────────────┐
│  + new          3.1 MiB  IMG_0431.jpg                                                            │
│  + new          2.7 MiB  IMG_0432.jpg                                                            │
│  ~ updated      1.1 KiB  album.json                                                              │
│  - deleted            —  old/IMG_0090.jpg                                                        │
│  = unchanged    2.2 MiB  IMG_0001.jpg                                                            │
│  = unchanged    2.0 MiB  IMG_0002.jpg                                                            │
│                                                                                                  │
│                                                                                                  │
└──────────────────────────────────────────────────────────────────────────────────────────────────┘
┌ Actions ─────────────────────────────────────────────────────────────────────────────────────────┐
│[Esc] Details  [Ctrl+Q] Quit                                                                      │
└──────────────────────────────────────────────────────────────────────────────────────────────────┘
```

</details>

The rehearsal mode is turned by one row in expanded details, and that row is the one exception to actions naming what they do: it states which mode the request is *in*, and carries the weight of being in it. `[Dry run — changes nothing]` is safe; `[⚠️ Real run — writes files]` is a caution, and says why. The warning asks for its emoji presentation, which is what makes it two columns wide to everyone measuring it rather than one width to a renderer and another to a font. Activating the row moves between them. Once the mode is off, Space means `Start` — and the stated command loses the argument that was keeping it harmless, which is the clearest statement of what changed.

<details open>
<summary>Ratatui rendering: a transfer in real mode</summary>

```text
┌──────────────────────────────────────────────────────────────────────────────────────────────────┐
│ rsynko  0.1.0  backup@nas.local:/volume1/photos/2026/                                            │
└──────────────────────────────────────────────────────────────────────────────────────────────────┘
┌ Transfers (1) ───────────────────────────────────────────────────────────────────────────────────┐
│▾ ○            0% backup@nas.local:/volume1/photos/2026/  Ready                                   │
│    Input      backup@nas.local:/volume1/photos/2026/                                             │
│    Output     /home/dev/photos/2026                                                              │
│    Transfer   copy adds and replaces, removes nothing, good for topping a folder up              │
│    Command    rsync --archive --partial --itemize-changes --itemize-changes --out-format=%i|%l|%…│
│    State      Ready                                                                              │
│    Downloaded —                                                                                  │
│    Speed      —                                                                                  │
│    Elapsed    00:00                                                                              │
│    Estimated  —                                                                                  │
│    Report     2 new, 1 updated, 1 deleted, 2 unchanged                                           │
│    Log        dry run disabled: the next run will perform the transfer                           │
│  ▸ [⚠️ Real run — writes files]                                                                  │
│    [Duplicate]                                                                                   │
│    [Delete]                                                                                      │
│                                                                                                  │
│                                                                                                  │
│                                                                                                  │
└──────────────────────────────────────────────────────────────────────────────────────────────────┘
┌ Actions ─────────────────────────────────────────────────────────────────────────────────────────┐
│[↑↓] Select field or action  [Enter] Activate  [Space] Start  [Esc] Transfers  [Ctrl+Q] Quit      │
└──────────────────────────────────────────────────────────────────────────────────────────────────┘
```

</details>

Everything above is manager meaning. What a folder transfer *is* — the two ends it joins, exactly the command that joins them, and what each line the running transfer writes means — is stated by `rsynko-rsync`, and `crates/rsynko-process` runs that command as an operating-system process. The manager never learns what `rsync` is; it learns that some requests can state what they would do instead of doing it.

## Download record

Every request keeps the record of what was observed about it: the source it was added from, the identity and title extraction recovered, every format the source described, retrieval start, publication, and failure. Notes are keyed by stable identity and observed in the order they were stated, so the record of one download never mixes with another's.

Expanded details state the record as a selectable `Log` field naming its most recent note, and activating that field opens the whole record. The record is reference material rather than the thing being watched, which is why an interpreter states it without competing for attention — Ratatui renders it in dim gray.

<details open>
<summary>Ratatui rendering: the download record</summary>

```text
┌──────────────────────────────────────────────────────────────────────────────────────────────────┐
│ rsynko  0.1.0  First fetched title  ›  Log                                                       │
└──────────────────────────────────────────────────────────────────────────────────────────────────┘
┌ Log (7) ─────────────────────────────────────────────────────────────────────────────────────────┐
│added https://www.youtube.com/watch?v=first                                                       │
│extracted Yd9s7MN8lPk: First fetched title                                                        │
│extraction described 3 formats                                                                    │
│  18   mp4   audio + video 360p                                                                   │
│  140  m4a   audio only    tiny                                                                   │
│  137  mp4   video only    1080p                                                                  │
│retrieval started                                                                                 │
│                                                                                                  │
└──────────────────────────────────────────────────────────────────────────────────────────────────┘
┌ Actions ─────────────────────────────────────────────────────────────────────────────────────────┐
│[Esc] Details  [Ctrl+Q] Quit                                                                      │
└──────────────────────────────────────────────────────────────────────────────────────────────────┘
```

</details>

## State and interpreter boundary

The manager specification decides:

- collection order and stable identity;
- which entry is selected;
- collection, editor, and expanded-details navigation;
- breadcrumb meaning;
- ready, waiting, active, paused, complete, and failed states;
- state-dependent Start, Pause, Resume, or absent Space meaning;
- page- and state-dependent menu-action availability;
- editable output file name, the preferred stream roles, the concrete format choice, and discovered format catalogs;
- the record of what was observed about one request, and the order its notes were stated in;
- the rehearsal mode of a request that has one, and that rehearsing is not starting;
- what a rehearsal stated would happen, and which of it would change anything;
- irreversible first-start locking and fresh-request duplication;
- how keyed progress and terminal observations update entries;
- removal, duplication, and safe-exit intentions;
- concise summaries versus diagnostic details;
- average byte rate, estimated remaining time, and completed share derived from observed progress.

An interpreter decides:

- keyboard, pointer, touch, or HTTP input mapping;
- list widgets, colors, borders, responsive layout, and how finely a bar draws a share;
- clipboard and terminal mechanisms;
- background source-inspection mechanisms;
- how a rehearsal and a transfer are actually performed, and what a folder is transferred by;
- sequential or parallel execution of entries denoted `Waiting`;
- persistence and notification mechanisms.

Exit is bound to `Ctrl+Q` on every page, and safe exit is what it states: every run is told to stop, and the reader is out once they have. `Ctrl+C` is bound to nothing, because it is the key people press out of habit to stop a terminal program — the Ratatui interpreter states what it is for and ends nothing, and reads two of them in quick succession as the exit it is; that reading is the interpreter's own and the manager hears nothing of a single one. Escape closes selectors or returns to the parent page and has no exit meaning on the root collection. The interpreter cancels its active stream immediately when that entry is removed or safe exit is requested.

[`ManagerIntent::apply_selected_space`] moves one selected ready entry to `Waiting`. Repeating that operation for other selected sources builds the execution queue, while the interpreter chooses how much concurrency to provide. The state model permits any number of waiting and active transfers.

## Laws

Every law below is checked against any interpreter by the extension it names:

- collection selection wraps and remains a stable identity — [`NavigationLaws::navigation_laws`](https://github.com/tgrospic/rsynko/blob/rsynko-manager-v0.1.2/rsynko-manager/src/laws.rs#L77-L145);
- adding sources focuses the first newly appended identity — [`NavigationLaws::navigation_laws`](https://github.com/tgrospic/rsynko/blob/rsynko-manager-v0.1.2/rsynko-manager/src/laws.rs#L77-L145);
- opening details denotes exactly the selected entry — [`NavigationLaws::navigation_laws`](https://github.com/tgrospic/rsynko/blob/rsynko-manager-v0.1.2/rsynko-manager/src/laws.rs#L77-L145);
- expanded details move an ordered cursor through visible fields and actions — [`NavigationLaws::navigation_laws`](https://github.com/tgrospic/rsynko/blob/rsynko-manager-v0.1.2/rsynko-manager/src/laws.rs#L77-L145);
- breadcrumbs are derived from page and entry identity — [`NavigationLaws::navigation_laws`](https://github.com/tgrospic/rsynko/blob/rsynko-manager-v0.1.2/rsynko-manager/src/laws.rs#L77-L145);
- removing selection chooses a remaining neighbor or no selection for an empty collection — [`NavigationLaws::navigation_laws`](https://github.com/tgrospic/rsynko/blob/rsynko-manager-v0.1.2/rsynko-manager/src/laws.rs#L77-L145);
- a draft is observed back exactly as it was stated, and submits the lines it names — [`DraftLaws::draft_laws`](https://github.com/tgrospic/rsynko/blob/rsynko-manager-v0.1.2/rsynko-manager/src/laws.rs#L166-L220);
- Space schedules exactly one selected entry in `Ready` state — [`TransitionLaws::transition_laws`](https://github.com/tgrospic/rsynko/blob/rsynko-manager-v0.1.2/rsynko-manager/src/laws.rs#L412-L513);
- Space names a collection entry, so it denotes nothing on the editor and selector pages — [`MenuLaws::menu_laws`](https://github.com/tgrospic/rsynko/blob/rsynko-manager-v0.1.2/rsynko-manager/src/laws.rs#L561-L627);
- Space pauses only active transfers that advertise cooperative pause support — [`TransitionLaws::transition_laws`](https://github.com/tgrospic/rsynko/blob/rsynko-manager-v0.1.2/rsynko-manager/src/laws.rs#L412-L513);
- unsupported, publishing, terminal, and failed transfers expose no pause action — [`MenuLaws::menu_laws`](https://github.com/tgrospic/rsynko/blob/rsynko-manager-v0.1.2/rsynko-manager/src/laws.rs#L561-L627);
- disabled menu actions remain observable but cannot be dispatched by an interpreter — [`MenuLaws::menu_laws`](https://github.com/tgrospic/rsynko/blob/rsynko-manager-v0.1.2/rsynko-manager/src/laws.rs#L561-L627);
- the first start irreversibly fixes request options — [`OptionsLaws::options_laws`](https://github.com/tgrospic/rsynko/blob/rsynko-manager-v0.1.2/rsynko-manager/src/laws.rs#L1237-L1313);
- extracted titles prefill portable output names and fall back to media identity — [`TransitionLaws::transition_laws`](https://github.com/tgrospic/rsynko/blob/rsynko-manager-v0.1.2/rsynko-manager/src/laws.rs#L412-L513);
- text insertion, deletion, and cursor movement preserve UTF-8 boundaries — [`TextLaws::text_editor_laws`](https://github.com/tgrospic/rsynko/blob/rsynko-manager-v0.1.2/rsynko-manager/src/laws.rs#L229-L260);
- failed requests restart without changing their fixed options — [`TransitionLaws::transition_laws`](https://github.com/tgrospic/rsynko/blob/rsynko-manager-v0.1.2/rsynko-manager/src/laws.rs#L412-L513);
- stream roles admit only formats with exactly the denoted audio/video composition — [`OptionsLaws::options_laws`](https://github.com/tgrospic/rsynko/blob/rsynko-manager-v0.1.2/rsynko-manager/src/laws.rs#L1237-L1313);
- one ordered selector states the preferred roles and then every discovered format — [`OptionsLaws::options_laws`](https://github.com/tgrospic/rsynko/blob/rsynko-manager-v0.1.2/rsynko-manager/src/laws.rs#L1237-L1313);
- preferring a role releases any fixed identity, and a fixed identity names exactly one format — [`OptionsLaws::options_laws`](https://github.com/tgrospic/rsynko/blob/rsynko-manager-v0.1.2/rsynko-manager/src/laws.rs#L1237-L1313);
- duplication creates a fresh editable identity and transfer state — [`TransitionLaws::transition_laws`](https://github.com/tgrospic/rsynko/blob/rsynko-manager-v0.1.2/rsynko-manager/src/laws.rs#L412-L513);
- duplication preserves media and format choices — [`TransitionLaws::transition_laws`](https://github.com/tgrospic/rsynko/blob/rsynko-manager-v0.1.2/rsynko-manager/src/laws.rs#L412-L513);
- every entry keeps the record of what was observed about it, keyed and in order — [`LogLaws::log_laws`](https://github.com/tgrospic/rsynko/blob/rsynko-manager-v0.1.2/rsynko-manager/src/laws.rs#L704-L743);
- transfer observations update only their addressed [`ManagerSorts::Id`] — [`QueueLaws::queue_laws`](https://github.com/tgrospic/rsynko/blob/rsynko-manager-v0.1.2/rsynko-manager/src/laws.rs#L299-L367);
- speed and estimated time are derived consistently for every interpreter — [`QueueLaws::queue_laws`](https://github.com/tgrospic/rsynko/blob/rsynko-manager-v0.1.2/rsynko-manager/src/laws.rs#L299-L367);
- full byte progress denotes `Publishing`, not `Complete` — [`QueueLaws::queue_laws`](https://github.com/tgrospic/rsynko/blob/rsynko-manager-v0.1.2/rsynko-manager/src/laws.rs#L299-L367);
- only terminal success denotes `Complete` — [`QueueLaws::queue_laws`](https://github.com/tgrospic/rsynko/blob/rsynko-manager-v0.1.2/rsynko-manager/src/laws.rs#L299-L367);
- a collection holds exactly the sources it was given, in the order given — [`DownloadsLaws::downloads_laws`](https://github.com/tgrospic/rsynko/blob/rsynko-manager-v0.1.2/rsynko-manager/src/laws.rs#L654-L680);
- stating what a transfer would do is not doing it — [`RehearsalLaws::rehearsal_laws`](https://github.com/tgrospic/rsynko/blob/rsynko-manager-v0.1.2/rsynko-manager/src/laws.rs#L959-L1048);
- a line nobody claimed is a path rather than something to fetch — [`SubmissionLaws::submission_laws`](https://github.com/tgrospic/rsynko/blob/rsynko-manager-v0.1.2/rsynko-manager/src/laws.rs#L795-L889);
- what wants work, what has begun, and what is left holding are the same statement — [`AttentionLaws::attention_laws`](https://github.com/tgrospic/rsynko/blob/rsynko-manager-v0.1.2/rsynko-manager/src/laws.rs#L1134-L1206);
- every intention denotes exactly the meaning derived for it — [`IntentLaws::intent_laws`](https://github.com/tgrospic/rsynko/blob/rsynko-manager-v0.1.2/rsynko-manager/src/laws.rs#L1381-L1435);
- renderer mechanisms do not choose manager state transitions.
