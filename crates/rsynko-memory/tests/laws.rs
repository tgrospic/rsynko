//! Every specification law scenario, run against the reference interpreters.
//!
//! The scenarios live in the specification crates and provide their own data, so this file names
//! them and supplies nothing. A scenario method and its test share a name, which makes the method
//! name the test's description.

use rsynko_download::DownloadLaws;
use rsynko_manager::*;
use rsynko_media::*;
use rsynko_memory::{
    MemoryManager, ReferenceLaws, ReferenceSession, ReferenceXEnv, ReferenceYoutubeLaws,
};
use rsynko_rsync::*;
use rsynko_session::*;
use rsynko_ui::*;
use rsynko_x::*;
use rsynko_yt::*;

/// Runs one named specification law scenario against the reference interpreters.
///
/// A scenario naming no context runs against the generic composition; Youtube retrieval consumes
/// the reified request instead of a URL, so those scenarios name their own.
macro_rules! law_test {
    ($(#[$meta:meta])* $name:ident) => {
        law_test!($(#[$meta])* $name in ReferenceLaws);
    };
    ($(#[$meta:meta])* $name:ident in $context:ty) => {
        $(#[$meta])*
        #[test]
        fn $name() {
            // Build the reference context once so the scenario runs against the same
            // interpreters the crate ships, then let the scenario drive itself.
            // Scenarios that configure their situation take `&mut self`; the rest do not.
            #[allow(unused_mut)]
            let mut reference = <$context>::default();
            reference
                .$name()
                .unwrap_or_else(|error| panic!("{} violated: {error:#}", stringify!($name)));
        }
    };
}

law_test!(observation_laws);
law_test!(format_laws);
law_test!(artifact_laws);
law_test!(selection_laws);
law_test!(processing_laws);
law_test!(processing_stage_laws);
law_test!(processing_failure_laws);
law_test!(output_naming_laws);
law_test!(extraction_laws);
law_test!(download_laws);
law_test!(download_fetch_failure_laws);
law_test!(download_publication_failure_laws);
law_test!(youtube_url_laws);
law_test!(youtube_granting_laws);
law_test!(youtube_solution_laws);
law_test!(youtube_request_laws);
law_test!(youtube_response_laws in ReferenceYoutubeLaws);
law_test!(youtube_extraction_laws in ReferenceYoutubeLaws);
law_test!(youtube_application_laws in ReferenceYoutubeLaws);
law_test!(media_program_laws);
law_test!(navigation_laws);
law_test!(draft_laws);
law_test!(text_editor_laws);
law_test!(queue_laws);
law_test!(transition_laws);
law_test!(menu_laws);
law_test!(options_laws);
law_test!(log_laws);
law_test!(intent_laws);
law_test!(downloads_laws in MemoryManager);
law_test!(key_binding_laws);
law_test!(menu_presentation_laws);
law_test!(gauge_laws);
law_test!(screen_laws);
law_test!(rehearsal_laws);
law_test!(sync_laws);
law_test!(submission_laws);
law_test!(attention_laws);
law_test!(x_laws in ReferenceXEnv);
law_test!(session_laws in ReferenceSession);
