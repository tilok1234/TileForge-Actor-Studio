use chrono::{SecondsFormat, Utc};
use serde::{Deserialize, Serialize};
use std::{
    env, fs,
    path::{Path, PathBuf},
};
use uuid::Uuid;

const CONTRACT_ID: &str = "tileforge-actor-32-v1";
const SESSION_ID_MAX_LENGTH: usize = 96;
const SESSION_SLUG_MAX_LENGTH: usize = 64;

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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            studio_status,
            create_sprite_session,
            get_sprite_session,
            list_sprite_sessions
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
}
