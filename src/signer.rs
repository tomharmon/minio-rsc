//！ This module implements all helpers for AWS Signature version '4' support.

use chrono::{DateTime, Utc};
use crypto::digest::Digest;
use crypto::hmac::Hmac;
use crypto::mac::Mac;
use crypto::sha2::Sha256;
use hyper::{header, HeaderMap, Method, Uri};

use crate::credentials::Credentials;
use crate::time::{aws_format_date, aws_format_time};

/// Return HMacSHA256 digest of given key and data.
fn _hmac_hash(key: &[u8], data: &str) -> Vec<u8> {
    let mut hasher = Hmac::new(Sha256::new(), key);
    hasher.input(data.as_bytes());
    hasher.result().code().to_vec()
}

/// Compute SHA-256 of string data and return hash as hex encoded value.
fn sha256_hash(date: &String) -> String {
    let mut hasher = Sha256::new();
    hasher.input_str(date);
    return hasher.result_str();
}

/// Get scope string.
fn _get_scope(date: &DateTime<Utc>, region: &str, service_name: &str) -> String {
    format!(
        "{}/{}/{}/aws4_request",
        aws_format_date(date),
        region,
        service_name
    )
}

/// Get canonical request hash.
///
/// CanonicalRequest =
///     HTTPRequestMethod + '\n' +
///     CanonicalURI + '\n' +
///     CanonicalQueryString + '\n' +
///     CanonicalHeaders + '\n\n' +
///     SignedHeaders + '\n' +
///     HexEncode(Hash(RequestPayload))
fn _get_canonical_request_hash(
    method: &Method,
    uri: &Uri,
    headers: &HeaderMap,
    content_sha256: &str,
) -> (String, String) {
    let mut canonical_hdrs: Vec<(String, String)> = headers
        .iter()
        .filter(|&(name, _)| name != header::USER_AGENT && name != header::AUTHORIZATION)
        .map(|(x, y)| {
            (
                x.clone().to_string(),
                format!("{}", y.clone().to_str().unwrap()),
            )
        })
        .collect();
    canonical_hdrs.sort_by_key(|x| x.0.clone());

    let signed_headers: String = canonical_hdrs
        .iter()
        .map(|x| x.0.clone())
        .collect::<Vec<String>>()
        .join(";");

    let canonical_headers: String = canonical_hdrs
        .iter()
        .map(|x| format!("{}:{}", x.0.clone(), x.1.clone()))
        .collect::<Vec<String>>()
        .join("\n");

    let canonical_query_string: String = uri.query().unwrap_or_else(|| "").to_string();

    let canonical_request = format!(
        "{}\n{}\n{}\n{}\n\n{}\n{}",
        method,
        uri.path(),
        canonical_query_string,
        canonical_headers,
        signed_headers,
        content_sha256
    );
    (sha256_hash(&canonical_request), signed_headers)
}

/// Get string-to-sign
fn _get_string_to_sign(date: &DateTime<Utc>, scope: &str, canonical_request_hash: &str) -> String {
    format!(
        "AWS4-HMAC-SHA256\n{}\n{}\n{}",
        &aws_format_time(date),
        scope,
        canonical_request_hash,
    )
}

/// Get signing key
fn _get_signing_key(
    secret_key: &String,
    date: &DateTime<Utc>,
    region: &str,
    service_name: &str,
) -> Vec<u8> {
    let date_key = _hmac_hash(
        format!("AWS4{}", secret_key).as_bytes(),
        aws_format_date(date).as_str(),
    );
    let date_region_key = _hmac_hash(date_key.as_ref(), region);
    let date_region_service_key = _hmac_hash(date_region_key.as_ref(), service_name);
    _hmac_hash(date_region_service_key.as_ref(), "aws4_request")
}

/// Get authorization format
fn _get_authorization(
    access_key: &str,
    scope: &str,
    signed_headers: &str,
    signature: &str,
) -> String {
    format!(
        "AWS4-HMAC-SHA256 Credential={}/{}, SignedHeaders={}, Signature={}",
        access_key, scope, signed_headers, signature
    )
}

pub fn sign_v4_authorization(
    method: &Method,
    uri: &Uri,
    region: &str,
    server_name: &str,
    headers: &HeaderMap,
    credentials: &Credentials,
    content_sha256: &str,
    date: &DateTime<Utc>,
) -> String {
    let scope = _get_scope(&date, region, server_name);
    let (canonical_request_hash, signed_headers) =
        _get_canonical_request_hash(method, uri, headers, content_sha256);

    let string_to_sign = _get_string_to_sign(&date, &scope, &canonical_request_hash);

    let signing_key = _get_signing_key(&credentials.secret_key, &date, region, server_name);

    let signature = _hmac_hash(signing_key.as_ref(), &string_to_sign);
    let signature = hex::encode(signature);
    _get_authorization(&credentials.access_key, &scope, &signed_headers, &signature)
}
