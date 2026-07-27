use chrono::{SecondsFormat, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{HashMap, HashSet},
    env,
    ffi::OsString,
    fs,
    io::Cursor,
    path::{Path, PathBuf},
};
use uuid::Uuid;

const CONTRACT_ID: &str = "tileforge-actor-32-v1";
const SESSION_ID_MAX_LENGTH: usize = 96;
const SESSION_SLUG_MAX_LENGTH: usize = 64;
const CANDIDATE_ID_MAX_LENGTH: usize = 96;
const GENERATION_REQUEST_ID_MAX_LENGTH: usize = 96;
const CONCEPT_PNG_MAX_BYTES: usize = 1_048_576;
const VALIDATION_REPORT_VERSION: u32 = 1;
const STRUCTURAL_VALIDATOR_ID: &str = "tileforge-actor-32-structural-v1";
const TURNAROUND_VALIDATOR_ID: &str = "tileforge-actor-32-turnaround-structural-v1";
const WALK_CYCLE_VALIDATOR_ID: &str = "tileforge-actor-32-walk-cycle-structural-v1";
const WORLD_TEST_VALIDATOR_ID: &str = "tileforge-actor-32-world-test-ground-luma-v1";
const EXPORT_VALIDATOR_ID: &str = "tileforge-actor-32-export-package-v1";
const WORLD_TEST_REFERENCE_PACK_ID: &str = "tileforge-world-test-v1";
const WALK_CYCLE_FRAMES_PER_DIRECTION: usize = 4;
const WALK_CYCLE_FRAME_DURATION_MS: u32 = 300;
const FRAME_WIDTH: u32 = 32;
const FRAME_HEIGHT: u32 = 32;
const ACTOR_HEIGHT_MIN: u32 = 22;
const ACTOR_HEIGHT_MAX: u32 = 30;
const FOOT_ANCHOR_X: u32 = 16;
const FOOT_ANCHOR_Y: u32 = 28;
const WALK_GROUND_CONTACT_Y: u32 = FOOT_ANCHOR_Y;
const PALETTE_MAX_COLORS: usize = 16;
const MINIMUM_GROUND_LUMA_DISTANCE: u32 = 15;
const WORLD_TEST_PREVIEW_WIDTH: u32 = 640;
const WORLD_TEST_PREVIEW_HEIGHT: u32 = 384;
const EXPORT_SHEET_WIDTH: u32 = FRAME_WIDTH * WALK_CYCLE_FRAMES_PER_DIRECTION as u32;
const EXPORT_SHEET_HEIGHT: u32 = FRAME_HEIGHT * 4;
const EXPORT_SHEET_FILE: &str = "sprite-sheet.png";
const EXPORT_METADATA_FILE: &str = "metadata.json";
const EXPORT_PROVENANCE_FILE: &str = "provenance.json";
const EXPORT_SHEET_LAYOUT: &str = "direction-rows-frame-columns-v1";
const WINDOWS_VENDOR_DIRECTORY: &str = "TileForge";
const WINDOWS_PRODUCT_DIRECTORY: &str = "Actor Studio";
const WORLD_TEST_SCENES: [&str; 4] = ["scale-lineup", "forest-clearing", "crownhold", "tidewater"];
const WORLD_TEST_THEMES: [&str; 4] = ["forest", "autumn", "dusk", "winter"];
const WORLD_TEST_REFERENCE_MANIFEST: &[u8] =
    include_bytes!("../../reference-packs/tileforge-world-test-v1/manifest.json");

#[derive(Debug, Clone, Copy)]
enum StructuralContactMode {
    ExactAnchor,
    FootAnchorRow,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum ActorKind {
    Mob,
    Npc,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ActorBrief {
    name: String,
    kind: ActorKind,
    description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct StudioSession {
    id: String,
    revision: u32,
    stage: String,
    brief: ActorBrief,
    contract_id: String,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SessionList {
    workspace_root: PathBuf,
    sessions: Vec<StudioSession>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct GenerationRequestExecution {
    mode: String,
    additional_paid_services: String,
    api_credentials: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct GenerationRequestOutput {
    artifact: String,
    direction: String,
    width: u32,
    height: u32,
    mime_type: String,
    import_tool: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct GenerationRequestAuthority {
    agents_may_generate: bool,
    agents_may_import: bool,
    agents_may_approve: bool,
    approval_owner: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ConceptGenerationRequest {
    schema_version: u32,
    id: String,
    revision: u32,
    session_id: String,
    stage: String,
    contract_id: String,
    created_at: String,
    prompt: String,
    requested_candidates: u32,
    execution: GenerationRequestExecution,
    output: GenerationRequestOutput,
    authority: GenerationRequestAuthority,
    lifecycle: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum CandidateSource {
    Imported,
    Generated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct CandidateProvenance {
    source: CandidateSource,
    #[serde(skip_serializing_if = "Option::is_none")]
    original_filename: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    provider: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    model: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct CandidateIntakeValidation {
    file_type: String,
    dimensions: String,
    alpha_channel: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ConceptCandidate {
    schema_version: u32,
    id: String,
    revision: u32,
    session_id: String,
    stage: String,
    direction: String,
    contract_id: String,
    source_file: String,
    mime_type: String,
    sha256: String,
    byte_length: usize,
    width: u32,
    height: u32,
    created_at: String,
    provenance: CandidateProvenance,
    intake_validation: CandidateIntakeValidation,
    review_status: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ConceptCandidatePayload {
    candidate: ConceptCandidate,
    png_bytes: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum TurnaroundDirection {
    Down,
    Right,
    Up,
    Left,
}

impl TurnaroundDirection {
    fn as_str(self) -> &'static str {
        match self {
            Self::Down => "down",
            Self::Right => "right",
            Self::Up => "up",
            Self::Left => "left",
        }
    }
}

const TURNAROUND_DIRECTIONS: [TurnaroundDirection; 4] = [
    TurnaroundDirection::Down,
    TurnaroundDirection::Right,
    TurnaroundDirection::Up,
    TurnaroundDirection::Left,
];

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct TurnaroundSourceSelection {
    candidate_id: String,
    candidate_sha256: String,
    selected_by: String,
    selected_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct TurnaroundDirectionSource {
    direction: TurnaroundDirection,
    source_file: String,
    sha256: String,
    byte_length: usize,
    width: u32,
    height: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct TurnaroundCandidate {
    schema_version: u32,
    id: String,
    revision: u32,
    session_id: String,
    stage: String,
    contract_id: String,
    source_selection: TurnaroundSourceSelection,
    directions: Vec<TurnaroundDirectionSource>,
    created_at: String,
    provenance: CandidateProvenance,
    review_status: String,
    identity_judgment: VisualJudgment,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct TurnaroundPngBytes {
    down: Vec<u8>,
    right: Vec<u8>,
    up: Vec<u8>,
    left: Vec<u8>,
}

impl TurnaroundPngBytes {
    fn get(&self, direction: TurnaroundDirection) -> &[u8] {
        match direction {
            TurnaroundDirection::Down => &self.down,
            TurnaroundDirection::Right => &self.right,
            TurnaroundDirection::Up => &self.up,
            TurnaroundDirection::Left => &self.left,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct TurnaroundCandidatePayload {
    candidate: TurnaroundCandidate,
    png_bytes: TurnaroundPngBytes,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct WalkCycleAcceptedDirectionSource {
    direction: TurnaroundDirection,
    sha256: String,
    byte_length: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct WalkCycleSourceTurnaround {
    turnaround_id: String,
    direction_sources: Vec<WalkCycleAcceptedDirectionSource>,
    accepted_by: String,
    accepted_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct WalkCycleFrameSource {
    direction: TurnaroundDirection,
    frame_index: usize,
    source_file: String,
    sha256: String,
    byte_length: usize,
    width: u32,
    height: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct WalkCycleCandidate {
    schema_version: u32,
    id: String,
    revision: u32,
    session_id: String,
    stage: String,
    contract_id: String,
    source_turnaround: WalkCycleSourceTurnaround,
    clip: String,
    frames_per_direction: usize,
    frame_duration_ms: u32,
    frames: Vec<WalkCycleFrameSource>,
    created_at: String,
    provenance: CandidateProvenance,
    review_status: String,
    motion_judgment: VisualJudgment,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct WalkCyclePngBytes {
    down: Vec<Vec<u8>>,
    right: Vec<Vec<u8>>,
    up: Vec<Vec<u8>>,
    left: Vec<Vec<u8>>,
}

impl WalkCyclePngBytes {
    fn get(&self, direction: TurnaroundDirection, frame_index: usize) -> Option<&[u8]> {
        let frames = match direction {
            TurnaroundDirection::Down => &self.down,
            TurnaroundDirection::Right => &self.right,
            TurnaroundDirection::Up => &self.up,
            TurnaroundDirection::Left => &self.left,
        };
        frames.get(frame_index).map(Vec::as_slice)
    }

    fn direction(&self, direction: TurnaroundDirection) -> &[Vec<u8>] {
        match direction {
            TurnaroundDirection::Down => &self.down,
            TurnaroundDirection::Right => &self.right,
            TurnaroundDirection::Up => &self.up,
            TurnaroundDirection::Left => &self.left,
        }
    }

    fn direction_mut(&mut self, direction: TurnaroundDirection) -> &mut Vec<Vec<u8>> {
        match direction {
            TurnaroundDirection::Down => &mut self.down,
            TurnaroundDirection::Right => &mut self.right,
            TurnaroundDirection::Up => &mut self.up,
            TurnaroundDirection::Left => &mut self.left,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct WalkCycleCandidatePayload {
    candidate: WalkCycleCandidate,
    png_bytes: WalkCyclePngBytes,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct WorldTestAcceptedFrameSource {
    direction: TurnaroundDirection,
    frame_index: usize,
    sha256: String,
    byte_length: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct WorldTestSourceWalkCycle {
    walk_cycle_id: String,
    frame_sources: Vec<WorldTestAcceptedFrameSource>,
    accepted_by: String,
    accepted_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct WorldTestReferenceReceipt {
    id: String,
    version: u32,
    manifest_sha256: String,
    checkout_commit: String,
    generated_engine_commit: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct WorldTestPreviewSource {
    scene: String,
    theme: String,
    source_file: String,
    sha256: String,
    byte_length: usize,
    width: u32,
    height: u32,
    reference_source_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct WorldTestPreparation {
    method: String,
    additional_ai_cost: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct WorldTestCandidate {
    schema_version: u32,
    id: String,
    revision: u32,
    session_id: String,
    stage: String,
    contract_id: String,
    source_walk_cycle: WorldTestSourceWalkCycle,
    reference_pack: WorldTestReferenceReceipt,
    previews: Vec<WorldTestPreviewSource>,
    created_at: String,
    preparation: WorldTestPreparation,
    review_status: String,
    final_art_judgment: VisualJudgment,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct WorldTestCandidatePayload {
    candidate: WorldTestCandidate,
    preview_png_bytes: HashMap<String, Vec<u8>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ExportPreviewSource {
    scene: String,
    theme: String,
    source_file: String,
    sha256: String,
    byte_length: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ExportApprovedWorldTest {
    world_test_id: String,
    document_sha256: String,
    preview_sources: Vec<ExportPreviewSource>,
    approved_by: String,
    approved_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ExportSourceWalkCycle {
    walk_cycle_id: String,
    frame_sources: Vec<WorldTestAcceptedFrameSource>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ExportFileReceipt {
    source_file: String,
    sha256: String,
    byte_length: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ExportSheetReceipt {
    source_file: String,
    sha256: String,
    byte_length: usize,
    width: u32,
    height: u32,
    cell_width: u32,
    cell_height: u32,
    layout: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ExportPackage {
    sprite_sheet: ExportSheetReceipt,
    metadata: ExportFileReceipt,
    provenance: ExportFileReceipt,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ExportPreparation {
    method: String,
    additional_ai_cost: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct PublishingBoundary {
    status: String,
    authority: String,
    message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ExportCandidate {
    schema_version: u32,
    id: String,
    revision: u32,
    session_id: String,
    stage: String,
    contract_id: String,
    approved_world_test: ExportApprovedWorldTest,
    source_walk_cycle: ExportSourceWalkCycle,
    package: ExportPackage,
    created_at: String,
    preparation: ExportPreparation,
    status: String,
    publishing: PublishingBoundary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ExportActorMetadata {
    name: String,
    kind: ActorKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ExportSheetMetadata {
    source_file: String,
    width: u32,
    height: u32,
    cell_width: u32,
    cell_height: u32,
    layout: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ExportAnimationMetadata {
    clip: String,
    directions: Vec<TurnaroundDirection>,
    frames_per_direction: usize,
    frame_duration_ms: u32,
    foot_anchor: Vec<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ExportFrameMetadata {
    direction: TurnaroundDirection,
    frame_index: usize,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    sha256: String,
    byte_length: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ExportMetadata {
    schema_version: u32,
    contract_id: String,
    actor: ExportActorMetadata,
    sheet: ExportSheetMetadata,
    animation: ExportAnimationMetadata,
    frames: Vec<ExportFrameMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ExportProvenance {
    schema_version: u32,
    export_id: String,
    session_id: String,
    approved_world_test: ExportApprovedWorldTest,
    source_walk_cycle: ExportSourceWalkCycle,
    preparation: ExportPreparation,
    publishing: PublishingBoundary,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ExportCandidatePayload {
    candidate: ExportCandidate,
    sprite_sheet_png_bytes: Vec<u8>,
    metadata: ExportMetadata,
    provenance: ExportProvenance,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ReferencePackSource {
    repository: String,
    checkout_commit: String,
    generated_engine_commit: String,
    generated: String,
    render_path: String,
    scale: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ReferencePackPreview {
    width: u32,
    height: u32,
    actor_direction: String,
    actor_frame_index: usize,
    compositor: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ReferenceRectangle {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ReferencePoint {
    x: u32,
    y: u32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ReferencePackEntry {
    scene: String,
    theme: String,
    source_file: String,
    source_sha256: String,
    source_byte_length: usize,
    source_width: u32,
    source_height: u32,
    viewport: ReferenceRectangle,
    actor_placement: ReferencePoint,
    ground_sample: ReferenceRectangle,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct WorldTestReferencePack {
    schema_version: u32,
    id: String,
    version: u32,
    contract_id: String,
    source: ReferencePackSource,
    preview: ReferencePackPreview,
    entries: Vec<ReferencePackEntry>,
}

#[derive(Debug)]
struct LoadedReferencePack {
    manifest: WorldTestReferencePack,
    manifest_sha256: String,
    sources: Vec<Vec<u8>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ValidationStatus {
    Pass,
    Fail,
    NotAssessed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ValidationRuleId {
    CanvasDimensions,
    HardAlpha,
    ActorHeight,
    FootAnchor,
    PaletteMaxColors,
    GroundLumaSeparation,
    FrameEdgeClipping,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ValidationRuleResult {
    id: ValidationRuleId,
    status: ValidationStatus,
    expected: String,
    observed: Option<String>,
    message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ValidationSummary {
    pass: u32,
    fail: u32,
    not_assessed: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum VisualJudgmentStatus {
    NotAssessed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct VisualJudgment {
    status: VisualJudgmentStatus,
    authority: String,
    message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ValidationReport {
    schema_version: u32,
    validator_id: String,
    candidate_id: String,
    candidate_sha256: String,
    contract_id: String,
    results: Vec<ValidationRuleResult>,
    summary: ValidationSummary,
    visual_judgment: VisualJudgment,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct TurnaroundDirectionReport {
    direction: TurnaroundDirection,
    report: ValidationReport,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct TurnaroundValidationReport {
    schema_version: u32,
    validator_id: String,
    turnaround_id: String,
    contract_id: String,
    directions: Vec<TurnaroundDirectionReport>,
    summary: ValidationSummary,
    identity_judgment: VisualJudgment,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct WalkCycleFrameReport {
    direction: TurnaroundDirection,
    frame_index: usize,
    report: ValidationReport,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct WalkCycleValidationReport {
    schema_version: u32,
    validator_id: String,
    walk_cycle_id: String,
    contract_id: String,
    frames: Vec<WalkCycleFrameReport>,
    summary: ValidationSummary,
    motion_judgment: VisualJudgment,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct WorldTestLumaMeasurement {
    scene: String,
    theme: String,
    direction: TurnaroundDirection,
    frame_index: usize,
    actor_mean_luma: u32,
    ground_mean_luma: u32,
    distance: u32,
    minimum: u32,
    status: ValidationStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct WorldTestValidationReport {
    schema_version: u32,
    validator_id: String,
    world_test_id: String,
    contract_id: String,
    measurements: Vec<WorldTestLumaMeasurement>,
    summary: ValidationSummary,
    final_art_judgment: VisualJudgment,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ExportValidationCheckId {
    ApprovedWorldTest,
    SourceWalkCycle,
    SpriteSheetIdentity,
    SpriteSheetPixels,
    MetadataIdentity,
    ProvenanceIdentity,
    PublishingBoundary,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ExportValidationCheck {
    id: ExportValidationCheckId,
    status: ValidationStatus,
    message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ExportValidationReport {
    schema_version: u32,
    validator_id: String,
    export_id: String,
    contract_id: String,
    checks: Vec<ExportValidationCheck>,
    summary: ValidationSummary,
    publishing: PublishingBoundary,
}

#[derive(Debug)]
struct DecodedPng {
    width: u32,
    height: u32,
    pixels: Vec<[u8; 4]>,
}

fn repository_workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("src-tauri must have a repository parent")
        .join(".studio")
}

fn workspace_root_from(
    override_root: Option<OsString>,
    local_app_data: Option<OsString>,
) -> PathBuf {
    if let Some(root) = override_root.filter(|value| !value.is_empty()) {
        return PathBuf::from(root);
    }
    if cfg!(target_os = "windows") {
        if let Some(root) = local_app_data.filter(|value| !value.is_empty()) {
            return PathBuf::from(root)
                .join(WINDOWS_VENDOR_DIRECTORY)
                .join(WINDOWS_PRODUCT_DIRECTORY)
                .join(".studio");
        }
    }
    repository_workspace_root()
}

fn workspace_root() -> PathBuf {
    workspace_root_from(env::var_os("TFAS_WORKSPACE"), env::var_os("LOCALAPPDATA"))
}

fn validate_brief(mut brief: ActorBrief) -> Result<ActorBrief, String> {
    brief.name = brief.name.trim().to_owned();
    brief.description = brief.description.trim().to_owned();

    let name_length = brief.name.chars().count();
    if name_length == 0 {
        return Err("Name is required.".to_owned());
    }
    if name_length > 80 {
        return Err("Name must be 80 characters or fewer.".to_owned());
    }

    let description_length = brief.description.chars().count();
    if description_length == 0 {
        return Err("Description is required.".to_owned());
    }
    if description_length > 2_000 {
        return Err("Description must be 2000 characters or fewer.".to_owned());
    }

    Ok(brief)
}

fn actor_slug(name: &str) -> String {
    let mut slug = String::new();
    let mut pending_separator = false;

    for character in name.to_ascii_lowercase().chars() {
        if character.is_ascii_alphanumeric() {
            if pending_separator && !slug.is_empty() && slug.len() < SESSION_SLUG_MAX_LENGTH {
                slug.push('-');
            }
            pending_separator = false;
            if slug.len() < SESSION_SLUG_MAX_LENGTH {
                slug.push(character);
            }
        } else {
            pending_separator = true;
        }
    }

    while slug.ends_with('-') {
        slug.pop();
    }
    if slug.is_empty() {
        "untitled".to_owned()
    } else {
        slug
    }
}

fn validate_session_id(session_id: &str) -> Result<(), String> {
    let valid_length = (3..=SESSION_ID_MAX_LENGTH).contains(&session_id.len());
    let valid_characters = session_id
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || character == '-');
    let valid_start = session_id
        .chars()
        .next()
        .is_some_and(|character| character.is_ascii_alphanumeric());

    if valid_length && valid_characters && valid_start {
        Ok(())
    } else {
        Err("Invalid session id.".to_owned())
    }
}

fn validate_candidate_id(candidate_id: &str) -> Result<(), String> {
    let valid_length = (3..=CANDIDATE_ID_MAX_LENGTH).contains(&candidate_id.len());
    let valid_characters = candidate_id
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || character == '-');
    let valid_start = candidate_id
        .chars()
        .next()
        .is_some_and(|character| character.is_ascii_alphanumeric());

    if valid_length && valid_characters && valid_start {
        Ok(())
    } else {
        Err("Invalid candidate id.".to_owned())
    }
}

fn session_file(root: &Path, session_id: &str) -> Result<PathBuf, String> {
    validate_session_id(session_id)?;
    Ok(root.join("sessions").join(session_id).join("session.json"))
}

fn validate_session(session: StudioSession) -> Result<StudioSession, String> {
    validate_session_id(&session.id)?;
    if session.revision == 0 {
        return Err("Session revision must be at least 1.".to_owned());
    }
    if !matches!(
        session.stage.as_str(),
        "brief" | "concept" | "turnaround" | "animate" | "world-test" | "export"
    ) {
        return Err("Invalid session stage.".to_owned());
    }
    if session.contract_id != CONTRACT_ID {
        return Err("Session contract id is incompatible.".to_owned());
    }
    validate_brief(session.brief.clone())?;
    Ok(session)
}

fn read_session(root: &Path, session_id: &str) -> Result<StudioSession, String> {
    let raw = fs::read_to_string(session_file(root, session_id)?)
        .map_err(|error| format!("Could not read session: {error}"))?;
    let session: StudioSession =
        serde_json::from_str(&raw).map_err(|error| format!("Invalid session document: {error}"))?;
    validate_session(session)
}

fn create_session_at(
    root: &Path,
    brief: ActorBrief,
    timestamp: &str,
    id_suffix: &str,
    temporary_suffix: &str,
) -> Result<StudioSession, String> {
    let brief = validate_brief(brief)?;
    let timestamp_digits: String = timestamp
        .chars()
        .filter(char::is_ascii_digit)
        .take(14)
        .collect();
    let session = StudioSession {
        id: format!(
            "{}-{}-{}",
            actor_slug(&brief.name),
            timestamp_digits,
            id_suffix
        ),
        revision: 1,
        stage: "concept".to_owned(),
        brief,
        contract_id: CONTRACT_ID.to_owned(),
        created_at: timestamp.to_owned(),
        updated_at: timestamp.to_owned(),
    };
    validate_session_id(&session.id)?;

    let sessions_root = root.join("sessions");
    fs::create_dir_all(&sessions_root)
        .map_err(|error| format!("Could not create session storage: {error}"))?;
    let final_directory = sessions_root.join(&session.id);
    let temporary_directory =
        sessions_root.join(format!(".{}.{}.tmp", session.id, temporary_suffix));
    fs::create_dir(&temporary_directory)
        .map_err(|error| format!("Could not stage session: {error}"))?;

    let publish_result = (|| -> Result<(), String> {
        let document = format!(
            "{}\n",
            serde_json::to_string_pretty(&session)
                .map_err(|error| format!("Could not serialize session: {error}"))?
        );
        fs::write(temporary_directory.join("session.json"), document)
            .map_err(|error| format!("Could not write session: {error}"))?;
        fs::create_dir(temporary_directory.join("candidates"))
            .map_err(|error| format!("Could not create candidate storage: {error}"))?;
        fs::create_dir(temporary_directory.join("generation-requests"))
            .map_err(|error| format!("Could not create generation request storage: {error}"))?;
        fs::rename(&temporary_directory, &final_directory)
            .map_err(|error| format!("Could not publish session: {error}"))
    })();

    if publish_result.is_err() {
        let _ = fs::remove_dir_all(&temporary_directory);
    }
    publish_result?;
    Ok(session)
}

fn validate_generation_request_id(request_id: &str) -> Result<(), String> {
    let valid_length = (3..=GENERATION_REQUEST_ID_MAX_LENGTH).contains(&request_id.len());
    let valid_characters = request_id
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || character == '-');
    let valid_start = request_id
        .chars()
        .next()
        .is_some_and(|character| character.is_ascii_alphanumeric());

    if valid_length && valid_characters && valid_start {
        Ok(())
    } else {
        Err("Invalid generation request id.".to_owned())
    }
}

fn compile_actor_prompt(brief: &ActorBrief) -> String {
    let kind = match &brief.kind {
        ActorKind::Mob => "mob",
        ActorKind::Npc => "npc",
    };
    format!(
        "Create one 32x32 pixel-art {kind} named \"{}\".\n{}\n\n\
Locked world contract:\n\
- The visible actor is 22-30px tall.\n\
- Feet remain anchored at (16, 28).\n\
- Lighting comes from the north-west.\n\
- Use selective shaded-side outline using the actor's own dark ramp.\n\
- Use at most 16 colors with hard transparent pixels.\n\
- Preserve at least 15 luma separation from walkable TileForge grounds.\n\
- Render a single down-facing approval concept. Do not create a full sheet yet.\n\
- No background, text, border, mockup, or soft antialiasing.\n\
- Return a candidate only. Only the user may approve final art.",
        brief.name, brief.description
    )
}

fn validate_generation_request(
    request: ConceptGenerationRequest,
) -> Result<ConceptGenerationRequest, String> {
    validate_generation_request_id(&request.id)?;
    validate_session_id(&request.session_id)?;
    if request.schema_version != 1 {
        return Err("Unsupported generation request document version.".to_owned());
    }
    if request.revision == 0 {
        return Err("Generation request revision must be at least 1.".to_owned());
    }
    if request.stage != "concept" || request.contract_id != CONTRACT_ID {
        return Err("Generation request contract is incompatible.".to_owned());
    }
    if request.prompt.is_empty() || request.prompt.chars().count() > 12_000 {
        return Err("Generation request prompt is invalid.".to_owned());
    }
    if !(1..=4).contains(&request.requested_candidates) {
        return Err("Generation request candidate count must be between 1 and 4.".to_owned());
    }
    if request.execution.mode != "connected-client-native-image-generation"
        || request.execution.additional_paid_services != "forbidden"
        || request.execution.api_credentials != "not-used"
    {
        return Err("Generation request cost boundary is incompatible.".to_owned());
    }
    if request.output.artifact != "concept-candidate"
        || request.output.direction != "down"
        || request.output.width != FRAME_WIDTH
        || request.output.height != FRAME_HEIGHT
        || request.output.mime_type != "image/png"
        || request.output.import_tool != "import_concept_candidate"
    {
        return Err("Generation request output contract is incompatible.".to_owned());
    }
    if !request.authority.agents_may_generate
        || !request.authority.agents_may_import
        || request.authority.agents_may_approve
        || request.authority.approval_owner != "user"
    {
        return Err("Generation request approval boundary is incompatible.".to_owned());
    }
    if request.lifecycle != "immutable-request" {
        return Err("Generation request lifecycle is incompatible.".to_owned());
    }
    Ok(request)
}

fn generation_requests_root(root: &Path, session_id: &str) -> Result<PathBuf, String> {
    validate_session_id(session_id)?;
    Ok(root
        .join("sessions")
        .join(session_id)
        .join("generation-requests"))
}

fn generation_request_directory(
    root: &Path,
    session_id: &str,
    request_id: &str,
) -> Result<PathBuf, String> {
    validate_generation_request_id(request_id)?;
    Ok(generation_requests_root(root, session_id)?.join(request_id))
}

fn read_generation_request(
    root: &Path,
    session_id: &str,
    request_id: &str,
) -> Result<ConceptGenerationRequest, String> {
    let session = read_session(root, session_id)?;
    let raw = fs::read_to_string(
        generation_request_directory(root, &session.id, request_id)?.join("request.json"),
    )
    .map_err(|error| format!("Could not read generation request: {error}"))?;
    let request: ConceptGenerationRequest = serde_json::from_str(&raw)
        .map_err(|error| format!("Invalid generation request document: {error}"))?;
    let request = validate_generation_request(request)?;
    if request.session_id != session.id || request.id != request_id {
        return Err("Generation request identity does not match its storage path.".to_owned());
    }
    Ok(request)
}

fn list_generation_requests_at(
    root: &Path,
    session_id: &str,
) -> Result<Vec<ConceptGenerationRequest>, String> {
    let session = read_session(root, session_id)?;
    let requests_root = generation_requests_root(root, &session.id)?;
    fs::create_dir_all(&requests_root)
        .map_err(|error| format!("Could not create generation request storage: {error}"))?;

    let mut requests = Vec::new();
    for entry in fs::read_dir(&requests_root)
        .map_err(|error| format!("Could not list generation requests: {error}"))?
    {
        let Ok(entry) = entry else {
            continue;
        };
        let file_name = entry.file_name();
        let Some(request_id) = file_name.to_str() else {
            continue;
        };
        if request_id.starts_with('.') || !entry.path().is_dir() {
            continue;
        }
        if let Ok(request) = read_generation_request(root, &session.id, request_id) {
            requests.push(request);
        }
    }
    requests.sort_by(|left, right| right.revision.cmp(&left.revision));
    Ok(requests)
}

#[allow(clippy::too_many_arguments)]
fn create_concept_generation_request_at(
    root: &Path,
    session_id: &str,
    requested_candidates: u32,
    timestamp: &str,
    id_suffix: &str,
    temporary_suffix: &str,
    forced_revision: Option<u32>,
) -> Result<ConceptGenerationRequest, String> {
    let session = read_session(root, session_id)?;
    let requests = list_generation_requests_at(root, &session.id)?;
    let revision = forced_revision.unwrap_or_else(|| {
        requests
            .iter()
            .map(|request| request.revision)
            .max()
            .unwrap_or(0)
            + 1
    });
    let timestamp_digits: String = timestamp
        .chars()
        .filter(char::is_ascii_digit)
        .take(14)
        .collect();
    let request = validate_generation_request(ConceptGenerationRequest {
        schema_version: 1,
        id: format!(
            "concept-gen-r{:04}-{}-{}",
            revision, timestamp_digits, id_suffix
        ),
        revision,
        session_id: session.id.clone(),
        stage: "concept".to_owned(),
        contract_id: CONTRACT_ID.to_owned(),
        created_at: timestamp.to_owned(),
        prompt: compile_actor_prompt(&session.brief),
        requested_candidates,
        execution: GenerationRequestExecution {
            mode: "connected-client-native-image-generation".to_owned(),
            additional_paid_services: "forbidden".to_owned(),
            api_credentials: "not-used".to_owned(),
        },
        output: GenerationRequestOutput {
            artifact: "concept-candidate".to_owned(),
            direction: "down".to_owned(),
            width: FRAME_WIDTH,
            height: FRAME_HEIGHT,
            mime_type: "image/png".to_owned(),
            import_tool: "import_concept_candidate".to_owned(),
        },
        authority: GenerationRequestAuthority {
            agents_may_generate: true,
            agents_may_import: true,
            agents_may_approve: false,
            approval_owner: "user".to_owned(),
        },
        lifecycle: "immutable-request".to_owned(),
    })?;

    let requests_root = generation_requests_root(root, &session.id)?;
    fs::create_dir_all(&requests_root)
        .map_err(|error| format!("Could not create generation request storage: {error}"))?;
    let final_directory = generation_request_directory(root, &session.id, &request.id)?;
    let temporary_directory =
        requests_root.join(format!(".{}.{}.tmp", request.id, temporary_suffix));
    fs::create_dir(&temporary_directory)
        .map_err(|error| format!("Could not stage generation request: {error}"))?;

    let publish_result = (|| -> Result<(), String> {
        let document = format!(
            "{}\n",
            serde_json::to_string_pretty(&request)
                .map_err(|error| format!("Could not serialize generation request: {error}"))?
        );
        fs::write(temporary_directory.join("request.json"), document)
            .map_err(|error| format!("Could not write generation request: {error}"))?;
        fs::rename(&temporary_directory, &final_directory)
            .map_err(|error| format!("Could not publish generation request: {error}"))
    })();

    if publish_result.is_err() {
        let _ = fs::remove_dir_all(&temporary_directory);
    }
    publish_result?;
    Ok(request)
}

fn validate_provenance(mut provenance: CandidateProvenance) -> Result<CandidateProvenance, String> {
    provenance.original_filename = provenance
        .original_filename
        .map(|value| value.trim().to_owned());
    provenance.provider = provenance.provider.map(|value| value.trim().to_owned());
    provenance.model = provenance.model.map(|value| value.trim().to_owned());

    for (label, value, maximum) in [
        (
            "Original filename",
            provenance.original_filename.as_deref(),
            255,
        ),
        ("Provider", provenance.provider.as_deref(), 120),
        ("Model", provenance.model.as_deref(), 120),
    ] {
        if let Some(value) = value {
            if value.is_empty() {
                return Err(format!("{label} cannot be empty."));
            }
            if value.chars().count() > maximum {
                return Err(format!("{label} is too long."));
            }
        }
    }

    if matches!(provenance.source, CandidateSource::Generated) && provenance.provider.is_none() {
        return Err("Generated candidates require provider provenance.".to_owned());
    }

    Ok(provenance)
}

fn validate_candidate(candidate: ConceptCandidate) -> Result<ConceptCandidate, String> {
    validate_candidate_id(&candidate.id)?;
    validate_session_id(&candidate.session_id)?;
    if candidate.schema_version != 1 {
        return Err("Unsupported candidate document version.".to_owned());
    }
    if candidate.revision == 0 {
        return Err("Candidate revision must be at least 1.".to_owned());
    }
    if candidate.stage != "concept" || candidate.direction != "down" {
        return Err("Candidate is not a down-facing Concept document.".to_owned());
    }
    if candidate.contract_id != CONTRACT_ID {
        return Err("Candidate contract id is incompatible.".to_owned());
    }
    if candidate.source_file != "source.png" || candidate.mime_type != "image/png" {
        return Err("Candidate source file contract is incompatible.".to_owned());
    }
    if candidate.sha256.len() != 64
        || !candidate
            .sha256
            .chars()
            .all(|character| character.is_ascii_hexdigit() && !character.is_ascii_uppercase())
    {
        return Err("Candidate SHA-256 is invalid.".to_owned());
    }
    if candidate.byte_length == 0 || candidate.byte_length > CONCEPT_PNG_MAX_BYTES {
        return Err("Candidate byte length is invalid.".to_owned());
    }
    if candidate.width != 32 || candidate.height != 32 {
        return Err("Candidate dimensions are incompatible.".to_owned());
    }
    if candidate.intake_validation.file_type != "pass"
        || candidate.intake_validation.dimensions != "pass"
        || candidate.intake_validation.alpha_channel != "pass"
    {
        return Err("Candidate intake evidence is incomplete.".to_owned());
    }
    if candidate.review_status != "unreviewed" {
        return Err("Candidate creation cannot imply visual approval.".to_owned());
    }
    validate_provenance(candidate.provenance.clone())?;
    Ok(candidate)
}

fn candidate_root(root: &Path, session_id: &str) -> Result<PathBuf, String> {
    validate_session_id(session_id)?;
    Ok(root.join("sessions").join(session_id).join("candidates"))
}

fn candidate_directory(
    root: &Path,
    session_id: &str,
    candidate_id: &str,
) -> Result<PathBuf, String> {
    validate_candidate_id(candidate_id)?;
    Ok(candidate_root(root, session_id)?.join(candidate_id))
}

fn read_candidate(
    root: &Path,
    session_id: &str,
    candidate_id: &str,
) -> Result<ConceptCandidate, String> {
    let session = read_session(root, session_id)?;
    let raw = fs::read_to_string(
        candidate_directory(root, &session.id, candidate_id)?.join("candidate.json"),
    )
    .map_err(|error| format!("Could not read candidate: {error}"))?;
    let candidate: ConceptCandidate = serde_json::from_str(&raw)
        .map_err(|error| format!("Invalid candidate document: {error}"))?;
    let candidate = validate_candidate(candidate)?;
    if candidate.session_id != session.id || candidate.id != candidate_id {
        return Err("Candidate identity does not match its storage path.".to_owned());
    }
    Ok(candidate)
}

fn list_candidates_at(root: &Path, session_id: &str) -> Result<Vec<ConceptCandidate>, String> {
    let session = read_session(root, session_id)?;
    let candidates_root = candidate_root(root, &session.id)?;
    fs::create_dir_all(&candidates_root)
        .map_err(|error| format!("Could not create candidate storage: {error}"))?;

    let mut candidates = Vec::new();
    for entry in fs::read_dir(&candidates_root)
        .map_err(|error| format!("Could not list candidates: {error}"))?
    {
        let Ok(entry) = entry else {
            continue;
        };
        let file_name = entry.file_name();
        let Some(candidate_id) = file_name.to_str() else {
            continue;
        };
        if candidate_id.starts_with('.') || !entry.path().is_dir() {
            continue;
        }
        if let Ok(candidate) = read_candidate(root, &session.id, candidate_id) {
            candidates.push(candidate);
        }
    }
    candidates.sort_by(|left, right| right.revision.cmp(&left.revision));
    Ok(candidates)
}

fn decode_png_rgba(png_bytes: &[u8]) -> Result<DecodedPng, String> {
    if png_bytes.is_empty() {
        return Err("PNG file is empty.".to_owned());
    }
    if png_bytes.len() > CONCEPT_PNG_MAX_BYTES {
        return Err("PNG file exceeds the 1 MiB intake limit.".to_owned());
    }

    let mut decoder = png::Decoder::new(Cursor::new(png_bytes));
    decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
    let mut reader = decoder
        .read_info()
        .map_err(|_| "File is not a valid PNG.".to_owned())?;
    if reader.info().animation_control.is_some() {
        return Err("Animated PNG files are not supported for Concept intake.".to_owned());
    }
    let mut decoded = vec![0; reader.output_buffer_size()];
    let output = reader
        .next_frame(&mut decoded)
        .map_err(|_| "File is not a valid PNG.".to_owned())?;
    let decoded = &decoded[..output.buffer_size()];
    let pixels = match output.color_type {
        png::ColorType::Grayscale => decoded
            .iter()
            .map(|gray| [*gray, *gray, *gray, 255])
            .collect(),
        png::ColorType::GrayscaleAlpha => decoded
            .chunks_exact(2)
            .map(|pixel| [pixel[0], pixel[0], pixel[0], pixel[1]])
            .collect(),
        png::ColorType::Rgb => decoded
            .chunks_exact(3)
            .map(|pixel| [pixel[0], pixel[1], pixel[2], 255])
            .collect(),
        png::ColorType::Rgba => decoded
            .chunks_exact(4)
            .map(|pixel| [pixel[0], pixel[1], pixel[2], pixel[3]])
            .collect(),
        png::ColorType::Indexed => {
            return Err("File is not a valid expanded PNG.".to_owned());
        }
    };

    Ok(DecodedPng {
        width: output.width,
        height: output.height,
        pixels,
    })
}

fn validate_concept_png(png_bytes: &[u8]) -> Result<(u32, u32, String), String> {
    let decoded = decode_png_rgba(png_bytes)?;
    if decoded.width != FRAME_WIDTH || decoded.height != FRAME_HEIGHT {
        return Err("Concept PNG must be exactly 32 x 32 pixels.".to_owned());
    }
    let has_transparency = decoded.pixels.iter().any(|pixel| pixel[3] < 255);
    if !has_transparency {
        return Err("Concept PNG must contain an alpha channel with transparency.".to_owned());
    }

    Ok((
        decoded.width,
        decoded.height,
        format!("{:x}", Sha256::digest(png_bytes)),
    ))
}

#[allow(clippy::too_many_arguments)]
fn create_concept_candidate_at(
    root: &Path,
    session_id: &str,
    png_bytes: &[u8],
    provenance: CandidateProvenance,
    timestamp: &str,
    id_suffix: &str,
    temporary_suffix: &str,
    forced_revision: Option<u32>,
) -> Result<ConceptCandidate, String> {
    let session = read_session(root, session_id)?;
    let provenance = validate_provenance(provenance)?;
    let (width, height, sha256) = validate_concept_png(png_bytes)?;
    let candidates = list_candidates_at(root, &session.id)?;
    let revision = forced_revision.unwrap_or_else(|| {
        candidates
            .iter()
            .map(|candidate| candidate.revision)
            .max()
            .unwrap_or(0)
            + 1
    });
    let timestamp_digits: String = timestamp
        .chars()
        .filter(char::is_ascii_digit)
        .take(14)
        .collect();
    let candidate = validate_candidate(ConceptCandidate {
        schema_version: 1,
        id: format!(
            "concept-r{:04}-{}-{}",
            revision, timestamp_digits, id_suffix
        ),
        revision,
        session_id: session.id.clone(),
        stage: "concept".to_owned(),
        direction: "down".to_owned(),
        contract_id: CONTRACT_ID.to_owned(),
        source_file: "source.png".to_owned(),
        mime_type: "image/png".to_owned(),
        sha256,
        byte_length: png_bytes.len(),
        width,
        height,
        created_at: timestamp.to_owned(),
        provenance,
        intake_validation: CandidateIntakeValidation {
            file_type: "pass".to_owned(),
            dimensions: "pass".to_owned(),
            alpha_channel: "pass".to_owned(),
        },
        review_status: "unreviewed".to_owned(),
    })?;

    let candidates_root = candidate_root(root, &session.id)?;
    fs::create_dir_all(&candidates_root)
        .map_err(|error| format!("Could not create candidate storage: {error}"))?;
    let final_directory = candidate_directory(root, &session.id, &candidate.id)?;
    let temporary_directory =
        candidates_root.join(format!(".{}.{}.tmp", candidate.id, temporary_suffix));
    fs::create_dir(&temporary_directory)
        .map_err(|error| format!("Could not stage candidate: {error}"))?;

    let publish_result = (|| -> Result<(), String> {
        fs::write(temporary_directory.join("source.png"), png_bytes)
            .map_err(|error| format!("Could not write candidate PNG: {error}"))?;
        let document = format!(
            "{}\n",
            serde_json::to_string_pretty(&candidate)
                .map_err(|error| format!("Could not serialize candidate: {error}"))?
        );
        fs::write(temporary_directory.join("candidate.json"), document)
            .map_err(|error| format!("Could not write candidate document: {error}"))?;
        fs::rename(&temporary_directory, &final_directory)
            .map_err(|error| format!("Could not publish candidate: {error}"))
    })();

    if publish_result.is_err() {
        let _ = fs::remove_dir_all(&temporary_directory);
    }
    publish_result?;
    Ok(candidate)
}

fn read_candidate_payload(
    root: &Path,
    session_id: &str,
    candidate_id: &str,
) -> Result<ConceptCandidatePayload, String> {
    let candidate = read_candidate(root, session_id, candidate_id)?;
    let png_bytes =
        fs::read(candidate_directory(root, session_id, candidate_id)?.join(&candidate.source_file))
            .map_err(|error| format!("Could not read candidate PNG: {error}"))?;
    if png_bytes.len() != candidate.byte_length
        || format!("{:x}", Sha256::digest(&png_bytes)) != candidate.sha256
    {
        return Err("Candidate source bytes no longer match immutable provenance.".to_owned());
    }
    Ok(ConceptCandidatePayload {
        candidate,
        png_bytes,
    })
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .chars()
            .all(|character| character.is_ascii_hexdigit() && !character.is_ascii_uppercase())
}

fn validate_turnaround(candidate: TurnaroundCandidate) -> Result<TurnaroundCandidate, String> {
    validate_candidate_id(&candidate.id)?;
    validate_session_id(&candidate.session_id)?;
    validate_candidate_id(&candidate.source_selection.candidate_id)?;
    if candidate.schema_version != 1 {
        return Err("Unsupported Turnaround document version.".to_owned());
    }
    if candidate.revision == 0 {
        return Err("Turnaround revision must be at least 1.".to_owned());
    }
    if candidate.stage != "turnaround" || candidate.contract_id != CONTRACT_ID {
        return Err("Turnaround stage or contract id is incompatible.".to_owned());
    }
    if candidate.source_selection.selected_by != "user"
        || candidate.source_selection.selected_at.is_empty()
        || !valid_sha256(&candidate.source_selection.candidate_sha256)
    {
        return Err("Turnaround source selection is invalid.".to_owned());
    }
    if candidate.directions.len() != TURNAROUND_DIRECTIONS.len() {
        return Err("Turnaround must contain four direction sources.".to_owned());
    }
    for (source, expected_direction) in candidate.directions.iter().zip(TURNAROUND_DIRECTIONS) {
        if source.direction != expected_direction
            || source.source_file != format!("{}.png", expected_direction.as_str())
        {
            return Err("Turnaround directions must use canonical order and filenames.".to_owned());
        }
        if !valid_sha256(&source.sha256)
            || source.byte_length == 0
            || source.byte_length > CONCEPT_PNG_MAX_BYTES
            || source.width != FRAME_WIDTH
            || source.height != FRAME_HEIGHT
        {
            return Err("Turnaround direction evidence is invalid.".to_owned());
        }
    }
    if candidate.directions[0].sha256 != candidate.source_selection.candidate_sha256 {
        return Err("Down view must preserve the selected Concept bytes.".to_owned());
    }
    if candidate.created_at.is_empty() {
        return Err("Turnaround creation time is required.".to_owned());
    }
    validate_provenance(candidate.provenance.clone())?;
    if candidate.review_status != "unreviewed" {
        return Err("Turnaround creation cannot imply visual approval.".to_owned());
    }
    if candidate.identity_judgment.status != VisualJudgmentStatus::NotAssessed
        || candidate.identity_judgment.authority != "user"
        || candidate.identity_judgment.message.is_empty()
    {
        return Err("Turnaround crossed the user-owned identity gate.".to_owned());
    }
    Ok(candidate)
}

fn turnaround_root(root: &Path, session_id: &str) -> Result<PathBuf, String> {
    validate_session_id(session_id)?;
    Ok(root.join("sessions").join(session_id).join("turnarounds"))
}

fn turnaround_directory(
    root: &Path,
    session_id: &str,
    turnaround_id: &str,
) -> Result<PathBuf, String> {
    validate_candidate_id(turnaround_id)?;
    Ok(turnaround_root(root, session_id)?.join(turnaround_id))
}

fn read_turnaround(
    root: &Path,
    session_id: &str,
    turnaround_id: &str,
) -> Result<TurnaroundCandidate, String> {
    let session = read_session(root, session_id)?;
    let raw = fs::read_to_string(
        turnaround_directory(root, &session.id, turnaround_id)?.join("turnaround.json"),
    )
    .map_err(|error| format!("Could not read Turnaround: {error}"))?;
    let candidate: TurnaroundCandidate = serde_json::from_str(&raw)
        .map_err(|error| format!("Invalid Turnaround document: {error}"))?;
    let candidate = validate_turnaround(candidate)?;
    if candidate.session_id != session.id || candidate.id != turnaround_id {
        return Err("Turnaround identity does not match its storage path.".to_owned());
    }
    Ok(candidate)
}

fn list_turnarounds_at(root: &Path, session_id: &str) -> Result<Vec<TurnaroundCandidate>, String> {
    let session = read_session(root, session_id)?;
    let candidates_root = turnaround_root(root, &session.id)?;
    fs::create_dir_all(&candidates_root)
        .map_err(|error| format!("Could not create Turnaround storage: {error}"))?;

    let mut candidates = Vec::new();
    for entry in fs::read_dir(&candidates_root)
        .map_err(|error| format!("Could not list Turnarounds: {error}"))?
    {
        let Ok(entry) = entry else {
            continue;
        };
        let file_name = entry.file_name();
        let Some(turnaround_id) = file_name.to_str() else {
            continue;
        };
        if turnaround_id.starts_with('.') || !entry.path().is_dir() {
            continue;
        }
        if let Ok(candidate) = read_turnaround(root, &session.id, turnaround_id) {
            candidates.push(candidate);
        }
    }
    candidates.sort_by(|left, right| right.revision.cmp(&left.revision));
    Ok(candidates)
}

#[allow(clippy::too_many_arguments)]
fn create_turnaround_candidate_at(
    root: &Path,
    session_id: &str,
    source_concept_id: &str,
    png_bytes: &TurnaroundPngBytes,
    provenance: CandidateProvenance,
    timestamp: &str,
    id_suffix: &str,
    temporary_suffix: &str,
    forced_revision: Option<u32>,
) -> Result<TurnaroundCandidate, String> {
    let session = read_session(root, session_id)?;
    let source_concept = read_candidate_payload(root, &session.id, source_concept_id)?;
    let provenance = validate_provenance(provenance)?;
    if png_bytes.down != source_concept.png_bytes {
        return Err(
            "Down view must preserve the exact user-selected Concept PNG bytes.".to_owned(),
        );
    }

    let mut directions = Vec::new();
    for direction in TURNAROUND_DIRECTIONS {
        let bytes = png_bytes.get(direction);
        let (width, height, sha256) = validate_concept_png(bytes)?;
        directions.push(TurnaroundDirectionSource {
            direction,
            source_file: format!("{}.png", direction.as_str()),
            sha256,
            byte_length: bytes.len(),
            width,
            height,
        });
    }

    let candidates = list_turnarounds_at(root, &session.id)?;
    let revision = forced_revision.unwrap_or_else(|| {
        candidates
            .iter()
            .map(|candidate| candidate.revision)
            .max()
            .unwrap_or(0)
            + 1
    });
    let timestamp_digits: String = timestamp
        .chars()
        .filter(char::is_ascii_digit)
        .take(14)
        .collect();
    let candidate = validate_turnaround(TurnaroundCandidate {
        schema_version: 1,
        id: format!(
            "turnaround-r{:04}-{}-{}",
            revision, timestamp_digits, id_suffix
        ),
        revision,
        session_id: session.id.clone(),
        stage: "turnaround".to_owned(),
        contract_id: CONTRACT_ID.to_owned(),
        source_selection: TurnaroundSourceSelection {
            candidate_id: source_concept.candidate.id,
            candidate_sha256: source_concept.candidate.sha256,
            selected_by: "user".to_owned(),
            selected_at: timestamp.to_owned(),
        },
        directions,
        created_at: timestamp.to_owned(),
        provenance,
        review_status: "unreviewed".to_owned(),
        identity_judgment: VisualJudgment {
            status: VisualJudgmentStatus::NotAssessed,
            authority: "user".to_owned(),
            message: "Only the user can accept identity consistency across turnaround views."
                .to_owned(),
        },
    })?;

    let candidates_root = turnaround_root(root, &session.id)?;
    fs::create_dir_all(&candidates_root)
        .map_err(|error| format!("Could not create Turnaround storage: {error}"))?;
    let final_directory = turnaround_directory(root, &session.id, &candidate.id)?;
    let temporary_directory =
        candidates_root.join(format!(".{}.{}.tmp", candidate.id, temporary_suffix));
    fs::create_dir(&temporary_directory)
        .map_err(|error| format!("Could not stage Turnaround: {error}"))?;

    let publish_result = (|| -> Result<(), String> {
        for direction in TURNAROUND_DIRECTIONS {
            fs::write(
                temporary_directory.join(format!("{}.png", direction.as_str())),
                png_bytes.get(direction),
            )
            .map_err(|error| {
                format!(
                    "Could not write {} Turnaround PNG: {error}",
                    direction.as_str()
                )
            })?;
        }
        let document = format!(
            "{}\n",
            serde_json::to_string_pretty(&candidate)
                .map_err(|error| format!("Could not serialize Turnaround: {error}"))?
        );
        fs::write(temporary_directory.join("turnaround.json"), document)
            .map_err(|error| format!("Could not write Turnaround document: {error}"))?;
        fs::rename(&temporary_directory, &final_directory)
            .map_err(|error| format!("Could not publish Turnaround: {error}"))
    })();

    if publish_result.is_err() {
        let _ = fs::remove_dir_all(&temporary_directory);
    }
    publish_result?;
    Ok(candidate)
}

fn read_turnaround_payload(
    root: &Path,
    session_id: &str,
    turnaround_id: &str,
) -> Result<TurnaroundCandidatePayload, String> {
    let candidate = read_turnaround(root, session_id, turnaround_id)?;
    let directory = turnaround_directory(root, session_id, turnaround_id)?;
    let png_bytes = TurnaroundPngBytes {
        down: fs::read(directory.join("down.png"))
            .map_err(|error| format!("Could not read down Turnaround PNG: {error}"))?,
        right: fs::read(directory.join("right.png"))
            .map_err(|error| format!("Could not read right Turnaround PNG: {error}"))?,
        up: fs::read(directory.join("up.png"))
            .map_err(|error| format!("Could not read up Turnaround PNG: {error}"))?,
        left: fs::read(directory.join("left.png"))
            .map_err(|error| format!("Could not read left Turnaround PNG: {error}"))?,
    };
    for source in &candidate.directions {
        let bytes = png_bytes.get(source.direction);
        if bytes.len() != source.byte_length
            || format!("{:x}", Sha256::digest(bytes)) != source.sha256
        {
            return Err(format!(
                "{} source bytes no longer match immutable provenance.",
                source.direction.as_str()
            ));
        }
    }
    Ok(TurnaroundCandidatePayload {
        candidate,
        png_bytes,
    })
}

fn validate_turnaround_report(
    report: TurnaroundValidationReport,
) -> Result<TurnaroundValidationReport, String> {
    validate_candidate_id(&report.turnaround_id)?;
    if report.schema_version != 1
        || report.validator_id != TURNAROUND_VALIDATOR_ID
        || report.contract_id != CONTRACT_ID
    {
        return Err("Unsupported Turnaround validation report.".to_owned());
    }
    if report.directions.len() != TURNAROUND_DIRECTIONS.len()
        || report
            .directions
            .iter()
            .zip(TURNAROUND_DIRECTIONS)
            .any(|(direction, expected)| direction.direction != expected)
    {
        return Err("Turnaround validation directions must use canonical order.".to_owned());
    }
    let mut counted = ValidationSummary {
        pass: 0,
        fail: 0,
        not_assessed: 0,
    };
    for direction in &report.directions {
        validate_structural_report(direction.report.clone())?;
        if direction.report.candidate_id != report.turnaround_id
            || direction.report.contract_id != report.contract_id
        {
            return Err("Direction validation identity does not match Turnaround.".to_owned());
        }
        counted.pass += direction.report.summary.pass;
        counted.fail += direction.report.summary.fail;
        counted.not_assessed += direction.report.summary.not_assessed;
    }
    if counted != report.summary {
        return Err("Turnaround validation summary does not match direction reports.".to_owned());
    }
    if report.identity_judgment.status != VisualJudgmentStatus::NotAssessed
        || report.identity_judgment.authority != "user"
        || report.identity_judgment.message.is_empty()
    {
        return Err("Turnaround validation crossed the user-owned identity gate.".to_owned());
    }
    Ok(report)
}

fn validate_turnaround_pngs(
    payload: &TurnaroundCandidatePayload,
) -> Result<TurnaroundValidationReport, String> {
    let mut directions = Vec::new();
    let mut summary = ValidationSummary {
        pass: 0,
        fail: 0,
        not_assessed: 0,
    };
    for source in &payload.candidate.directions {
        let report = validate_png_structural_evidence(
            &payload.candidate.id,
            &source.sha256,
            source.byte_length,
            &payload.candidate.contract_id,
            payload.png_bytes.get(source.direction),
            StructuralContactMode::ExactAnchor,
        )?;
        summary.pass += report.summary.pass;
        summary.fail += report.summary.fail;
        summary.not_assessed += report.summary.not_assessed;
        directions.push(TurnaroundDirectionReport {
            direction: source.direction,
            report,
        });
    }
    validate_turnaround_report(TurnaroundValidationReport {
        schema_version: 1,
        validator_id: TURNAROUND_VALIDATOR_ID.to_owned(),
        turnaround_id: payload.candidate.id.clone(),
        contract_id: payload.candidate.contract_id.clone(),
        directions,
        summary,
        identity_judgment: payload.candidate.identity_judgment.clone(),
    })
}

fn validate_walk_cycle(candidate: WalkCycleCandidate) -> Result<WalkCycleCandidate, String> {
    validate_candidate_id(&candidate.id)?;
    validate_session_id(&candidate.session_id)?;
    validate_candidate_id(&candidate.source_turnaround.turnaround_id)?;
    if candidate.schema_version != 1 {
        return Err("Unsupported Walk Cycle document version.".to_owned());
    }
    if candidate.revision == 0 {
        return Err("Walk Cycle revision must be at least 1.".to_owned());
    }
    if candidate.stage != "animate" || candidate.contract_id != CONTRACT_ID {
        return Err("Walk Cycle stage or contract id is incompatible.".to_owned());
    }
    if candidate.source_turnaround.accepted_by != "user"
        || candidate.source_turnaround.accepted_at.is_empty()
    {
        return Err("Walk Cycle source Turnaround acceptance is invalid.".to_owned());
    }
    if candidate.source_turnaround.direction_sources.len() != TURNAROUND_DIRECTIONS.len() {
        return Err("Walk Cycle source receipt must contain four directions.".to_owned());
    }
    for (source, expected_direction) in candidate
        .source_turnaround
        .direction_sources
        .iter()
        .zip(TURNAROUND_DIRECTIONS)
    {
        if source.direction != expected_direction
            || !valid_sha256(&source.sha256)
            || source.byte_length == 0
            || source.byte_length > CONCEPT_PNG_MAX_BYTES
        {
            return Err("Walk Cycle source Turnaround evidence is invalid.".to_owned());
        }
    }
    if candidate.clip != "walk"
        || candidate.frames_per_direction != WALK_CYCLE_FRAMES_PER_DIRECTION
        || candidate.frame_duration_ms != WALK_CYCLE_FRAME_DURATION_MS
    {
        return Err("Walk Cycle clip or timing is incompatible.".to_owned());
    }
    if candidate.frames.len() != TURNAROUND_DIRECTIONS.len() * WALK_CYCLE_FRAMES_PER_DIRECTION {
        return Err("Walk Cycle must contain sixteen frame sources.".to_owned());
    }
    for (direction_index, direction) in TURNAROUND_DIRECTIONS.iter().enumerate() {
        let accepted_source = &candidate.source_turnaround.direction_sources[direction_index];
        for frame_index in 0..WALK_CYCLE_FRAMES_PER_DIRECTION {
            let flat_index = direction_index * WALK_CYCLE_FRAMES_PER_DIRECTION + frame_index;
            let frame = &candidate.frames[flat_index];
            if frame.direction != *direction
                || frame.frame_index != frame_index
                || frame.source_file != format!("{}-{frame_index}.png", direction.as_str())
            {
                return Err(
                    "Walk Cycle frames must use canonical direction, index, and filenames."
                        .to_owned(),
                );
            }
            if !valid_sha256(&frame.sha256)
                || frame.byte_length == 0
                || frame.byte_length > CONCEPT_PNG_MAX_BYTES
                || frame.width != FRAME_WIDTH
                || frame.height != FRAME_HEIGHT
            {
                return Err("Walk Cycle frame evidence is invalid.".to_owned());
            }
            if frame_index == 0
                && (frame.sha256 != accepted_source.sha256
                    || frame.byte_length != accepted_source.byte_length)
            {
                return Err(
                    "Frame 0 must preserve the accepted Turnaround direction bytes.".to_owned(),
                );
            }
        }
    }
    if candidate.created_at.is_empty() {
        return Err("Walk Cycle creation time is required.".to_owned());
    }
    validate_provenance(candidate.provenance.clone())?;
    if candidate.review_status != "unreviewed" {
        return Err("Walk Cycle creation cannot imply motion approval.".to_owned());
    }
    if candidate.motion_judgment.status != VisualJudgmentStatus::NotAssessed
        || candidate.motion_judgment.authority != "user"
        || candidate.motion_judgment.message.is_empty()
    {
        return Err("Walk Cycle crossed the user-owned motion gate.".to_owned());
    }
    Ok(candidate)
}

fn walk_cycle_root(root: &Path, session_id: &str) -> Result<PathBuf, String> {
    validate_session_id(session_id)?;
    Ok(root.join("sessions").join(session_id).join("walk-cycles"))
}

fn walk_cycle_directory(
    root: &Path,
    session_id: &str,
    walk_cycle_id: &str,
) -> Result<PathBuf, String> {
    validate_candidate_id(walk_cycle_id)?;
    Ok(walk_cycle_root(root, session_id)?.join(walk_cycle_id))
}

fn read_walk_cycle(
    root: &Path,
    session_id: &str,
    walk_cycle_id: &str,
) -> Result<WalkCycleCandidate, String> {
    let session = read_session(root, session_id)?;
    let raw = fs::read_to_string(
        walk_cycle_directory(root, &session.id, walk_cycle_id)?.join("walk-cycle.json"),
    )
    .map_err(|error| format!("Could not read Walk Cycle: {error}"))?;
    let candidate: WalkCycleCandidate = serde_json::from_str(&raw)
        .map_err(|error| format!("Invalid Walk Cycle document: {error}"))?;
    let candidate = validate_walk_cycle(candidate)?;
    if candidate.session_id != session.id || candidate.id != walk_cycle_id {
        return Err("Walk Cycle identity does not match its storage path.".to_owned());
    }
    Ok(candidate)
}

fn list_walk_cycles_at(root: &Path, session_id: &str) -> Result<Vec<WalkCycleCandidate>, String> {
    let session = read_session(root, session_id)?;
    let candidates_root = walk_cycle_root(root, &session.id)?;
    fs::create_dir_all(&candidates_root)
        .map_err(|error| format!("Could not create Walk Cycle storage: {error}"))?;

    let mut candidates = Vec::new();
    for entry in fs::read_dir(&candidates_root)
        .map_err(|error| format!("Could not list Walk Cycles: {error}"))?
    {
        let Ok(entry) = entry else {
            continue;
        };
        let file_name = entry.file_name();
        let Some(walk_cycle_id) = file_name.to_str() else {
            continue;
        };
        if walk_cycle_id.starts_with('.') || !entry.path().is_dir() {
            continue;
        }
        if let Ok(candidate) = read_walk_cycle(root, &session.id, walk_cycle_id) {
            candidates.push(candidate);
        }
    }
    candidates.sort_by(|left, right| right.revision.cmp(&left.revision));
    Ok(candidates)
}

#[allow(clippy::too_many_arguments)]
fn create_walk_cycle_candidate_at(
    root: &Path,
    session_id: &str,
    source_turnaround_id: &str,
    png_bytes: &WalkCyclePngBytes,
    provenance: CandidateProvenance,
    timestamp: &str,
    id_suffix: &str,
    temporary_suffix: &str,
    forced_revision: Option<u32>,
) -> Result<WalkCycleCandidate, String> {
    let session = read_session(root, session_id)?;
    let source_turnaround = read_turnaround_payload(root, &session.id, source_turnaround_id)?;
    let provenance = validate_provenance(provenance)?;

    let mut frames = Vec::new();
    for direction in TURNAROUND_DIRECTIONS {
        let direction_frames = png_bytes.direction(direction);
        if direction_frames.len() != WALK_CYCLE_FRAMES_PER_DIRECTION {
            return Err(format!(
                "Walk Cycle {} must contain exactly four frames.",
                direction.as_str()
            ));
        }
        if direction_frames[0] != source_turnaround.png_bytes.get(direction) {
            return Err(format!(
                "Frame 0 for {} must preserve the exact user-accepted Turnaround PNG bytes.",
                direction.as_str()
            ));
        }
        for (frame_index, bytes) in direction_frames.iter().enumerate() {
            let (width, height, sha256) = validate_concept_png(bytes)?;
            frames.push(WalkCycleFrameSource {
                direction,
                frame_index,
                source_file: format!("{}-{frame_index}.png", direction.as_str()),
                sha256,
                byte_length: bytes.len(),
                width,
                height,
            });
        }
    }

    let candidates = list_walk_cycles_at(root, &session.id)?;
    let revision = forced_revision.unwrap_or_else(|| {
        candidates
            .iter()
            .map(|candidate| candidate.revision)
            .max()
            .unwrap_or(0)
            + 1
    });
    let timestamp_digits: String = timestamp
        .chars()
        .filter(char::is_ascii_digit)
        .take(14)
        .collect();
    let candidate = validate_walk_cycle(WalkCycleCandidate {
        schema_version: 1,
        id: format!(
            "walk-cycle-r{:04}-{}-{}",
            revision, timestamp_digits, id_suffix
        ),
        revision,
        session_id: session.id.clone(),
        stage: "animate".to_owned(),
        contract_id: CONTRACT_ID.to_owned(),
        source_turnaround: WalkCycleSourceTurnaround {
            turnaround_id: source_turnaround.candidate.id,
            direction_sources: source_turnaround
                .candidate
                .directions
                .iter()
                .map(|source| WalkCycleAcceptedDirectionSource {
                    direction: source.direction,
                    sha256: source.sha256.clone(),
                    byte_length: source.byte_length,
                })
                .collect(),
            accepted_by: "user".to_owned(),
            accepted_at: timestamp.to_owned(),
        },
        clip: "walk".to_owned(),
        frames_per_direction: WALK_CYCLE_FRAMES_PER_DIRECTION,
        frame_duration_ms: WALK_CYCLE_FRAME_DURATION_MS,
        frames,
        created_at: timestamp.to_owned(),
        provenance,
        review_status: "unreviewed".to_owned(),
        motion_judgment: VisualJudgment {
            status: VisualJudgmentStatus::NotAssessed,
            authority: "user".to_owned(),
            message: "Only the user can accept Walk Cycle motion and readability.".to_owned(),
        },
    })?;

    let candidates_root = walk_cycle_root(root, &session.id)?;
    fs::create_dir_all(&candidates_root)
        .map_err(|error| format!("Could not create Walk Cycle storage: {error}"))?;
    let final_directory = walk_cycle_directory(root, &session.id, &candidate.id)?;
    let temporary_directory =
        candidates_root.join(format!(".{}.{}.tmp", candidate.id, temporary_suffix));
    fs::create_dir(&temporary_directory)
        .map_err(|error| format!("Could not stage Walk Cycle: {error}"))?;

    let publish_result = (|| -> Result<(), String> {
        for frame in &candidate.frames {
            let bytes = png_bytes
                .get(frame.direction, frame.frame_index)
                .ok_or_else(|| "Walk Cycle frame bytes are incomplete.".to_owned())?;
            fs::write(temporary_directory.join(&frame.source_file), bytes).map_err(|error| {
                format!(
                    "Could not write {} frame {} PNG: {error}",
                    frame.direction.as_str(),
                    frame.frame_index
                )
            })?;
        }
        let document = format!(
            "{}\n",
            serde_json::to_string_pretty(&candidate)
                .map_err(|error| format!("Could not serialize Walk Cycle: {error}"))?
        );
        fs::write(temporary_directory.join("walk-cycle.json"), document)
            .map_err(|error| format!("Could not write Walk Cycle document: {error}"))?;
        fs::rename(&temporary_directory, &final_directory)
            .map_err(|error| format!("Could not publish Walk Cycle: {error}"))
    })();

    if publish_result.is_err() {
        let _ = fs::remove_dir_all(&temporary_directory);
    }
    publish_result?;
    Ok(candidate)
}

fn read_walk_cycle_payload(
    root: &Path,
    session_id: &str,
    walk_cycle_id: &str,
) -> Result<WalkCycleCandidatePayload, String> {
    let candidate = read_walk_cycle(root, session_id, walk_cycle_id)?;
    let directory = walk_cycle_directory(root, session_id, walk_cycle_id)?;
    let mut png_bytes = WalkCyclePngBytes {
        down: Vec::new(),
        right: Vec::new(),
        up: Vec::new(),
        left: Vec::new(),
    };
    for source in &candidate.frames {
        let bytes = fs::read(directory.join(&source.source_file)).map_err(|error| {
            format!(
                "Could not read {} frame {} PNG: {error}",
                source.direction.as_str(),
                source.frame_index
            )
        })?;
        if bytes.len() != source.byte_length
            || format!("{:x}", Sha256::digest(&bytes)) != source.sha256
        {
            return Err(format!(
                "{} frame {} bytes no longer match immutable provenance.",
                source.direction.as_str(),
                source.frame_index
            ));
        }
        png_bytes.direction_mut(source.direction).push(bytes);
    }
    for direction in TURNAROUND_DIRECTIONS {
        if png_bytes.direction(direction).len() != WALK_CYCLE_FRAMES_PER_DIRECTION {
            return Err("Walk Cycle frame bytes are incomplete.".to_owned());
        }
    }
    Ok(WalkCycleCandidatePayload {
        candidate,
        png_bytes,
    })
}

fn validate_walk_cycle_report(
    report: WalkCycleValidationReport,
) -> Result<WalkCycleValidationReport, String> {
    validate_candidate_id(&report.walk_cycle_id)?;
    if report.schema_version != 1
        || report.validator_id != WALK_CYCLE_VALIDATOR_ID
        || report.contract_id != CONTRACT_ID
    {
        return Err("Unsupported Walk Cycle validation report.".to_owned());
    }
    if report.frames.len() != TURNAROUND_DIRECTIONS.len() * WALK_CYCLE_FRAMES_PER_DIRECTION {
        return Err("Walk Cycle validation must contain sixteen frame reports.".to_owned());
    }
    let mut counted = ValidationSummary {
        pass: 0,
        fail: 0,
        not_assessed: 0,
    };
    for (direction_index, direction) in TURNAROUND_DIRECTIONS.iter().enumerate() {
        for frame_index in 0..WALK_CYCLE_FRAMES_PER_DIRECTION {
            let flat_index = direction_index * WALK_CYCLE_FRAMES_PER_DIRECTION + frame_index;
            let frame = &report.frames[flat_index];
            if frame.direction != *direction || frame.frame_index != frame_index {
                return Err(
                    "Walk Cycle validation frames must use canonical direction and index order."
                        .to_owned(),
                );
            }
            validate_structural_report(frame.report.clone())?;
            if frame.report.candidate_id != report.walk_cycle_id
                || frame.report.contract_id != report.contract_id
            {
                return Err("Frame validation identity does not match Walk Cycle.".to_owned());
            }
            counted.pass += frame.report.summary.pass;
            counted.fail += frame.report.summary.fail;
            counted.not_assessed += frame.report.summary.not_assessed;
        }
    }
    if counted != report.summary {
        return Err("Walk Cycle validation summary does not match frame reports.".to_owned());
    }
    if report.motion_judgment.status != VisualJudgmentStatus::NotAssessed
        || report.motion_judgment.authority != "user"
        || report.motion_judgment.message.is_empty()
    {
        return Err("Walk Cycle validation crossed the user-owned motion gate.".to_owned());
    }
    Ok(report)
}

fn validate_walk_cycle_pngs(
    payload: &WalkCycleCandidatePayload,
) -> Result<WalkCycleValidationReport, String> {
    let mut frames = Vec::new();
    let mut summary = ValidationSummary {
        pass: 0,
        fail: 0,
        not_assessed: 0,
    };
    for source in &payload.candidate.frames {
        let bytes = payload
            .png_bytes
            .get(source.direction, source.frame_index)
            .ok_or_else(|| "Walk Cycle frame bytes are incomplete.".to_owned())?;
        let report = validate_png_structural_evidence(
            &payload.candidate.id,
            &source.sha256,
            source.byte_length,
            &payload.candidate.contract_id,
            bytes,
            StructuralContactMode::FootAnchorRow,
        )?;
        summary.pass += report.summary.pass;
        summary.fail += report.summary.fail;
        summary.not_assessed += report.summary.not_assessed;
        frames.push(WalkCycleFrameReport {
            direction: source.direction,
            frame_index: source.frame_index,
            report,
        });
    }
    validate_walk_cycle_report(WalkCycleValidationReport {
        schema_version: 1,
        validator_id: WALK_CYCLE_VALIDATOR_ID.to_owned(),
        walk_cycle_id: payload.candidate.id.clone(),
        contract_id: payload.candidate.contract_id.clone(),
        frames,
        summary,
        motion_judgment: payload.candidate.motion_judgment.clone(),
    })
}

fn reference_source_bytes(source_file: &str) -> Option<&'static [u8]> {
    match source_file {
        "images/scale-lineup-forest.png" => Some(include_bytes!(
            "../../reference-packs/tileforge-world-test-v1/images/scale-lineup-forest.png"
        )),
        "images/scale-lineup-autumn.png" => Some(include_bytes!(
            "../../reference-packs/tileforge-world-test-v1/images/scale-lineup-autumn.png"
        )),
        "images/scale-lineup-dusk.png" => Some(include_bytes!(
            "../../reference-packs/tileforge-world-test-v1/images/scale-lineup-dusk.png"
        )),
        "images/scale-lineup-winter.png" => Some(include_bytes!(
            "../../reference-packs/tileforge-world-test-v1/images/scale-lineup-winter.png"
        )),
        "images/forest-clearing-forest.png" => Some(include_bytes!(
            "../../reference-packs/tileforge-world-test-v1/images/forest-clearing-forest.png"
        )),
        "images/forest-clearing-autumn.png" => Some(include_bytes!(
            "../../reference-packs/tileforge-world-test-v1/images/forest-clearing-autumn.png"
        )),
        "images/forest-clearing-dusk.png" => Some(include_bytes!(
            "../../reference-packs/tileforge-world-test-v1/images/forest-clearing-dusk.png"
        )),
        "images/forest-clearing-winter.png" => Some(include_bytes!(
            "../../reference-packs/tileforge-world-test-v1/images/forest-clearing-winter.png"
        )),
        "images/crownhold-forest.png" => Some(include_bytes!(
            "../../reference-packs/tileforge-world-test-v1/images/crownhold-forest.png"
        )),
        "images/crownhold-autumn.png" => Some(include_bytes!(
            "../../reference-packs/tileforge-world-test-v1/images/crownhold-autumn.png"
        )),
        "images/crownhold-dusk.png" => Some(include_bytes!(
            "../../reference-packs/tileforge-world-test-v1/images/crownhold-dusk.png"
        )),
        "images/crownhold-winter.png" => Some(include_bytes!(
            "../../reference-packs/tileforge-world-test-v1/images/crownhold-winter.png"
        )),
        "images/tidewater-forest.png" => Some(include_bytes!(
            "../../reference-packs/tileforge-world-test-v1/images/tidewater-forest.png"
        )),
        "images/tidewater-autumn.png" => Some(include_bytes!(
            "../../reference-packs/tileforge-world-test-v1/images/tidewater-autumn.png"
        )),
        "images/tidewater-dusk.png" => Some(include_bytes!(
            "../../reference-packs/tileforge-world-test-v1/images/tidewater-dusk.png"
        )),
        "images/tidewater-winter.png" => Some(include_bytes!(
            "../../reference-packs/tileforge-world-test-v1/images/tidewater-winter.png"
        )),
        _ => None,
    }
}

fn load_world_test_reference_pack() -> Result<LoadedReferencePack, String> {
    let manifest: WorldTestReferencePack = serde_json::from_slice(WORLD_TEST_REFERENCE_MANIFEST)
        .map_err(|error| format!("Invalid pinned reference manifest: {error}"))?;
    if manifest.schema_version != 1
        || manifest.id != WORLD_TEST_REFERENCE_PACK_ID
        || manifest.version != 1
        || manifest.contract_id != CONTRACT_ID
        || manifest.source.repository.is_empty()
        || manifest.source.checkout_commit.len() != 40
        || !manifest
            .source
            .checkout_commit
            .chars()
            .all(|character| character.is_ascii_hexdigit())
        || manifest.source.generated_engine_commit.len() < 7
        || manifest.source.generated_engine_commit.len() > 40
        || !manifest
            .source
            .generated_engine_commit
            .chars()
            .all(|character| character.is_ascii_hexdigit())
        || manifest.source.generated.is_empty()
        || manifest.source.render_path.is_empty()
        || manifest.source.scale != "1x"
        || manifest.preview.width != WORLD_TEST_PREVIEW_WIDTH
        || manifest.preview.height != WORLD_TEST_PREVIEW_HEIGHT
        || manifest.preview.actor_direction != "down"
        || manifest.preview.actor_frame_index != 0
        || manifest.preview.compositor != "nearest-neighbor-hard-alpha-v1"
        || manifest.entries.len() != WORLD_TEST_SCENES.len() * WORLD_TEST_THEMES.len()
    {
        return Err("Pinned World Test reference manifest is incompatible.".to_owned());
    }

    let mut sources = Vec::new();
    for (scene_index, scene) in WORLD_TEST_SCENES.iter().enumerate() {
        for (theme_index, theme) in WORLD_TEST_THEMES.iter().enumerate() {
            let index = scene_index * WORLD_TEST_THEMES.len() + theme_index;
            let entry = &manifest.entries[index];
            if entry.scene != *scene
                || entry.theme != *theme
                || !entry.source_file.starts_with("images/")
                || !entry.source_file.ends_with(".png")
                || !valid_sha256(&entry.source_sha256)
                || entry.viewport.width != WORLD_TEST_PREVIEW_WIDTH
                || entry.viewport.height != WORLD_TEST_PREVIEW_HEIGHT
                || entry.viewport.x + entry.viewport.width > entry.source_width
                || entry.viewport.y + entry.viewport.height > entry.source_height
                || entry.actor_placement.x + FRAME_WIDTH > entry.viewport.width
                || entry.actor_placement.y + FRAME_HEIGHT > entry.viewport.height
                || entry.ground_sample.x + entry.ground_sample.width > entry.viewport.width
                || entry.ground_sample.y + entry.ground_sample.height > entry.viewport.height
            {
                return Err("Pinned World Test reference entry is incompatible.".to_owned());
            }
            let bytes = reference_source_bytes(&entry.source_file)
                .ok_or_else(|| "Pinned World Test reference source is unavailable.".to_owned())?;
            let decoded = decode_png_rgba(bytes)?;
            if bytes.len() != entry.source_byte_length
                || format!("{:x}", Sha256::digest(bytes)) != entry.source_sha256
                || decoded.width != entry.source_width
                || decoded.height != entry.source_height
            {
                return Err(format!(
                    "Pinned reference {}/{} no longer matches its manifest.",
                    entry.scene, entry.theme
                ));
            }
            sources.push(bytes.to_vec());
        }
    }

    Ok(LoadedReferencePack {
        manifest,
        manifest_sha256: format!("{:x}", Sha256::digest(WORLD_TEST_REFERENCE_MANIFEST)),
        sources,
    })
}

fn encode_png_rgba(width: u32, height: u32, pixels: &[[u8; 4]]) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut bytes, width, height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder
            .write_header()
            .map_err(|error| format!("Could not encode World Test preview: {error}"))?;
        let flat: Vec<u8> = pixels
            .iter()
            .flat_map(|pixel| pixel.iter().copied())
            .collect();
        writer
            .write_image_data(&flat)
            .map_err(|error| format!("Could not encode World Test preview: {error}"))?;
    }
    Ok(bytes)
}

fn render_world_test_preview(
    entry: &ReferencePackEntry,
    reference_bytes: &[u8],
    actor_bytes: &[u8],
) -> Result<Vec<u8>, String> {
    let source = decode_png_rgba(reference_bytes)?;
    let actor = decode_png_rgba(actor_bytes)?;
    if actor.width != FRAME_WIDTH || actor.height != FRAME_HEIGHT {
        return Err("World Test actor frame dimensions are incompatible.".to_owned());
    }
    let mut pixels = vec![[0_u8; 4]; (entry.viewport.width * entry.viewport.height) as usize];
    for y in 0..entry.viewport.height {
        for x in 0..entry.viewport.width {
            let source_index =
                ((entry.viewport.y + y) * source.width + entry.viewport.x + x) as usize;
            let preview_index = (y * entry.viewport.width + x) as usize;
            pixels[preview_index] = source.pixels[source_index];
        }
    }
    for y in 0..actor.height {
        for x in 0..actor.width {
            let actor_pixel = actor.pixels[(y * actor.width + x) as usize];
            if actor_pixel[3] == 0 {
                continue;
            }
            let preview_index = ((entry.actor_placement.y + y) * entry.viewport.width
                + entry.actor_placement.x
                + x) as usize;
            pixels[preview_index] = actor_pixel;
        }
    }
    encode_png_rgba(entry.viewport.width, entry.viewport.height, &pixels)
}

fn preview_filename(entry: &ReferencePackEntry) -> String {
    format!("{}-{}.png", entry.scene, entry.theme)
}

fn validate_world_test(candidate: WorldTestCandidate) -> Result<WorldTestCandidate, String> {
    validate_candidate_id(&candidate.id)?;
    validate_session_id(&candidate.session_id)?;
    validate_candidate_id(&candidate.source_walk_cycle.walk_cycle_id)?;
    if candidate.schema_version != 1
        || candidate.revision == 0
        || candidate.stage != "world-test"
        || candidate.contract_id != CONTRACT_ID
    {
        return Err("World Test document is incompatible.".to_owned());
    }
    if candidate.source_walk_cycle.frame_sources.len()
        != TURNAROUND_DIRECTIONS.len() * WALK_CYCLE_FRAMES_PER_DIRECTION
        || candidate.source_walk_cycle.accepted_by != "user"
        || candidate.source_walk_cycle.accepted_at.is_empty()
    {
        return Err("World Test accepted Walk Cycle receipt is invalid.".to_owned());
    }
    for (direction_index, direction) in TURNAROUND_DIRECTIONS.iter().enumerate() {
        for frame_index in 0..WALK_CYCLE_FRAMES_PER_DIRECTION {
            let index = direction_index * WALK_CYCLE_FRAMES_PER_DIRECTION + frame_index;
            let source = &candidate.source_walk_cycle.frame_sources[index];
            if source.direction != *direction
                || source.frame_index != frame_index
                || !valid_sha256(&source.sha256)
                || source.byte_length == 0
                || source.byte_length > CONCEPT_PNG_MAX_BYTES
            {
                return Err(
                    "World Test Walk Cycle frames must use canonical immutable order.".to_owned(),
                );
            }
        }
    }
    if candidate.reference_pack.id != WORLD_TEST_REFERENCE_PACK_ID
        || candidate.reference_pack.version != 1
        || !valid_sha256(&candidate.reference_pack.manifest_sha256)
        || candidate.reference_pack.checkout_commit.len() != 40
        || candidate.reference_pack.generated_engine_commit.len() < 7
        || candidate.reference_pack.generated_engine_commit.len() > 40
    {
        return Err("World Test reference-pack receipt is invalid.".to_owned());
    }
    if candidate.previews.len() != WORLD_TEST_SCENES.len() * WORLD_TEST_THEMES.len() {
        return Err("World Test must contain sixteen previews.".to_owned());
    }
    for (scene_index, scene) in WORLD_TEST_SCENES.iter().enumerate() {
        for (theme_index, theme) in WORLD_TEST_THEMES.iter().enumerate() {
            let index = scene_index * WORLD_TEST_THEMES.len() + theme_index;
            let preview = &candidate.previews[index];
            if preview.scene != *scene
                || preview.theme != *theme
                || preview.source_file != format!("{scene}-{theme}.png")
                || !valid_sha256(&preview.sha256)
                || !valid_sha256(&preview.reference_source_sha256)
                || preview.byte_length == 0
                || preview.width != WORLD_TEST_PREVIEW_WIDTH
                || preview.height != WORLD_TEST_PREVIEW_HEIGHT
            {
                return Err(
                    "World Test previews must use canonical scene and theme order.".to_owned(),
                );
            }
        }
    }
    if candidate.created_at.is_empty()
        || candidate.preparation.method != "local-deterministic-compositor-v1"
        || candidate.preparation.additional_ai_cost
        || candidate.review_status != "unreviewed"
    {
        return Err("World Test preparation or review status is invalid.".to_owned());
    }
    if candidate.final_art_judgment.status != VisualJudgmentStatus::NotAssessed
        || candidate.final_art_judgment.authority != "user"
        || candidate.final_art_judgment.message.is_empty()
    {
        return Err("World Test crossed the user-owned final-art gate.".to_owned());
    }
    Ok(candidate)
}

fn world_test_root(root: &Path, session_id: &str) -> Result<PathBuf, String> {
    validate_session_id(session_id)?;
    Ok(root.join("sessions").join(session_id).join("world-tests"))
}

fn world_test_directory(
    root: &Path,
    session_id: &str,
    world_test_id: &str,
) -> Result<PathBuf, String> {
    validate_candidate_id(world_test_id)?;
    Ok(world_test_root(root, session_id)?.join(world_test_id))
}

fn read_world_test(
    root: &Path,
    session_id: &str,
    world_test_id: &str,
) -> Result<WorldTestCandidate, String> {
    let session = read_session(root, session_id)?;
    let raw = fs::read_to_string(
        world_test_directory(root, &session.id, world_test_id)?.join("world-test.json"),
    )
    .map_err(|error| format!("Could not read World Test: {error}"))?;
    let candidate: WorldTestCandidate = serde_json::from_str(&raw)
        .map_err(|error| format!("Invalid World Test document: {error}"))?;
    let candidate = validate_world_test(candidate)?;
    if candidate.session_id != session.id || candidate.id != world_test_id {
        return Err("World Test identity does not match its storage path.".to_owned());
    }
    Ok(candidate)
}

fn list_world_tests_at(root: &Path, session_id: &str) -> Result<Vec<WorldTestCandidate>, String> {
    let session = read_session(root, session_id)?;
    let candidates_root = world_test_root(root, &session.id)?;
    fs::create_dir_all(&candidates_root)
        .map_err(|error| format!("Could not create World Test storage: {error}"))?;
    let mut candidates = Vec::new();
    for entry in fs::read_dir(&candidates_root)
        .map_err(|error| format!("Could not list World Tests: {error}"))?
    {
        let Ok(entry) = entry else {
            continue;
        };
        let file_name = entry.file_name();
        let Some(world_test_id) = file_name.to_str() else {
            continue;
        };
        if world_test_id.starts_with('.') || !entry.path().is_dir() {
            continue;
        }
        if let Ok(candidate) = read_world_test(root, &session.id, world_test_id) {
            candidates.push(candidate);
        }
    }
    candidates.sort_by(|left, right| right.revision.cmp(&left.revision));
    Ok(candidates)
}

#[allow(clippy::too_many_arguments)]
fn create_world_test_candidate_at(
    root: &Path,
    session_id: &str,
    source_walk_cycle_id: &str,
    timestamp: &str,
    id_suffix: &str,
    temporary_suffix: &str,
    forced_revision: Option<u32>,
) -> Result<WorldTestCandidate, String> {
    let session = read_session(root, session_id)?;
    let source_walk_cycle = read_walk_cycle_payload(root, &session.id, source_walk_cycle_id)?;
    let reference_pack = load_world_test_reference_pack()?;
    let candidates = list_world_tests_at(root, &session.id)?;
    let revision = forced_revision.unwrap_or_else(|| {
        candidates
            .iter()
            .map(|candidate| candidate.revision)
            .max()
            .unwrap_or(0)
            + 1
    });
    let timestamp_digits: String = timestamp
        .chars()
        .filter(char::is_ascii_digit)
        .take(14)
        .collect();
    let down_frame_zero = source_walk_cycle
        .png_bytes
        .get(TurnaroundDirection::Down, 0)
        .ok_or_else(|| "Accepted Walk Cycle down frame 0 is unavailable.".to_owned())?;
    let mut preview_bytes = Vec::new();
    for (entry, reference_bytes) in reference_pack
        .manifest
        .entries
        .iter()
        .zip(reference_pack.sources.iter())
    {
        preview_bytes.push(render_world_test_preview(
            entry,
            reference_bytes,
            down_frame_zero,
        )?);
    }
    let candidate = validate_world_test(WorldTestCandidate {
        schema_version: 1,
        id: format!(
            "world-test-r{:04}-{}-{}",
            revision, timestamp_digits, id_suffix
        ),
        revision,
        session_id: session.id.clone(),
        stage: "world-test".to_owned(),
        contract_id: CONTRACT_ID.to_owned(),
        source_walk_cycle: WorldTestSourceWalkCycle {
            walk_cycle_id: source_walk_cycle.candidate.id.clone(),
            frame_sources: source_walk_cycle
                .candidate
                .frames
                .iter()
                .map(|frame| WorldTestAcceptedFrameSource {
                    direction: frame.direction,
                    frame_index: frame.frame_index,
                    sha256: frame.sha256.clone(),
                    byte_length: frame.byte_length,
                })
                .collect(),
            accepted_by: "user".to_owned(),
            accepted_at: timestamp.to_owned(),
        },
        reference_pack: WorldTestReferenceReceipt {
            id: reference_pack.manifest.id.clone(),
            version: reference_pack.manifest.version,
            manifest_sha256: reference_pack.manifest_sha256.clone(),
            checkout_commit: reference_pack.manifest.source.checkout_commit.clone(),
            generated_engine_commit: reference_pack
                .manifest
                .source
                .generated_engine_commit
                .clone(),
        },
        previews: reference_pack
            .manifest
            .entries
            .iter()
            .zip(preview_bytes.iter())
            .map(|(entry, bytes)| WorldTestPreviewSource {
                scene: entry.scene.clone(),
                theme: entry.theme.clone(),
                source_file: preview_filename(entry),
                sha256: format!("{:x}", Sha256::digest(bytes)),
                byte_length: bytes.len(),
                width: WORLD_TEST_PREVIEW_WIDTH,
                height: WORLD_TEST_PREVIEW_HEIGHT,
                reference_source_sha256: entry.source_sha256.clone(),
            })
            .collect(),
        created_at: timestamp.to_owned(),
        preparation: WorldTestPreparation {
            method: "local-deterministic-compositor-v1".to_owned(),
            additional_ai_cost: false,
        },
        review_status: "unreviewed".to_owned(),
        final_art_judgment: VisualJudgment {
            status: VisualJudgmentStatus::NotAssessed,
            authority: "user".to_owned(),
            message: "Only the user can approve final art after reviewing World Test evidence."
                .to_owned(),
        },
    })?;

    let candidates_root = world_test_root(root, &session.id)?;
    fs::create_dir_all(&candidates_root)
        .map_err(|error| format!("Could not create World Test storage: {error}"))?;
    let final_directory = world_test_directory(root, &session.id, &candidate.id)?;
    let temporary_directory =
        candidates_root.join(format!(".{}.{}.tmp", candidate.id, temporary_suffix));
    fs::create_dir(&temporary_directory)
        .map_err(|error| format!("Could not stage World Test: {error}"))?;
    let publish_result = (|| -> Result<(), String> {
        for (preview, bytes) in candidate.previews.iter().zip(preview_bytes.iter()) {
            fs::write(temporary_directory.join(&preview.source_file), bytes).map_err(|error| {
                format!(
                    "Could not write {}/{} World Test preview: {error}",
                    preview.scene, preview.theme
                )
            })?;
        }
        let document = format!(
            "{}\n",
            serde_json::to_string_pretty(&candidate)
                .map_err(|error| format!("Could not serialize World Test: {error}"))?
        );
        fs::write(temporary_directory.join("world-test.json"), document)
            .map_err(|error| format!("Could not write World Test document: {error}"))?;
        fs::rename(&temporary_directory, &final_directory)
            .map_err(|error| format!("Could not publish World Test: {error}"))
    })();
    if publish_result.is_err() {
        let _ = fs::remove_dir_all(&temporary_directory);
    }
    publish_result?;
    Ok(candidate)
}

fn read_world_test_payload(
    root: &Path,
    session_id: &str,
    world_test_id: &str,
) -> Result<WorldTestCandidatePayload, String> {
    let candidate = read_world_test(root, session_id, world_test_id)?;
    let directory = world_test_directory(root, session_id, world_test_id)?;
    let mut preview_png_bytes = HashMap::new();
    for preview in &candidate.previews {
        let bytes = fs::read(directory.join(&preview.source_file)).map_err(|error| {
            format!(
                "Could not read {}/{} World Test preview: {error}",
                preview.scene, preview.theme
            )
        })?;
        let decoded = decode_png_rgba(&bytes)?;
        if bytes.len() != preview.byte_length
            || format!("{:x}", Sha256::digest(&bytes)) != preview.sha256
            || decoded.width != preview.width
            || decoded.height != preview.height
        {
            return Err(format!(
                "{}/{} preview no longer matches immutable provenance.",
                preview.scene, preview.theme
            ));
        }
        preview_png_bytes.insert(format!("{}/{}", preview.scene, preview.theme), bytes);
    }
    Ok(WorldTestCandidatePayload {
        candidate,
        preview_png_bytes,
    })
}

fn rounded_pixel_luma(pixel: [u8; 4]) -> u32 {
    (299 * u32::from(pixel[0]) + 587 * u32::from(pixel[1]) + 114 * u32::from(pixel[2]) + 500) / 1000
}

fn actor_mean_luma(png_bytes: &[u8]) -> Result<u32, String> {
    let decoded = decode_png_rgba(png_bytes)?;
    let mut total = 0_u32;
    let mut count = 0_u32;
    for pixel in decoded.pixels {
        if pixel[3] == 0 {
            continue;
        }
        total += rounded_pixel_luma(pixel);
        count += 1;
    }
    if count == 0 {
        return Err("Walk Cycle frame has no visible actor pixels.".to_owned());
    }
    Ok((total + count / 2) / count)
}

fn ground_mean_luma(entry: &ReferencePackEntry, png_bytes: &[u8]) -> Result<u32, String> {
    let decoded = decode_png_rgba(png_bytes)?;
    let mut total = 0_u32;
    let mut count = 0_u32;
    for y in entry.ground_sample.y..entry.ground_sample.y + entry.ground_sample.height {
        for x in entry.ground_sample.x..entry.ground_sample.x + entry.ground_sample.width {
            let source_x = entry.viewport.x + x;
            let source_y = entry.viewport.y + y;
            total +=
                rounded_pixel_luma(decoded.pixels[(source_y * decoded.width + source_x) as usize]);
            count += 1;
        }
    }
    Ok((total + count / 2) / count)
}

fn assert_world_test_source_walk_cycle(
    candidate: &WorldTestCandidate,
    source: &WalkCycleCandidatePayload,
) -> Result<(), String> {
    if candidate.source_walk_cycle.walk_cycle_id != source.candidate.id {
        return Err("World Test source Walk Cycle identity changed.".to_owned());
    }
    for (receipt, frame) in candidate
        .source_walk_cycle
        .frame_sources
        .iter()
        .zip(source.candidate.frames.iter())
    {
        if receipt.direction != frame.direction
            || receipt.frame_index != frame.frame_index
            || receipt.sha256 != frame.sha256
            || receipt.byte_length != frame.byte_length
        {
            return Err("World Test source Walk Cycle bytes changed.".to_owned());
        }
    }
    Ok(())
}

fn validate_world_test_report(
    report: WorldTestValidationReport,
) -> Result<WorldTestValidationReport, String> {
    validate_candidate_id(&report.world_test_id)?;
    if report.schema_version != 1
        || report.validator_id != WORLD_TEST_VALIDATOR_ID
        || report.contract_id != CONTRACT_ID
        || report.measurements.len()
            != WORLD_TEST_SCENES.len()
                * WORLD_TEST_THEMES.len()
                * TURNAROUND_DIRECTIONS.len()
                * WALK_CYCLE_FRAMES_PER_DIRECTION
    {
        return Err("Unsupported World Test validation report.".to_owned());
    }
    let mut index = 0;
    let mut counted = ValidationSummary {
        pass: 0,
        fail: 0,
        not_assessed: 0,
    };
    for scene in WORLD_TEST_SCENES {
        for theme in WORLD_TEST_THEMES {
            for direction in TURNAROUND_DIRECTIONS {
                for frame_index in 0..WALK_CYCLE_FRAMES_PER_DIRECTION {
                    let measurement = &report.measurements[index];
                    if measurement.scene != scene
                        || measurement.theme != theme
                        || measurement.direction != direction
                        || measurement.frame_index != frame_index
                        || measurement.minimum != MINIMUM_GROUND_LUMA_DISTANCE
                        || measurement.distance
                            != measurement
                                .actor_mean_luma
                                .abs_diff(measurement.ground_mean_luma)
                        || (measurement.status == ValidationStatus::Pass)
                            != (measurement.distance >= measurement.minimum)
                        || measurement.status == ValidationStatus::NotAssessed
                    {
                        return Err(
                            "World Test luma measurement order or evidence is invalid.".to_owned()
                        );
                    }
                    match measurement.status {
                        ValidationStatus::Pass => counted.pass += 1,
                        ValidationStatus::Fail => counted.fail += 1,
                        ValidationStatus::NotAssessed => counted.not_assessed += 1,
                    }
                    index += 1;
                }
            }
        }
    }
    if counted != report.summary || report.summary.not_assessed != 0 {
        return Err("World Test validation summary does not match measurements.".to_owned());
    }
    if report.final_art_judgment.status != VisualJudgmentStatus::NotAssessed
        || report.final_art_judgment.authority != "user"
        || report.final_art_judgment.message.is_empty()
    {
        return Err("World Test validation crossed the user-owned final-art gate.".to_owned());
    }
    Ok(report)
}

fn validate_world_test_pngs(
    root: &Path,
    payload: &WorldTestCandidatePayload,
) -> Result<WorldTestValidationReport, String> {
    let source = read_walk_cycle_payload(
        root,
        &payload.candidate.session_id,
        &payload.candidate.source_walk_cycle.walk_cycle_id,
    )?;
    assert_world_test_source_walk_cycle(&payload.candidate, &source)?;
    let reference_pack = load_world_test_reference_pack()?;
    if payload.candidate.reference_pack.manifest_sha256 != reference_pack.manifest_sha256
        || payload.candidate.reference_pack.checkout_commit
            != reference_pack.manifest.source.checkout_commit
        || payload.candidate.reference_pack.generated_engine_commit
            != reference_pack.manifest.source.generated_engine_commit
    {
        return Err("World Test reference-pack receipt no longer matches.".to_owned());
    }
    let mut actor_lumas = HashMap::new();
    for frame in &source.candidate.frames {
        let bytes = source
            .png_bytes
            .get(frame.direction, frame.frame_index)
            .ok_or_else(|| "Walk Cycle frame bytes are incomplete.".to_owned())?;
        actor_lumas.insert(
            (frame.direction, frame.frame_index),
            actor_mean_luma(bytes)?,
        );
    }
    let mut measurements = Vec::new();
    let mut summary = ValidationSummary {
        pass: 0,
        fail: 0,
        not_assessed: 0,
    };
    for (entry, reference_bytes) in reference_pack
        .manifest
        .entries
        .iter()
        .zip(reference_pack.sources.iter())
    {
        let ground = ground_mean_luma(entry, reference_bytes)?;
        for direction in TURNAROUND_DIRECTIONS {
            for frame_index in 0..WALK_CYCLE_FRAMES_PER_DIRECTION {
                let actor = *actor_lumas
                    .get(&(direction, frame_index))
                    .ok_or_else(|| "Walk Cycle luma evidence is incomplete.".to_owned())?;
                let distance = actor.abs_diff(ground);
                let status = if distance >= MINIMUM_GROUND_LUMA_DISTANCE {
                    summary.pass += 1;
                    ValidationStatus::Pass
                } else {
                    summary.fail += 1;
                    ValidationStatus::Fail
                };
                measurements.push(WorldTestLumaMeasurement {
                    scene: entry.scene.clone(),
                    theme: entry.theme.clone(),
                    direction,
                    frame_index,
                    actor_mean_luma: actor,
                    ground_mean_luma: ground,
                    distance,
                    minimum: MINIMUM_GROUND_LUMA_DISTANCE,
                    status,
                });
            }
        }
    }
    validate_world_test_report(WorldTestValidationReport {
        schema_version: 1,
        validator_id: WORLD_TEST_VALIDATOR_ID.to_owned(),
        world_test_id: payload.candidate.id.clone(),
        contract_id: payload.candidate.contract_id.clone(),
        measurements,
        summary,
        final_art_judgment: payload.candidate.final_art_judgment.clone(),
    })
}

fn json_document_bytes<T: Serialize>(value: &T, label: &str) -> Result<Vec<u8>, String> {
    Ok(format!(
        "{}\n",
        serde_json::to_string_pretty(value)
            .map_err(|error| format!("Could not serialize {label}: {error}"))?
    )
    .into_bytes())
}

fn json_equal<T: Serialize, U: Serialize>(left: &T, right: &U) -> Result<bool, String> {
    let left = serde_json::to_value(left)
        .map_err(|error| format!("Could not compare Export receipt: {error}"))?;
    let right = serde_json::to_value(right)
        .map_err(|error| format!("Could not compare Export receipt: {error}"))?;
    Ok(left == right)
}

fn validate_export_document(candidate: ExportCandidate) -> Result<ExportCandidate, String> {
    validate_candidate_id(&candidate.id)?;
    validate_session_id(&candidate.session_id)?;
    validate_candidate_id(&candidate.approved_world_test.world_test_id)?;
    validate_candidate_id(&candidate.source_walk_cycle.walk_cycle_id)?;
    if candidate.schema_version != 1
        || candidate.revision == 0
        || candidate.stage != "export"
        || candidate.contract_id != CONTRACT_ID
    {
        return Err("Export document is incompatible.".to_owned());
    }
    if !valid_sha256(&candidate.approved_world_test.document_sha256)
        || candidate.approved_world_test.approved_by != "user"
        || candidate.approved_world_test.approved_at.is_empty()
        || candidate.approved_world_test.preview_sources.len()
            != WORLD_TEST_SCENES.len() * WORLD_TEST_THEMES.len()
    {
        return Err("Export approved World Test receipt is invalid.".to_owned());
    }
    for (scene_index, scene) in WORLD_TEST_SCENES.iter().enumerate() {
        for (theme_index, theme) in WORLD_TEST_THEMES.iter().enumerate() {
            let index = scene_index * WORLD_TEST_THEMES.len() + theme_index;
            let preview = &candidate.approved_world_test.preview_sources[index];
            if preview.scene != *scene
                || preview.theme != *theme
                || preview.source_file != format!("{scene}-{theme}.png")
                || !valid_sha256(&preview.sha256)
                || preview.byte_length == 0
            {
                return Err(
                    "Export approved World Test previews must use canonical order.".to_owned(),
                );
            }
        }
    }
    if candidate.source_walk_cycle.frame_sources.len()
        != TURNAROUND_DIRECTIONS.len() * WALK_CYCLE_FRAMES_PER_DIRECTION
    {
        return Err("Export source Walk Cycle receipt is incomplete.".to_owned());
    }
    for (direction_index, direction) in TURNAROUND_DIRECTIONS.iter().enumerate() {
        for frame_index in 0..WALK_CYCLE_FRAMES_PER_DIRECTION {
            let index = direction_index * WALK_CYCLE_FRAMES_PER_DIRECTION + frame_index;
            let frame = &candidate.source_walk_cycle.frame_sources[index];
            if frame.direction != *direction
                || frame.frame_index != frame_index
                || !valid_sha256(&frame.sha256)
                || frame.byte_length == 0
                || frame.byte_length > CONCEPT_PNG_MAX_BYTES
            {
                return Err("Export source frames must use canonical immutable order.".to_owned());
            }
        }
    }
    let sheet = &candidate.package.sprite_sheet;
    if sheet.source_file != EXPORT_SHEET_FILE
        || !valid_sha256(&sheet.sha256)
        || sheet.byte_length == 0
        || sheet.width != EXPORT_SHEET_WIDTH
        || sheet.height != EXPORT_SHEET_HEIGHT
        || sheet.cell_width != FRAME_WIDTH
        || sheet.cell_height != FRAME_HEIGHT
        || sheet.layout != EXPORT_SHEET_LAYOUT
    {
        return Err("Export sprite-sheet receipt is invalid.".to_owned());
    }
    for (receipt, expected_file) in [
        (&candidate.package.metadata, EXPORT_METADATA_FILE),
        (&candidate.package.provenance, EXPORT_PROVENANCE_FILE),
    ] {
        if receipt.source_file != expected_file
            || !valid_sha256(&receipt.sha256)
            || receipt.byte_length == 0
        {
            return Err("Export JSON file receipt is invalid.".to_owned());
        }
    }
    if candidate.created_at.is_empty()
        || candidate.preparation.method != "local-deterministic-sheet-v1"
        || candidate.preparation.additional_ai_cost
        || candidate.status != "draft"
    {
        return Err("Export preparation or draft status is invalid.".to_owned());
    }
    if candidate.publishing.status != "not_approved"
        || candidate.publishing.authority != "user"
        || candidate.publishing.message.is_empty()
    {
        return Err("Export crossed the user-owned publishing gate.".to_owned());
    }
    Ok(candidate)
}

fn validate_export_metadata(metadata: ExportMetadata) -> Result<ExportMetadata, String> {
    if metadata.schema_version != 1
        || metadata.contract_id != CONTRACT_ID
        || metadata.actor.name.is_empty()
        || metadata.actor.name.chars().count() > 80
        || metadata.sheet.source_file != EXPORT_SHEET_FILE
        || metadata.sheet.width != EXPORT_SHEET_WIDTH
        || metadata.sheet.height != EXPORT_SHEET_HEIGHT
        || metadata.sheet.cell_width != FRAME_WIDTH
        || metadata.sheet.cell_height != FRAME_HEIGHT
        || metadata.sheet.layout != EXPORT_SHEET_LAYOUT
        || metadata.animation.clip != "walk"
        || metadata.animation.directions != TURNAROUND_DIRECTIONS
        || metadata.animation.frames_per_direction != WALK_CYCLE_FRAMES_PER_DIRECTION
        || metadata.animation.frame_duration_ms != WALK_CYCLE_FRAME_DURATION_MS
        || metadata.animation.foot_anchor != [FOOT_ANCHOR_X, FOOT_ANCHOR_Y]
        || metadata.frames.len() != TURNAROUND_DIRECTIONS.len() * WALK_CYCLE_FRAMES_PER_DIRECTION
    {
        return Err("Export metadata document is incompatible.".to_owned());
    }
    for (direction_index, direction) in TURNAROUND_DIRECTIONS.iter().enumerate() {
        for frame_index in 0..WALK_CYCLE_FRAMES_PER_DIRECTION {
            let index = direction_index * WALK_CYCLE_FRAMES_PER_DIRECTION + frame_index;
            let frame = &metadata.frames[index];
            if frame.direction != *direction
                || frame.frame_index != frame_index
                || frame.x != frame_index as u32 * FRAME_WIDTH
                || frame.y != direction_index as u32 * FRAME_HEIGHT
                || frame.width != FRAME_WIDTH
                || frame.height != FRAME_HEIGHT
                || !valid_sha256(&frame.sha256)
                || frame.byte_length == 0
            {
                return Err("Export metadata frames do not match the sheet layout.".to_owned());
            }
        }
    }
    Ok(metadata)
}

fn validate_export_provenance(provenance: ExportProvenance) -> Result<ExportProvenance, String> {
    validate_candidate_id(&provenance.export_id)?;
    validate_session_id(&provenance.session_id)?;
    if provenance.schema_version != 1
        || provenance.preparation.method != "local-deterministic-sheet-v1"
        || provenance.preparation.additional_ai_cost
        || provenance.publishing.status != "not_approved"
        || provenance.publishing.authority != "user"
        || provenance.publishing.message.is_empty()
    {
        return Err("Export provenance document is incompatible.".to_owned());
    }
    let probe = ExportCandidate {
        schema_version: 1,
        id: provenance.export_id.clone(),
        revision: 1,
        session_id: provenance.session_id.clone(),
        stage: "export".to_owned(),
        contract_id: CONTRACT_ID.to_owned(),
        approved_world_test: provenance.approved_world_test.clone(),
        source_walk_cycle: provenance.source_walk_cycle.clone(),
        package: ExportPackage {
            sprite_sheet: ExportSheetReceipt {
                source_file: EXPORT_SHEET_FILE.to_owned(),
                sha256: "0".repeat(64),
                byte_length: 1,
                width: EXPORT_SHEET_WIDTH,
                height: EXPORT_SHEET_HEIGHT,
                cell_width: FRAME_WIDTH,
                cell_height: FRAME_HEIGHT,
                layout: EXPORT_SHEET_LAYOUT.to_owned(),
            },
            metadata: ExportFileReceipt {
                source_file: EXPORT_METADATA_FILE.to_owned(),
                sha256: "0".repeat(64),
                byte_length: 1,
            },
            provenance: ExportFileReceipt {
                source_file: EXPORT_PROVENANCE_FILE.to_owned(),
                sha256: "0".repeat(64),
                byte_length: 1,
            },
        },
        created_at: provenance.approved_world_test.approved_at.clone(),
        preparation: provenance.preparation.clone(),
        status: "draft".to_owned(),
        publishing: provenance.publishing.clone(),
    };
    validate_export_document(probe)?;
    Ok(provenance)
}

fn render_export_sheet(source: &WalkCycleCandidatePayload) -> Result<Vec<u8>, String> {
    let mut pixels = vec![[0_u8; 4]; (EXPORT_SHEET_WIDTH * EXPORT_SHEET_HEIGHT) as usize];
    for (direction_index, direction) in TURNAROUND_DIRECTIONS.iter().enumerate() {
        for frame_index in 0..WALK_CYCLE_FRAMES_PER_DIRECTION {
            let bytes = source
                .png_bytes
                .get(*direction, frame_index)
                .ok_or_else(|| "Export source frame set is incomplete.".to_owned())?;
            let frame = decode_png_rgba(bytes)?;
            if frame.width != FRAME_WIDTH || frame.height != FRAME_HEIGHT {
                return Err("Export source frame dimensions are incompatible.".to_owned());
            }
            for y in 0..FRAME_HEIGHT {
                for x in 0..FRAME_WIDTH {
                    let source_index = (y * FRAME_WIDTH + x) as usize;
                    let target_x = frame_index as u32 * FRAME_WIDTH + x;
                    let target_y = direction_index as u32 * FRAME_HEIGHT + y;
                    let target_index = (target_y * EXPORT_SHEET_WIDTH + target_x) as usize;
                    pixels[target_index] = frame.pixels[source_index];
                }
            }
        }
    }
    encode_png_rgba(EXPORT_SHEET_WIDTH, EXPORT_SHEET_HEIGHT, &pixels)
}

fn build_export_metadata(
    session: &StudioSession,
    source: &WalkCycleCandidatePayload,
) -> Result<ExportMetadata, String> {
    validate_export_metadata(ExportMetadata {
        schema_version: 1,
        contract_id: CONTRACT_ID.to_owned(),
        actor: ExportActorMetadata {
            name: session.brief.name.clone(),
            kind: session.brief.kind.clone(),
        },
        sheet: ExportSheetMetadata {
            source_file: EXPORT_SHEET_FILE.to_owned(),
            width: EXPORT_SHEET_WIDTH,
            height: EXPORT_SHEET_HEIGHT,
            cell_width: FRAME_WIDTH,
            cell_height: FRAME_HEIGHT,
            layout: EXPORT_SHEET_LAYOUT.to_owned(),
        },
        animation: ExportAnimationMetadata {
            clip: "walk".to_owned(),
            directions: TURNAROUND_DIRECTIONS.to_vec(),
            frames_per_direction: WALK_CYCLE_FRAMES_PER_DIRECTION,
            frame_duration_ms: WALK_CYCLE_FRAME_DURATION_MS,
            foot_anchor: vec![FOOT_ANCHOR_X, FOOT_ANCHOR_Y],
        },
        frames: source
            .candidate
            .frames
            .iter()
            .enumerate()
            .map(|(index, frame)| ExportFrameMetadata {
                direction: frame.direction,
                frame_index: frame.frame_index,
                x: frame.frame_index as u32 * FRAME_WIDTH,
                y: (index / WALK_CYCLE_FRAMES_PER_DIRECTION) as u32 * FRAME_HEIGHT,
                width: FRAME_WIDTH,
                height: FRAME_HEIGHT,
                sha256: frame.sha256.clone(),
                byte_length: frame.byte_length,
            })
            .collect(),
    })
}

fn build_export_provenance(
    export_id: &str,
    session_id: &str,
    approved_world_test: ExportApprovedWorldTest,
    source_walk_cycle: ExportSourceWalkCycle,
) -> Result<ExportProvenance, String> {
    validate_export_provenance(ExportProvenance {
        schema_version: 1,
        export_id: export_id.to_owned(),
        session_id: session_id.to_owned(),
        approved_world_test,
        source_walk_cycle,
        preparation: ExportPreparation {
            method: "local-deterministic-sheet-v1".to_owned(),
            additional_ai_cost: false,
        },
        publishing: PublishingBoundary {
            status: "not_approved".to_owned(),
            authority: "user".to_owned(),
            message: "This draft export is local only. Publishing requires a separate explicit user decision."
                .to_owned(),
        },
    })
}

fn export_root(root: &Path, session_id: &str) -> Result<PathBuf, String> {
    validate_session_id(session_id)?;
    Ok(root.join("sessions").join(session_id).join("exports"))
}

fn export_directory(root: &Path, session_id: &str, export_id: &str) -> Result<PathBuf, String> {
    validate_candidate_id(export_id)?;
    Ok(export_root(root, session_id)?.join(export_id))
}

fn read_export(root: &Path, session_id: &str, export_id: &str) -> Result<ExportCandidate, String> {
    let session = read_session(root, session_id)?;
    let raw =
        fs::read_to_string(export_directory(root, &session.id, export_id)?.join("export.json"))
            .map_err(|error| format!("Could not read Export: {error}"))?;
    let candidate: ExportCandidate =
        serde_json::from_str(&raw).map_err(|error| format!("Invalid Export document: {error}"))?;
    let candidate = validate_export_document(candidate)?;
    if candidate.session_id != session.id || candidate.id != export_id {
        return Err("Export identity does not match its storage path.".to_owned());
    }
    Ok(candidate)
}

fn list_exports_at(root: &Path, session_id: &str) -> Result<Vec<ExportCandidate>, String> {
    let session = read_session(root, session_id)?;
    let candidates_root = export_root(root, &session.id)?;
    fs::create_dir_all(&candidates_root)
        .map_err(|error| format!("Could not create Export storage: {error}"))?;
    let mut candidates = Vec::new();
    for entry in fs::read_dir(&candidates_root)
        .map_err(|error| format!("Could not list Exports: {error}"))?
    {
        let Ok(entry) = entry else {
            continue;
        };
        let file_name = entry.file_name();
        let Some(export_id) = file_name.to_str() else {
            continue;
        };
        if export_id.starts_with('.') || !entry.path().is_dir() {
            continue;
        }
        if let Ok(candidate) = read_export(root, &session.id, export_id) {
            candidates.push(candidate);
        }
    }
    candidates.sort_by(|left, right| right.revision.cmp(&left.revision));
    Ok(candidates)
}

#[allow(clippy::too_many_arguments)]
fn create_export_candidate_at(
    root: &Path,
    session_id: &str,
    source_world_test_id: &str,
    timestamp: &str,
    id_suffix: &str,
    temporary_suffix: &str,
    forced_revision: Option<u32>,
) -> Result<ExportCandidate, String> {
    let session = read_session(root, session_id)?;
    let world_test = read_world_test_payload(root, &session.id, source_world_test_id)?;
    let walk_cycle = read_walk_cycle_payload(
        root,
        &session.id,
        &world_test.candidate.source_walk_cycle.walk_cycle_id,
    )?;
    assert_world_test_source_walk_cycle(&world_test.candidate, &walk_cycle)?;
    let world_test_document = fs::read(
        world_test_directory(root, &session.id, &world_test.candidate.id)?.join("world-test.json"),
    )
    .map_err(|error| format!("Could not read approved World Test document: {error}"))?;
    let revision = forced_revision.unwrap_or_else(|| {
        list_exports_at(root, &session.id)
            .ok()
            .and_then(|candidates| candidates.iter().map(|candidate| candidate.revision).max())
            .unwrap_or(0)
            + 1
    });
    let timestamp_digits: String = timestamp
        .chars()
        .filter(char::is_ascii_digit)
        .take(14)
        .collect();
    let export_id = format!("export-r{:04}-{}-{}", revision, timestamp_digits, id_suffix);
    let approved_world_test = ExportApprovedWorldTest {
        world_test_id: world_test.candidate.id.clone(),
        document_sha256: format!("{:x}", Sha256::digest(&world_test_document)),
        preview_sources: world_test
            .candidate
            .previews
            .iter()
            .map(|preview| ExportPreviewSource {
                scene: preview.scene.clone(),
                theme: preview.theme.clone(),
                source_file: preview.source_file.clone(),
                sha256: preview.sha256.clone(),
                byte_length: preview.byte_length,
            })
            .collect(),
        approved_by: "user".to_owned(),
        approved_at: timestamp.to_owned(),
    };
    let source_walk_cycle = ExportSourceWalkCycle {
        walk_cycle_id: walk_cycle.candidate.id.clone(),
        frame_sources: walk_cycle
            .candidate
            .frames
            .iter()
            .map(|frame| WorldTestAcceptedFrameSource {
                direction: frame.direction,
                frame_index: frame.frame_index,
                sha256: frame.sha256.clone(),
                byte_length: frame.byte_length,
            })
            .collect(),
    };
    let sprite_sheet_png_bytes = render_export_sheet(&walk_cycle)?;
    let metadata = build_export_metadata(&session, &walk_cycle)?;
    let provenance = build_export_provenance(
        &export_id,
        &session.id,
        approved_world_test.clone(),
        source_walk_cycle.clone(),
    )?;
    let metadata_bytes = json_document_bytes(&metadata, "Export metadata")?;
    let provenance_bytes = json_document_bytes(&provenance, "Export provenance")?;
    let candidate = validate_export_document(ExportCandidate {
        schema_version: 1,
        id: export_id,
        revision,
        session_id: session.id.clone(),
        stage: "export".to_owned(),
        contract_id: CONTRACT_ID.to_owned(),
        approved_world_test,
        source_walk_cycle,
        package: ExportPackage {
            sprite_sheet: ExportSheetReceipt {
                source_file: EXPORT_SHEET_FILE.to_owned(),
                sha256: format!("{:x}", Sha256::digest(&sprite_sheet_png_bytes)),
                byte_length: sprite_sheet_png_bytes.len(),
                width: EXPORT_SHEET_WIDTH,
                height: EXPORT_SHEET_HEIGHT,
                cell_width: FRAME_WIDTH,
                cell_height: FRAME_HEIGHT,
                layout: EXPORT_SHEET_LAYOUT.to_owned(),
            },
            metadata: ExportFileReceipt {
                source_file: EXPORT_METADATA_FILE.to_owned(),
                sha256: format!("{:x}", Sha256::digest(&metadata_bytes)),
                byte_length: metadata_bytes.len(),
            },
            provenance: ExportFileReceipt {
                source_file: EXPORT_PROVENANCE_FILE.to_owned(),
                sha256: format!("{:x}", Sha256::digest(&provenance_bytes)),
                byte_length: provenance_bytes.len(),
            },
        },
        created_at: timestamp.to_owned(),
        preparation: provenance.preparation.clone(),
        status: "draft".to_owned(),
        publishing: provenance.publishing.clone(),
    })?;

    let candidates_root = export_root(root, &session.id)?;
    fs::create_dir_all(&candidates_root)
        .map_err(|error| format!("Could not create Export storage: {error}"))?;
    let final_directory = export_directory(root, &session.id, &candidate.id)?;
    let temporary_directory =
        candidates_root.join(format!(".{}.{}.tmp", candidate.id, temporary_suffix));
    fs::create_dir(&temporary_directory)
        .map_err(|error| format!("Could not stage Export: {error}"))?;
    let publish_result = (|| -> Result<(), String> {
        fs::write(
            temporary_directory.join(EXPORT_SHEET_FILE),
            &sprite_sheet_png_bytes,
        )
        .map_err(|error| format!("Could not write Export sprite sheet: {error}"))?;
        fs::write(
            temporary_directory.join(EXPORT_METADATA_FILE),
            &metadata_bytes,
        )
        .map_err(|error| format!("Could not write Export metadata: {error}"))?;
        fs::write(
            temporary_directory.join(EXPORT_PROVENANCE_FILE),
            &provenance_bytes,
        )
        .map_err(|error| format!("Could not write Export provenance: {error}"))?;
        fs::write(
            temporary_directory.join("export.json"),
            json_document_bytes(&candidate, "Export document")?,
        )
        .map_err(|error| format!("Could not write Export document: {error}"))?;
        fs::rename(&temporary_directory, &final_directory)
            .map_err(|error| format!("Could not publish immutable Export: {error}"))
    })();
    if publish_result.is_err() {
        let _ = fs::remove_dir_all(&temporary_directory);
    }
    publish_result?;
    Ok(candidate)
}

fn read_export_payload(
    root: &Path,
    session_id: &str,
    export_id: &str,
) -> Result<ExportCandidatePayload, String> {
    let candidate = read_export(root, session_id, export_id)?;
    let directory = export_directory(root, session_id, export_id)?;
    let sprite_sheet_png_bytes = fs::read(directory.join(EXPORT_SHEET_FILE))
        .map_err(|error| format!("Could not read Export sprite sheet: {error}"))?;
    let metadata_bytes = fs::read(directory.join(EXPORT_METADATA_FILE))
        .map_err(|error| format!("Could not read Export metadata: {error}"))?;
    let provenance_bytes = fs::read(directory.join(EXPORT_PROVENANCE_FILE))
        .map_err(|error| format!("Could not read Export provenance: {error}"))?;
    let decoded = decode_png_rgba(&sprite_sheet_png_bytes)?;
    if sprite_sheet_png_bytes.len() != candidate.package.sprite_sheet.byte_length
        || format!("{:x}", Sha256::digest(&sprite_sheet_png_bytes))
            != candidate.package.sprite_sheet.sha256
        || decoded.width != EXPORT_SHEET_WIDTH
        || decoded.height != EXPORT_SHEET_HEIGHT
    {
        return Err("Export sprite sheet no longer matches immutable provenance.".to_owned());
    }
    if metadata_bytes.len() != candidate.package.metadata.byte_length
        || format!("{:x}", Sha256::digest(&metadata_bytes)) != candidate.package.metadata.sha256
    {
        return Err("Export metadata no longer matches immutable provenance.".to_owned());
    }
    if provenance_bytes.len() != candidate.package.provenance.byte_length
        || format!("{:x}", Sha256::digest(&provenance_bytes)) != candidate.package.provenance.sha256
    {
        return Err("Export provenance no longer matches immutable provenance.".to_owned());
    }
    let metadata: ExportMetadata = serde_json::from_slice(&metadata_bytes)
        .map_err(|error| format!("Invalid Export metadata: {error}"))?;
    let metadata = validate_export_metadata(metadata)?;
    let provenance: ExportProvenance = serde_json::from_slice(&provenance_bytes)
        .map_err(|error| format!("Invalid Export provenance: {error}"))?;
    let provenance = validate_export_provenance(provenance)?;
    if provenance.export_id != candidate.id
        || provenance.session_id != candidate.session_id
        || !json_equal(
            &provenance.approved_world_test,
            &candidate.approved_world_test,
        )?
        || !json_equal(&provenance.source_walk_cycle, &candidate.source_walk_cycle)?
        || provenance.preparation != candidate.preparation
        || provenance.publishing != candidate.publishing
    {
        return Err("Export provenance document does not match its receipt.".to_owned());
    }
    Ok(ExportCandidatePayload {
        candidate,
        sprite_sheet_png_bytes,
        metadata,
        provenance,
    })
}

fn validated_export_directory(
    root: &Path,
    session_id: &str,
    export_id: &str,
) -> Result<PathBuf, String> {
    read_export_payload(root, session_id, export_id)?;
    export_directory(root, session_id, export_id)
}

fn open_export_folder_at<F>(
    root: &Path,
    session_id: &str,
    export_id: &str,
    open_folder: F,
) -> Result<String, String>
where
    F: FnOnce(&Path) -> Result<(), String>,
{
    let directory = validated_export_directory(root, session_id, export_id)?;
    open_folder(&directory)?;
    Ok(directory.to_string_lossy().into_owned())
}

#[cfg(target_os = "windows")]
fn launch_export_folder(directory: &Path) -> Result<(), String> {
    std::process::Command::new("explorer.exe")
        .arg(directory)
        .spawn()
        .map(|_| ())
        .map_err(|error| format!("Could not open the Export folder: {error}"))
}

#[cfg(not(target_os = "windows"))]
fn launch_export_folder(_directory: &Path) -> Result<(), String> {
    Err("Open Export Folder is currently available only on Windows.".to_owned())
}

fn validate_export_package(
    root: &Path,
    payload: &ExportCandidatePayload,
) -> Result<ExportValidationReport, String> {
    let session = read_session(root, &payload.candidate.session_id)?;
    let world_test = read_world_test_payload(
        root,
        &session.id,
        &payload.candidate.approved_world_test.world_test_id,
    )?;
    let world_test_document = fs::read(
        world_test_directory(root, &session.id, &world_test.candidate.id)?.join("world-test.json"),
    )
    .map_err(|error| format!("Could not read approved World Test document: {error}"))?;
    if payload.candidate.approved_world_test.world_test_id != world_test.candidate.id
        || payload.candidate.approved_world_test.document_sha256
            != format!("{:x}", Sha256::digest(&world_test_document))
    {
        return Err("Export approved World Test document no longer matches.".to_owned());
    }
    for (receipt, preview) in payload
        .candidate
        .approved_world_test
        .preview_sources
        .iter()
        .zip(world_test.candidate.previews.iter())
    {
        if receipt.scene != preview.scene
            || receipt.theme != preview.theme
            || receipt.source_file != preview.source_file
            || receipt.sha256 != preview.sha256
            || receipt.byte_length != preview.byte_length
        {
            return Err("Export approved World Test previews no longer match.".to_owned());
        }
    }
    let walk_cycle = read_walk_cycle_payload(
        root,
        &session.id,
        &payload.candidate.source_walk_cycle.walk_cycle_id,
    )?;
    assert_world_test_source_walk_cycle(&world_test.candidate, &walk_cycle)?;
    for (receipt, frame) in payload
        .candidate
        .source_walk_cycle
        .frame_sources
        .iter()
        .zip(walk_cycle.candidate.frames.iter())
    {
        if receipt.direction != frame.direction
            || receipt.frame_index != frame.frame_index
            || receipt.sha256 != frame.sha256
            || receipt.byte_length != frame.byte_length
        {
            return Err("Export source Walk Cycle bytes changed.".to_owned());
        }
    }
    let expected_sheet = decode_png_rgba(&render_export_sheet(&walk_cycle)?)?;
    let actual_sheet = decode_png_rgba(&payload.sprite_sheet_png_bytes)?;
    if actual_sheet.pixels != expected_sheet.pixels {
        return Err("Export sprite-sheet pixels no longer match source frames.".to_owned());
    }
    let expected_metadata = build_export_metadata(&session, &walk_cycle)?;
    if !json_equal(&payload.metadata, &expected_metadata)? {
        return Err("Export metadata no longer describes the source frames.".to_owned());
    }
    let expected_provenance = build_export_provenance(
        &payload.candidate.id,
        &session.id,
        payload.candidate.approved_world_test.clone(),
        payload.candidate.source_walk_cycle.clone(),
    )?;
    if !json_equal(&payload.provenance, &expected_provenance)? {
        return Err("Export provenance no longer matches its source receipts.".to_owned());
    }
    let checks = [
        (
            ExportValidationCheckId::ApprovedWorldTest,
            "Approved World Test document and all sixteen previews are SHA-256 bound.",
        ),
        (
            ExportValidationCheckId::SourceWalkCycle,
            "All sixteen source Walk Cycle frames match the approved World Test receipt.",
        ),
        (
            ExportValidationCheckId::SpriteSheetIdentity,
            "The immutable 128 x 128 sprite sheet matches its SHA-256 receipt.",
        ),
        (
            ExportValidationCheckId::SpriteSheetPixels,
            "Every sheet cell is pixel-identical to its source Walk Cycle frame.",
        ),
        (
            ExportValidationCheckId::MetadataIdentity,
            "Metadata matches the actor contract, layout, timing, anchor, and source frames.",
        ),
        (
            ExportValidationCheckId::ProvenanceIdentity,
            "Provenance matches the exact World Test approval and Walk Cycle receipts.",
        ),
        (
            ExportValidationCheckId::PublishingBoundary,
            "Publishing remains not approved and requires a separate user decision.",
        ),
    ]
    .into_iter()
    .map(|(id, message)| ExportValidationCheck {
        id,
        status: ValidationStatus::Pass,
        message: message.to_owned(),
    })
    .collect();
    Ok(ExportValidationReport {
        schema_version: 1,
        validator_id: EXPORT_VALIDATOR_ID.to_owned(),
        export_id: payload.candidate.id.clone(),
        contract_id: payload.candidate.contract_id.clone(),
        checks,
        summary: ValidationSummary {
            pass: 7,
            fail: 0,
            not_assessed: 0,
        },
        publishing: payload.candidate.publishing.clone(),
    })
}

fn validation_rule(
    id: ValidationRuleId,
    status: ValidationStatus,
    expected: impl Into<String>,
    observed: Option<String>,
    message: impl Into<String>,
) -> ValidationRuleResult {
    ValidationRuleResult {
        id,
        status,
        expected: expected.into(),
        observed,
        message: message.into(),
    }
}

fn pixel_label(count: usize) -> String {
    format!("{count} pixel{}", if count == 1 { "" } else { "s" })
}

fn validate_structural_report(report: ValidationReport) -> Result<ValidationReport, String> {
    if report.schema_version != VALIDATION_REPORT_VERSION
        || report.validator_id != STRUCTURAL_VALIDATOR_ID
    {
        return Err("Unsupported validation report version.".to_owned());
    }
    validate_candidate_id(&report.candidate_id)?;
    if report.candidate_sha256.len() != 64
        || !report
            .candidate_sha256
            .chars()
            .all(|character| character.is_ascii_hexdigit() && !character.is_ascii_uppercase())
    {
        return Err("Validation candidate SHA-256 is invalid.".to_owned());
    }
    if report.contract_id != CONTRACT_ID {
        return Err("Validation contract id is incompatible.".to_owned());
    }

    let rule_order = [
        ValidationRuleId::CanvasDimensions,
        ValidationRuleId::HardAlpha,
        ValidationRuleId::ActorHeight,
        ValidationRuleId::FootAnchor,
        ValidationRuleId::PaletteMaxColors,
        ValidationRuleId::GroundLumaSeparation,
        ValidationRuleId::FrameEdgeClipping,
    ];
    if report.results.len() != rule_order.len()
        || report
            .results
            .iter()
            .zip(rule_order)
            .any(|(result, expected)| result.id != expected)
    {
        return Err("Validation rules must use the canonical order.".to_owned());
    }
    if report.results.iter().any(|result| {
        result.expected.is_empty()
            || result.message.is_empty()
            || result.observed.as_ref().is_some_and(String::is_empty)
    }) {
        return Err("Validation evidence cannot be empty.".to_owned());
    }

    let mut counted = ValidationSummary {
        pass: 0,
        fail: 0,
        not_assessed: 0,
    };
    for result in &report.results {
        match result.status {
            ValidationStatus::Pass => counted.pass += 1,
            ValidationStatus::Fail => counted.fail += 1,
            ValidationStatus::NotAssessed => counted.not_assessed += 1,
        }
    }
    if counted != report.summary {
        return Err("Validation summary does not match its rule results.".to_owned());
    }
    if report.visual_judgment.authority != "user"
        || report.visual_judgment.message.is_empty()
        || report.visual_judgment.status != VisualJudgmentStatus::NotAssessed
    {
        return Err("Validation report crossed the human approval boundary.".to_owned());
    }
    Ok(report)
}

fn validate_candidate_png(
    candidate: &ConceptCandidate,
    png_bytes: &[u8],
) -> Result<ValidationReport, String> {
    let candidate = validate_candidate(candidate.clone())?;
    validate_png_structural_evidence(
        &candidate.id,
        &candidate.sha256,
        candidate.byte_length,
        &candidate.contract_id,
        png_bytes,
        StructuralContactMode::ExactAnchor,
    )
}

fn validate_png_structural_evidence(
    artifact_id: &str,
    sha256: &str,
    byte_length: usize,
    contract_id: &str,
    png_bytes: &[u8],
    contact_mode: StructuralContactMode,
) -> Result<ValidationReport, String> {
    validate_candidate_id(artifact_id)?;
    if png_bytes.len() != byte_length || format!("{:x}", Sha256::digest(png_bytes)) != sha256 {
        return Err("Artifact source bytes no longer match immutable provenance.".to_owned());
    }
    let decoded = decode_png_rgba(png_bytes)
        .map_err(|_| "Candidate source is not a valid PNG.".to_owned())?;

    let mut visible_colors = HashSet::<[u8; 3]>::new();
    let mut edge_sides = [false; 4];
    let mut edge_pixel_count = 0usize;
    let mut semi_transparent_pixel_count = 0usize;
    let mut visible_pixel_count = 0usize;
    let mut min_y = decoded.height;
    let mut max_y = 0u32;
    let mut exact_foot_anchor_contact = false;
    let mut foot_anchor_row_contact = false;

    for (index, pixel) in decoded.pixels.iter().enumerate() {
        let x = index as u32 % decoded.width;
        let y = index as u32 / decoded.width;
        let alpha = pixel[3];

        if (1..=254).contains(&alpha) {
            semi_transparent_pixel_count += 1;
        }
        if alpha == 0 {
            continue;
        }

        visible_pixel_count += 1;
        visible_colors.insert([pixel[0], pixel[1], pixel[2]]);
        min_y = min_y.min(y);
        max_y = max_y.max(y);
        if x == FOOT_ANCHOR_X && y == FOOT_ANCHOR_Y {
            exact_foot_anchor_contact = true;
        }
        if y == WALK_GROUND_CONTACT_Y {
            foot_anchor_row_contact = true;
        }
        if x == 0 || y == 0 || x == decoded.width - 1 || y == decoded.height - 1 {
            edge_pixel_count += 1;
            if y == 0 {
                edge_sides[0] = true;
            }
            if x == decoded.width - 1 {
                edge_sides[1] = true;
            }
            if y == decoded.height - 1 {
                edge_sides[2] = true;
            }
            if x == 0 {
                edge_sides[3] = true;
            }
        }
    }

    let actor_height = if visible_pixel_count == 0 {
        0
    } else {
        max_y - min_y + 1
    };
    let dimensions_pass = decoded.width == FRAME_WIDTH && decoded.height == FRAME_HEIGHT;
    let hard_alpha_pass = semi_transparent_pixel_count == 0;
    let actor_height_pass = (ACTOR_HEIGHT_MIN..=ACTOR_HEIGHT_MAX).contains(&actor_height);
    let palette_pass = visible_colors.len() <= PALETTE_MAX_COLORS;
    let edge_pass = edge_pixel_count == 0;
    let edge_observed = if edge_pass {
        "No edge contact".to_owned()
    } else {
        let names = ["top", "right", "bottom", "left"]
            .into_iter()
            .zip(edge_sides)
            .filter_map(|(name, contacted)| contacted.then_some(name))
            .collect::<Vec<_>>()
            .join(", ");
        format!("{} on {names}", pixel_label(edge_pixel_count))
    };
    let foot_contact = match contact_mode {
        StructuralContactMode::ExactAnchor => exact_foot_anchor_contact,
        StructuralContactMode::FootAnchorRow => foot_anchor_row_contact,
    };
    let contact_expected = match contact_mode {
        StructuralContactMode::ExactAnchor => {
            format!("Visible pixel at ({FOOT_ANCHOR_X}, {FOOT_ANCHOR_Y})")
        }
        StructuralContactMode::FootAnchorRow => {
            format!("Visible pixel on foot-anchor row y={WALK_GROUND_CONTACT_Y}")
        }
    };
    let contact_message = match contact_mode {
        StructuralContactMode::ExactAnchor => {
            if foot_contact {
                "The actor contacts the contract foot anchor."
            } else {
                "The contract foot anchor is transparent."
            }
        }
        StructuralContactMode::FootAnchorRow => {
            if foot_contact {
                "The walk frame contacts the contract foot-anchor row."
            } else {
                "The contract foot-anchor row has no visible contact."
            }
        }
    };

    let results = vec![
        validation_rule(
            ValidationRuleId::CanvasDimensions,
            if dimensions_pass {
                ValidationStatus::Pass
            } else {
                ValidationStatus::Fail
            },
            format!("{FRAME_WIDTH} x {FRAME_HEIGHT} px"),
            Some(format!("{} x {} px", decoded.width, decoded.height)),
            if dimensions_pass {
                "Decoded canvas matches the contract."
            } else {
                "Decoded canvas dimensions do not match the contract."
            },
        ),
        validation_rule(
            ValidationRuleId::HardAlpha,
            if hard_alpha_pass {
                ValidationStatus::Pass
            } else {
                ValidationStatus::Fail
            },
            "Only alpha 0 or 255",
            Some(format!(
                "{} with alpha from 1 to 254",
                pixel_label(semi_transparent_pixel_count)
            )),
            if hard_alpha_pass {
                "All pixels use hard alpha."
            } else {
                "Semi-transparent pixels violate the hard-alpha contract."
            },
        ),
        validation_rule(
            ValidationRuleId::ActorHeight,
            if actor_height_pass {
                ValidationStatus::Pass
            } else {
                ValidationStatus::Fail
            },
            format!("{ACTOR_HEIGHT_MIN}-{ACTOR_HEIGHT_MAX} px"),
            Some(format!("{actor_height} px")),
            if actor_height_pass {
                "Visible actor height is within the contract range."
            } else {
                "Visible actor height is outside the contract range."
            },
        ),
        validation_rule(
            ValidationRuleId::FootAnchor,
            if foot_contact {
                ValidationStatus::Pass
            } else {
                ValidationStatus::Fail
            },
            contact_expected,
            Some(
                if foot_contact {
                    "Contact"
                } else {
                    "No contact"
                }
                .to_owned(),
            ),
            contact_message,
        ),
        validation_rule(
            ValidationRuleId::PaletteMaxColors,
            if palette_pass {
                ValidationStatus::Pass
            } else {
                ValidationStatus::Fail
            },
            format!("{PALETTE_MAX_COLORS} visible RGB colors or fewer"),
            Some(format!(
                "{} visible RGB color{}",
                visible_colors.len(),
                if visible_colors.len() == 1 { "" } else { "s" }
            )),
            if palette_pass {
                "Visible palette is within the contract maximum."
            } else {
                "Visible palette exceeds the contract maximum."
            },
        ),
        validation_rule(
            ValidationRuleId::GroundLumaSeparation,
            ValidationStatus::NotAssessed,
            format!("At least {MINIMUM_GROUND_LUMA_DISTANCE} luma from pinned ground"),
            None,
            "A pinned ground reference is required before this rule can be measured.",
        ),
        validation_rule(
            ValidationRuleId::FrameEdgeClipping,
            if edge_pass {
                ValidationStatus::Pass
            } else {
                ValidationStatus::Fail
            },
            "No visible pixels on the frame edge",
            Some(edge_observed),
            if edge_pass {
                "No visible pixel touches the frame edge."
            } else {
                "Visible edge contact indicates possible clipping."
            },
        ),
    ];

    let mut summary = ValidationSummary {
        pass: 0,
        fail: 0,
        not_assessed: 0,
    };
    for result in &results {
        match result.status {
            ValidationStatus::Pass => summary.pass += 1,
            ValidationStatus::Fail => summary.fail += 1,
            ValidationStatus::NotAssessed => summary.not_assessed += 1,
        }
    }

    validate_structural_report(ValidationReport {
        schema_version: VALIDATION_REPORT_VERSION,
        validator_id: STRUCTURAL_VALIDATOR_ID.to_owned(),
        candidate_id: artifact_id.to_owned(),
        candidate_sha256: sha256.to_owned(),
        contract_id: contract_id.to_owned(),
        results,
        summary,
        visual_judgment: VisualJudgment {
            status: VisualJudgmentStatus::NotAssessed,
            authority: "user".to_owned(),
            message: "Only the user can make the visual-acceptance decision.".to_owned(),
        },
    })
}

#[tauri::command]
fn studio_status() -> serde_json::Value {
    serde_json::json!({
        "name": "TileForge Actor Studio",
        "version": env!("CARGO_PKG_VERSION"),
        "contract": CONTRACT_ID,
        "approvalOwner": "user"
    })
}

#[tauri::command]
fn create_sprite_session(brief: ActorBrief) -> Result<StudioSession, String> {
    let timestamp = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
    let id_suffix = Uuid::new_v4().simple().to_string()[..8].to_owned();
    let temporary_suffix = Uuid::new_v4().simple().to_string();
    create_session_at(
        &workspace_root(),
        brief,
        &timestamp,
        &id_suffix,
        &temporary_suffix,
    )
}

#[tauri::command]
fn get_sprite_session(session_id: String) -> Result<StudioSession, String> {
    read_session(&workspace_root(), &session_id)
}

#[tauri::command]
fn list_sprite_sessions() -> Result<SessionList, String> {
    let root = workspace_root();
    let sessions_root = root.join("sessions");
    fs::create_dir_all(&sessions_root)
        .map_err(|error| format!("Could not create session storage: {error}"))?;

    let mut sessions = Vec::new();
    for entry in
        fs::read_dir(&sessions_root).map_err(|error| format!("Could not list sessions: {error}"))?
    {
        let Ok(entry) = entry else {
            continue;
        };
        let file_name = entry.file_name();
        let Some(session_id) = file_name.to_str() else {
            continue;
        };
        if session_id.starts_with('.') || !entry.path().is_dir() {
            continue;
        }
        if let Ok(session) = read_session(&root, session_id) {
            sessions.push(session);
        }
    }
    sessions.sort_by(|left, right| right.updated_at.cmp(&left.updated_at));

    Ok(SessionList {
        workspace_root: root,
        sessions,
    })
}

#[tauri::command]
fn create_concept_generation_request(
    session_id: String,
    requested_candidates: u32,
) -> Result<ConceptGenerationRequest, String> {
    let timestamp = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
    let id_suffix = Uuid::new_v4().simple().to_string()[..8].to_owned();
    let temporary_suffix = Uuid::new_v4().simple().to_string();
    create_concept_generation_request_at(
        &workspace_root(),
        &session_id,
        requested_candidates,
        &timestamp,
        &id_suffix,
        &temporary_suffix,
        None,
    )
}

#[tauri::command]
fn list_concept_generation_requests(
    session_id: String,
) -> Result<Vec<ConceptGenerationRequest>, String> {
    list_generation_requests_at(&workspace_root(), &session_id)
}

#[tauri::command]
fn get_concept_generation_request(
    session_id: String,
    request_id: String,
) -> Result<ConceptGenerationRequest, String> {
    read_generation_request(&workspace_root(), &session_id, &request_id)
}

#[tauri::command]
fn import_concept_candidate(
    session_id: String,
    png_bytes: Vec<u8>,
    provenance: CandidateProvenance,
) -> Result<ConceptCandidate, String> {
    let timestamp = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
    let id_suffix = Uuid::new_v4().simple().to_string()[..8].to_owned();
    let temporary_suffix = Uuid::new_v4().simple().to_string();
    create_concept_candidate_at(
        &workspace_root(),
        &session_id,
        &png_bytes,
        provenance,
        &timestamp,
        &id_suffix,
        &temporary_suffix,
        None,
    )
}

#[tauri::command]
fn list_concept_candidates(session_id: String) -> Result<Vec<ConceptCandidate>, String> {
    list_candidates_at(&workspace_root(), &session_id)
}

#[tauri::command]
fn get_concept_candidate(
    session_id: String,
    candidate_id: String,
) -> Result<ConceptCandidatePayload, String> {
    read_candidate_payload(&workspace_root(), &session_id, &candidate_id)
}

#[tauri::command]
fn validate_concept_candidate(
    session_id: String,
    candidate_id: String,
) -> Result<ValidationReport, String> {
    let payload = read_candidate_payload(&workspace_root(), &session_id, &candidate_id)?;
    validate_candidate_png(&payload.candidate, &payload.png_bytes)
}

#[tauri::command]
fn create_turnaround_candidate(
    session_id: String,
    source_concept_id: String,
    png_bytes: TurnaroundPngBytes,
    provenance: CandidateProvenance,
) -> Result<TurnaroundCandidate, String> {
    let timestamp = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
    let id_suffix = Uuid::new_v4().simple().to_string()[..8].to_owned();
    let temporary_suffix = Uuid::new_v4().simple().to_string();
    create_turnaround_candidate_at(
        &workspace_root(),
        &session_id,
        &source_concept_id,
        &png_bytes,
        provenance,
        &timestamp,
        &id_suffix,
        &temporary_suffix,
        None,
    )
}

#[tauri::command]
fn list_turnaround_candidates(session_id: String) -> Result<Vec<TurnaroundCandidate>, String> {
    list_turnarounds_at(&workspace_root(), &session_id)
}

#[tauri::command]
fn get_turnaround_candidate(
    session_id: String,
    turnaround_id: String,
) -> Result<TurnaroundCandidatePayload, String> {
    read_turnaround_payload(&workspace_root(), &session_id, &turnaround_id)
}

#[tauri::command]
fn validate_turnaround_candidate(
    session_id: String,
    turnaround_id: String,
) -> Result<TurnaroundValidationReport, String> {
    let payload = read_turnaround_payload(&workspace_root(), &session_id, &turnaround_id)?;
    validate_turnaround_pngs(&payload)
}

#[tauri::command]
fn create_walk_cycle_candidate(
    session_id: String,
    source_turnaround_id: String,
    png_bytes: WalkCyclePngBytes,
    provenance: CandidateProvenance,
) -> Result<WalkCycleCandidate, String> {
    let timestamp = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
    let id_suffix = Uuid::new_v4().simple().to_string()[..8].to_owned();
    let temporary_suffix = Uuid::new_v4().simple().to_string();
    create_walk_cycle_candidate_at(
        &workspace_root(),
        &session_id,
        &source_turnaround_id,
        &png_bytes,
        provenance,
        &timestamp,
        &id_suffix,
        &temporary_suffix,
        None,
    )
}

#[tauri::command]
fn list_walk_cycle_candidates(session_id: String) -> Result<Vec<WalkCycleCandidate>, String> {
    list_walk_cycles_at(&workspace_root(), &session_id)
}

#[tauri::command]
fn get_walk_cycle_candidate(
    session_id: String,
    walk_cycle_id: String,
) -> Result<WalkCycleCandidatePayload, String> {
    read_walk_cycle_payload(&workspace_root(), &session_id, &walk_cycle_id)
}

#[tauri::command]
fn validate_walk_cycle_candidate(
    session_id: String,
    walk_cycle_id: String,
) -> Result<WalkCycleValidationReport, String> {
    let payload = read_walk_cycle_payload(&workspace_root(), &session_id, &walk_cycle_id)?;
    validate_walk_cycle_pngs(&payload)
}

#[tauri::command]
fn create_world_test_candidate(
    session_id: String,
    source_walk_cycle_id: String,
) -> Result<WorldTestCandidate, String> {
    let timestamp = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
    let id_suffix = Uuid::new_v4().simple().to_string()[..8].to_owned();
    let temporary_suffix = Uuid::new_v4().simple().to_string();
    create_world_test_candidate_at(
        &workspace_root(),
        &session_id,
        &source_walk_cycle_id,
        &timestamp,
        &id_suffix,
        &temporary_suffix,
        None,
    )
}

#[tauri::command]
fn list_world_test_candidates(session_id: String) -> Result<Vec<WorldTestCandidate>, String> {
    list_world_tests_at(&workspace_root(), &session_id)
}

#[tauri::command]
fn get_world_test_candidate(
    session_id: String,
    world_test_id: String,
) -> Result<WorldTestCandidatePayload, String> {
    read_world_test_payload(&workspace_root(), &session_id, &world_test_id)
}

#[tauri::command]
fn validate_world_test_candidate(
    session_id: String,
    world_test_id: String,
) -> Result<WorldTestValidationReport, String> {
    let root = workspace_root();
    let payload = read_world_test_payload(&root, &session_id, &world_test_id)?;
    validate_world_test_pngs(&root, &payload)
}

#[tauri::command]
fn create_export_candidate(
    session_id: String,
    source_world_test_id: String,
) -> Result<ExportCandidate, String> {
    let timestamp = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
    let id_suffix = Uuid::new_v4().simple().to_string()[..8].to_owned();
    let temporary_suffix = Uuid::new_v4().simple().to_string();
    create_export_candidate_at(
        &workspace_root(),
        &session_id,
        &source_world_test_id,
        &timestamp,
        &id_suffix,
        &temporary_suffix,
        None,
    )
}

#[tauri::command]
fn list_export_candidates(session_id: String) -> Result<Vec<ExportCandidate>, String> {
    list_exports_at(&workspace_root(), &session_id)
}

#[tauri::command]
fn get_export_candidate(
    session_id: String,
    export_id: String,
) -> Result<ExportCandidatePayload, String> {
    read_export_payload(&workspace_root(), &session_id, &export_id)
}

#[tauri::command]
fn validate_export_candidate(
    session_id: String,
    export_id: String,
) -> Result<ExportValidationReport, String> {
    let root = workspace_root();
    let payload = read_export_payload(&root, &session_id, &export_id)?;
    validate_export_package(&root, &payload)
}

#[tauri::command]
fn open_export_folder(session_id: String, export_id: String) -> Result<String, String> {
    open_export_folder_at(
        &workspace_root(),
        &session_id,
        &export_id,
        launch_export_folder,
    )
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            studio_status,
            create_sprite_session,
            get_sprite_session,
            list_sprite_sessions,
            create_concept_generation_request,
            list_concept_generation_requests,
            get_concept_generation_request,
            import_concept_candidate,
            list_concept_candidates,
            get_concept_candidate,
            validate_concept_candidate,
            create_turnaround_candidate,
            list_turnaround_candidates,
            get_turnaround_candidate,
            validate_turnaround_candidate,
            create_walk_cycle_candidate,
            list_walk_cycle_candidates,
            get_walk_cycle_candidate,
            validate_walk_cycle_candidate,
            create_world_test_candidate,
            list_world_test_candidates,
            get_world_test_candidate,
            validate_world_test_candidate,
            create_export_candidate,
            list_export_candidates,
            get_export_candidate,
            validate_export_candidate,
            open_export_folder
        ])
        .run(tauri::generate_context!())
        .expect("error while running TileForge Actor Studio");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_root(label: &str) -> PathBuf {
        env::temp_dir().join(format!("tfas-{label}-{}", Uuid::new_v4().simple()))
    }

    fn brief() -> ActorBrief {
        ActorBrief {
            name: "Mirelight Pilgrim".to_owned(),
            kind: ActorKind::Mob,
            description: "A quiet reed-cloaked marsh pilgrim.".to_owned(),
        }
    }

    fn provenance(source: CandidateSource) -> CandidateProvenance {
        CandidateProvenance {
            source,
            original_filename: Some("mirelight-pilgrim.png".to_owned()),
            provider: None,
            model: None,
        }
    }

    fn concept_png(width: u32, height: u32, transparent: bool) -> Vec<u8> {
        let mut pixels = vec![0; (width * height * 4) as usize];
        for pixel in pixels.chunks_exact_mut(4) {
            pixel.copy_from_slice(&[42, 74, 55, if transparent { 0 } else { 255 }]);
        }
        if !pixels.is_empty() {
            let center = ((height / 2 * width + width / 2) * 4) as usize;
            pixels[center..center + 4].copy_from_slice(&[104, 198, 178, 255]);
        }

        let mut bytes = Vec::new();
        {
            let mut encoder = png::Encoder::new(&mut bytes, width, height);
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);
            encoder
                .write_header()
                .unwrap()
                .write_image_data(&pixels)
                .unwrap();
        }
        bytes
    }

    fn validation_png(failing: bool, width: u32, height: u32) -> Vec<u8> {
        let mut pixels = vec![0; (width * height * 4) as usize];
        let first_y = if failing { 12 } else { 5 };
        let mut color_index = 0u8;

        for y in first_y..=28.min(height.saturating_sub(1)) {
            for x in 10..=21.min(width.saturating_sub(1)) {
                let index = ((y * width + x) * 4) as usize;
                let color = if failing {
                    let color = color_index % 20;
                    color_index = color_index.wrapping_add(1);
                    [color * 11, 40 + color * 7, 220 - color * 9, 255]
                } else if y % 2 == 0 {
                    [104, 198, 178, 255]
                } else {
                    [42, 74, 55, 255]
                };
                pixels[index..index + 4].copy_from_slice(&color);
            }
        }

        if failing {
            pixels[((12 * width + 10) * 4 + 3) as usize] = 128;
            pixels[((28 * width + 16) * 4 + 3) as usize] = 0;
            let edge = (20 * width * 4) as usize;
            pixels[edge..edge + 4].copy_from_slice(&[42, 74, 55, 255]);
            let bottom_edge = ((31 * width + 15) * 4) as usize;
            pixels[bottom_edge..bottom_edge + 4].copy_from_slice(&[42, 74, 55, 255]);
        }

        let mut bytes = Vec::new();
        {
            let mut encoder = png::Encoder::new(&mut bytes, width, height);
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);
            encoder
                .write_header()
                .unwrap()
                .write_image_data(&pixels)
                .unwrap();
        }
        bytes
    }

    fn walk_frame_png(source: &[u8], include_ground_contact: bool) -> Vec<u8> {
        let mut decoded = decode_png_rgba(source).unwrap();
        let anchor_index = (FOOT_ANCHOR_Y * decoded.width + FOOT_ANCHOR_X) as usize;
        decoded.pixels[anchor_index][3] = 0;
        if !include_ground_contact {
            for x in 0..decoded.width {
                let index = (WALK_GROUND_CONTACT_Y * decoded.width + x) as usize;
                decoded.pixels[index][3] = 0;
            }
        }
        encode_png_rgba(decoded.width, decoded.height, &decoded.pixels).unwrap()
    }

    #[test]
    fn desktop_adapter_reads_shared_session_fixture() {
        let root = test_root("fixture");
        let fixture = include_str!("../../tests/fixtures/session-v1.json");
        let fixture_session: StudioSession = serde_json::from_str(fixture).unwrap();
        let directory = root.join("sessions").join(&fixture_session.id);
        fs::create_dir_all(directory.join("candidates")).unwrap();
        fs::write(directory.join("session.json"), fixture).unwrap();

        let reread = read_session(&root, &fixture_session.id).unwrap();
        assert_eq!(reread.id, fixture_session.id);
        assert_eq!(reread.contract_id, CONTRACT_ID);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn desktop_adapter_reads_shared_generation_request_fixture() {
        let root = test_root("generation-request-fixture");
        let session_fixture = include_str!("../../tests/fixtures/session-v1.json");
        let session: StudioSession = serde_json::from_str(session_fixture).unwrap();
        let request_fixture =
            include_str!("../../tests/fixtures/concept-generation-request-v1.json");
        let request: ConceptGenerationRequest = serde_json::from_str(request_fixture).unwrap();
        let request = validate_generation_request(request).unwrap();
        let request_directory = root
            .join("sessions")
            .join(&session.id)
            .join("generation-requests")
            .join(&request.id);
        fs::create_dir_all(&request_directory).unwrap();
        fs::write(
            root.join("sessions").join(&session.id).join("session.json"),
            session_fixture,
        )
        .unwrap();
        fs::write(request_directory.join("request.json"), request_fixture).unwrap();

        let reread = read_generation_request(&root, &session.id, &request.id).unwrap();
        assert_eq!(reread.id, request.id);
        assert_eq!(reread.execution.additional_paid_services, "forbidden");
        assert!(!reread.authority.agents_may_approve);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn generation_request_is_immutable_and_failure_cleans_up() {
        let root = test_root("generation-request");
        let session = create_session_at(
            &root,
            brief(),
            "2026-07-27T12:00:00.000Z",
            "request1",
            "session",
        )
        .unwrap();
        let first = create_concept_generation_request_at(
            &root,
            &session.id,
            3,
            "2026-07-27T12:01:00.000Z",
            "request1",
            "first",
            None,
        )
        .unwrap();
        assert!(first.prompt.contains("Only the user may approve final art"));
        assert_eq!(first.execution.api_credentials, "not-used");

        let collision = create_concept_generation_request_at(
            &root,
            &session.id,
            3,
            "2026-07-27T12:01:00.000Z",
            "request1",
            "collision",
            Some(1),
        );
        assert!(collision.is_err());
        let invalid_count = create_concept_generation_request_at(
            &root,
            &session.id,
            5,
            "2026-07-27T12:02:00.000Z",
            "invalid1",
            "invalid",
            None,
        );
        assert!(invalid_count.is_err());

        let entries: Vec<_> = fs::read_dir(
            root.join("sessions")
                .join(&session.id)
                .join("generation-requests"),
        )
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .collect();
        assert_eq!(entries, vec![OsString::from(first.id)]);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn workspace_root_prefers_override_then_windows_local_app_data() {
        let override_root = PathBuf::from(r"D:\TileForge Workspace");
        let local_app_data = PathBuf::from(r"C:\Users\artist\AppData\Local");

        assert_eq!(
            workspace_root_from(
                Some(override_root.clone().into_os_string()),
                Some(local_app_data.clone().into_os_string()),
            ),
            override_root
        );
        assert_eq!(
            workspace_root_from(None, Some(local_app_data.clone().into_os_string())),
            if cfg!(target_os = "windows") {
                local_app_data
                    .join(WINDOWS_VENDOR_DIRECTORY)
                    .join(WINDOWS_PRODUCT_DIRECTORY)
                    .join(".studio")
            } else {
                repository_workspace_root()
            }
        );
    }

    #[test]
    fn invalid_brief_creates_no_partial_storage() {
        let root = test_root("invalid");
        let mut invalid = brief();
        invalid.name = " ".to_owned();

        let result = create_session_at(
            &root,
            invalid,
            "2026-07-26T21:01:40.000Z",
            "invalid1",
            "temporary",
        );

        assert!(result.is_err());
        assert!(!root.exists());
    }

    #[test]
    fn collision_does_not_overwrite_or_leave_temporary_directory() {
        let root = test_root("collision");
        let timestamp = "2026-07-26T21:01:40.000Z";
        let first = create_session_at(&root, brief(), timestamp, "same0001", "first").unwrap();
        let second = create_session_at(&root, brief(), timestamp, "same0001", "second");

        assert!(second.is_err());
        let entries: Vec<_> = fs::read_dir(root.join("sessions"))
            .unwrap()
            .map(|entry| entry.unwrap().file_name())
            .collect();
        assert_eq!(entries, vec![std::ffi::OsString::from(first.id)]);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn desktop_adapter_reads_shared_candidate_fixture() {
        let fixture = include_str!("../../tests/fixtures/concept-candidate-v1.json");
        let candidate: ConceptCandidate = serde_json::from_str(fixture).unwrap();
        let candidate = validate_candidate(candidate).unwrap();

        assert_eq!(candidate.contract_id, CONTRACT_ID);
        assert_eq!(candidate.review_status, "unreviewed");
    }

    #[test]
    fn concept_candidate_preserves_original_bytes() {
        let root = test_root("candidate");
        let session = create_session_at(
            &root,
            brief(),
            "2026-07-26T21:30:00.000Z",
            "candidate",
            "session",
        )
        .unwrap();
        let bytes = concept_png(32, 32, true);
        let candidate = create_concept_candidate_at(
            &root,
            &session.id,
            &bytes,
            provenance(CandidateSource::Imported),
            "2026-07-26T21:31:00.000Z",
            "native01",
            "candidate",
            None,
        )
        .unwrap();
        let payload = read_candidate_payload(&root, &session.id, &candidate.id).unwrap();

        assert_eq!(payload.png_bytes, bytes);
        assert_eq!(payload.candidate.review_status, "unreviewed");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn malformed_candidate_intake_leaves_no_partial_directory() {
        let root = test_root("candidate-invalid");
        let session = create_session_at(
            &root,
            brief(),
            "2026-07-26T21:30:00.000Z",
            "invalidc",
            "session",
        )
        .unwrap();

        for bytes in [
            concept_png(31, 32, true),
            concept_png(32, 32, false),
            b"not a png".to_vec(),
        ] {
            let result = create_concept_candidate_at(
                &root,
                &session.id,
                &bytes,
                provenance(CandidateSource::Imported),
                "2026-07-26T21:31:00.000Z",
                "invalid1",
                "candidate",
                None,
            );
            assert!(result.is_err());
        }

        assert_eq!(
            fs::read_dir(root.join("sessions").join(&session.id).join("candidates"))
                .unwrap()
                .count(),
            0
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn candidate_collision_does_not_overwrite_or_leave_temporary_directory() {
        let root = test_root("candidate-collision");
        let session = create_session_at(
            &root,
            brief(),
            "2026-07-26T21:30:00.000Z",
            "collidec",
            "session",
        )
        .unwrap();
        let bytes = concept_png(32, 32, true);
        let first = create_concept_candidate_at(
            &root,
            &session.id,
            &bytes,
            provenance(CandidateSource::Imported),
            "2026-07-26T21:31:00.000Z",
            "same0001",
            "first",
            Some(1),
        )
        .unwrap();
        let second = create_concept_candidate_at(
            &root,
            &session.id,
            &concept_png(32, 32, true),
            provenance(CandidateSource::Imported),
            "2026-07-26T21:31:00.000Z",
            "same0001",
            "second",
            Some(1),
        );

        assert!(second.is_err());
        let entries: Vec<_> =
            fs::read_dir(root.join("sessions").join(&session.id).join("candidates"))
                .unwrap()
                .map(|entry| entry.unwrap().file_name())
                .collect();
        assert_eq!(entries, vec![std::ffi::OsString::from(first.id)]);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn structural_validation_matches_shared_fixture_and_decoded_dimensions() {
        let root = test_root("validation-fixture");
        let session = create_session_at(
            &root,
            brief(),
            "2026-07-26T22:00:00.000Z",
            "validatr",
            "session",
        )
        .unwrap();
        let bytes = validation_png(false, 32, 32);
        let candidate = create_concept_candidate_at(
            &root,
            &session.id,
            &bytes,
            provenance(CandidateSource::Imported),
            "2026-07-26T22:01:00.000Z",
            "passing1",
            "candidate",
            None,
        )
        .unwrap();
        let payload = read_candidate_payload(&root, &session.id, &candidate.id).unwrap();
        let report = validate_candidate_png(&payload.candidate, &payload.png_bytes).unwrap();
        let fixture: ValidationReport = serde_json::from_str(include_str!(
            "../../tests/fixtures/validation-report-v1.json"
        ))
        .unwrap();
        let fixture = validate_structural_report(fixture).unwrap();

        assert_eq!(report.results, fixture.results);
        assert_eq!(report.summary, fixture.summary);
        assert_eq!(report.visual_judgment, fixture.visual_judgment);
        assert_eq!(
            fs::read_dir(
                root.join("sessions")
                    .join(&session.id)
                    .join("candidates")
                    .join(&candidate.id)
            )
            .unwrap()
            .count(),
            2
        );

        let narrow_bytes = validation_png(false, 31, 32);
        let mut narrow_candidate = candidate;
        narrow_candidate.sha256 = format!("{:x}", Sha256::digest(&narrow_bytes));
        narrow_candidate.byte_length = narrow_bytes.len();
        let narrow_report = validate_candidate_png(&narrow_candidate, &narrow_bytes).unwrap();
        assert_eq!(
            narrow_report.results[0].status,
            ValidationStatus::Fail,
            "validator trusted candidate metadata instead of decoded dimensions"
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn structural_validation_reports_independent_failures() {
        let root = test_root("validation-failures");
        let session = create_session_at(
            &root,
            brief(),
            "2026-07-26T22:00:00.000Z",
            "failures",
            "session",
        )
        .unwrap();
        let bytes = validation_png(true, 32, 32);
        let candidate = create_concept_candidate_at(
            &root,
            &session.id,
            &bytes,
            provenance(CandidateSource::Imported),
            "2026-07-26T22:02:00.000Z",
            "failing1",
            "candidate",
            None,
        )
        .unwrap();
        let report = validate_candidate_png(&candidate, &bytes).unwrap();
        let failed_rules = report
            .results
            .iter()
            .filter(|result| result.status == ValidationStatus::Fail)
            .map(|result| result.id)
            .collect::<Vec<_>>();

        assert_eq!(
            failed_rules,
            vec![
                ValidationRuleId::HardAlpha,
                ValidationRuleId::ActorHeight,
                ValidationRuleId::FootAnchor,
                ValidationRuleId::PaletteMaxColors,
                ValidationRuleId::FrameEdgeClipping,
            ]
        );
        assert_eq!(
            report.summary,
            ValidationSummary {
                pass: 1,
                fail: 5,
                not_assessed: 1,
            }
        );
        assert_eq!(
            report
                .results
                .iter()
                .find(|result| result.id == ValidationRuleId::FrameEdgeClipping)
                .and_then(|result| result.observed.as_deref()),
            Some("2 pixels on bottom, left")
        );
        assert_eq!(
            report.visual_judgment.status,
            VisualJudgmentStatus::NotAssessed
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn desktop_adapter_reads_shared_turnaround_fixture() {
        let fixture = include_str!("../../tests/fixtures/turnaround-candidate-v1.json");
        let candidate: TurnaroundCandidate = serde_json::from_str(fixture).unwrap();
        let candidate = validate_turnaround(candidate).unwrap();

        assert_eq!(candidate.contract_id, CONTRACT_ID);
        assert_eq!(candidate.stage, "turnaround");
        assert_eq!(candidate.source_selection.selected_by, "user");
        assert_eq!(
            candidate.identity_judgment.status,
            VisualJudgmentStatus::NotAssessed
        );
    }

    #[test]
    fn turnaround_preserves_selected_concept_and_four_direction_bytes() {
        let root = test_root("turnaround");
        let session = create_session_at(
            &root,
            brief(),
            "2026-07-27T20:00:00.000Z",
            "turnrond",
            "session",
        )
        .unwrap();
        let down = validation_png(false, 32, 32);
        let concept = create_concept_candidate_at(
            &root,
            &session.id,
            &down,
            provenance(CandidateSource::Imported),
            "2026-07-27T20:01:00.000Z",
            "selected",
            "candidate",
            None,
        )
        .unwrap();
        let views = TurnaroundPngBytes {
            down: down.clone(),
            right: down.clone(),
            up: down.clone(),
            left: down.clone(),
        };
        let generated = CandidateProvenance {
            source: CandidateSource::Generated,
            original_filename: None,
            provider: Some("subscription-image-tool".to_owned()),
            model: Some("built-in".to_owned()),
        };
        let turnaround = create_turnaround_candidate_at(
            &root,
            &session.id,
            &concept.id,
            &views,
            generated.clone(),
            "2026-07-27T20:30:00.000Z",
            "native01",
            "turnaround",
            None,
        )
        .unwrap();
        let payload = read_turnaround_payload(&root, &session.id, &turnaround.id).unwrap();
        let report = validate_turnaround_pngs(&payload).unwrap();

        assert_eq!(payload.png_bytes.down, down);
        assert_eq!(payload.png_bytes.right, views.right);
        assert_eq!(payload.png_bytes.up, views.up);
        assert_eq!(payload.png_bytes.left, views.left);
        assert_eq!(payload.candidate.source_selection.candidate_id, concept.id);
        assert_eq!(payload.candidate.source_selection.selected_by, "user");
        assert_eq!(payload.candidate.review_status, "unreviewed");
        assert_eq!(
            payload.candidate.identity_judgment.status,
            VisualJudgmentStatus::NotAssessed
        );
        assert_eq!(
            report.summary,
            ValidationSummary {
                pass: 24,
                fail: 0,
                not_assessed: 4,
            }
        );
        assert_eq!(
            fs::read_dir(
                root.join("sessions")
                    .join(&session.id)
                    .join("turnarounds")
                    .join(&turnaround.id)
            )
            .unwrap()
            .count(),
            5
        );

        let collision = create_turnaround_candidate_at(
            &root,
            &session.id,
            &concept.id,
            &views,
            generated.clone(),
            "2026-07-27T20:31:00.000Z",
            "same0001",
            "first",
            Some(2),
        )
        .unwrap();
        let second = create_turnaround_candidate_at(
            &root,
            &session.id,
            &concept.id,
            &views,
            generated,
            "2026-07-27T20:31:00.000Z",
            "same0001",
            "second",
            Some(2),
        );
        assert!(second.is_err());
        assert_eq!(
            read_turnaround_payload(&root, &session.id, &collision.id)
                .unwrap()
                .png_bytes
                .down,
            down
        );
        assert!(
            fs::read_dir(root.join("sessions").join(&session.id).join("turnarounds"))
                .unwrap()
                .all(|entry| !entry
                    .unwrap()
                    .file_name()
                    .to_string_lossy()
                    .starts_with('.'))
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn turnaround_rejects_down_replacement_without_partial_storage() {
        let root = test_root("turnaround-invalid");
        let session = create_session_at(
            &root,
            brief(),
            "2026-07-27T20:00:00.000Z",
            "invalidt",
            "session",
        )
        .unwrap();
        let down = validation_png(false, 32, 32);
        let concept = create_concept_candidate_at(
            &root,
            &session.id,
            &down,
            provenance(CandidateSource::Imported),
            "2026-07-27T20:01:00.000Z",
            "selected",
            "candidate",
            None,
        )
        .unwrap();
        let replacement = concept_png(32, 32, true);
        let views = TurnaroundPngBytes {
            down: replacement,
            right: down.clone(),
            up: down.clone(),
            left: down,
        };
        let result = create_turnaround_candidate_at(
            &root,
            &session.id,
            &concept.id,
            &views,
            CandidateProvenance {
                source: CandidateSource::Generated,
                original_filename: None,
                provider: Some("subscription-image-tool".to_owned()),
                model: None,
            },
            "2026-07-27T20:30:00.000Z",
            "invalid1",
            "turnaround",
            None,
        );

        assert!(result.is_err());
        assert!(!root
            .join("sessions")
            .join(&session.id)
            .join("turnarounds")
            .exists());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn desktop_adapter_reads_shared_walk_cycle_fixture() {
        let fixture = include_str!("../../tests/fixtures/walk-cycle-candidate-v1.json");
        let candidate: WalkCycleCandidate = serde_json::from_str(fixture).unwrap();
        let candidate = validate_walk_cycle(candidate).unwrap();

        assert_eq!(candidate.contract_id, CONTRACT_ID);
        assert_eq!(candidate.stage, "animate");
        assert_eq!(candidate.source_turnaround.accepted_by, "user");
        assert_eq!(candidate.frames.len(), 16);
        assert_eq!(candidate.frame_duration_ms, WALK_CYCLE_FRAME_DURATION_MS);
        assert_eq!(
            candidate.motion_judgment.status,
            VisualJudgmentStatus::NotAssessed
        );
    }

    #[test]
    fn walk_cycle_preserves_accepted_turnaround_and_sixteen_frame_bytes() {
        let root = test_root("walk-cycle");
        let session = create_session_at(
            &root,
            brief(),
            "2026-07-27T23:00:00.000Z",
            "walkcycl",
            "session",
        )
        .unwrap();
        let down = validation_png(false, 32, 32);
        let concept = create_concept_candidate_at(
            &root,
            &session.id,
            &down,
            provenance(CandidateSource::Imported),
            "2026-07-27T23:01:00.000Z",
            "selected",
            "candidate",
            None,
        )
        .unwrap();
        let views = TurnaroundPngBytes {
            down: down.clone(),
            right: down.clone(),
            up: down.clone(),
            left: down.clone(),
        };
        let turnaround = create_turnaround_candidate_at(
            &root,
            &session.id,
            &concept.id,
            &views,
            CandidateProvenance {
                source: CandidateSource::Generated,
                original_filename: None,
                provider: Some("subscription-image-tool".to_owned()),
                model: Some("built-in".to_owned()),
            },
            "2026-07-27T23:02:00.000Z",
            "accepted",
            "turnaround",
            None,
        )
        .unwrap();
        let frames = WalkCyclePngBytes {
            down: vec![
                down.clone(),
                walk_frame_png(&down, true),
                walk_frame_png(&down, true),
                walk_frame_png(&down, true),
            ],
            right: vec![
                views.right.clone(),
                walk_frame_png(&views.right, true),
                walk_frame_png(&views.right, true),
                walk_frame_png(&views.right, true),
            ],
            up: vec![
                views.up.clone(),
                walk_frame_png(&views.up, true),
                walk_frame_png(&views.up, true),
                walk_frame_png(&views.up, true),
            ],
            left: vec![
                views.left.clone(),
                walk_frame_png(&views.left, true),
                walk_frame_png(&views.left, true),
                walk_frame_png(&views.left, true),
            ],
        };
        let walk_cycle = create_walk_cycle_candidate_at(
            &root,
            &session.id,
            &turnaround.id,
            &frames,
            provenance(CandidateSource::Imported),
            "2026-07-27T23:30:00.000Z",
            "native01",
            "walk-cycle",
            None,
        )
        .unwrap();
        let payload = read_walk_cycle_payload(&root, &session.id, &walk_cycle.id).unwrap();
        let report = validate_walk_cycle_pngs(&payload).unwrap();

        assert_eq!(payload.png_bytes.down[0], down);
        assert_eq!(
            payload.candidate.source_turnaround.turnaround_id,
            turnaround.id
        );
        assert_eq!(payload.candidate.source_turnaround.accepted_by, "user");
        assert_eq!(payload.candidate.review_status, "unreviewed");
        assert_eq!(
            payload.candidate.motion_judgment.status,
            VisualJudgmentStatus::NotAssessed
        );
        assert_eq!(
            report.summary,
            ValidationSummary {
                pass: 96,
                fail: 0,
                not_assessed: 16,
            }
        );
        for frame in report.frames.iter().filter(|frame| frame.frame_index > 0) {
            let contact = frame
                .report
                .results
                .iter()
                .find(|result| result.id == ValidationRuleId::FootAnchor)
                .unwrap();
            assert_eq!(contact.status, ValidationStatus::Pass);
            assert_eq!(
                contact.expected,
                format!("Visible pixel on foot-anchor row y={WALK_GROUND_CONTACT_Y}")
            );
        }

        let ungrounded = walk_frame_png(&down, false);
        let ungrounded_report = validate_png_structural_evidence(
            "walk-ground-contact",
            &format!("{:x}", Sha256::digest(&ungrounded)),
            ungrounded.len(),
            CONTRACT_ID,
            &ungrounded,
            StructuralContactMode::FootAnchorRow,
        )
        .unwrap();
        assert_eq!(
            ungrounded_report
                .results
                .iter()
                .find(|result| result.id == ValidationRuleId::FootAnchor)
                .unwrap()
                .status,
            ValidationStatus::Fail
        );
        assert_eq!(
            fs::read_dir(
                root.join("sessions")
                    .join(&session.id)
                    .join("walk-cycles")
                    .join(&walk_cycle.id)
            )
            .unwrap()
            .count(),
            17
        );

        let collision = create_walk_cycle_candidate_at(
            &root,
            &session.id,
            &turnaround.id,
            &frames,
            provenance(CandidateSource::Imported),
            "2026-07-27T23:31:00.000Z",
            "same0001",
            "first",
            Some(2),
        )
        .unwrap();
        let second = create_walk_cycle_candidate_at(
            &root,
            &session.id,
            &turnaround.id,
            &frames,
            provenance(CandidateSource::Imported),
            "2026-07-27T23:31:00.000Z",
            "same0001",
            "second",
            Some(2),
        );
        assert!(second.is_err());
        assert_eq!(
            read_walk_cycle_payload(&root, &session.id, &collision.id)
                .unwrap()
                .png_bytes
                .down[0],
            frames.down[0]
        );
        assert!(
            fs::read_dir(root.join("sessions").join(&session.id).join("walk-cycles"))
                .unwrap()
                .all(|entry| !entry
                    .unwrap()
                    .file_name()
                    .to_string_lossy()
                    .starts_with('.'))
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn walk_cycle_rejects_frame_zero_replacement_without_partial_storage() {
        let root = test_root("walk-cycle-invalid");
        let session = create_session_at(
            &root,
            brief(),
            "2026-07-27T23:00:00.000Z",
            "invalidw",
            "session",
        )
        .unwrap();
        let down = validation_png(false, 32, 32);
        let concept = create_concept_candidate_at(
            &root,
            &session.id,
            &down,
            provenance(CandidateSource::Imported),
            "2026-07-27T23:01:00.000Z",
            "selected",
            "candidate",
            None,
        )
        .unwrap();
        let views = TurnaroundPngBytes {
            down: down.clone(),
            right: down.clone(),
            up: down.clone(),
            left: down.clone(),
        };
        let turnaround = create_turnaround_candidate_at(
            &root,
            &session.id,
            &concept.id,
            &views,
            CandidateProvenance {
                source: CandidateSource::Generated,
                original_filename: None,
                provider: Some("subscription-image-tool".to_owned()),
                model: None,
            },
            "2026-07-27T23:02:00.000Z",
            "accepted",
            "turnaround",
            None,
        )
        .unwrap();
        let replacement = concept_png(32, 32, true);
        let frames = WalkCyclePngBytes {
            down: vec![replacement, down.clone(), down.clone(), down.clone()],
            right: vec![down.clone(), down.clone(), down.clone(), down.clone()],
            up: vec![down.clone(), down.clone(), down.clone(), down.clone()],
            left: vec![down.clone(), down.clone(), down.clone(), down],
        };
        let result = create_walk_cycle_candidate_at(
            &root,
            &session.id,
            &turnaround.id,
            &frames,
            provenance(CandidateSource::Imported),
            "2026-07-27T23:30:00.000Z",
            "invalid1",
            "walk-cycle",
            None,
        );

        assert!(result.is_err());
        assert!(!root
            .join("sessions")
            .join(&session.id)
            .join("walk-cycles")
            .exists());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn desktop_adapter_reads_shared_world_test_fixture() {
        let fixture = include_str!("../../tests/fixtures/world-test-candidate-v1.json");
        let candidate: WorldTestCandidate = serde_json::from_str(fixture).unwrap();
        let candidate = validate_world_test(candidate).unwrap();

        assert_eq!(candidate.contract_id, CONTRACT_ID);
        assert_eq!(candidate.stage, "world-test");
        assert_eq!(candidate.source_walk_cycle.accepted_by, "user");
        assert_eq!(candidate.source_walk_cycle.frame_sources.len(), 16);
        assert_eq!(candidate.previews.len(), 16);
        assert!(!candidate.preparation.additional_ai_cost);
        assert_eq!(
            candidate.final_art_judgment.status,
            VisualJudgmentStatus::NotAssessed
        );
    }

    #[test]
    fn world_test_preserves_receipts_previews_and_user_gate() {
        let root = test_root("world-test");
        let session = create_session_at(
            &root,
            brief(),
            "2026-07-28T00:00:00.000Z",
            "worldtes",
            "session",
        )
        .unwrap();
        let down = validation_png(false, 32, 32);
        let concept = create_concept_candidate_at(
            &root,
            &session.id,
            &down,
            provenance(CandidateSource::Imported),
            "2026-07-28T00:01:00.000Z",
            "selected",
            "candidate",
            None,
        )
        .unwrap();
        let views = TurnaroundPngBytes {
            down: down.clone(),
            right: down.clone(),
            up: down.clone(),
            left: down.clone(),
        };
        let turnaround = create_turnaround_candidate_at(
            &root,
            &session.id,
            &concept.id,
            &views,
            provenance(CandidateSource::Imported),
            "2026-07-28T00:02:00.000Z",
            "accepted",
            "turnaround",
            None,
        )
        .unwrap();
        let frames = WalkCyclePngBytes {
            down: vec![down.clone(), down.clone(), down.clone(), down.clone()],
            right: vec![down.clone(), down.clone(), down.clone(), down.clone()],
            up: vec![down.clone(), down.clone(), down.clone(), down.clone()],
            left: vec![down.clone(), down.clone(), down.clone(), down],
        };
        let walk_cycle = create_walk_cycle_candidate_at(
            &root,
            &session.id,
            &turnaround.id,
            &frames,
            provenance(CandidateSource::Imported),
            "2026-07-28T00:03:00.000Z",
            "accepted",
            "walk-cycle",
            None,
        )
        .unwrap();

        assert!(create_world_test_candidate_at(
            &root,
            &session.id,
            "missing-walk-cycle",
            "2026-07-28T00:04:00.000Z",
            "invalid1",
            "missing",
            None,
        )
        .is_err());

        let world_test = create_world_test_candidate_at(
            &root,
            &session.id,
            &walk_cycle.id,
            "2026-07-28T00:04:00.000Z",
            "native01",
            "world-test",
            None,
        )
        .unwrap();
        let payload = read_world_test_payload(&root, &session.id, &world_test.id).unwrap();
        let report = validate_world_test_pngs(&root, &payload).unwrap();
        assert_eq!(
            payload.candidate.source_walk_cycle.walk_cycle_id,
            walk_cycle.id
        );
        assert_eq!(payload.candidate.source_walk_cycle.accepted_by, "user");
        assert_eq!(payload.preview_png_bytes.len(), 16);
        assert!(!payload.candidate.preparation.additional_ai_cost);
        assert_eq!(report.measurements.len(), 256);
        assert_eq!(report.summary.pass + report.summary.fail, 256);
        assert_eq!(report.summary.not_assessed, 0);
        assert_eq!(
            report.final_art_judgment.status,
            VisualJudgmentStatus::NotAssessed
        );
        assert_eq!(
            fs::read_dir(
                root.join("sessions")
                    .join(&session.id)
                    .join("world-tests")
                    .join(&world_test.id)
            )
            .unwrap()
            .count(),
            17
        );

        let collision = create_world_test_candidate_at(
            &root,
            &session.id,
            &walk_cycle.id,
            "2026-07-28T00:05:00.000Z",
            "same0001",
            "first",
            Some(2),
        )
        .unwrap();
        assert!(create_world_test_candidate_at(
            &root,
            &session.id,
            &walk_cycle.id,
            "2026-07-28T00:05:00.000Z",
            "same0001",
            "collision-cleanup",
            Some(2),
        )
        .is_err());
        assert_eq!(
            read_world_test_payload(&root, &session.id, &collision.id)
                .unwrap()
                .preview_png_bytes
                .len(),
            16
        );
        assert!(
            fs::read_dir(root.join("sessions").join(&session.id).join("world-tests"))
                .unwrap()
                .all(|entry| !entry
                    .unwrap()
                    .file_name()
                    .to_string_lossy()
                    .starts_with('.'))
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn desktop_adapter_reads_shared_export_fixture() {
        let fixture = include_str!("../../tests/fixtures/export-candidate-v1.json");
        let candidate: ExportCandidate = serde_json::from_str(fixture).unwrap();
        let candidate = validate_export_document(candidate).unwrap();

        assert_eq!(candidate.contract_id, CONTRACT_ID);
        assert_eq!(candidate.stage, "export");
        assert_eq!(candidate.approved_world_test.approved_by, "user");
        assert_eq!(candidate.source_walk_cycle.frame_sources.len(), 16);
        assert_eq!(candidate.package.sprite_sheet.width, 128);
        assert_eq!(candidate.package.sprite_sheet.height, 128);
        assert!(!candidate.preparation.additional_ai_cost);
        assert_eq!(candidate.status, "draft");
        assert_eq!(candidate.publishing.status, "not_approved");
        assert_eq!(candidate.publishing.authority, "user");
    }

    #[test]
    fn export_preserves_receipts_package_and_publishing_gate() {
        let root = test_root("export");
        let session = create_session_at(
            &root,
            brief(),
            "2026-07-28T01:00:00.000Z",
            "export01",
            "session",
        )
        .unwrap();
        let down = validation_png(false, 32, 32);
        let concept = create_concept_candidate_at(
            &root,
            &session.id,
            &down,
            provenance(CandidateSource::Imported),
            "2026-07-28T01:01:00.000Z",
            "selected",
            "candidate",
            None,
        )
        .unwrap();
        let views = TurnaroundPngBytes {
            down: down.clone(),
            right: down.clone(),
            up: down.clone(),
            left: down.clone(),
        };
        let turnaround = create_turnaround_candidate_at(
            &root,
            &session.id,
            &concept.id,
            &views,
            provenance(CandidateSource::Imported),
            "2026-07-28T01:02:00.000Z",
            "accepted",
            "turnaround",
            None,
        )
        .unwrap();
        let frames = WalkCyclePngBytes {
            down: vec![down.clone(), down.clone(), down.clone(), down.clone()],
            right: vec![down.clone(), down.clone(), down.clone(), down.clone()],
            up: vec![down.clone(), down.clone(), down.clone(), down.clone()],
            left: vec![down.clone(), down.clone(), down.clone(), down],
        };
        let walk_cycle = create_walk_cycle_candidate_at(
            &root,
            &session.id,
            &turnaround.id,
            &frames,
            provenance(CandidateSource::Imported),
            "2026-07-28T01:03:00.000Z",
            "accepted",
            "walk-cycle",
            None,
        )
        .unwrap();
        let world_test = create_world_test_candidate_at(
            &root,
            &session.id,
            &walk_cycle.id,
            "2026-07-28T01:04:00.000Z",
            "approved",
            "world-test",
            None,
        )
        .unwrap();

        assert!(create_export_candidate_at(
            &root,
            &session.id,
            "missing-world-test",
            "2026-07-28T01:05:00.000Z",
            "invalid1",
            "missing",
            None,
        )
        .is_err());

        let export = create_export_candidate_at(
            &root,
            &session.id,
            &world_test.id,
            "2026-07-28T01:05:00.000Z",
            "native01",
            "export",
            None,
        )
        .unwrap();
        let payload = read_export_payload(&root, &session.id, &export.id).unwrap();
        assert_eq!(
            validated_export_directory(&root, &session.id, &export.id).unwrap(),
            root.join("sessions")
                .join(&session.id)
                .join("exports")
                .join(&export.id)
        );
        let expected_export_directory = root
            .join("sessions")
            .join(&session.id)
            .join("exports")
            .join(&export.id);
        assert_eq!(
            open_export_folder_at(&root, &session.id, &export.id, |directory| {
                assert_eq!(directory, expected_export_directory);
                Ok(())
            })
            .unwrap(),
            expected_export_directory.to_string_lossy()
        );
        let mut launched_missing_export = false;
        assert!(
            open_export_folder_at(&root, &session.id, "missing-export", |_| {
                launched_missing_export = true;
                Ok(())
            },)
            .is_err(),
            "Folder resolution accepted a missing Export."
        );
        assert!(
            !launched_missing_export,
            "Folder launcher ran before Export validation."
        );
        let report = validate_export_package(&root, &payload).unwrap();
        let sheet = decode_png_rgba(&payload.sprite_sheet_png_bytes).unwrap();
        assert_eq!(
            payload.candidate.approved_world_test.world_test_id,
            world_test.id
        );
        assert_eq!(payload.candidate.approved_world_test.approved_by, "user");
        assert_eq!(
            payload.candidate.source_walk_cycle.walk_cycle_id,
            walk_cycle.id
        );
        assert_eq!(sheet.width, EXPORT_SHEET_WIDTH);
        assert_eq!(sheet.height, EXPORT_SHEET_HEIGHT);
        assert_eq!(payload.metadata.frames.len(), 16);
        assert!(!payload.candidate.preparation.additional_ai_cost);
        assert_eq!(payload.candidate.status, "draft");
        assert_eq!(payload.candidate.publishing.status, "not_approved");
        assert_eq!(report.summary.pass, 7);
        assert_eq!(report.summary.fail, 0);
        assert_eq!(report.summary.not_assessed, 0);
        assert_eq!(
            fs::read_dir(
                root.join("sessions")
                    .join(&session.id)
                    .join("exports")
                    .join(&export.id)
            )
            .unwrap()
            .count(),
            4
        );

        let collision = create_export_candidate_at(
            &root,
            &session.id,
            &world_test.id,
            "2026-07-28T01:06:00.000Z",
            "same0001",
            "first",
            Some(2),
        )
        .unwrap();
        assert!(create_export_candidate_at(
            &root,
            &session.id,
            &world_test.id,
            "2026-07-28T01:06:00.000Z",
            "same0001",
            "collision-cleanup",
            Some(2),
        )
        .is_err());
        read_export_payload(&root, &session.id, &collision.id).unwrap();
        assert!(
            fs::read_dir(root.join("sessions").join(&session.id).join("exports"))
                .unwrap()
                .all(|entry| !entry
                    .unwrap()
                    .file_name()
                    .to_string_lossy()
                    .starts_with('.'))
        );

        fs::write(
            root.join("sessions")
                .join(&session.id)
                .join("exports")
                .join(&collision.id)
                .join(EXPORT_METADATA_FILE),
            "{}\n",
        )
        .unwrap();
        assert!(read_export_payload(&root, &session.id, &collision.id).is_err());
        fs::remove_dir_all(root).unwrap();
    }
}
