//! Law scenarios for reading a tweet, stated once over the capabilities.
//!
//! Everything a tweet source decides before it asks anything is decided here: which addresses
//! name a tweet, what asking about one is, and which files a way of taking accepts. A scenario
//! authors its own addresses, so it constrains any interpreter of the sorts.

use crate::*;
use alux_ext::ext;
use anyhow::{Result, bail};

/// Names one tweet every scenario reads addresses of.
const LAW_STATUS: &str = "1234567890123456789";

/// Authors the tweet-reading laws.
#[ext(name = XLaws)]
pub impl<This> This
where
    This: XRequestAlg + XRequestViewAlg,
{
    /// Checks that an address, a request, and a way of taking each mean one thing.
    ///
    /// The laws checked are:
    ///
    /// 1. every address of one tweet names that tweet, however it was written;
    /// 2. an address naming no tweet names none, whatever else it looks like;
    /// 3. asking about one tweet is a function of which tweet it is, and always carries a token;
    /// 4. every way of taking is named by its own word and says what it does;
    /// 5. taking everything accepts every kind, and each other way accepts its own and no other;
    /// 6. every kind the answer names reads back as itself.
    ///
    /// # Errors
    ///
    /// Returns the first violated law.
    fn x_laws(&self) -> Result<()> {
        self.check_address_laws()?;
        self.check_request_laws()?;
        check_taking_laws()
    }

    /// Checks which addresses name a tweet.
    ///
    /// # Errors
    ///
    /// Returns the first violated law.
    fn check_address_laws(&self) -> Result<()> {
        for host in X_HOSTS {
            for shape in [
                format!("https://{host}/somebody/status/{LAW_STATUS}"),
                format!("http://{host}/somebody/status/{LAW_STATUS}"),
                format!("https://{host}/somebody/statuses/{LAW_STATUS}"),
                format!("https://{host}/somebody/status/{LAW_STATUS}?s=20&t=abc"),
                format!("https://{host}/somebody/status/{LAW_STATUS}/photo/1"),
                format!("  https://{host}/somebody/status/{LAW_STATUS}  "),
            ] {
                if status_id(&shape).as_deref() != Some(LAW_STATUS) {
                    bail!("{shape} does not name the tweet it addresses");
                }
            }
        }
        for shape in [
            "https://example.test/somebody/status/1234567890123456789",
            "https://x.com/somebody",
            "https://x.com/somebody/status/",
            "https://x.com/somebody/status/not-a-number",
            "https://x.com.example.test/somebody/status/1234567890123456789",
            "/home/dev/x.com/somebody/status/1234567890123456789",
            "https://www.youtube.com/watch?v=VIDEO_ID",
        ] {
            if let Some(named) = status_id(shape) {
                bail!("{shape} names the tweet {named}, and addresses no tweet");
            }
        }
        Ok(())
    }

    /// Checks what asking about one tweet is.
    ///
    /// # Errors
    ///
    /// Returns the first violated law.
    fn check_request_laws(&self) -> Result<()> {
        let asked = self.status_request(LAW_STATUS);
        let asked_again = self.status_request(LAW_STATUS);
        let address = self.request_address(&asked).to_owned();
        if address != self.request_address(&asked_again) {
            bail!("asking twice about one tweet asks two different things");
        }
        if !address.starts_with(READING_ADDRESS) {
            bail!("a tweet is asked about somewhere other than where tweets are read");
        }
        if !address.contains(&format!("id={LAW_STATUS}")) {
            bail!("asking about a tweet does not name which tweet");
        }
        // The address answers with nothing at all unless it is asked with a token.
        let stated = reading_token(LAW_STATUS);
        if stated.is_empty() || !address.contains(&format!("token={stated}")) {
            bail!("asking about a tweet carries no token");
        }
        if reading_token("1") == reading_token("2") {
            bail!("two tweets are asked about with one token");
        }
        Ok(())
    }
}

/// Checks which files each way of taking accepts.
///
/// # Errors
///
/// Returns the first violated law.
fn check_taking_laws() -> Result<()> {
    for (place, way) in take::ALL.iter().copied().enumerate() {
        let named = take::to(way);
        if take::ALL[..place]
            .iter()
            .any(|earlier| take::to(*earlier) == named)
        {
            bail!("two ways of taking are named {named}");
        }
        if take::from(named) != Some(way) {
            bail!("{named} is not read back as itself");
        }
        if way.summary().is_empty() {
            bail!("{named} does not say what it does");
        }
    }
    if take::from("nothing anybody would call a way of taking").is_some() {
        bail!("a word naming no way of taking names one");
    }

    for kind in attachment_kind::ALL.iter().copied() {
        if !Take::Everything.accepts(kind) {
            bail!(
                "taking everything leaves {} behind",
                attachment_kind::to(kind)
            );
        }
        // Every kind is taken by exactly one of the ways that are not everything, so no file a
        // tweet carries is unreachable and none is offered twice.
        let ways = [Take::Videos, Take::Images]
            .into_iter()
            .filter(|way| way.accepts(kind))
            .count();
        if ways != 1 {
            bail!(
                "{} is taken by {ways} ways rather than one",
                attachment_kind::to(kind)
            );
        }
        if attachment_kind::from(attachment_kind::to(kind)) != Some(kind) {
            bail!("{} is not read back as itself", attachment_kind::to(kind));
        }
    }
    Ok(())
}
