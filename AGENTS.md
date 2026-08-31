# Rsynko repository guide

## Product

This workspace transfers paths. A transfer joins two ends, is rehearsed before it is performed, and
is performed by `rsync`, which the manager never learns anything about. Every other reading of a
submitted line exists because a source claimed it: retrieval from Youtube and X is the additional
source composed onto that, and its extraction follows yt-dlp, which is the behavioral oracle and
not an architecture template. Do not reproduce `YoutubeDL` or `InfoExtractor` god objects in Rust.

## Workspace

Specifications sit at the workspace root, interpreters under `crates/` named for the choice each
makes, and one executable composes them. [`README.md`](README.md) lists them all, and
[`ARCHITECTURE.md`](ARCHITECTURE.md) states the shape they make and which way the dependencies
point. Neither is repeated here.

## Authority

Read authority in this order:

1. Meanings and equalities in `DENOTATIONAL_DESIGN.md`.
2. Public values, capability traits, and first-order syntax in specification crates.
3. Extension bounds and bodies defining derived meaning.
4. Shared laws and scenarios.
5. Reference and production interpreters.
6. Operational documentation and compatibility notes.

When artifacts disagree, fix the smallest higher-authority layer that is wrong.

## Read order

For design work, read from meaning to machinery:

1. `README.md`
2. `DENOTATIONAL_DESIGN.md`
3. `ARCHITECTURE.md`
4. the README of the specification the change belongs to
5. its capabilities, then the extensions derived over them
6. its laws
7. the interpreter that would carry the change
8. the tests that compare interpretations

Do not begin with an interpreter merely because it is concrete.

## Commands

```sh
just fmt
just build
just clippy
just doc
just test
just package
```

`just ci` runs all six in that order, and the [`Justfile`](Justfile) holds the full command each
recipe stands for. Run the narrowest relevant test while working and `just ci` before finishing.

## Design rules

- Defines primitive domain meaning in small capability traits.
- Derives behavior through `alux_ext::ext` over explicit, minimal `where` clauses.
- Keeps specifications independent of serialization, runtimes, frameworks, storage, and processes.
- Reifies behavior as first-order syntax only when another interpreter must inspect or compose it.
- States a sequence as a carrier and a stepping observation. A materialized collection in a specification signature is a loop that already ran, and belongs to an interpreter.
- Represents domain elements through associated carrier types by default. Introduces a concrete
  struct or enum in a specification only when its constructors and observations are themselves
  shared first-order meaning; never places an interpreter's storage layout in a specification.
- Defines multi-capability compositions as extensions. Does not replace an extension with a generic
  free function carrying the same `where` clause, and does not use generated extension traits as
  bounds in place of their primitive capabilities.
- Keeps concrete interpreters thin and prevents them from duplicating derived behavior.
- Uses direct capability bounds for one receiver; does not introduce universal context traits.
- Uses projection only for genuine containment and delegation only for semantic substitution.
- Uses `Default` only when it denotes the abstraction's neutral element.
- Does not add zero-argument `new` methods that merely delegate to `Default`.
- States non-neutral policy explicitly instead of hiding it in `Default`.

## Tests and laws

- States every specification module's laws as extensions over that module's own capability bounds,
  and supplies what only an interpreter can hold through a fixture capability. An interpreter runs
  those extensions; it never restates a law as an assertion of its own.
- Tests public observations and laws rather than private fields or call sequences without meaning.
- Treats an example as finite evidence. Where an obligation must hold for every interpretation, it
  is a law rather than one more test.

## Rust conventions

### Manifests

- Inherits edition, Rust version, license, and shared metadata from the workspace manifest.
- Declares external dependencies in root `[workspace.dependencies]`; crates inherit them with
  `dependency.workspace = true`.
- Uses major versions for stable dependencies and minor versions for `0.x` ones, and enables only
  the features a crate needs. A framework dependency belongs to an interpreter crate.
- Uses Rust 2024 and the pinned Rust 1.97 toolchain.

### Bounds and carriers

- Avoids bounds on trait and struct definitions. A carrier is an associated type, and the bounds it
  must satisfy go on the implementations and use sites that need them.
- Writes an extension over `This` with a `where` clause rather than a named parameter carrying
  inline bounds.
- Names an associated type as a type parameter where that makes a bound clearer, and reaches for
  `<This as Trait>::Assoc` only where the name is genuinely ambiguous. An inline multi-segment
  projection never appears in a signature, bound, or `impl` header.
- Expresses a capability alias as a trait with supertraits plus one blanket implementation, so it
  adds no dependency and states that one interpreter answers all of them.
- Derives operations with `alux_ext::ext`, imports the generated extension traits, and calls their
  methods directly. A generated extension trait is never used as a bound in place of the primitive
  capabilities it derives from.
- Forwards trait implementations with `ambassador::Delegate` and `#[delegatable_trait]` rather than
  by hand.

### Modules and imports

- Keeps root modules as intentional public surfaces and engineering modules private.
- Does not both declare a module `pub` and re-export its contents; one path to an item is enough.
- Orders module declarations by visibility: `pub mod`, `pub(crate) mod`, then private `mod`.
- After module declarations, groups re-exports and imports by the same visibility order: `pub`,
  `pub(crate)`, then private.
- Separates visibility groups with one blank line and keeps declarations within the same visibility
  group contiguous, regardless of whether a path begins with `crate`, an external crate, or `std`.
- Keeps imports at module scope unless narrowing scope avoids a concrete collision.
- Imports a specification with `*` once the named list grows long, whether that specification is another crate or the crate being written (`use crate::*`). A specification exports only capabilities, extensions, and shared syntax, so glob-importing one states a dependency on its whole surface rather than hiding an item. Interpreter crates stay explicitly imported.

### Code and comments

- Prefers iterator combinators where they make dataflow clearer than a loop.
- Uses `derive-new` and `derive_more` where they remove truthful boilerplate, and never combines
  `thiserror #[from]` with `derive_more::From` on the same variant.
- Uses `Default` only for a neutral element, and adds no zero-argument `new` that merely delegates.
- Orders struct fields outward-facing first — identity, foreign identities, edges — and intrinsic
  scalar payload last.
- Wraps code and doc comments at 100 columns, filling the width rather than wrapping narrow.
- Divides a file by the order of its items, not by banner comments.
- Keeps a comment that is still true when editing around it; updates it when it stops being true and
  removes it only then.
- Forbids unsafe code and treats Clippy and rustdoc warnings as errors.

### Documentation

- Documents every public item with what it means, in US English, beginning with a third-person
  singular verb where practical.
- Includes each crate's `README.md` as crate documentation with `#![doc = include_str!]`, and writes
  its examples as executed doctests rather than `ignore` blocks, so the published introduction
  cannot drift from the compiling surface.
- Leads specification READMEs with specification values, capabilities, derived operations, and laws.
  Places concrete interpreter examples only after that surface and wraps them in expandable
  `<details>` sections.

## Change workflow

1. State the meaning first, in the specification that owns it.
2. Derive what follows in an extension over the capabilities it needs.
3. State the obligation as a law, and let every interpreter run it.
4. Interpret it, in the crate named for the choice being made.
5. Update the README that carries that meaning, and run `just ci`.

## Commits

Commit messages are [Conventional Commits](https://www.conventionalcommits.org): a type, an optional
scope naming the crate, and one lower-case sentence saying what the commit does.

```text
feat(rsynko-x): state what one public tweet carries
fix(rsynko-ratatui): elide a menu too long for its pane
docs: separate specifications from interpretations
refactor: move the interpretations under crates/
test(rsynko-manager): check the ordering one pass states
chore: release
```

The title carries the change; a body is written only where the reason is not in the diff. Nothing
signs the message on the author's behalf.
