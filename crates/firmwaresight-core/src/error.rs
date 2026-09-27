/// Stable, machine-readable error taxonomy for the core layer.
///
/// Variant names follow `05_ENGINEERING/03_ERROR_MODEL.md`. `ArtifactTooLarge` is required by
/// `04_TECH/16_ARTIFACT_ANALYSIS_PIPELINE.md`, which mandates a typed size-guard error raised
/// before any full-buffer allocation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CoreError {
    /// A declared fact was asked to overwrite an observed one.
    DeclaredCannotOverwriteObserved { field: String },
    /// A snapshot was mutated after construction; snapshots are immutable by contract.
    SnapshotAlreadySealed,
    /// An invariant that should be structurally impossible was violated.
    InternalInvariant { detail: String },
}
