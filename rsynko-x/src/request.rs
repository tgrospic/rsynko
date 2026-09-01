use crate::XSorts;
use alux_ext::ext;
use ambassador::delegatable_trait;

/// Provides the carrier and constructor for exactly the request that asks about one tweet.
#[delegatable_trait]
pub trait XRequestAlg: XSorts {
    /// Defines the request asking one address what it holds.
    fn tweet_request(&self, address: impl Into<String>) -> Self::Request;
}

/// Specifies what one request states about itself.
pub trait XRequestViewAlg: XSorts {
    /// Observes the address the request asks.
    fn request_address<'a>(&self, request: &'a Self::Request) -> &'a str;
}

/// Derives the request that asks what one tweet carries.
#[ext(name = XRequestExt)]
pub impl<This> This
where
    This: XRequestAlg,
{
    /// States the request asking what the tweet one identity names carries.
    fn status_request(&self, id: &str) -> This::Request {
        self.tweet_request(format!("{READING_ADDRESS}?id={id}&lang=en&token={}", reading_token(id)))
    }
}

/// Names the address one public tweet is read from.
///
/// It is the address that answers an embedded tweet on somebody else's page, which is why it
/// answers at all without an account, a cookie, or anything anybody has to be given.
pub const READING_ADDRESS: &str = "https://cdn.syndication.twimg.com/tweet-result";

/// States the token one tweet is read with.
///
/// The address answers with nothing unless it is asked with a token. What the token *is* does not
/// decide the answer — only that there is one — so this derives one from the identity itself: the
/// request for a tweet stays a function of which tweet it is, and there is no shared secret and
/// nothing to keep in step with anybody.
#[must_use]
pub fn reading_token(id: &str) -> String {
    const DIGITS: &[u8; 36] = b"0123456789abcdefghijklmnopqrstuvwxyz";
    let mut remaining = id
        .bytes()
        .fold(0_u128, |carried, digit| carried.saturating_mul(10).saturating_add(u128::from(digit.wrapping_sub(b'0'))));
    let mut stated = Vec::new();
    while remaining > 0 {
        stated.push(DIGITS[usize::try_from(remaining % 36).unwrap_or_default()]);
        remaining /= 36;
    }
    if stated.is_empty() {
        stated.push(DIGITS[0]);
    }
    stated.reverse();
    String::from_utf8(stated).unwrap_or_default()
}
