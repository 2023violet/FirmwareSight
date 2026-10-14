//! Untrusted intake: stat, size guard, streaming hash, format detection, then one immutable
//! read.
//!
//! The ordering is the pipeline contract of `04_TECH/16_ARTIFACT_ANALYSIS_PIPELINE.md`:
//! the guard runs on the stat result, so an oversized artifact is rejected before any
//! allocation rather than after.

use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256 as Sha2Digest};

use crate::error::ArtifactError;
use firmwaresight_core::domain::identity::Sha256;

/// Read chunk for hashing. Fixed so that the digest work is not sensitive to buffer sizing.
const HASH_CHUNK_BYTES: usize = 64 * 1024;

/// Full-buffer ceiling. `04_TECH/06_PERFORMANCE_BUDGETS.md` and `16:126-138` fix this MVP
/// default; raising it is an explicit project/user policy, never automatic, and it must not
/// silently fall back to mmap.
pub const DEFAULT_MAX_FULL_BUFFER_BYTES: u64 = 512 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GuardConfig {
    pub max_full_buffer_bytes: u64,
}

impl Default for GuardConfig {
    fn default() -> Self {
        Self {
            max_full_buffer_bytes: DEFAULT_MAX_FULL_BUFFER_BYTES,
        }
    }
}

impl GuardConfig {
    #[must_use]
    pub fn with_limit(max_full_buffer_bytes: u64) -> Self {
        Self {
            max_full_buffer_bytes,
        }
    }
}

/// What the first bytes of a file look like.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetectedFormat {
    Elf32Le,
    Elf32Be,
    Elf64Le,
    Elf64Be,
    IntelHex,
    IhexLike,
    Binary,
    Empty,
    Unknown,
}

impl DetectedFormat {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Elf32Le | Self::Elf32Be | Self::Elf64Le | Self::Elf64Be => "elf",
            Self::IntelHex | Self::IhexLike => "intel-hex",
            Self::Binary => "binary",
            Self::Empty => "empty",
            Self::Unknown => "unknown",
        }
    }

    #[must_use]
    pub fn is_elf(self) -> bool {
        matches!(
            self,
            Self::Elf32Le | Self::Elf32Be | Self::Elf64Le | Self::Elf64Be
        )
    }
}

/// A file that passed the guard and was read exactly once, together with the facts gathered
/// while streaming it in.
#[derive(Debug, Clone)]
pub struct GuardedInput {
    pub path: PathBuf,
    pub byte_size: u64,
    pub sha256: Sha256,
    pub format: DetectedFormat,
    pub bytes: Vec<u8>,
    /// Phase timings in milliseconds, kept separate from the content facts so a deterministic
    /// payload never embeds wall-clock drift.
    pub hash_ms: u128,
    pub read_ms: u128,
}

impl GuardedInput {
    /// stat -> guard -> bounded read and SHA-256 -> detect.
    pub fn load(path: &Path, guard: GuardConfig) -> Result<Self, ArtifactError> {
        let metadata = std::fs::metadata(path).map_err(|err| match err.kind() {
            std::io::ErrorKind::NotFound => crate::error::not_found(path),
            _ => crate::error::unreadable(path, err),
        })?;

        if !metadata.is_file() {
            return Err(ArtifactError::MalformedArtifact {
                detail: "the path is not a regular file".to_owned(),
            });
        }

        let byte_size = metadata.len();
        if byte_size > guard.max_full_buffer_bytes {
            // Deliberately before any allocation of the file contents.
            return Err(ArtifactError::ArtifactTooLarge {
                path: crate::error::display_path(path),
                byte_size,
                limit_bytes: guard.max_full_buffer_bytes,
            });
        }

        let mut file = File::open(path).map_err(|err| crate::error::unreadable(path, err))?;
        // Hash the exact bytes handed to the parser. The initial stat bounds the buffer, and a
        // same-handle metadata check after the read detects size changes without reading past it.
        let (bytes, digest, hash_ms, read_ms) = read_after_stat(
            &mut file,
            byte_size,
            guard.max_full_buffer_bytes,
        )
        .map_err(|error| match error {
            ReadAfterStatError::Io(err) => crate::error::unreadable(path, err),
            ReadAfterStatError::Changed { expected, actual } => ArtifactError::MalformedArtifact {
                detail: format!(
                    "file changed while being read: stat said {expected} bytes, read {actual}"
                ),
            },
        })?;
        let final_size = file
            .metadata()
            .map_err(|err| crate::error::unreadable(path, err))?
            .len();
        verify_size(byte_size, final_size).map_err(|_| ArtifactError::MalformedArtifact {
            detail: format!(
                "file changed while being read: stat said {byte_size} bytes, now {final_size}"
            ),
        })?;

        let format = detect_format(&bytes);

        Ok(Self {
            path: path.to_path_buf(),
            byte_size,
            sha256: Sha256::parse(&lowercase_hex(&digest)).map_err(|_| {
                ArtifactError::InternalBug {
                    detail: "digest produced non-hex output".to_owned(),
                }
            })?,
            format,
            bytes,
            hash_ms,
            read_ms,
        })
    }
}

#[derive(Debug)]
enum ReadAfterStatError {
    Io(std::io::Error),
    Changed { expected: u64, actual: u64 },
}

fn read_after_stat<R: Read>(
    reader: &mut R,
    expected_size: u64,
    limit_bytes: u64,
) -> Result<(Vec<u8>, [u8; 32], u128, u128), ReadAfterStatError> {
    let cap = expected_size.min(limit_bytes);
    let mut limited = reader.take(cap);
    let mut bytes = Vec::with_capacity(usize::try_from(cap).unwrap_or(usize::MAX));
    let mut chunk = [0_u8; HASH_CHUNK_BYTES];
    let mut hasher = Sha2Digest::new();
    let mut hash_time = std::time::Duration::ZERO;
    let mut read_time = std::time::Duration::ZERO;

    loop {
        let started = std::time::Instant::now();
        let read = limited.read(&mut chunk).map_err(ReadAfterStatError::Io)?;
        read_time += started.elapsed();
        if read == 0 {
            break;
        }
        let started = std::time::Instant::now();
        hasher.update(&chunk[..read]);
        bytes.extend_from_slice(&chunk[..read]);
        hash_time += started.elapsed();
    }

    verify_size(expected_size, bytes.len() as u64)?;

    Ok((
        bytes,
        hasher.finalize().into(),
        hash_time.as_millis(),
        read_time.as_millis(),
    ))
}

fn verify_size(expected: u64, actual: u64) -> Result<(), ReadAfterStatError> {
    if actual == expected {
        Ok(())
    } else {
        Err(ReadAfterStatError::Changed { expected, actual })
    }
}

/// Inspect only the leading bytes; never trusts the declared sizes inside.
#[must_use]
pub fn detect_format(bytes: &[u8]) -> DetectedFormat {
    if bytes.is_empty() {
        return DetectedFormat::Empty;
    }
    if bytes.starts_with(b"\x7fELF") {
        // ei_class: 1 = 32-bit, 2 = 64-bit; ei_data: 1 = LE, 2 = BE.
        let class = bytes.get(4).copied().unwrap_or(0);
        let data = bytes.get(5).copied().unwrap_or(0);
        return match (class, data) {
            (1, 1) => DetectedFormat::Elf32Le,
            (1, 2) => DetectedFormat::Elf32Be,
            (2, 1) => DetectedFormat::Elf64Le,
            (2, 2) => DetectedFormat::Elf64Be,
            _ => DetectedFormat::Unknown,
        };
    }
    if bytes.starts_with(b":") {
        return DetectedFormat::IntelHex;
    }
    if bytes.len() >= 8 && bytes.iter().all(u8::is_ascii_graphic) {
        return DetectedFormat::IhexLike;
    }
    DetectedFormat::Binary
}

/// Lowercase hex, written here rather than pulling in a `hex` crate that the dependency
/// baseline does not sanction.
#[must_use]
pub fn lowercase_hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(char::from(DIGITS[usize::from(byte >> 4)]));
        out.push(char::from(DIGITS[usize::from(byte & 0x0f)]));
    }
    out
}

/// Write a synthetic workload for guard and benchmark runs. Never committed to Git.
pub fn write_sparse_workload(path: &Path, size_bytes: u64, prefix: &[u8]) -> std::io::Result<()> {
    let mut file = File::create(path)?;
    file.write_all(prefix)?;
    if size_bytes > prefix.len() as u64 {
        file.set_len(size_bytes)?;
    }
    file.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn bounded_read_rejects_growth_after_stat_without_reading_past_limit() {
        let mut source = CountingReader::new(b"0123456789");

        let result = read_after_stat(&mut source, 2, 4);

        let (bytes, _, _, _) = result.unwrap();
        assert_eq!(bytes, b"01");
        assert_eq!(source.bytes_read, 2, "the stat size is a hard read limit");
        assert!(matches!(
            verify_size(2, 10),
            Err(ReadAfterStatError::Changed {
                expected: 2,
                actual: 10
            })
        ));
    }

    #[test]
    fn bounded_read_rejects_short_read() {
        let mut source = Cursor::new(b"abc".as_slice());

        let result = read_after_stat(&mut source, 4, 8);

        assert!(matches!(
            result,
            Err(ReadAfterStatError::Changed {
                expected: 4,
                actual: 3
            })
        ));
    }

    #[test]
    fn digest_is_computed_from_the_same_bytes_returned_for_parsing() {
        let observed = b"changed after stat";
        let mut source = Cursor::new(observed.as_slice());

        let (bytes, digest, _, _) =
            read_after_stat(&mut source, observed.len() as u64, 64).unwrap();

        assert_eq!(bytes, observed);
        assert_eq!(digest.as_slice(), Sha2Digest::digest(observed).as_slice());
    }

    struct CountingReader<'a> {
        bytes: &'a [u8],
        bytes_read: usize,
    }

    impl<'a> CountingReader<'a> {
        fn new(bytes: &'a [u8]) -> Self {
            Self {
                bytes,
                bytes_read: 0,
            }
        }
    }

    impl Read for CountingReader<'_> {
        fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
            let count = buffer.len().min(self.bytes.len() - self.bytes_read);
            buffer[..count].copy_from_slice(&self.bytes[self.bytes_read..self.bytes_read + count]);
            self.bytes_read += count;
            Ok(count)
        }
    }
    #[test]
    fn lowercase_hex_is_stable_and_lower_case() {
        assert_eq!(lowercase_hex(&[0x00, 0x0f, 0xff]), "000fff");
        assert_eq!(lowercase_hex(&[]), "");
        assert!(!lowercase_hex(&[0xab]).contains('A'));
    }

    #[test]
    fn sha256_type_rejects_uppercase() {
        // The stable identity representation is lowercase; anything else is a bug upstream.
        assert!(Sha256::parse(&"A".repeat(64)).is_err());
        assert!(Sha256::parse(&"0".repeat(64)).is_ok());
    }

    #[test]
    fn elf_class_and_endian_come_from_the_header_not_from_assumption() {
        assert_eq!(detect_format(b"\x7fELF\x01\x01"), DetectedFormat::Elf32Le);
        assert_eq!(detect_format(b"\x7fELF\x01\x02"), DetectedFormat::Elf32Be);
        assert_eq!(detect_format(b"\x7fELF\x02\x01"), DetectedFormat::Elf64Le);
        assert_eq!(detect_format(b"\x7fELF\x02\x02"), DetectedFormat::Elf64Be);
        // Declared ELF magic with an impossible class is unknown, not "assume 32-bit LE".
        assert_eq!(detect_format(b"\x7fELF\x09\x09"), DetectedFormat::Unknown);
    }

    #[test]
    fn empty_and_non_elf_inputs_are_classified_without_error() {
        assert_eq!(detect_format(&[]), DetectedFormat::Empty);
        assert_eq!(detect_format(b"MZ\x90\x00"), DetectedFormat::Binary);
        assert_eq!(detect_format(b":020000041000EA"), DetectedFormat::IntelHex);
    }
}
