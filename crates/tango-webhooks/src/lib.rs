//! HMAC-SHA256 signing and verification for Tango webhook deliveries.
//!
//! Tango signs each webhook delivery with an HTTP header of the form:
//!
//! ```text
//! X-Tango-Signature: sha256=<lowercase hex HMAC-SHA256 of raw body>
//! ```
//!
//! The signature is computed over the **raw request body bytes**, keyed by the
//! endpoint's secret. Verify against the bytes you received off the wire —
//! re-serializing a parsed JSON document will produce a different signature
//! because of whitespace, key ordering, and float formatting differences.
//!
//! This crate has no transport dependency. It pulls in only `hmac`, `sha2`,
//! `subtle`, and `hex`, so a webhook receiver can verify deliveries without
//! linking the full SDK.
//!
//! # Quick start
//!
//! ```
//! use tango_webhooks::{generate, verify, SIGNATURE_HEADER};
//!
//! let body = br#"{"event_type":"alerts.contract.match"}"#;
//! let secret = "topsecret";
//!
//! // Server side (or in tests): produce a signature for `body`.
//! let header = generate(body, secret);
//! assert!(header.starts_with("sha256="));
//!
//! // Receiver side: verify the header you read off the request.
//! assert!(verify(body, &header, secret));
//! assert!(!verify(body, &header, "wrong-secret"));
//!
//! // The canonical header name to look up on the request:
//! assert_eq!(SIGNATURE_HEADER, "X-Tango-Signature");
//! ```
//!
//! # Constant-time comparison
//!
//! [`verify`] decodes both signatures to bytes and compares them with
//! [`subtle::ConstantTimeEq`]. The comparison does not short-circuit on the
//! first differing byte, which protects against timing-based signature
//! recovery attacks.
//!
//! # Why no axum/actix middleware?
//!
//! Transport adapters live behind cargo features added in a later release.
//! This crate intentionally stays tiny — a verifier service depends on
//! `tango-webhooks` alone, not the full SDK.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use hmac::{Hmac, Mac};
use sha2::Sha256;
use subtle::ConstantTimeEq;

type HmacSha256 = Hmac<Sha256>;

/// The HTTP header name Tango uses to sign webhook deliveries.
pub const SIGNATURE_HEADER: &str = "X-Tango-Signature";

/// The algorithm prefix on the header value (`"sha256="`).
pub const SIGNATURE_PREFIX: &str = "sha256=";

/// Compute the wire-format signature for `body` keyed by `secret`.
///
/// Returns a string of the form `"sha256=<lowercase hex>"`. The hex digest is
/// 64 characters long (32 bytes of HMAC-SHA256 output).
///
/// # Panics
///
/// Never. HMAC-SHA256 accepts keys of any length, so the underlying
/// `Hmac::new_from_slice` call cannot fail; the `expect` documents the
/// invariant.
///
/// # Examples
///
/// ```
/// use tango_webhooks::generate;
///
/// let sig = generate(b"hello", "shh");
/// assert_eq!(
///     sig,
///     "sha256=0e396369ee043c5b6b922743631745b2249cf7cb2c4722e61e802447d5d14c70",
/// );
/// ```
#[must_use]
pub fn generate(body: &[u8], secret: &str) -> String {
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .expect("HMAC-SHA256 accepts keys of any length");
    mac.update(body);
    let digest = mac.finalize().into_bytes();
    let mut out = String::with_capacity(SIGNATURE_PREFIX.len() + digest.len() * 2);
    out.push_str(SIGNATURE_PREFIX);
    out.push_str(&hex::encode(digest));
    out
}

/// Verify that `header` is a valid Tango signature of `body` keyed by `secret`.
///
/// Returns `false` for absent, malformed, or mismatched headers — never panics.
/// The comparison is constant-time via [`subtle::ConstantTimeEq`] on the
/// decoded digest bytes, so a caller cannot probe a valid signature byte by
/// byte using response-time differences.
///
/// Accepts both the canonical `"sha256=<hex>"` form and a bare hex string
/// (legacy compatibility, mirroring the Node and Python SDKs).
///
/// # Examples
///
/// ```
/// use tango_webhooks::{generate, verify};
///
/// let body = b"payload";
/// let header = generate(body, "secret");
/// assert!(verify(body, &header, "secret"));
/// assert!(!verify(body, &header, "wrong-secret"));
/// assert!(!verify(b"tampered", &header, "secret"));
/// assert!(!verify(body, "", "secret"));
/// ```
#[must_use]
pub fn verify(body: &[u8], header: &str, secret: &str) -> bool {
    let Some(parsed) = parse(header) else {
        return false;
    };
    if parsed.algorithm != "sha256" {
        return false;
    }
    // The expected hex is everything after the prefix in `generate`'s output.
    let expected_full = generate(body, secret);
    let Some(expected_hex) = expected_full.strip_prefix(SIGNATURE_PREFIX) else {
        // Should never happen — `generate` always emits the prefix.
        return false;
    };

    // Cheap length-based short-circuit before any decoding. Lengths are not
    // secret, so comparing them in non-constant time is fine.
    if expected_hex.len() != parsed.signature.len() {
        return false;
    }

    let Ok(expected_bytes) = hex::decode(expected_hex) else {
        return false;
    };
    let Ok(actual_bytes) = hex::decode(&parsed.signature) else {
        return false;
    };

    expected_bytes.ct_eq(&actual_bytes).into()
}

/// The decomposed form of an `X-Tango-Signature` header value.
///
/// Returned by [`parse`]. `algorithm` is always lowercase; `signature` is the
/// raw lowercase hex digest with no prefix.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedSignature {
    /// The signing algorithm (always `"sha256"` today).
    pub algorithm: String,
    /// The lowercase hex digest, without any `sha256=` prefix.
    pub signature: String,
}

/// Decompose an `X-Tango-Signature` header value.
///
/// Accepts both the canonical `"sha256=<hex>"` form and a bare hex string
/// (legacy compatibility); in the bare-hex case `algorithm` defaults to
/// `"sha256"`. Returns `None` for empty, malformed, or non-hex inputs.
///
/// # Examples
///
/// ```
/// use tango_webhooks::parse;
///
/// let canonical = parse("sha256=0e396369ee043c5b6b922743631745b2249cf7cb2c4722e61e802447d5d14c70")
///     .expect("canonical form parses");
/// assert_eq!(canonical.algorithm, "sha256");
/// assert_eq!(canonical.signature.len(), 64);
///
/// // Bare hex (legacy) defaults to sha256.
/// let bare = parse("0e396369ee043c5b6b922743631745b2249cf7cb2c4722e61e802447d5d14c70")
///     .expect("bare hex parses");
/// assert_eq!(bare.algorithm, "sha256");
///
/// // Garbage in, None out.
/// assert!(parse("").is_none());
/// assert!(parse("   ").is_none());
/// assert!(parse("sha256=").is_none());
/// assert!(parse("sha256=zzzz").is_none());
/// assert!(parse("not-hex").is_none());
/// ```
#[must_use]
pub fn parse(header: &str) -> Option<ParsedSignature> {
    let stripped = header.trim();
    if stripped.is_empty() {
        return None;
    }

    let (alg, sig) = match stripped.find('=') {
        Some(0) => return None, // empty algorithm prefix like "=abc"
        Some(i) => (&stripped[..i], &stripped[i + 1..]),
        None => ("sha256", stripped),
    };

    if sig.is_empty() || !is_hex(sig) {
        return None;
    }

    Some(ParsedSignature {
        algorithm: alg.to_ascii_lowercase(),
        signature: sig.to_ascii_lowercase(),
    })
}

fn is_hex(s: &str) -> bool {
    !s.is_empty() && s.bytes().all(|b| b.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    /// HMAC-SHA256 of `"hello"` with secret `"shh"`. Same vector the Go,
    /// Node, and Python SDKs use. Locks the implementation to the canonical
    /// algorithm — if this changes, every receiver in the world breaks.
    const KNOWN_VECTOR: &str =
        "sha256=0e396369ee043c5b6b922743631745b2249cf7cb2c4722e61e802447d5d14c70";

    #[test]
    fn generate_matches_known_vector() {
        assert_eq!(generate(b"hello", "shh"), KNOWN_VECTOR);
    }

    #[test]
    fn generate_is_deterministic() {
        // HMAC is deterministic given the same key + message.
        assert_eq!(generate(b"hello", "shh"), generate(b"hello", "shh"));
    }

    #[test]
    fn verify_roundtrip() {
        let body = br#"{"event":"contract.updated","id":"123"}"#;
        let header = generate(body, "topsecret");
        assert!(verify(body, &header, "topsecret"));
    }

    #[test]
    fn verify_rejects_wrong_secret() {
        let body = b"payload";
        let header = generate(body, "right");
        assert!(!verify(body, &header, "wrong"));
    }

    #[test]
    fn verify_rejects_tampered_body() {
        let body = b"original";
        let header = generate(body, "secret");
        assert!(!verify(b"tampered", &header, "secret"));
    }

    #[test]
    fn verify_rejects_empty_header() {
        assert!(!verify(b"body", "", "secret"));
        assert!(!verify(b"body", "   ", "secret"));
    }

    #[test]
    fn verify_rejects_malformed_headers() {
        let body = b"body";
        for h in [
            "",
            "    ",
            "sha256=",
            "sha256=zzz",
            "md5=abc",
            "not-hex",
            "=abc",
            "sha256=0e39", // valid hex, wrong length
        ] {
            assert!(
                !verify(body, h, "secret"),
                "verify unexpectedly accepted malformed header {h:?}",
            );
        }
    }

    #[test]
    fn verify_accepts_bare_hex_legacy() {
        let body = b"payload";
        let with_prefix = generate(body, "s");
        let bare = with_prefix
            .strip_prefix(SIGNATURE_PREFIX)
            .expect("generate always emits the prefix");
        assert!(verify(body, bare, "s"));
    }

    #[test]
    fn verify_is_case_insensitive_on_hex() {
        let body = b"payload";
        let header = generate(body, "secret");
        let upper = header.to_uppercase();
        // Hex is normalized to lowercase in `parse`, so an uppercase signature
        // verifies fine.
        assert!(verify(body, &upper, "secret"));
    }

    #[test]
    fn parse_accepts_canonical_form() {
        let p = parse(KNOWN_VECTOR).expect("canonical form parses");
        assert_eq!(p.algorithm, "sha256");
        assert_eq!(
            p.signature,
            "0e396369ee043c5b6b922743631745b2249cf7cb2c4722e61e802447d5d14c70",
        );
    }

    #[test]
    fn parse_accepts_bare_hex_form() {
        let bare = "0e396369ee043c5b6b922743631745b2249cf7cb2c4722e61e802447d5d14c70";
        let p = parse(bare).expect("bare hex parses");
        assert_eq!(p.algorithm, "sha256");
        assert_eq!(p.signature, bare);
    }

    #[test]
    fn parse_normalizes_case() {
        let upper = "SHA256=0E396369EE043C5B6B922743631745B2249CF7CB2C4722E61E802447D5D14C70";
        let p = parse(upper).expect("uppercase form parses");
        assert_eq!(p.algorithm, "sha256");
        assert_eq!(
            p.signature,
            "0e396369ee043c5b6b922743631745b2249cf7cb2c4722e61e802447d5d14c70",
        );
    }

    #[test]
    fn parse_trims_whitespace() {
        let p = parse("   sha256=deadbeef   ").expect("padded form parses");
        assert_eq!(p.algorithm, "sha256");
        assert_eq!(p.signature, "deadbeef");
    }

    #[test]
    fn parse_rejects_empty_and_whitespace() {
        assert!(parse("").is_none());
        assert!(parse("   ").is_none());
    }

    #[test]
    fn parse_rejects_empty_signature() {
        assert!(parse("sha256=").is_none());
    }

    #[test]
    fn parse_rejects_empty_algorithm_prefix() {
        // A leading "=" with nothing before it isn't a valid algorithm name.
        assert!(parse("=deadbeef").is_none());
    }

    #[test]
    fn parse_rejects_non_hex_signature() {
        assert!(parse("sha256=zzzz").is_none());
        assert!(parse("sha256=dead beef").is_none());
        assert!(parse("not-hex").is_none());
        // Odd-length hex is rejected at verify time (hex::decode fails) and
        // accepted as a "shape" by parse — that's fine, parse only checks
        // each char is hex.
    }

    #[test]
    fn parse_preserves_non_sha256_algorithm() {
        // `parse` is content-agnostic; `verify` is the one that gates on alg.
        let p = parse("md5=deadbeef").expect("any alg parses if hex");
        assert_eq!(p.algorithm, "md5");
        assert!(!verify(b"body", "md5=deadbeef", "secret"));
    }

    #[test]
    fn parsed_signature_is_clonable_and_comparable() {
        let a = parse(KNOWN_VECTOR).unwrap();
        let b = a.clone();
        assert_eq!(a, b);
    }
}
