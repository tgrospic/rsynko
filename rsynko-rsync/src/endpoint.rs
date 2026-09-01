use crate::*;
use alux_ext::ext;
use ambassador::delegatable_trait;

/// Provides the carrier and constructor for one end of a transfer.
#[delegatable_trait]
pub trait RsyncEndpointAlg: RsyncSorts {
    /// Defines one endpoint from the account, the machine, and the path it names.
    ///
    /// An endpoint naming no machine names a path on this one.
    fn endpoint(&self, user: Option<String>, host: Option<String>, path: impl Into<String>) -> Self::Endpoint;
}

/// Specifies what one endpoint states about itself.
#[delegatable_trait]
pub trait RsyncEndpointViewAlg: RsyncSorts {
    /// Observes the account the far end is reached as, when one was stated.
    fn endpoint_user<'a>(&self, endpoint: &'a Self::Endpoint) -> Option<&'a str>;
    /// Observes the machine the path rests on, and states nothing when the path is here.
    fn endpoint_host<'a>(&self, endpoint: &'a Self::Endpoint) -> Option<&'a str>;
    /// Observes the path itself.
    fn endpoint_path<'a>(&self, endpoint: &'a Self::Endpoint) -> &'a str;
}

/// Names the scheme a daemon endpoint states itself with.
pub const RSYNC_SCHEME: &str = "rsync://";

/// Names the word a whole transfer command begins with.
pub const RSYNC_WORD: &str = "rsync";

/// Derives an endpoint from what somebody submitted, and states one back.
#[ext(name = RsyncEndpointExt)]
pub impl<This> This
where
    This: RsyncEndpointAlg + RsyncEndpointViewAlg,
{
    /// Reads one endpoint from the text somebody submitted.
    ///
    /// Three shapes name a path on another machine: a daemon URL, and a machine and a path
    /// separated by a colon with nothing path-like before it. A text naming any other scheme
    /// names a resource somewhere else, and a text naming neither names a path here.
    fn read_endpoint(&self, input: &str) -> This::Endpoint {
        let input = input.trim();
        if let Some(rest) = input.strip_prefix(RSYNC_SCHEME) {
            let (authority, path) = rest.split_once('/').unwrap_or((rest, ""));
            let (user, host) = split_account(authority);
            return self.endpoint(user, Some(host.to_owned()), format!("/{path}"));
        }
        if !input.contains("://")
            && let Some((authority, path)) = input.split_once(':')
            && !authority.contains('/')
            && !authority.is_empty()
        {
            let (user, host) = split_account(authority);
            return self.endpoint(user, Some(host.to_owned()), path);
        }
        self.endpoint(None, None, input)
    }

    /// Reads one submitted line as the two ends of a transfer, when it names two.
    ///
    /// A transfer is written the way it is run: what comes from, then what it comes to. Anything
    /// naming how rather than what — the program itself, and the arguments given to it — is
    /// already stated by the transfer, so a whole command can be submitted and still be read.
    fn read_transfer(&self, line: &str) -> Option<(This::Endpoint, This::Endpoint)> {
        let mut ends = line.split_whitespace().filter(|word| !word.starts_with('-') && *word != RSYNC_WORD);
        let source = ends.next()?;
        let destination = ends.next()?;
        // A line naming a third end names something this specification cannot read.
        ends.next().is_none().then(|| (self.read_endpoint(source), self.read_endpoint(destination)))
    }

    /// Observes whether the path rests on another machine.
    fn endpoint_remote(&self, endpoint: &This::Endpoint) -> bool {
        self.endpoint_host(endpoint).is_some()
    }

    /// States the endpoint the way the transfer program is given it.
    fn endpoint_text(&self, endpoint: &This::Endpoint) -> String {
        let path = self.endpoint_path(endpoint);
        let Some(host) = self.endpoint_host(endpoint) else {
            return path.to_owned();
        };
        match self.endpoint_user(endpoint) {
            Some(user) => format!("{user}@{host}:{path}"),
            None => format!("{host}:{path}"),
        }
    }

    /// Names what the path ends with, without the path that leads to it.
    fn endpoint_name<'a>(&self, endpoint: &'a This::Endpoint) -> &'a str {
        let path = self.endpoint_path(endpoint);
        path.trim_end_matches('/').rsplit('/').find(|segment| !segment.is_empty()).unwrap_or(path)
    }
}

/// Splits an account from the machine it reaches.
fn split_account(authority: &str) -> (Option<String>, &str) {
    authority.split_once('@').map_or((None, authority), |(user, host)| (Some(user.to_owned()), host))
}
