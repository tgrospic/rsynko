# rsynko-session

`rsynko-session` states the relation between what is *intended* and what is *happening*. A collection states that a request wants work done; the world holds runs — threads, processes, anything this program cannot reach into. Attendance is the reconciliation of the two, and it is the same reconciliation whether the run downloads a resource, transfers a folder, or reads what a source offers.

Attendance is not recorded twice. A request that has a run is no longer one that wants a run, so what the requests themselves state is the whole record of what is running.

## Specification surface

An interpreter states five sorts through [`SessionSorts`] — the request identity, the run, what a run says, how it ends, and how it refuses — and implements four capabilities over them:

- [`UndertakingAlg`] states which requests want work and what beginning it is;
- [`RunReadAlg`] states whether a run is over, what it has said since it was last read, and how it ended;
- [`RunHoldAlg`] holds a run still, lets it go, and tells one nobody wants to end;
- [`AttentionAlg`] states what all of that means to whoever asked, and reads back what they want now as [`Wanted`].

Time is read through [`ClockAlg`], so how long a run has been running is a statement about a clock the interpreter supplies. [`Attending`] carries one run together with the time it has spent running and held still.

## Attention

[`SessionExt::attend`] is one pass of attention, derived from those capabilities and stating nothing of its own beyond the order it takes them in. A program written over the same capabilities composes with it:

```rust
use alux_ext::ext;
use rsynko_session::*;

#[ext(name = ExampleSessionExt)]
impl<This> This
where
    This: UndertakingAlg + RunReadAlg + RunHoldAlg + AttentionAlg + ClockAlg,
{
    /// Attends until nothing is running and nothing is left wanting to run.
    fn example_attend_until_settled(
        &mut self,
        running: &mut Vec<Attending<This::Id, This::Run, This::Moment>>,
    ) {
        while !running.is_empty() || !self.unattended().is_empty() {
            self.attend(running);
        }
    }
}
```

The order that pass takes is the meaning it adds. Ending comes before beginning, so leaving never races starting. A run is seen to be over before it is read, so everything it wrote before ending is read. What it said is stated before how it ended, so nothing it said is lost to the ending that followed it. And a run nobody wants is told to end on every pass until it is over, because a run that has not begun yet cannot be told to stop.

Leaving is not a separate path: an interpreter whose [`Wanted`] answers `Unwanted` once the reader is leaving ends every run through the ordinary pass.

## Laws

[`SessionLaws::session_laws`] checks all of that against any interpreter, over the runs a [`SessionLawFixture`] authors, reading back what each request was told as [`Telling`].
