use chrono::{SecondsFormat, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
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

fn validate_concept_png(png_bytes: &[u8]) -> Result<(u32, u32, String), String> {
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
    if reader.info().width != 32 || reader.info().height != 32 {
        return Err("Concept PNG must be exactly 32 x 32 pixels.".to_owned());
    }

    let mut decoded = vec![0; reader.output_buffer_size()];
    let output = reader
        .next_frame(&mut decoded)
        .map_err(|_| "File is not a valid PNG.".to_owned())?;
    let decoded = &decoded[..output.buffer_size()];
    let has_transparency = match output.color_type {
        png::ColorType::Rgba => decoded.chunks_exact(4).any(|pixel| pixel[3] < 255),
        png::ColorType::GrayscaleAlpha => decoded.chunks_exact(2).any(|pixel| pixel[1] < 255),
        _ => false,
    };
    if !has_transparency {
        return Err("Concept PNG must contain an alpha channel with transparency.".to_owned());
    }

    Ok((
        output.width,
        output.height,
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
            get_concept_candidate
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
}
