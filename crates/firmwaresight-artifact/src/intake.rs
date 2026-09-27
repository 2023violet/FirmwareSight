//! Untrusted intake: stat, size guard, streaming hash, format detection, then one immutable
//! read.
//!
//! The ordering is the pipeline contract of `04_TECH/16_ARTIFACT_ANALYSIS_PIPELINE.md`:
//! the guard runs on the stat result, so an oversized artifact is rejected before any
//! allocation rather than after.

use std::fs::File;
use std::io::{Read, Seek, Write};
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
    /// stat -> guard -> streaming SHA-256 -> detect -> immutable read.
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

        let hash_start = std::time::Instant::now();
        let mut file = File::open(path).map_err(|err| crate::error::unreadable(path, err))?;
        let mut hasher = Sha2Digest::new();
        let mut chunk = vec![0_u8; HASH_CHUNK_BYTES];
        loop {
            let read = file
                .read(&mut chunk)
                .map_err(|err| crate::error::unreadable(path, err))?;
            if read == 0 {
                break;
            }
            hasher.update(&chunk[..read]);
        }
        let hash_ms = hash_start.elapsed().as_millis();

        // Second pass from the same open handle: the file is small enough by now, and reading
        // once for hashing and once for content keeps the hash streaming rather than forcing a
        // full read up front.
        let read_start = std::time::Instant::now();
        let mut bytes = Vec::with_capacity(
            usize::try_from(byte_size)
                .unwrap_or(usize::try_from(guard.max_full_buffer_bytes).unwrap_or(usize::MAX)),
        );
        file.rewind()
            .map_err(|err| crate::error::unreadable(path, err))?;
        file.read_to_end(&mut bytes)
            .map_err(|err| crate::error::unreadable(path, err))?;
        let read_ms = read_start.elapsed().as_millis();

        if bytes.len() as u64 != byte_size {
            return Err(ArtifactError::MalformedArtifact {
                detail: format!(
                    "file changed while being read: stat said {byte_size} bytes, read {}",
                    bytes.len()
                ),
            });
        }

        let format = detect_format(&bytes);

        Ok(Self {
            path: path.to_path_buf(),
            byte_size,
            sha256: Sha256::parse(&lowercase_hex(&hasher.finalize())).map_err(|_| {
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
