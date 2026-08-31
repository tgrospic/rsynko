#![doc = include_str!("../README.md")]

mod attention;
mod clock;
mod hold;
mod laws;
mod run;
mod session;
mod sorts;
mod undertaking;

pub use attention::{AttentionAlg, Wanted};
pub use clock::{Attending, ClockAlg};
pub use hold::RunHoldAlg;
pub use laws::{SessionLawFixture, SessionLaws, Telling};
pub use run::RunReadAlg;
pub use session::SessionExt;
pub use sorts::SessionSorts;
pub use undertaking::UndertakingAlg;

// Ambassador's generated delegation macros, re-exported so interpreters can compose.
pub use attention::ambassador_impl_AttentionAlg;
pub use clock::ambassador_impl_ClockAlg;
pub use hold::ambassador_impl_RunHoldAlg;
pub use run::ambassador_impl_RunReadAlg;
pub use undertaking::ambassador_impl_UndertakingAlg;
