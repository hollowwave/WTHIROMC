use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

/// Result of checking one executable's Authenticode signature.
#[derive(Debug, Clone)]
pub struct SignatureInfo {
    pub is_signed: bool,
    /// The signer's organization name, e.g. "Microsoft Corporation", if available.
    pub publisher: Option<String>,
    /// Why the file is considered unsigned, when it is. `None` for signed
    /// files. Distinguishes "no signature present" from "verification
    /// actually failed" (revoked cert, tampered file, etc.) so explanations
    /// can be more specific than a flat "unsigned" for every case.
    pub detail: Option<String>,
}

impl SignatureInfo {
    fn unsigned_no_signature() -> Self {
        SignatureInfo {
            is_signed: false,
            publisher: None,
            detail: Some("no signature was found on the file".to_string()),
        }
    }

    fn unsigned_verification_failed(reason: String) -> Self {
        SignatureInfo {
            is_signed: false,
            publisher: None,
            detail: Some(format!("signature verification failed ({reason})")),
        }
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
/// On non-Windows targets (e.g. if you ever build this for testing on
/// another OS), this always reports unsigned — signature checking is
/// Windows-only for v1.
pub fn check_signature(exe_path: &str) -> SignatureInfo {
    if exe_path.is_empty() {
        return SignatureInfo::unsigned_no_signature();
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
                detail: None,
            },
            // The underlying error's Display impl is used for the detail
            // message - since it implements std::error::Error, Display is
            // guaranteed, so this gives a real, specific reason (e.g.
            // "Unsigned", or a revoked/tampered cert message) rather than
            // us trying to enumerate every variant ourselves.
            Err(e) => SignatureInfo::unsigned_verification_failed(format!("{:?}", e)),
        },
        Err(e) => SignatureInfo::unsigned_verification_failed(format!("{:?}", e)),
    }
}

#[cfg(not(windows))]
fn verify_uncached(_exe_path: &str) -> SignatureInfo {
    SignatureInfo::unsigned_no_signature()
}

