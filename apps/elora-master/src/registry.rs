//! List of registered servers – pure logic without network (for tests).

use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::time::{Duration, Instant};

use crate::EXPIRY;

/// Minimum interval between two registrations of the same address.
pub const MIN_REREGISTER: Duration = Duration::from_secs(5);
/// At most this many servers per IP address.
pub const MAX_PER_IP: usize = 32;
/// At most this many servers in total (protection against flooding).
pub const MAX_SERVERS: usize = 8192;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Refused {
    TooOften,
    TooManyForIp,
    Full,
}

impl Refused {
    pub fn message(self) -> &'static str {
        match self {
            Self::TooOften => "zu häufige Anmeldung",
            Self::TooManyForIp => "zu viele Server von dieser Adresse",
            Self::Full => "Liste voll",
        }
    }
}

#[derive(Debug, Default)]
pub struct Registry {
    /// Verified servers and when they expire.
    listed: HashMap<SocketAddr, Instant>,
    /// Last registration per address (rate limit).
    last_request: HashMap<SocketAddr, Instant>,
}

impl Registry {
    /// Check a registration; on `Ok` the caller must query the server via UDP and
    /// report the result with [`Registry::verified`].
    ///
    /// # Errors
    /// On too frequent registrations or exceeded limits.
    pub fn request(&mut self, addr: SocketAddr, now: Instant) -> Result<(), Refused> {
        if self
            .last_request
            .get(&addr)
            .is_some_and(|t| now - *t < MIN_REREGISTER)
        {
            return Err(Refused::TooOften);
        }
        if !self.listed.contains_key(&addr) {
            let same_ip = self.listed.keys().filter(|a| a.ip() == addr.ip()).count();
            if same_ip >= MAX_PER_IP {
                return Err(Refused::TooManyForIp);
            }
            if self.listed.len() >= MAX_SERVERS {
                return Err(Refused::Full);
            }
        }
        self.last_request.insert(addr, now);
        Ok(())
    }

    /// Result of the UDP check: reachable and matching version → listed (further on).
    pub fn verified(&mut self, addr: SocketAddr, ok: bool, now: Instant) {
        if ok {
            self.listed.insert(addr, now + EXPIRY);
        }
    }

    /// Remove expired entries.
    pub fn expire(&mut self, now: Instant) {
        self.listed.retain(|_, until| now < *until);
        self.last_request
            .retain(|_, t| now - *t < MIN_REREGISTER.max(EXPIRY));
    }

    /// Current list (sorted so that replies are stable).
    pub fn list(&self) -> Vec<SocketAddr> {
        let mut v: Vec<SocketAddr> = self.listed.keys().copied().collect();
        v.sort();
        v
    }

    pub fn count_for(&self, ip: IpAddr) -> usize {
        self.listed.keys().filter(|a| a.ip() == ip).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn a(port: u16) -> SocketAddr {
        SocketAddr::from(([10, 0, 0, 1], port))
    }

    #[test]
    fn register_verify_expire() {
        let mut r = Registry::default();
        let t0 = Instant::now();
        r.request(a(8303), t0).unwrap();
        assert!(r.list().is_empty(), "erst nach der UDP-Prüfung gelistet");
        r.verified(a(8303), true, t0);
        assert_eq!(r.list(), vec![a(8303)]);
        assert_eq!(
            r.request(a(8303), t0 + Duration::from_secs(1)),
            Err(Refused::TooOften)
        );
        // re-registering extends
        r.request(a(8303), t0 + Duration::from_secs(30)).unwrap();
        r.verified(a(8303), true, t0 + Duration::from_secs(30));
        r.expire(t0 + EXPIRY + Duration::from_secs(1));
        assert_eq!(r.list(), vec![a(8303)]);
        r.expire(t0 + Duration::from_secs(30) + EXPIRY);
        assert!(r.list().is_empty());
        // unreachable → not listed
        r.request(a(9000), t0).unwrap();
        r.verified(a(9000), false, t0);
        assert!(r.list().is_empty());
    }

    #[test]
    fn per_ip_limit() {
        let mut r = Registry::default();
        let t = Instant::now();
        for p in 0..u16::try_from(MAX_PER_IP).unwrap() {
            r.request(a(10000 + p), t).unwrap();
            r.verified(a(10000 + p), true, t);
        }
        assert_eq!(r.request(a(20000), t), Err(Refused::TooManyForIp));
        assert_eq!(r.count_for(a(1).ip()), MAX_PER_IP);
        // a different IP still works
        let other = SocketAddr::from(([10, 0, 0, 2], 8303));
        assert!(r.request(other, t).is_ok());
    }
}
