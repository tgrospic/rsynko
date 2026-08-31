/// Names the hosts one tweet is addressed by.
///
/// The service has been called two things and serves the same tweet under both, along with the
/// hosts its own applications use. A tweet is one tweet however it was linked to.
pub const X_HOSTS: [&str; 6] = [
    "x.com",
    "www.x.com",
    "mobile.x.com",
    "twitter.com",
    "www.twitter.com",
    "mobile.twitter.com",
];

/// Names what stands between the person and the tweet in an address.
const STATUS_WORDS: [&str; 2] = ["status", "statuses"];

/// Observes the tweet one address names, when it names one.
///
/// What is read is the identity and nothing else: who posted it, what was linked from where, and
/// everything after the question mark say nothing about which tweet it is.
#[must_use]
pub fn status_id(input: &str) -> Option<String> {
    let without_scheme = input
        .trim()
        .strip_prefix("https://")
        .or_else(|| input.trim().strip_prefix("http://"))?;
    let (authority, path) = without_scheme.split_once('/')?;
    let host = authority.split('@').next_back()?.split(':').next()?;
    if !X_HOSTS.contains(&host) {
        return None;
    }
    let path = path.split(['?', '#']).next()?;
    let mut segments = path.split('/').filter(|segment| !segment.is_empty());
    let _person = segments.next()?;
    if !STATUS_WORDS.contains(&segments.next()?) {
        return None;
    }
    let id = segments.next()?;
    (!id.is_empty() && id.chars().all(|digit| digit.is_ascii_digit())).then(|| id.to_owned())
}
