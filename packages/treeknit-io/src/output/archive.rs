//! The ZIP archive of the result files.

use crate::analysis;
use crate::output::RESULTS_DIR;
use std::collections::BTreeMap;
use std::collections::btree_map::Entry;
use std::io::{Cursor, Write};
use zip::result::ZipError;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, DateTime, System, ZipWriter};

/// Media type of the ZIP archive of the results.
pub const ARCHIVE_MEDIA_TYPE: &str = "application/zip";

/// A ZIP archive under construction, so that each file can be added while its text exists. Each
/// file is under `treeknit_results/` at its path. Every entry is deflated, dated 1980-01-01 00:00
/// (the earliest ZIP time), and made by a Unix system with the permissions `rw-r--r--`, so equal
/// files give a byte-identical archive on every host.
pub struct Archive {
  zip: ZipWriter<Cursor<Vec<u8>>>,
  /// The path of each file by its `analysis::label_key`, to reject paths that differ only in case.
  paths: BTreeMap<String, String>,
}

impl Archive {
  /// An archive whose buffer holds `bytes` before it grows: the sum of the text lengths of the
  /// files bounds the archive of texts that deflate compresses, apart from the headers.
  pub fn with_capacity(bytes: usize) -> Archive {
    Archive {
      zip: ZipWriter::new(Cursor::new(Vec::with_capacity(bytes))),
      paths: BTreeMap::new(),
    }
  }

  /// Add the file `text` at `path`; fails when the archive has a file at `path` or at a path that
  /// differs only in case, because extracting it on macOS or Windows would overwrite one file
  /// with the other.
  pub fn add(&mut self, path: &str, text: &str) -> Result<(), ArchiveError> {
    match self.paths.entry(analysis::label_key(path)) {
      Entry::Occupied(e) => {
        return Err(ArchiveError::Repeated {
          first: e.get().clone(),
          path: path.to_owned(),
        });
      },
      Entry::Vacant(e) => {
        e.insert(path.to_owned());
      },
    }
    let options = SimpleFileOptions::default()
      .compression_method(CompressionMethod::Deflated)
      .last_modified_time(DateTime::DEFAULT)
      .system(System::Unix)
      .unix_permissions(0o644);
    self.zip.start_file(format!("{RESULTS_DIR}/{path}"), options)?;
    self.zip.write_all(text.as_bytes()).map_err(ZipError::from)?;
    Ok(())
  }

  /// The bytes of the archive.
  pub fn finish(self) -> Result<Vec<u8>, ArchiveError> {
    Ok(self.zip.finish()?.into_inner())
  }
}

/// Failure to build a ZIP archive.
#[derive(Debug, thiserror::Error)]
pub enum ArchiveError {
  /// Two files have the same path, ignoring case.
  #[error("cannot build the ZIP archive: {}", repeated_paths(.first, .path))]
  Repeated { first: String, path: String },
  /// The ZIP writer failed.
  #[error("cannot build the ZIP archive: {0}")]
  Zip(#[from] ZipError),
}

/// The two paths of [`ArchiveError::Repeated`], which are equal or differ only in case.
fn repeated_paths(first: &str, path: &str) -> String {
  if first == path {
    format!("two files are named {path:?}")
  } else {
    format!("the files {first:?} and {path:?} differ only in case")
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::output::OutputFile;
  use pretty_assertions::assert_eq;
  use rstest::rstest;
  use treeknit_testing::assert_err;

  fn fixed_files() -> Vec<OutputFile> {
    vec![
      OutputFile::new("MCCs.json".to_owned(), "{}\n".to_owned()),
      OutputFile::new("ARG/arg.nwk".to_owned(), "(A,B);\n".to_owned()),
      OutputFile::new("MCCs.dat".to_owned(), "A,B".to_owned()),
    ]
  }

  #[test]
  fn zip_archive_holds_every_file_under_the_results_directory() {
    let bytes = zip_archive(&fixed_files()).unwrap();
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).unwrap();
    let entries: Vec<ZipEntry> = (0..archive.len())
      .map(|i| {
        let mut entry = archive.by_index(i).unwrap();
        let mut text = String::new();
        std::io::Read::read_to_string(&mut entry, &mut text).unwrap();
        ZipEntry {
          name: entry.name().to_owned(),
          text,
          modified: entry.last_modified(),
          compression: entry.compression(),
          unix_mode: entry.unix_mode(),
        }
      })
      .collect();
    let entry = |name: &str, text: &str| ZipEntry {
      name: name.to_owned(),
      text: text.to_owned(),
      modified: Some(DateTime::DEFAULT),
      compression: CompressionMethod::Deflated,
      // Oracle: a regular file (S_IFREG, 0o100000) with the permissions rw-r--r--.
      unix_mode: Some(0o100_644),
    };
    let expected = vec![
      entry("treeknit_results/MCCs.json", "{}\n"),
      entry("treeknit_results/ARG/arg.nwk", "(A,B);\n"),
      entry("treeknit_results/MCCs.dat", "A,B"),
    ];
    assert_eq!(expected, entries);
  }

  /// An entry of a ZIP archive as `zip::ZipArchive` reads it back.
  #[derive(Debug, PartialEq)]
  struct ZipEntry {
    name: String,
    text: String,
    modified: Option<DateTime>,
    compression: CompressionMethod,
    unix_mode: Option<u32>,
  }

  #[test]
  fn zip_archive_entries_are_made_by_unix_on_every_host() {
    // Oracle: APPNOTE.TXT 4.4.2 and 4.4.15: byte 5 of a central directory header is the host
    // system of "version made by" (3 for Unix), and bytes 38 to 41 are the external attributes,
    // which hold the Unix mode in their upper 16 bits.
    let bytes = zip_archive(&fixed_files()).unwrap();
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes.clone())).unwrap();
    let headers: Vec<(u8, u32)> = (0..archive.len())
      .map(|i| {
        let start = usize::try_from(archive.by_index(i).unwrap().central_header_start()).unwrap();
        #[expect(clippy::little_endian_bytes, reason = "ZIP headers are little-endian")]
        let attributes = u32::from_le_bytes(bytes[start + 38..start + 42].try_into().unwrap());
        (bytes[start + 5], attributes)
      })
      .collect();
    assert_eq!(vec![(3, 0o100_644 << 16); 3], headers);
  }

  #[test]
  fn zip_archive_of_equal_files_is_byte_identical_within_a_run() {
    // The fixed time, host system, and permissions of the tests around this one make the bytes
    // independent of the host.
    assert_eq!(
      zip_archive(&fixed_files()).unwrap(),
      zip_archive(&fixed_files()).unwrap()
    );
  }

  #[test]
  fn zip_archive_keeps_non_ascii_names_as_utf8() {
    let files = vec![OutputFile::new("Ålesund_résolu.nwk".to_owned(), "(A,B);\n".to_owned())];
    let mut archive = zip::ZipArchive::new(Cursor::new(zip_archive(&files).unwrap())).unwrap();
    assert_eq!(
      "treeknit_results/Ålesund_résolu.nwk",
      archive.by_index(0).unwrap().name()
    );
  }

  #[test]
  fn zip_archive_dates_entries_at_the_earliest_zip_time() {
    // Oracle: the MS-DOS date of 1980-01-01 is 0x0021 (day 1, month 1, year 1980 + 0) and the
    // time of 00:00:00 is 0; the local file header holds the time at byte 10 and the date at byte
    // 12 (APPNOTE.TXT 4.3.7).
    let bytes = zip_archive(&fixed_files()).unwrap();
    assert_eq!(
      [0x50, 0x4b, 0x03, 0x04, 0x00, 0x00, 0x21, 0x00],
      [&bytes[..4], &bytes[10..14]].concat()[..]
    );
  }

  #[rustfmt::skip]
  #[rstest]
  #[case::repeated( "MCCs.json", "cannot build the ZIP archive: two files are named \"MCCs.json\"")]
  #[case::case_only("mccs.JSON", "cannot build the ZIP archive: the files \"MCCs.json\" and \"mccs.JSON\" differ only in case")]
  #[trace]
  fn zip_archive_rejects_a_path_repeated_ignoring_case(#[case] second: &str, #[case] message: &str) {
    let files = vec![
      OutputFile::new("MCCs.json".to_owned(), String::new()),
      OutputFile::new(second.to_owned(), String::new()),
    ];
    assert_err!(zip_archive(&files), message);
  }
  #[rustfmt::skip]
  #[rstest]
  #[case::json(  "MCCs.json",   "application/json")]
  #[case::upper( "A.JSON",      "application/json")]
  #[case::newick("ha.nwk",      "text/plain")]
  #[case::table( "ARG/nodes.dat", "text/plain")]
  #[case::none(  "ha_resolved", "text/plain")]
  #[case::svg(   "ARG/arg.svg", "image/svg+xml")]
  #[trace]
  fn output_file_media_type_follows_the_extension(#[case] path: &str, #[case] expected: &str) {
    assert_eq!(expected, OutputFile::new(path.to_owned(), String::new()).media_type);
  }

  /// A ZIP archive of `files`, as `Archive` writes it.
  fn zip_archive(files: &[OutputFile]) -> Result<Vec<u8>, ArchiveError> {
    let mut archive = Archive::with_capacity(files.iter().map(|f| f.text.len()).sum());
    for f in files {
      archive.add(&f.path, &f.text)?;
    }
    archive.finish()
  }
}
