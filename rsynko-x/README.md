# rsynko-x

`rsynko-x` states what one public tweet carries, and how to ask. A tweet is a *bundle*: it may hold pictures, a video at several sizes, or an animation, and whoever asks either takes all of it or picks one file out of it. Nothing here fetches anything — asking is a request value, and reading an answer is what an interpreter does with it.

Only public tweets are read, through the address that answers an embedded tweet on somebody else's page. There is no account, no cookie, and nothing anybody has to be given.

## Specification surface

[`status_id`] observes the tweet one address names, and nothing else about it: which of [`X_HOSTS`] it was linked by, who posted it, and everything after the question mark say nothing about which tweet it is.

```rust
use rsynko_x::status_id;

let named = "https://x.com/somebody/status/1234567890123456789";
assert_eq!(status_id(named).as_deref(), Some("1234567890123456789"));
assert_eq!(status_id("https://x.com/somebody").as_deref(), None);
assert_eq!(status_id("/home/dev/photos/2026").as_deref(), None);
```

[`AttachmentKind`] names what a tweet carries, in the words the answer itself uses, and [`Take`] names the ways of taking them. Both read back from what they are called, and `ALL` states every one:

```rust
use rsynko_x::{AttachmentKind, Take, attachment_kind, take};

assert_eq!(attachment_kind::from("animated_gif"), Some(AttachmentKind::Animation));
assert_eq!(take::to(Take::Images), "images");
assert_eq!(take::from("everything"), Some(Take::Everything));

assert!(Take::Videos.accepts(AttachmentKind::Animation));
assert!(!Take::Videos.accepts(AttachmentKind::Photo));
```

## Extension

An interpreter states two sorts through [`XSorts`] — the request and the attachment — and implements [`XRequestAlg`] to say what a request *is*. `XRequestExt::status_request` then derives the whole of asking about one tweet:

```rust
use rsynko_x::{XRequestAlg, XRequestExt, XRequestViewAlg, XSorts};

/// Asks by writing the address down.
struct Written;

impl XSorts for Written {
    type Request = String;
    type Attachment = (String, String);
}

impl XRequestAlg for Written {
    fn tweet_request(&self, address: impl Into<String>) -> String {
        address.into()
    }
}

impl XRequestViewAlg for Written {
    fn request_address<'a>(&self, request: &'a String) -> &'a str {
        request
    }
}

let asked = Written.status_request("1234567890123456789");
assert!(asked.starts_with("https://cdn.syndication.twimg.com/tweet-result"));
assert!(asked.contains("id=1234567890123456789"));
assert!(asked.contains("token="));
```

The token is derived from the identity rather than agreed with anybody: the address answers with nothing unless it is asked with one, and what the token *is* does not decide the answer — only that there is one.

[`XAttachmentAlg`] and [`XAttachmentViewAlg`] state one file the tweet carries: what it is called among the others, what kind it is, and where its bytes are fetched from.

## Laws

[`XLaws::x_laws`] checks all of it against any interpreter: that every address of one tweet names it however it was written, that an address naming no tweet names none, that asking about one tweet is a function of which tweet it is and always carries a token, that every way of taking is named by its own word, and that every kind a tweet carries is taken by exactly one way besides everything.
