use chrono::{SecondsFormat, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::HashSet,
    env, fs,
    io::Cursor,
    path::{Path, PathBuf},
};
use uuid::Uuid;

const CONTRACT_ID: &str = "tileforge-actor-32-v1";
const SESSION_ID_MAX_LENGTH: usize = 96;
const SESSION_SLUG_MAX_LENGTH: usize = 64;
const CANDIDATE_ID_MAX_LENGTH: usize = 96;
const CONCEPT_PNG_MAX_BYTES: usize = 1_048_576;
const VALIDATION_REPORT_VERSION: u32 = 1;
const STRUCTURAL_VALIDATOR_ID: &str = "tileforge-actor-32-structural-v1";
const TURNAROUND_VALIDATOR_ID: &str = "tileforge-actor-32-turnaround-structural-v1";
const FRAME_WIDTH: u32 = 32;
const FRAME_HEIGHT: u32 = 32;
const ACTOR_HEIGHT_MIN: u32 = 22;
const ACTOR_HEIGHT_MAX: u32 = 30;
const FOOT_ANCHOR_X: u32 = 16;
const FOOT_ANCHOR_Y: u32 = 28;
const PALETTE_MAX_COLORS: usize = 16;
const MINIMUM_GROUND_LUMA_DISTANCE: u32 = 15;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Debug)]
struct DecodedPng {
    width: u32,
    height: u32,
    pixels: Vec<[u8; 4]>,
}

fn workspace_root() -> PathBuf {
    env::var_os("TFAS_WORKSPACE").map_or_else(
        || {
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .expect("src-tauri must have a repository parent")
                .join(".studio")
        },
        PathBuf::from,
    )
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
        fs::rename(&temporary_directory, &final_directory)
            .map_err(|error| format!("Could not publish session: {error}"))
    })();

    if publish_result.is_err() {
        let _ = fs::remove_dir_all(&temporary_directory);
    }
    publish_result?;
    Ok(session)
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
    )
}

fn validate_png_structural_evidence(
    artifact_id: &str,
    sha256: &str,
    byte_length: usize,
    contract_id: &str,
    png_bytes: &[u8],
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
    let mut foot_anchor_contact = false;

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
            foot_anchor_contact = true;
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
            if foot_anchor_contact {
                ValidationStatus::Pass
            } else {
                ValidationStatus::Fail
            },
            format!("Visible pixel at ({FOOT_ANCHOR_X}, {FOOT_ANCHOR_Y})"),
            Some(
                if foot_anchor_contact {
                    "Contact"
                } else {
                    "No contact"
                }
                .to_owned(),
            ),
            if foot_anchor_contact {
                "The actor contacts the contract foot anchor."
            } else {
                "The contract foot anchor is transparent."
            },
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            studio_status,
            create_sprite_session,
            get_sprite_session,
            list_sprite_sessions,
            import_concept_candidate,
            list_concept_candidates,
            get_concept_candidate,
            validate_concept_candidate,
            create_turnaround_candidate,
            list_turnaround_candidates,
            get_turnaround_candidate,
            validate_turnaround_candidate
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
}
