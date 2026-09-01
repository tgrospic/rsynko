//! Checks that construction stays parametric in the carrier, using a counting interpreter.

use rsynko_media::{MediaSorts, OutputNameAlg, OutputNameExt};
use rsynko_memory::MediaSyntax;
use std::path::PathBuf;

struct LengthSyntax;

impl MediaSorts for LengthSyntax {
    type Value = ();
    type Metadata = ();
    type Format = ();
    type Artifact = ();
    type Media = ();
    type Extraction = ();
    type Extractor = ();
    type Predicate = ();
    type Selection = ();
    type Output = usize;
}

impl OutputNameAlg for LengthSyntax {
    fn output_name(&self, component: String) -> Self::Output {
        component.len()
    }
}

#[test]
fn normalization_is_independent_of_the_output_carrier() {
    let path = MediaSyntax.portable_output_name(Some("A/B"), "id", Some("mp4"));
    assert_eq!(path, PathBuf::from("A_B.mp4"));
    assert_eq!(LengthSyntax.portable_output_name(Some("A/B"), "id", Some("mp4")), 7);
}
