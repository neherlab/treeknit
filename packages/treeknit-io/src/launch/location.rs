//! The locations of the files of a link: `https:` addresses and `data:` texts, and the bytes they
//! hold.

use base64::Engine;
use base64::alphabet;
use base64::engine::DecodePaddingMode;
use base64::engine::general_purpose::{GeneralPurpose, GeneralPurposeConfig};
use flate2::read::MultiGzDecoder;
use percent_encoding::{AsciiSet, CONTROLS, percent_decode_str, utf8_percent_encode};
use serde::Serialize;
use std::io::{self, Read};
use std::string::FromUtf8Error;
#[cfg(feature = "tsify")]
use tsify::Tsify;

/// Largest text that a compressed tree or session file may expand to. A wrong link must not
/// exhaust the memory of the 32-bit WebAssembly module, and a Newick tree of 100,000 leaves is
/// about 5 MB.
pub const MAX_TEXT_BYTES: usize = MAX_TEXT_MIB * 1024 * 1024;

/// [`MAX_TEXT_BYTES`] in MiB.
const MAX_TEXT_MIB: usize = 256;

/// First two bytes of a gzip member (RFC 1952, section 2.3.1).
const GZIP_MAGIC: [u8; 2] = [0x1f, 0x8b];

/// Longest time that reading a file of a link may take, in seconds, so that a wrong address
/// cannot hang the web app or the command line.
pub const FETCH_TIMEOUT_SECONDS: u32 = 60;

/// Largest file that a link may download, counted while it arrives, so that a wrong address
/// cannot exhaust the memory of a browser tab: a Newick tree of 100,000 leaves is about 5 MB.
pub const MAX_DOWNLOAD_BYTES: usize = 64 * 1024 * 1024;

/// The characters that a URI cannot hold (RFC 3986): every ASCII character outside its
/// unreserved and reserved sets and `%`.
const URI_SET: &AsciiSet = &CONTROLS
  .add(b' ')
  .add(b'"')
  .add(b'<')
  .add(b'>')
  .add(b'\\')
  .add(b'^')
  .add(b'`')
  .add(b'{')
  .add(b'|')
  .add(b'}');

/// The location of `value`: an `https:` URL, rewritten to the address that serves the file to
/// other sites (see [`fetch_url`]), or a `data:` text, decoded.
pub fn parse_location(value: &str) -> Result<LinkLocation, LocationError> {
  let scheme = value.split_once(':').map(|(s, _)| s.to_ascii_lowercase());
  match scheme.as_deref() {
    Some("https") => {
      check_https(value)?;
      let UrlPlace { host, file_name } = url_place(value);
      Ok(LinkLocation::Url {
        url: value.to_owned(),
        fetch: fetch_url(value),
        host,
        file_name,
      })
    },
    Some("data") => Ok(LinkLocation::Data {
      text: decode_data_url(value)?,
    }),
    Some("http") => Err(LocationError::Http(value.to_owned())),
    _ => Err(LocationError::Unsupported(value.to_owned())),
  }
}

/// The address that serves the file of `url` to other sites, which browsers require: a GitHub
/// file page (`github.com/<owner>/<repo>/blob/<ref>/<path>` or `.../raw/...`) becomes its file
/// on `raw.githubusercontent.com`, and a Zenodo file (`zenodo.org/records/<id>/files/<name>`)
/// its `/content` address of the Zenodo API. Other URLs stay as they are.
pub(crate) fn fetch_url(url: &str) -> String {
  let Some(parts) = UrlParts::of(url) else {
    return url.to_owned();
  };
  let host = parts.host.to_ascii_lowercase();
  let segments: Vec<&str> = parts.path.split('/').filter(|s| !s.is_empty()).collect();
  match (host.as_str(), segments.as_slice()) {
    ("github.com" | "www.github.com", [owner, repo, "blob" | "raw", git_ref, path @ ..]) if !path.is_empty() => {
      format!(
        "https://raw.githubusercontent.com/{owner}/{repo}/{git_ref}/{}",
        path.join("/")
      )
    },
    ("zenodo.org" | "www.zenodo.org", ["records", id, "files", name @ ..]) if !name.is_empty() => {
      format!("https://zenodo.org/api/records/{id}/files/{}/content", name.join("/"))
    },
    _ => url.to_owned(),
  }
}

/// The host and the file name of an `https:` URL, as messages and tree rows name the place of a
/// file: the host without the port, and the last path segment, percent-decoded (an invalid UTF-8
/// sequence becomes U+FFFD), or an empty text without one. Labels of unlabeled trees come from the
/// file name. Both are empty for a text that is not an `https:` URL.
pub fn url_place(url: &str) -> UrlPlace {
  let parts = UrlParts::of(url);
  UrlPlace {
    host: parts.as_ref().map(|p| p.host.to_owned()).unwrap_or_default(),
    file_name: parts
      .and_then(|p| p.path.rsplit('/').find(|s| !s.is_empty()))
      .map(|segment| percent_decode_str(segment).decode_utf8_lossy().into_owned())
      .unwrap_or_default(),
  }
}

/// The place of a file at an `https:` URL ([`url_place`]).
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct UrlPlace {
  pub host: String,
  pub file_name: String,
}

/// The text of a tree or session file: gzip-compressed bytes (they start with [`GZIP_MAGIC`]) are
/// decompressed first, up to [`MAX_TEXT_BYTES`]; the text must be UTF-8, and a web page (a text
/// that starts with `<!doctype html` or `<html`) is rejected, because a wrong address often
/// serves one.
pub fn decode_tree_bytes(bytes: &[u8]) -> Result<String, DecodeError> {
  let bytes = if bytes.starts_with(&GZIP_MAGIC) {
    let mut text = Vec::new();
    let limit = u64::try_from(MAX_TEXT_BYTES + 1).unwrap_or(u64::MAX);
    MultiGzDecoder::new(bytes)
      .take(limit)
      .read_to_end(&mut text)
      .map_err(DecodeError::Gzip)?;
    if text.len() > MAX_TEXT_BYTES {
      return Err(DecodeError::TooLarge);
    }
    text
  } else {
    bytes.to_vec()
  };
  let text = String::from_utf8(bytes).map_err(DecodeError::NotUtf8)?;
  let start = text
    .trim_start()
    .chars()
    .take(14)
    .collect::<String>()
    .to_ascii_lowercase();
  if start.starts_with("<!doctype html") || start.starts_with("<html") {
    return Err(DecodeError::WebPage);
  }
  Ok(text)
}

/// Why a value is not the location of a tree or session file, from [`parse_location`].
#[derive(Debug, thiserror::Error)]
pub enum LocationError {
  #[error("{0:?} is an http: address; use https:, because a page served over https cannot read http: addresses")]
  Http(String),
  #[error("{0:?} is neither an https: address nor a data: text")]
  Unsupported(String),
  #[error("{0:?} is not an https:// address")]
  NotHttps(String),
  #[error("{0:?} has no host")]
  NoHost(String),
  #[error("{0:?} must not contain a user name or password")]
  UserInfo(String),
  #[error("a data: location needs a comma before its data, as in data:,((A,B),C);")]
  DataWithoutComma,
  #[error("the base64 data is invalid: {0}")]
  Base64(#[source] base64::DecodeError),
  #[error(transparent)]
  Decode(#[from] DecodeError),
}

/// Why bytes are not the text of a tree or session file, from [`decode_tree_bytes`].
#[derive(Debug, thiserror::Error)]
pub enum DecodeError {
  #[error("not a valid gzip file: {0}")]
  Gzip(#[source] io::Error),
  #[error("the decompressed file is larger than {} MiB", MAX_TEXT_MIB)]
  TooLarge,
  #[error("not a UTF-8 text file: {}", .0.utf8_error())]
  NotUtf8(#[source] FromUtf8Error),
  #[error("a web page, not a tree file")]
  WebPage,
}

/// `url` with every character that a URI cannot hold percent-encoded (spaces, non-ASCII
/// characters, `"`, `<`, `>`, ...), as browsers send it: links keep such characters readable,
/// and an HTTP client needs them encoded.
pub fn request_url(url: &str) -> String {
  utf8_percent_encode(url, URI_SET).to_string()
}

/// Where a file of a link is.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum LinkLocation {
  /// An `https:` URL as the link gives it, the address to read it from ([`fetch_url`]), and the
  /// host and file name of its place ([`url_place`]).
  Url {
    url: String,
    fetch: String,
    host: String,
    #[serde(rename = "fileName")]
    file_name: String,
  },
  /// The decoded text of a `data:` location.
  Data { text: String },
}

/// Check an `https:` URL: it has a host and no user name or password.
fn check_https(url: &str) -> Result<(), LocationError> {
  let parts = UrlParts::of(url).ok_or_else(|| LocationError::NotHttps(url.to_owned()))?;
  if parts.host.is_empty() {
    return Err(LocationError::NoHost(url.to_owned()));
  }
  if parts.has_user {
    return Err(LocationError::UserInfo(url.to_owned()));
  }
  Ok(())
}

/// The host and the path of an `https://` URL.
struct UrlParts<'a> {
  host: &'a str,
  /// The authority holds a user name or password (`user@host`).
  has_user: bool,
  path: &'a str,
}

impl<'a> UrlParts<'a> {
  fn of(url: &'a str) -> Option<Self> {
    let rest = url
      .get(..8)
      .filter(|scheme| scheme.eq_ignore_ascii_case("https://"))
      .and_then(|_| url.get(8..))?;
    let (authority, after) = rest.split_at(rest.find(['/', '?', '#']).unwrap_or(rest.len()));
    let path = after.split(['?', '#']).next().unwrap_or_default();
    let (has_user, host_port) = match authority.rsplit_once('@') {
      Some((_, host)) => (true, host),
      None => (false, authority),
    };
    let host = host_port.rsplit_once(':').map_or(host_port, |(h, _)| h);
    Some(UrlParts { host, has_user, path })
  }
}

/// The text of a `data:` location (RFC 2397): its data after the first `,`, decoded from base64
/// when the media type ends with `;base64`, and percent-decoded otherwise, then decoded by
/// [`decode_tree_bytes`].
fn decode_data_url(value: &str) -> Result<String, LocationError> {
  let (meta, data) = value
    .get(5..)
    .and_then(|rest| rest.split_once(','))
    .ok_or(LocationError::DataWithoutComma)?;
  let bytes = if meta.trim_end().to_ascii_lowercase().ends_with(";base64") {
    decode_base64(data)?
  } else {
    percent_decode_str(data).collect()
  };
  Ok(decode_tree_bytes(&bytes)?)
}

/// Bytes of base64 text in the standard or the URL-safe alphabet, with or without padding. A
/// space stands for `+`, which a reader of form data turns into a space.
fn decode_base64(text: &str) -> Result<Vec<u8>, LocationError> {
  let config = GeneralPurposeConfig::new().with_decode_padding_mode(DecodePaddingMode::Indifferent);
  let text = text.replace(' ', "+");
  let text = text.trim();
  GeneralPurpose::new(&alphabet::STANDARD, config)
    .decode(text)
    .or_else(|_standard_error| GeneralPurpose::new(&alphabet::URL_SAFE, config).decode(text))
    .map_err(LocationError::Base64)
}

#[cfg(test)]
mod tests {
  use super::*;
  use base64::engine::general_purpose;
  use flate2::Compression;
  use flate2::write::GzEncoder;
  use pretty_assertions::assert_eq;
  use rstest::rstest;
  use std::io::Write;
  use treeknit_testing::assert_err;

  fn gzip(bytes: &[u8]) -> Vec<u8> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(bytes).unwrap();
    encoder.finish().unwrap()
  }

  #[rustfmt::skip]
  #[rstest]
  #[case::file_name_decoded(     "https://x/a/seg%204.nwk?download=1",   ("x",           "seg 4.nwk"))]
  #[case::port_left_out(         "https://example.org:8443/a/ha.nwk",    ("example.org", "ha.nwk"))]
  #[case::user_left_out(         "https://user@example.org/ha.nwk#x",    ("example.org", "ha.nwk"))]
  #[case::last_segment_of_a_dir( "https://x/c/",                         ("x",           "c"))]
  #[case::no_file_name(          "https://example.org/",                 ("example.org", ""))]
  #[case::invalid_utf8_replaced( "https://x/a/%E2%28.nwk",               ("x",           "\u{fffd}(.nwk"))]
  #[case::not_https(             "http://x/ha.nwk",                      ("",            ""))]
  #[trace]
  fn url_place_is_the_host_without_port_and_the_decoded_last_segment(
    #[case] url: &str,
    #[case] (host, file_name): (&str, &str),
  ) {
    let expected = UrlPlace { host: host.to_owned(), file_name: file_name.to_owned() };
    assert_eq!(expected, url_place(url));
  }

  #[rustfmt::skip]
  #[rstest]
  #[case::github_blob(  "https://github.com/org/repo/blob/main/data/ha.nwk",          "https://raw.githubusercontent.com/org/repo/main/data/ha.nwk")]
  #[case::github_raw(   "https://github.com/org/repo/raw/v1.0/ha.nwk",                "https://raw.githubusercontent.com/org/repo/v1.0/ha.nwk")]
  #[case::github_query( "https://github.com/org/repo/blob/main/ha.nwk?raw=true",      "https://raw.githubusercontent.com/org/repo/main/ha.nwk")]
  #[case::zenodo(       "https://zenodo.org/records/123/files/ha.nwk",               "https://zenodo.org/api/records/123/files/ha.nwk/content")]
  #[case::zenodo_query( "https://zenodo.org/records/123/files/ha.nwk?download=1",    "https://zenodo.org/api/records/123/files/ha.nwk/content")]
  #[case::zenodo_folder("https://zenodo.org/records/123/files/org/ha.nwk",           "https://zenodo.org/api/records/123/files/org/ha.nwk/content")]
  #[case::raw_github(   "https://raw.githubusercontent.com/org/repo/main/ha.nwk",    "https://raw.githubusercontent.com/org/repo/main/ha.nwk")]
  #[case::github_repo(  "https://github.com/org/repo",                               "https://github.com/org/repo")]
  #[case::other_host(   "https://data.nextstrain.org/files/ha.nwk",                  "https://data.nextstrain.org/files/ha.nwk")]
  #[trace]
  fn copied_addresses_are_rewritten_to_their_file_addresses(#[case] given: &str, #[case] fetch: &str) {
    assert_eq!(fetch, fetch_url(given));
  }

  #[rustfmt::skip]
  #[rstest]
  #[case::http(      "http://x/ha.nwk",          "\"http://x/ha.nwk\" is an http: address; use https:, because a page served over https cannot read http: addresses")]
  #[case::ftp(       "ftp://x/ha.nwk",           "\"ftp://x/ha.nwk\" is neither an https: address nor a data: text")]
  #[case::file_name( "ha.nwk",                   "\"ha.nwk\" is neither an https: address nor a data: text")]
  #[case::user(      "https://me:pw@x/ha.nwk",   "\"https://me:pw@x/ha.nwk\" must not contain a user name or password")]
  #[case::no_host(   "https:///ha.nwk",          "\"https:///ha.nwk\" has no host")]
  #[case::no_slashes("https:ha.nwk",             "\"https:ha.nwk\" is not an https:// address")]
  #[case::no_comma(  "data:text/plain",          "a data: location needs a comma before its data, as in data:,((A,B),C);")]
  #[trace]
  fn invalid_locations_are_errors(#[case] value: &str, #[case] message: &str) {
    assert_err!(parse_location(value), message);
  }

  #[rustfmt::skip]
  #[rstest]
  #[case::plain(          "data:,((A,B),C);",                     "((A,B),C);")]
  #[case::percent(        "data:,((A%20x,B),C)%3B",               "((A x,B),C);")]
  #[case::invalid_percent("data:,(A%zz,B);",                      "(A%zz,B);")]
  #[case::signed_percent( "data:,(A%+5,B);",                      "(A%+5,B);")]
  #[case::base64(         "data:text/plain;base64,KChBLEIpLEMpOw==", "((A,B),C);")]
  #[case::base64_unpadded("data:;base64,KChBLEIpLEMpOw",          "((A,B),C);")]
  #[case::base64_url(     "data:;base64,KChBLEIpLEMpOz8-",        "((A,B),C);?>")]
  #[case::base64_spaces(  "data:;base64,KChBLEIpLEMpOz8 ",        "((A,B),C);?>")]
  #[trace]
  fn data_locations_are_decoded(#[case] value: &str, #[case] text: &str) {
    assert_eq!(LinkLocation::Data { text: text.to_owned() }, parse_location(value).unwrap());
  }

  #[test]
  fn gzip_data_is_decompressed() {
    let encoded = general_purpose::STANDARD.encode(gzip(b"((A,B),C);"));
    assert_eq!(
      LinkLocation::Data {
        text: "((A,B),C);".to_owned()
      },
      parse_location(&format!("data:application/gzip;base64,{encoded}")).unwrap()
    );
  }

  #[rustfmt::skip]
  #[rstest]
  #[case::doctype(  b"  <!DOCTYPE html><html></html>".as_slice(), Err("a web page, not a tree file".to_owned()))]
  #[case::html(     b"\n<HTML lang=en>".as_slice(),              Err("a web page, not a tree file".to_owned()))]
  #[case::not_utf8( b"(A,\xff);".as_slice(),                     Err("not a UTF-8 text file: invalid utf-8 sequence of 1 bytes from index 3".to_owned()))]
  #[case::broken_gz(b"\x1f\x8b\x08\x00garbage".as_slice(),        Err("not a valid gzip file: incomplete deflate stream".to_owned()))]
  #[case::newick(   b"((A,B),C);\n".as_slice(),                  Ok("((A,B),C);\n".to_owned()))]
  #[case::html_name(b"(<html>,B);".as_slice(),                   Ok("(<html>,B);".to_owned()))]
  #[trace]
  fn tree_bytes_are_checked(#[case] bytes: &[u8], #[case] expected: Result<String, String>) {
    #[cfg_attr(
      dylint_lib = "mordant",
      expect(stringified_error, reason = "the test checks the messages that users see")
    )]
    let actual = decode_tree_bytes(bytes).map_err(|e| e.to_string());
    assert_eq!(expected, actual);
  }

  #[test]
  fn gzip_members_are_concatenated() {
    let mut bytes = gzip(b"((A,B),");
    bytes.extend(gzip(b"C);"));
    assert_eq!("((A,B),C);", decode_tree_bytes(&bytes).unwrap());
  }

  #[test]
  fn request_url_encodes_what_a_uri_cannot_hold() {
    assert_eq!(
      "https://x/a%20b/%C3%A9.nwk?q=1&r=%22#f",
      request_url("https://x/a b/é.nwk?q=1&r=\"#f")
    );
  }
}
