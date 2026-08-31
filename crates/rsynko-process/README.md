# rsynko-process

`rsynko-process` runs the transfers `rsynko-rsync` states, as an operating-system process.

It decides nothing. The command, including every argument, is stated by `rsynko-rsync`; this crate spawns it, hands each line it writes back in order, and reports how it ended. What those lines mean is read by the specification, not here — this crate supplies `SyncRunAlg` and `SyncWatchAlg` and takes every other meaning from the in-memory interpretation of the transfer sorts.

<details open>
<summary>Running one stated command</summary>

```rust no_run
use rsynko_process::ProcessSyncEnv;
use rsynko_rsync::{RsyncEndpointExt, SyncCommandExt, SyncMode, SyncProgramExt};
use std::sync::mpsc::channel;

let (observations, watched) = channel();
let environment = ProcessSyncEnv::new(observations);
let command = environment.transfer_command(
    &environment.read_endpoint("backup@nas.local:/volume1/photos"),
    &environment.read_endpoint("/home/dev/photos"),
    SyncMode::rehearsal(),
);

let changes = environment.run_sync(&command)?;
println!("{} paths would change", changes.len());
while let Ok(observation) = watched.try_recv() {
    println!("{observation:?}");
}
# Ok::<(), rsynko_process::ProcessSyncError>(())
```

</details>

A transfer writes its progress separated by carriage returns rather than newlines, so a line here ends at either one. The process is asked for its output only; whatever it wrote to its error stream is kept and stated as the reason a transfer failed.
