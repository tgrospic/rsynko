# rsynko-reqwest

This crate executes fully described requests with Reqwest and publishes bytes with the filesystem. Youtube URL recognition, endpoint selection, headers, request bodies, response parsing, and format ranking belong to the root `rsynko-yt` specification.

The assembled executable also provides a deterministic fixture interpreter:

- `fixture://single-video` denotes deterministic media with an exact byte payload;
- resources are fetched with Rustls-backed HTTP;
- final files are published by a same-directory `.part` file and atomic rename.

A running retrieval states what it is doing to whoever is watching: `runtime_observation_channel` states one channel of the specification's `DownloadObservationOp` values, sent from the thread that retrieves and read by whoever holds the other end. Nothing is answered and nothing is waited for — a retrieval states what it does whether or not it is read, and reading it is the application's own pass.

Downloading one audio-only or video-only adaptive stream requires no merger. Combining separate adaptive audio and video streams, signature deciphering, proof-of-origin tokens, and ffmpeg merging remain outside this interpretation.

X is read the same way. `rsynko-x` decides which addresses are tweets and what request reads one; this crate sends that request, parses the answer, and turns every image and video in the tweet into a format that can be chosen.
