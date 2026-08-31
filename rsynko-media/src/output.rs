use crate::MediaSorts;
use alux_ext::ext;
use ambassador::delegatable_trait;
use std::path::PathBuf;

/// Provides the carrier and constructor for one portable output name.
#[delegatable_trait]
pub trait OutputNameAlg: MediaSorts {
    /// Defines an output from one already normalized file component.
    fn output_name(&self, component: String) -> Self::Output;
}

/// Derives portable output names independently of their carrier.
#[ext(name = OutputNameExt)]
pub impl<This> This
where
    This: OutputNameAlg,
{
    /// Defines a portable file name from title, identity, and extension.
    fn portable_output_name(
        &self,
        title: Option<&str>,
        fallback: &str,
        extension: Option<&str>,
    ) -> This::Output {
        self.output_name(portable_file_component(title, fallback, extension))
    }

    /// Defines a portable edited file name.
    fn portable_user_output_name(
        &self,
        input: &str,
        fallback_extension: Option<&str>,
    ) -> This::Output {
        self.output_name(portable_user_file_component(input, fallback_extension))
    }
}

const MAX_STEM_BYTES: usize = 200;

/// Derives a portable single-component file stem from a title and fallback identity.
#[must_use]
pub fn portable_file_stem(title: Option<&str>, fallback: &str) -> String {
    let candidate = title
        .filter(|title| !title.trim().is_empty())
        .unwrap_or(fallback);
    let mut stem = String::new();
    let mut whitespace = false;
    for character in candidate.chars() {
        let character = if character.is_control()
            || matches!(
                character,
                '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*'
            ) {
            '_'
        } else if character.is_whitespace() {
            whitespace = true;
            continue;
        } else {
            character
        };
        if whitespace && !stem.is_empty() && stem.len() < MAX_STEM_BYTES {
            stem.push(' ');
        }
        whitespace = false;
        if stem.len().saturating_add(character.len_utf8()) > MAX_STEM_BYTES {
            break;
        }
        stem.push(character);
    }
    let stem = stem.trim_matches([' ', '.']);
    let stem = if stem.is_empty() { "download" } else { stem };
    if is_windows_reserved(stem) {
        format!("_{stem}")
    } else {
        stem.to_owned()
    }
}

/// Derives a portable file name from a title, fallback identity, and selected extension.
#[must_use]
pub fn portable_file_name(title: Option<&str>, fallback: &str, extension: Option<&str>) -> PathBuf {
    PathBuf::from(portable_file_component(title, fallback, extension))
}

/// Normalizes an edited file name while preserving a valid explicit extension.
#[must_use]
pub fn portable_user_file_name(input: &str, fallback_extension: Option<&str>) -> PathBuf {
    PathBuf::from(portable_user_file_component(input, fallback_extension))
}

/// Normalizes an edited name into one portable file component.
fn portable_user_file_component(input: &str, fallback_extension: Option<&str>) -> String {
    let trimmed = input.trim();
    let (stem, extension) =
        trimmed
            .rsplit_once('.')
            .map_or((trimmed, fallback_extension), |(stem, extension)| {
                if valid_extension(extension) {
                    (stem, Some(extension))
                } else {
                    (trimmed, fallback_extension)
                }
            });
    portable_file_component(Some(stem), "download", extension)
}

fn portable_file_component(title: Option<&str>, fallback: &str, extension: Option<&str>) -> String {
    let stem = portable_file_stem(title, fallback);
    let extension = extension
        .filter(|extension| valid_extension(extension))
        .unwrap_or("bin");
    format!("{stem}.{extension}")
}

fn valid_extension(extension: &str) -> bool {
    !extension.is_empty()
        && extension
            .chars()
            .all(|character| character.is_ascii_alphanumeric())
}

fn is_windows_reserved(stem: &str) -> bool {
    let basename = stem.split('.').next().unwrap_or(stem).to_ascii_uppercase();
    matches!(basename.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        || basename
            .strip_prefix("COM")
            .or_else(|| basename.strip_prefix("LPT"))
            .is_some_and(|number| {
                matches!(number, "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9")
            })
}

#[cfg(test)]
mod tests {
    use super::{portable_file_name, portable_file_stem, portable_user_file_name};
    use std::path::PathBuf;

    #[test]
    fn title_name_is_portable_across_linux_and_windows() {
        assert_eq!(
            portable_file_name(Some("  A/B: C?  "), "id", Some("mp4")),
            PathBuf::from("A_B_ C_.mp4")
        );
        assert_eq!(portable_file_stem(Some("CON.txt"), "id"), "_CON.txt");
        assert_eq!(portable_file_stem(Some("..."), "id"), "download");
    }

    #[test]
    fn absent_title_uses_the_media_identity() {
        assert_eq!(
            portable_file_name(None, "video/id", None),
            PathBuf::from("video_id.bin")
        );
    }

    #[test]
    fn edited_name_preserves_a_portable_extension() {
        assert_eq!(
            portable_user_file_name("My: video.webm", Some("mp4")),
            PathBuf::from("My_ video.webm")
        );
        assert_eq!(
            portable_user_file_name("My video", Some("mp4")),
            PathBuf::from("My video.mp4")
        );
    }
}
