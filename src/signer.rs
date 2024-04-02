//！ This module implements all helpers for AWS Signature version '4' support.
use bytes::Bytes;
use futures_util::{stream, StreamExt, TryStreamExt};
use hmac::{Hmac, Mac};
use hyper::{
    header::{self, InvalidHeaderValue}, HeaderMap, Method, Uri,
};
use reqwest::Body;
use sha2::{Digest, Sha256};

use crate::{
    data::Data,
    time::UtcTime,
    utils::{trim_bytes, urlencode, EMPTY_CONTENT_SHA256},
};

pub const MAX_MULTIPART_COUNT: usize = 10000; // 10000 parts
pub const MAX_MULTIPART_OBJECT_SIZE: usize = 5 * 1024 * 1024 * 1024 * 1024; // 5TiB
pub const MAX_PART_SIZE: usize = 5 * 1024 * 1024 * 1024; // 5GiB
pub const MIN_PART_SIZE: usize = 5 * 1024 * 1024; // 5MiB
pub const RECOMMEND_CHUNK_SIZE: usize = 64 * 1024; // 64 KB
pub const MIN_CHUNK_SIZE: usize = 8 * 1024; // 8 KB

type HmacSha256 = Hmac<Sha256>;
/// Return HMacSHA256 digest of given key and data.
fn _hmac_hash(key: &[u8], data: &str) -> Vec<u8> {
    let mut hasher = HmacSha256::new_from_slice(key).expect("");
    hasher.update(data.as_bytes());
    hasher.finalize().into_bytes().to_vec()
}

/// Compute Mac-SHA-256 of data and return hash as hex encoded value.
///
/// Return Hex(HMAC-SHA256(key, data)).
#[inline]
pub fn hmac_hash_hex(key: &[u8], data: &str) -> String {
    hex::encode(_hmac_hash(key.as_ref(), &data))
}

/// Compute SHA-256 of data and return hash as hex encoded value.
#[inline]
pub fn sha256_hash(date: &[u8]) -> String {
    hex::encode(Sha256::digest(date))
}

/// Get scope string.
///
/// `date.Format(<YYYYMMDD>) + "/" + <region> + "/" + <service> + "/aws4_request"`
#[inline]
fn _get_scope(date: &UtcTime, region: &str, service_name: &str) -> String {
    format!(
        "{}/{}/{}/aws4_request",
        date.aws_format_date(),
        region,
        service_name
    )
}

/// Get canonical query string.
///
/// query string parameters is assumed be URI-encoded
///
fn _get_canonical_query_string(query: &str) -> String {
    let mut querys: Vec<(&str, &str)> = query
        .split("&")
        .filter(|&x| !x.is_empty())
        .map(|q| {
            let i = q.find("=");
            if let Some(i) = i {
                (&q[0..i], &q[i + 1..])
            } else {
                (q, "")
            }
        })
        .collect();
    querys.sort_by_key(|x| x.0);
    querys
        .iter()
        .map(|&(k, v)| format!("{}={}", k, v))
        .collect::<Vec<String>>()
        .join("&")
}

/// Get canonical request hash and signed_headers.
///
/// `Hex(SHA256Hash(Canonical Request)))`
///
/// CanonicalRequest =
///     HTTPRequestMethod + '\n' +
///     CanonicalURI + '\n' +
///     CanonicalQueryString + '\n' +
///     CanonicalHeaders + '\n' +
///     SignedHeaders + '\n' +
///     HashedPayload
///
/// - `HTTPRequestMethod` is one of the HTTP methods, for example GET, PUT, HEAD, and DELETE.
/// - `CanonicalURI`  is the URI-encoded version of the absolute path component of the URI—everything
/// starting with the "/" that follows the domain name and up to the end of the string or to the
/// question mark character ('?') if you have query string parameters.
/// - `CanonicalQueryString`
/// - `CanonicalHeaders`
/// - `SignedHeaders` A semicolon-separated list of request headers that you used to compute Signature. The list includes header names only, and the header names must be in lowercase.
/// - `HashedPayload` is the hexadecimal value of the SHA256 hash of the request payload.
fn _get_canonical_request_hash(
    method: &Method,
    uri: &Uri,
    headers: &HeaderMap,
    content_sha256: &str,
) -> (String, String) {
    let mut cr: Vec<u8> = Vec::new();

    // HTTPRequestMethod
    cr.extend_from_slice(method.as_str().as_bytes());
    cr.push(b'\n');

    // CanonicalURI
    cr.extend_from_slice(uri.path().as_bytes());
    cr.push(b'\n');

    // CanonicalQueryString
    let querys = uri.query().unwrap_or_else(|| "");
    let canonical_query_string = _get_canonical_query_string(querys);
    cr.extend_from_slice(canonical_query_string.as_bytes());
    cr.push(b'\n');

    // CanonicalHeaders and SignedHeaders
    let mut canonical_hdrs = headers
        .iter()
        .filter(|&(name, _)| name != header::USER_AGENT && name != header::AUTHORIZATION)
        .collect::<Vec<_>>();
    canonical_hdrs.sort_by_key(|f| f.0.as_str());
    let mut signed_headers: String = String::new();
    canonical_hdrs.iter().for_each(|(h, v)| {
        let h = h.as_str().to_lowercase();
        cr.extend_from_slice(h.as_bytes());
        cr.push(b':');
        cr.extend_from_slice(trim_bytes(v.as_bytes()));
        cr.push(b'\n');

        signed_headers += h.as_str();
        signed_headers += ";";
    });
    cr.push(b'\n');
    signed_headers.pop();
    cr.extend_from_slice(signed_headers.as_bytes());
    cr.push(b'\n');

    // HashedPayload
    cr.extend_from_slice(content_sha256.as_bytes());

    (sha256_hash(&cr), signed_headers)
}

/// Get string-to-sign
///
/// "AWS4-HMAC-SHA256" + "\n" +
/// timeStampISO8601Format + "\n" +
/// <Scope> + "\n" +
/// Hex(SHA256Hash(Canonical Request)))
#[inline]
fn _get_string_to_sign(date: &UtcTime, scope: &str, canonical_request_hash: &str) -> String {
    format!(
        "AWS4-HMAC-SHA256\n{}\n{}\n{}",
        date.aws_format_time(),
        scope,
        canonical_request_hash,
    )
}

/// Get chunk string to sign
///
/// "AWS4-HMAC-SHA256-PAYLOAD" + "\n" +
/// timeStampISO8601Format + "\n" +
/// <Scope> + "\n" +
/// previousSignature + "\n" +
/// Hex(SHA256Hash(""))) + "\n" +
/// Hex(SHA256Hash(current-chunk-data)))
#[inline]
pub fn get_chunk_string_to_sign(
    date_time: &str,
    scope: &str,
    previous_signature: &str,
    chunk_hash: &str,
) -> String {
    format!(
        "AWS4-HMAC-SHA256-PAYLOAD\n{}\n{}\n{}\n{}\n{}",
        date_time, scope, previous_signature, EMPTY_CONTENT_SHA256, chunk_hash,
    )
}

/// get the header of chunk data sent
#[inline]
pub fn get_chunk_header(len: usize, signature: &str) -> String {
    format!("{:x};chunk-signature={}\r\n", len, signature)
}

/// Get signing key
///
/// DateKey = HMAC-SHA256("AWS4"+"<SecretAccessKey>", "<YYYYMMDD>")
/// DateRegionKey = HMAC-SHA256(<DateKey>, "<aws-region>")
/// DateRegionServiceKey = HMAC-SHA256(<DateRegionKey>, "<aws-service>")
/// SigningKey = HMAC-SHA256(<DateRegionServiceKey>, "aws4_request")
fn _get_signing_key(secret_key: &str, date: &UtcTime, region: &str, service_name: &str) -> Vec<u8> {
    let secret_access_key = format!("AWS4{}", secret_key);
    let date_key = _hmac_hash(secret_access_key.as_bytes(), &date.aws_format_date());
    let date_region_key = _hmac_hash(date_key.as_ref(), region);
    let date_region_service_key = _hmac_hash(date_region_key.as_ref(), service_name);
    _hmac_hash(date_region_service_key.as_ref(), "aws4_request")
}

/// Get authorization header value
#[inline]
fn _get_authorization_header_value(
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

/// Do signature V4 of given request for given service name
pub fn sign_v4_authorization(
    method: &Method,
    uri: &Uri,
    region: &str,
    server_name: &str,
    headers: &HeaderMap,
    access_key: &str,
    secret_key: &str,
    content_sha256: &str,
    date: &UtcTime,
) -> String {
    let scope = _get_scope(&date, region, server_name);
    let (canonical_request_hash, signed_headers) =
        _get_canonical_request_hash(method, uri, headers, content_sha256);

    let string_to_sign = _get_string_to_sign(&date, &scope, &canonical_request_hash);

    let signing_key = _get_signing_key(secret_key, &date, region, server_name);

    let signature = hmac_hash_hex(signing_key.as_ref(), &string_to_sign);

    _get_authorization_header_value(access_key, &scope, &signed_headers, &signature)
}

/// Get canonical request hash for presign request.
fn _get_presign_canonical_request_hash(
    method: &Method,
    uri: &Uri,
    access_key: &str,
    scope: &str,
    date: &UtcTime,
    expires: usize,
    security_token: Option<&str>,
) -> (String, String) {
    let x_amz_credential = urlencode(&(access_key.to_string() + "/" + scope), false);
    let mut canonical_headers = "host:".to_string() + uri.host().unwrap_or("").trim();
    if let Some(port) = uri.port_u16() {
        canonical_headers = canonical_headers + ":" + port.to_string().as_str();
    }
    let signed_headers = "host";

    let querys = uri
        .query()
        .map(|x| x.to_owned() + "&")
        .unwrap_or("".to_string());
    let mut querys = format!(
        "{}X-Amz-Algorithm=AWS4-HMAC-SHA256&X-Amz-Credential={}&X-Amz-Date={}&X-Amz-Expires={}&X-Amz-SignedHeaders={}",
        querys,x_amz_credential,date.aws_format_time(),expires,signed_headers);
    if let Some(security_token) = security_token {
        querys = querys + "&X-Amz-Security-Token=" + security_token;
    }
    let canonical_query_string = _get_canonical_query_string(&querys);
    let canonical_request = format!(
        "{}\n{}\n{}\n{}\n\n{}\nUNSIGNED-PAYLOAD",
        method,
        uri.path(),
        canonical_query_string,
        canonical_headers,
        signed_headers
    );
    (sha256_hash(canonical_request.as_bytes()), querys)
}

/// Do signature V4 of given presign request.
/// Returned `uri:Strig`
pub fn presign_v4(
    method: &Method,
    uri: &Uri,
    region: &str,
    access_key: &str,
    secret_key: &str,
    date: &UtcTime,
    expires: usize,
) -> String {
    let scope = _get_scope(&date, region, "s3");
    let (canonical_request_hash, querys) =
        _get_presign_canonical_request_hash(method, uri, access_key, &scope, date, expires, None);

    let string_to_sign = _get_string_to_sign(date, &scope, &canonical_request_hash);
    let signing_key = _get_signing_key(secret_key, date, region, "s3");
    let signature = _hmac_hash(signing_key.as_ref(), &string_to_sign);
    let signature = hex::encode(signature);
    let querys = querys + "&X-Amz-Signature=" + &urlencode(&signature, false);
    let scheme = uri
        .scheme_str()
        .map(|x| x.to_string() + "://")
        .unwrap_or("".to_string());
    format!(
        "{}{}{}?{}",
        scheme,
        uri.authority().map(|x| x.as_str()).unwrap_or(""),
        uri.path(),
        querys
    )
}

/// Do signature V4 of given request params,
/// add the headers required by S3 and convert [Data] to [Body].
///
/// return (uri: [String], Body: [Body])
#[allow(unused)]
pub fn sign_request_v4<E>(
    method: &Method,
    uri: &Uri,
    headers: &mut HeaderMap,
    region: &str,
    data: Data<E>,
    access_key: &str,
    secret_key: &str,
) -> std::result::Result<(String, Body), InvalidHeaderValue>
where
    E: std::error::Error + Send + Sync + 'static,
{
    let date = UtcTime::now();
    let server_name = "s3";

    // add s3 header
    if let Some(host) = uri.host() {
        headers.insert(header::HOST, host.parse()?);
    }
    headers.insert("x-amz-date", date.aws_format_time().parse()?);
    match &data {
        Data::Stream(_, len) => {
            headers.insert(header::CONTENT_ENCODING, "aws-chunked".parse()?);
            headers.insert("x-amz-decoded-content-length", len.to_string().parse()?);
        }
        Data::Bytes(data) => {
            headers.insert(header::CONTENT_LENGTH, data.len().to_string().parse()?);
        }
    };
    let payload_hash = data.payload_hash();
    let content_sha256 = payload_hash.as_str();
    headers.insert("x-amz-content-sha256", payload_hash.as_str().parse()?);

    // Calculate s3 signature
    let scope = _get_scope(&date, region, server_name);
    let (canonical_request_hash, signed_headers) =
        _get_canonical_request_hash(method, uri, &headers, content_sha256);

    let string_to_sign = _get_string_to_sign(&date, &scope, &canonical_request_hash);

    let signing_key = _get_signing_key(secret_key, &date, region, server_name);

    let mut signature = hmac_hash_hex(signing_key.as_ref(), &string_to_sign);

    let auth_header =
        _get_authorization_header_value(access_key, &scope, &signed_headers, &signature);

    let date_time = date.aws_format_time();

    // add authorization header
    headers.insert(header::AUTHORIZATION, auth_header.parse()?);

    // wrap data to http dody
    let body = match data {
        Data::Bytes(b) => Body::from(b),
        Data::Stream(s, _) => Body::wrap_stream(
            s.chain(stream::iter(vec![Ok(Bytes::new())]))
                .map_ok(move |chunk| {
                    let chunk_hash = sha256_hash(&chunk);
                    let string_to_sign =
                        get_chunk_string_to_sign(&date_time, &scope, &signature, &chunk_hash);
                    let signature_next = hmac_hash_hex(&signing_key, &string_to_sign);
                    let chunk_header = get_chunk_header(chunk.len(), &signature_next);
                    signature = signature_next;
                    vec![Bytes::from(chunk_header), chunk, Bytes::from("\r\n")]
                })
                .flat_map(|f| {
                    stream::iter(match f {
                        Ok(d) => d.into_iter().map(|f| Ok(f)).collect(),
                        Err(e) => vec![Err(e)],
                    })
                }),
        ),
    };

    Ok((uri.to_string(), body))
}
