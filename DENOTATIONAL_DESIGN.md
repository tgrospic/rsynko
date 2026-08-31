# Denotational design

## What this means

Design by meaning is a Rust adaptation of **Denotational Design**, the methodology named and developed by [Conal Elliott](http://conal.net/). Its central move is to give every type and operation a precise, simple, implementation-independent meaning, and to design the programming interface from that meaning rather than from the machinery that would carry it.

An implementation is not the definition of an operation. It is one representation that must preserve the operation's meaning.

```text
representation first                          meaning first
--------------------                          -------------
choose library, framework, and state          choose the observation or transformation
derive an API from that machinery             derive operations compositionally
generate framework code as the authority      keep a neutral first-order program
describe behavior afterward                   choose and check interpreters afterward
```

This is more than dependency inversion or trait-oriented programming. A trait can abstract machinery while leaving meaning vague, and a macro can remove syntax while hiding the real program. The method asks for the semantic object and its laws to be identified before any representation or generation choice reaches the interface.

## Authority

When artifacts disagree, authority descends in this order:

1. The meanings and equalities stated in this document.
2. Public value and capability signatures in the specification crates.
3. Extension bounds and bodies defining derived operations.
4. Reusable law checks and scenarios.
5. Reference and production interpreters.

An interpreter is never the authority for what something means, and a test is finite evidence rather than proof. Where stronger assurance is wanted, add property tests, exhaustive finite checks, or model comparison rather than calling an example a proof.

## Choose the carrier

A carrier is the world of values a boundary admits, not a state struct. A specification represents a domain element through an associated carrier unless the element is deliberately reified as first-order syntax that several interpreters must inspect or compose.

Concrete records for vectors, counters, cursors, locks, caches, or mutable state are interpreter choices and belong to interpreters. A derived composition belongs in an extension over primitive capability bounds; the same `where` clause must not be displaced into a generic free function or hidden behind an extension-trait bound.

## Sequences

A sequence is a carrier and a step, never a materialized collection.

`Vec` in a specification signature is a loop that has already run. It commits to when the producer is consumed, to the producer terminating, and to the whole result fitting in memory. Those are interpreter facts wearing a type, and stating them as meaning excludes every producer that does not satisfy them.

A specification states a sequence as a carrier together with one stepping observation, the coalgebra `Carrier -> Option<(Element, Carrier)>`. An interpreter then chooses whether stepping reads memory, walks a manifest, or waits on a network. `Vec` becomes a correct carrier for a reference interpreter and a wrong signature for a specification.

A derived operation returns a stream for the same reason. Where a barrier is genuinely part of the meaning, the barrier is stated, and only the barrier is eager.

## Derive behavior in extensions

A capability trait states one coherent primitive observation or effect. Everything composed from several of them is an extension over explicit bounds, so the bounds state what the derivation actually needs and nothing else.

An extension earns its place when its bounds and its result state a real composition. One that forwards to a single capability adds a name and no meaning.

## Compose, do not inherit

Semantic conjunction is expressed by bounds. Static policy composition is expressed by values. Runtime environments are introduced only where they genuinely carry independent interpreters.

| Meaning location | Composition |
| --- | --- |
| One receiver answers all capabilities | Direct extension bounds |
| One separately selected policy | Explicit parameter |
| Several independently selected policies | Product value |
| Environment carries separate interpreters | Projection |
| Wrapper truthfully substitutes for the inner value | Delegation |

Projection and delegation are not interchangeable: projection says *has*, delegation says *is*. Neither is used only to shorten a call.

## Keep interpreters neutral

An interpreter preserves meaning while choosing machinery. It does not rename an operation, reorder arguments, add a step the specification never stated, or select domain policy. Mechanical defaults may be derived; semantic defaults are stated.

Separate crates enforce that. A specification crate cannot name a runtime, an HTTP client, or a drawing library, so if neutral code seems to need one, the interpreter boundary has leaked.

## Keep macros below meaning

A procedural macro is a syntax translator, and its target is a public, manually constructible first-order representation:

```text
convenient syntax -> public operation values -> generic fold
```

Not the reverse, where framework-shaped syntax expands into opaque callbacks that documentation must then reverse-engineer. Expansion tests verify lowering; they do not replace laws over the first-order algebra. Diagnostics are part of the authoring surface: report the violated rule where it was written.

## Put bounds at use sites

Do not constrain traits, syntax nodes, or carriers preemptively. A bound belongs on the fold, extension, or interpreter operation that uses it — serialization on the interpreter that serializes, ordering on the operation that sorts. This keeps first-order meaning reusable and interpreter contracts readable.

## State laws once

An example shows one execution; a law states an obligation for every interpretation. Laws are written once, over the capability bounds, and checked against any interpreter through a fixture the interpreter supplies.

Prefer laws about preserved structure: identities behave as identities, composition is associative where claimed, ordering is preserved where claimed, and two interpretations of one program observe the same surface. Where a law must hold universally, encode a generic check or property test rather than trusting one fixture.

## Structure

A specification is capability traits, extensions, and the syntax reifying them. A package is a compilation and publishing unit. Meaning lives in modules and trait surfaces, so a package boundary states nothing about meaning, and one package may hold several specifications.

Inside a package, one module states one meaning. The root is the intentional public surface: it declares its modules privately and re-exports the values, traits, and extensions meant to be used. Engineering modules stay private and nothing is exposed twice.

A meaning earns its own package only when it has an independent consumer — one that would depend on it and not on the others. Splitting because the vocabulary changed produces packages that are always used together and whose bounds lengthen at every call site. Merging meanings whose consumers genuinely differ produces a dependency that cannot be refused. Direction is the other reason to split: a package cannot depend on one that depends on it, so a meaning two specifications both need is a package rather than a module inside either.

Specification packages sit at the workspace root. Interpreters sit under `crates/`, and an interpreter's name ends with the lower-level choice it makes: the library, storage, runtime, or hardware it selects. No specification depends on an interpreter.

When a package's name stops matching what it means, absorption is considered before renaming. A meaning with no independent consumer is a module of the package that uses it, and renaming only relocates the mismatch.

## Neutral elements

`Default` denotes a neutral element only where the specification supplies one: something that contributes nothing under its own composition. Such a type does not also offer a zero-argument `new`. A default that would select retry, format, output, or any other product policy is not neutral and stays explicit.

## Keep the surface intentional

Root exports are the vocabulary a reader learns. Export the algebras, the first-order syntax, the folds, the intentional interpreters, and the public macros; keep visitors, tuple machinery, helper types, and fixtures private unless their names carry stable public meaning.

A specification crate's dependency list is a promise about its boundary. Frameworks live in interpreter crates rather than behind a feature, because an optional dependency still makes framework meaning expressible in the specification.

## Review checklist

- Is the meaning stated before its representation or macro syntax?
- Does each capability expose one coherent primitive distinction?
- Is derived behavior an extension over minimal explicit bounds?
- Is a sequence stated as a carrier and a step rather than a collection?
- Are bounds located at the operations that require them?
- Can an independent interpreter implement the public algebra without the existing machinery?
- Is a reusable law added for a general obligation, rather than one more example?
- Do exports, crate boundaries, and documentation describe meaning rather than file topology?

If several answers are no, revise the semantic surface before adding machinery.

## Lineage and further reading

- [ALUX programming guidelines](https://alux-network.github.io/alux-programming/) teach this method in full, from denotations and laws to capability algebras and first-order programs in Rust.
- [`alux-rust`](https://github.com/alux-network/alux-rust) is where the `alux-*` crates this workspace depends on are developed, and where the same rules are applied to foundation crates.
- [Conal Elliott's website](http://conal.net/)
- [Denotational Design: from meanings to programs](https://github.com/conal/talk-2014-bayhac-denotational-design)
- [LambdaJam Denotational Design workshop](https://github.com/conal/talk-2014-lambdajam-denotational-design)
- [Denotational design with type class morphisms](http://conal.net/papers/type-class-morphisms/)
