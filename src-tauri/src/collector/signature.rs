use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

/// Result of checking one executable's Authenticode signature.
#[derive(Debug, Clone)]
pub struct SignatureInfo {
    pub is_signed: bool,
    /// The signer's organization name, e.g. "Microsoft Corporation", if available.
    pub publisher: Option<String>,
}

impl SignatureInfo {
    fn unsigned() -> Self {
        SignatureInfo { is_signed: false, publisher: None }
    }
}

/// Signature checks involve real disk I/O and cryptographic verification —
/// noticeably more expensive than the rest of the collector. Since an
/// executable's signature won't change while it's running, we cache results
/// for the lifetime of the app process, keyed by exe path.
fn cache() -> &'static Mutex<HashMap<String, SignatureInfo>> {
    static CACHE: OnceLock<Mutex<HashMap<String, SignatureInfo>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Checks whether the executable at `exe_path` has a valid Authenticode
/// signature, and if so, who signed it. Results are cached per path.
///
/// On non-Windows targets (e.g. if you ever build this for testing on another OS), this always reports unsigned — signature checking is
/// Windows-only for v1.
pub fn check_signature(exe_path: &str) -> SignatureInfo {
    if exe_path.is_empty() {
        return SignatureInfo::unsigned();
    }

    if let Some(cached) = cache().lock().unwrap().get(exe_path) {
        return cached.clone();
    }

    let result = verify_uncached(exe_path);
    cache().lock().unwrap().insert(exe_path.to_string(), result.clone());
    result
}

#[cfg(windows)]
fn verify_uncached(exe_path: &str) -> SignatureInfo {
    match verifysign::CodeSignVerifier::for_file(exe_path) {
        Ok(verifier) => match verifier.verify() {
            Ok(signature) => SignatureInfo {
                is_signed: true,
                publisher: signature.subject_name().organization.clone(),
            },
            // Err(Unsigned) and any other verification failure both mean
            // "don't trust this" for our purposes — we don't need to
            // distinguish "no signature" from "invalid signature".
            Err(_) => SignatureInfo::unsigned(),
        },
        Err(_) => SignatureInfo::unsigned(),
    }
}

#[cfg(not(windows))]
fn verify_uncached(_exe_path: &str) -> SignatureInfo {
    SignatureInfo::unsigned()
}
