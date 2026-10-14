//! Schematic banners prove refusal, never vendor compiler compatibility.
use firmwaresight_artifact::{ArtifactError, map};
use firmwaresight_artifact::{
    elf,
    intake::{GuardConfig, GuardedInput},
    pipeline::{self, AnalysisRequest},
};
use firmwaresight_core::domain::identity::{Architecture, Bitness, Endianness};
use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .into()
}

#[test]
fn foreign_linker_banners_cannot_fall_through_to_gnu_even_with_gnu_text() {
    for banner in [
        "ARM Linker map information",
        "Component: ARM Compiler 5.06 Tool: armlink [synthetic]",
        "IAR ELF Linker V7.80.1.28/LNX for ARM",
        "IAR Linker",
    ] {
        for suffix in [
            "",
            "\nMemory Configuration\nFLASH 0x08000000 0x1000 xr\nLinker script and memory map",
        ] {
            let text = format!("{}\n{banner}{suffix}", "synthetic preamble\n".repeat(5000));
            assert!(
                matches!(map::parse(&text), Err(ArtifactError::MapUnsupported { .. })),
                "foreign format must not be parsed as GNU"
            );
        }
    }
}

#[test]
fn clang_objects_prove_32_64_and_both_endian_metadata_without_vendor_claims() {
    for (target, class, endian, architecture) in [
        (
            "armv7-none-eabi",
            Bitness::Bits32,
            Endianness::Little,
            Architecture::Arm,
        ),
        (
            "armeb-none-eabi",
            Bitness::Bits32,
            Endianness::Big,
            Architecture::Arm,
        ),
        (
            "aarch64-none-elf",
            Bitness::Bits64,
            Endianness::Little,
            Architecture::AArch64,
        ),
        (
            "aarch64_be-none-elf",
            Bitness::Bits64,
            Endianness::Big,
            Architecture::AArch64,
        ),
    ] {
        let input = GuardedInput::load(
            &root().join(format!("fixtures/elf/interop-clang/{target}.o")),
            GuardConfig::default(),
        )
        .unwrap();
        assert_eq!(&input.bytes[..4], b"\x7fELF");
        assert_eq!(input.bytes[4], if class == Bitness::Bits32 { 1 } else { 2 });
        assert_eq!(
            input.bytes[5],
            if endian == Endianness::Little { 1 } else { 2 }
        );
        let parsed = elf::parse(&input.bytes, &input.sha256).unwrap();
        assert_eq!(
            (parsed.bitness, parsed.endianness, parsed.architecture),
            (class, endian, architecture)
        );
        assert!(parsed.symbol_table_present);
        assert!(
            !parsed.debug_sections_present,
            "recipe uses -g0, not DWARF decoding proof"
        );
        assert!(
            parsed
                .symbols
                .iter()
                .any(|s| s.name.value().map(String::as_str) == Some("fixture_function"))
        );
        let data = parsed
            .sections
            .iter()
            .find(|s| s.name.value().map(String::as_str) == Some(".data"))
            .unwrap();
        assert_eq!(data.memory_size.value(), Some(&4));
        assert_eq!(data.file_size, 4);
        let offset = *data.file_offset.value().unwrap() as usize;
        let raw: [u8; 4] = input.bytes[offset..offset + 4].try_into().unwrap();
        assert_eq!(
            if endian == Endianness::Little {
                u32::from_le_bytes(raw)
            } else {
                u32::from_be_bytes(raw)
            },
            7
        );
        let bss = parsed
            .sections
            .iter()
            .find(|s| s.name.value().map(String::as_str) == Some(".bss"))
            .unwrap();
        assert_eq!((bss.memory_size.value(), bss.file_size), (Some(&4), 0));
    }
}

#[test]
fn compatible_elf_only_works_while_explicit_foreign_or_invalid_map_is_refused() {
    let scratch = root()
        .join("target/interop-tests")
        .join(format!("固件-{}", std::process::id()));
    std::fs::create_dir_all(&scratch).unwrap();
    let fixture = root().join("fixtures/elf/p5-compat/clang-arm/firmware.elf");
    let expected = pipeline::analyze(&AnalysisRequest::new(&fixture)).unwrap();
    for extension in ["axf", "out"] {
        let renamed = scratch.join(format!("固件.{extension}"));
        std::fs::copy(&fixture, &renamed).unwrap();
        let alone = pipeline::analyze(&AnalysisRequest::new(&renamed)).unwrap();
        assert_eq!(alone.input.sha256, expected.input.sha256);
        assert!(alone.map_input.is_none() && alone.map_evidence.is_none());
        // This is a renamed Clang fixture, never a Keil/IAR-produced file.
        for bytes in [
            b"IAR ELF Linker synthetic\nMemory Configuration".as_slice(),
            b"Component: ARM Compiler Tool: armlink [synthetic]",
            b"\xff\xfe\0I\0A\0R",
            b"\x7fELF",
        ] {
            let path = scratch.join("foreign.map");
            std::fs::write(&path, bytes).unwrap();
            let err =
                pipeline::analyze(&AnalysisRequest::new(&renamed).with_map(&path)).unwrap_err();
            assert!(matches!(err, ArtifactError::MapUnsupported { .. }));
            assert!(err.remediation().contains("without a MAP"));
            assert_eq!(err.stable_code(), "ERR-MAP-3001");
        }
        std::fs::write(&renamed, b"NOT ELF").unwrap();
        assert!(matches!(
            pipeline::analyze(&AnalysisRequest::new(&renamed)),
            Err(ArtifactError::UnsupportedFormat { .. })
        ));
    }
    let malformed = scratch.join("truncated.axf");
    std::fs::write(&malformed, b"\x7fELF\x01\x01\x01").unwrap();
    assert!(pipeline::analyze(&AnalysisRequest::new(&malformed)).is_err());
    let oversized = scratch.join("large.map");
    std::fs::write(&oversized, vec![b' '; 8192]).unwrap();
    assert!(matches!(
        GuardedInput::load(&oversized, GuardConfig::with_limit(4096)),
        Err(ArtifactError::ArtifactTooLarge { .. })
    ));
    std::fs::remove_dir_all(scratch).unwrap();
}
