//! Admin authentication, ported from `server.ts`.
//!
//! Admin routes accept `Authorization: Bearer <ADMIN_TOKEN>` (used by
//! send-newsletter.mjs), HTTP Basic auth with the token as the password (any
//! username), or a session cookie from the dashboard's login form. Comparisons
//! are constant-time so a response-timing side channel can't leak the token.

use base64::Engine;
use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;

/// Name of the dashboard's session cookie.
pub const SESSION_COOKIE: &str = "nl_admin";
/// How long a dashboard login lasts.
pub const SESSION_TTL_SECS: i64 = 30 * 86_400;

/// Constant-time equality that leaks NOTHING about the inputs — not even their
/// length. We compare fixed-size SHA-256 digests, so the number of bytes examined
/// is always 32 regardless of how long (or short) the candidate token is. This
/// closes the length side channel that a plain `len() == len() && ct_eq` leaves
/// open (a mismatched-length request would otherwise return measurably faster).
pub fn safe_equal(a: &str, b: &str) -> bool {
    let ha = Sha256::digest(a.as_bytes());
    let hb = Sha256::digest(b.as_bytes());
    ha.ct_eq(&hb).into()
}

/// Validate an `Authorization` header value against the admin token.
pub fn check_admin_auth(authorization: Option<&str>, admin_token: &str) -> bool {
    let Some(auth) = authorization else {
        return false;
    };

    if let Some(token) = auth.strip_prefix("Bearer ") {
        return safe_equal(token, admin_token);
    }

    if let Some(encoded) = auth.strip_prefix("Basic ") {
        let Ok(decoded) = base64::engine::general_purpose::STANDARD.decode(encoded) else {
            return false;
        };
        let Ok(decoded) = String::from_utf8(decoded) else {
            return false;
        };
        // `user:password` — the password is everything after the first colon.
        let password = match decoded.find(':') {
            Some(i) => &decoded[i + 1..],
            None => "",
        };
        return safe_equal(password, admin_token);
    }

    false
}

// ---- dashboard sessions ----
//
// A session is `<expiry unix secs>.<hex HMAC-SHA256(ADMIN_TOKEN, "admin-session:v1:<expiry>")>`.
// Nothing is stored server-side: sessions survive restarts, and changing
// ADMIN_TOKEN signs every session out.

fn session_signature(admin_token: &str, expires: i64) -> String {
    let mut mac = Hmac::<Sha256>::new_from_slice(admin_token.as_bytes())
        .expect("HMAC accepts keys of any length");
    mac.update(format!("admin-session:v1:{expires}").as_bytes());
    mac.finalize()
        .into_bytes()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// A new session value, valid for [`SESSION_TTL_SECS`] from `now`.
pub fn new_session(admin_token: &str, now: i64) -> String {
    let expires = now + SESSION_TTL_SECS;
    format!("{expires}.{}", session_signature(admin_token, expires))
}

/// Whether a session value is authentic and unexpired at `now`.
pub fn check_session(value: &str, admin_token: &str, now: i64) -> bool {
    let Some((expires, signature)) = value.split_once('.') else {
        return false;
    };
    let Ok(expires) = expires.parse::<i64>() else {
        return false;
    };
    // An expiry further out than any session we'd issue can't be ours.
    if expires <= now || expires > now + SESSION_TTL_SECS {
        return false;
    }
    safe_equal(signature, &session_signature(admin_token, expires))
}

/// The session value from a `Cookie` request header, if present.
pub fn session_from_cookies(cookie_header: &str) -> Option<&str> {
    cookie_header.split(';').find_map(|pair| {
        let (name, value) = pair.trim().split_once('=')?;
        (name == SESSION_COOKIE).then_some(value)
    })
}
#[cfg(test)]
mod tests {
    use super::*;

    const TOKEN: &str = "s3cret-token";

    fn basic(creds: &[u8]) -> String {
        format!(
            "Basic {}",
            base64::engine::general_purpose::STANDARD.encode(creds)
        )
    }

    #[test]
    fn bearer_token() {
        assert!(check_admin_auth(Some("Bearer s3cret-token"), TOKEN));
        assert!(!check_admin_auth(Some("Bearer s3cret-toke"), TOKEN));
        assert!(!check_admin_auth(Some("Bearer s3cret-token "), TOKEN));
        assert!(!check_admin_auth(Some("Bearer "), TOKEN));
        assert!(!check_admin_auth(Some("s3cret-token"), TOKEN), "no scheme");
        assert!(!check_admin_auth(None, TOKEN));
    }

    #[test]
    fn basic_auth_uses_the_password_with_any_username() {
        assert!(check_admin_auth(Some(&basic(b"admin:s3cret-token")), TOKEN));
        assert!(check_admin_auth(Some(&basic(b":s3cret-token")), TOKEN));
        assert!(!check_admin_auth(Some(&basic(b"s3cret-token:x")), TOKEN));
        // Only the first colon separates: a password may contain colons.
        assert!(check_admin_auth(Some(&basic(b"u:a:b")), "a:b"));
    }

    #[test]
    fn malformed_basic_auth_is_rejected() {
        assert!(
            !check_admin_auth(Some(&basic(b"s3cret-token")), TOKEN),
            "no colon"
        );
        assert!(!check_admin_auth(Some("Basic %%%not-base64"), TOKEN));
        assert!(
            !check_admin_auth(Some(&basic(b"u:\xff\xfe")), TOKEN),
            "not UTF-8"
        );
    }

    const NOW: i64 = 1_800_000_000;

    #[test]
    fn a_fresh_session_checks_out_until_it_expires() {
        let session = new_session(TOKEN, NOW);
        assert!(check_session(&session, TOKEN, NOW));
        assert!(check_session(&session, TOKEN, NOW + SESSION_TTL_SECS - 1));
        assert!(!check_session(&session, TOKEN, NOW + SESSION_TTL_SECS));
    }

    #[test]
    fn sessions_cannot_be_forged_or_extended() {
        let session = new_session(TOKEN, NOW);
        assert!(
            !check_session(&session, "rotated-token", NOW),
            "token change signs out"
        );

        let (_, signature) = session.split_once('.').unwrap();
        let extended = format!("{}.{signature}", NOW + SESSION_TTL_SECS + 100);
        assert!(!check_session(
            &extended,
            TOKEN,
            NOW + SESSION_TTL_SECS + 50
        ));

        let far = NOW + 10 * SESSION_TTL_SECS;
        let far_session = format!("{far}.{}", session_signature(TOKEN, far));
        assert!(
            !check_session(&far_session, TOKEN, NOW),
            "beyond the max lifetime"
        );

        for bad in ["", ".", "abc", "123", "123.", "x.y", &format!("{NOW}.00")] {
            assert!(!check_session(bad, TOKEN, NOW), "{bad:?}");
        }
    }

    #[test]
    fn finds_the_session_among_other_cookies() {
        assert_eq!(session_from_cookies("a=1; nl_admin=v.s; b=2"), Some("v.s"));
        assert_eq!(session_from_cookies("nl_admin=v.s"), Some("v.s"));
        assert_eq!(session_from_cookies("xnl_admin=v; nl_adminx=w"), None);
        assert_eq!(session_from_cookies(""), None);
    }
}
