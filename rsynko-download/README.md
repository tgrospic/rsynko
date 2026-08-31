# rsynko-download

This crate specifies one-resource download meaning independently of HTTP and filesystems.

## Capabilities

[`FetchStreamAlg`] denotes incremental retrieval as a carrier and a step, so an interpreter chooses whether stepping reads memory, walks a manifest, or waits on a network. [`AtomicPublishAlg`] denotes publication without an observable partial destination: an unpublished carrier is written, then committed or abandoned. [`DownloadObservationAlg`] defines the progress and terminal observations, and [`DownloadProgressAlg`] and [`DownloadReportAlg`] emit them.

An interpreter chooses the stream, the unpublished carrier, and the observation carriers; a reference interpreter reifying the observations lives in `rsynko-memory`.

## The derived program

[`DownloadExt::download_resource`] derives the complete operation from exactly those capabilities: it fetches into an unpublished carrier, commits it atomically at the final path, and emits exactly one terminal observation. A caller composes it further without acquiring anything new:

```rust
use alux_ext::ext;
use rsynko_download::{
    AtomicPublishAlg, DownloadError, DownloadExt, DownloadObservationAlg, DownloadProgressAlg,
    DownloadReportAlg, FetchStreamAlg,
};
use std::fmt::Display;
use std::path::Path;

#[ext(name = DownloadBatchExt)]
impl<This, Source, FetchError, Stream, PublishError, Event, Progress> This
where
    Source: ?Sized,
    This: FetchStreamAlg<Source, Error = FetchError, Stream = Stream>
        + AtomicPublishAlg<Error = PublishError>
        + DownloadObservationAlg<Event = Event, Progress = Progress>
        + DownloadProgressAlg<Progress = Progress>
        + DownloadReportAlg<Event = Event>,
    FetchError: Display,
    PublishError: Display,
{
    /// Publishes several resources in declaration order and stops at the first failure.
    fn download_resources<'a>(
        &self,
        requests: impl IntoIterator<Item = (&'a Source, &'a Path)>,
    ) -> Result<u64, DownloadError<FetchError, PublishError>>
    where
        Source: 'a,
    {
        let mut published = 0;
        for (source, destination) in requests {
            published += self.download_resource(source, destination)?;
        }
        Ok(published)
    }
}
```

## Observation operations

`DownloadObservationOp` reifies progress and terminal reports as inspectable stream elements, and `DownloadObservationInterpreter` interprets one element without choosing a channel or owning a consumption loop. Both are generated from the observation vocabulary, so an interpreter states what an observation means and never restates its shape.

## Laws

- progress begins at zero and retrieved byte counts are monotonic;
- successful publication preserves the fetched bytes exactly;
- the final path never denotes partial bytes;
- every execution emits exactly one terminal event;
- success is reported only after atomic publication succeeds.
