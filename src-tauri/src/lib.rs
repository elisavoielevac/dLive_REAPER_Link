mod bridge;
pub mod reaper_osc;

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tauri::{Emitter, Manager};
use tauri_plugin_opener::OpenerExt;

const DUPLICATE_SCENE_RECALL_WINDOW: Duration = Duration::from_millis(250);
const SCENE_SLOT_COUNT: usize = 500;

struct MidiInputSelection {
    path: PathBuf,
}

impl MidiInputSelection {
    fn load(&self) -> Result<Option<String>, std::io::Error> {
        match fs::read_to_string(&self.path) {
            Ok(port_name) => {
                let port_name = port_name.trim().to_string();
                Ok((!port_name.is_empty()).then_some(port_name))
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(error),
        }
    }

    fn save(&self, port_name: &str) -> Result<(), std::io::Error> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(&self.path, port_name)
    }
}

struct ReaperOscSettings {
    config: Arc<RwLock<reaper_osc::ReaperOscConfig>>,
    path: PathBuf,
}

#[derive(Clone, Copy, PartialEq)]
enum MarkerMode {
    Always,
    RecordingOnly,
}

impl MarkerMode {
    fn as_str(self) -> &'static str {
        match self {
            Self::Always => "always",
            Self::RecordingOnly => "recording_only",
        }
    }

    fn from_str(value: &str) -> Option<Self> {
        match value.trim() {
            "always" => Some(Self::Always),
            "recording_only" => Some(Self::RecordingOnly),
            _ => None,
        }
    }
}

struct MarkerModeState {
    mode: Arc<RwLock<MarkerMode>>,
}

impl MarkerModeState {
    fn new() -> Self {
        Self {
            mode: Arc::new(RwLock::new(MarkerMode::RecordingOnly)),
        }
    }

    fn get(&self) -> Result<MarkerMode, String> {
        self.mode
            .read()
            .map(|mode| *mode)
            .map_err(|_| "marker mode is unavailable".to_string())
    }

    fn save(&self, mode: MarkerMode) -> Result<(), String> {
        *self
            .mode
            .write()
            .map_err(|_| "marker mode is unavailable".to_string())? = mode;
        Ok(())
    }
}

impl ReaperOscSettings {
    fn load(path: PathBuf) -> Result<Self, std::io::Error> {
        let mut config = reaper_osc::ReaperOscConfig::default();

        match fs::read_to_string(&path) {
            Ok(contents) => {
                let mut lines = contents.lines();
                if let Some(host) = lines.next().filter(|host| !host.trim().is_empty()) {
                    config.host = host.trim().to_string();
                }
                if let Some(port) = lines
                    .next()
                    .and_then(|port| port.trim().parse::<u16>().ok())
                    .filter(|port| *port != 0)
                {
                    config.receive_port = port;
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error),
        }

        Ok(Self {
            config: Arc::new(RwLock::new(config)),
            path,
        })
    }

    fn get(&self) -> Result<(String, u16), String> {
        let config = self
            .config
            .read()
            .map_err(|_| "REAPER OSC settings are unavailable".to_string())?;
        Ok((config.host.clone(), config.receive_port))
    }

    fn feedback_config(&self) -> Result<reaper_osc::ReaperOscConfig, String> {
        self.config
            .read()
            .map(|config| config.clone())
            .map_err(|_| "REAPER OSC settings are unavailable".to_string())
    }

    fn save(&self, host: String, port: u16) -> Result<(), String> {
        let host = host.trim().to_string();
        if host.is_empty() {
            return Err("REAPER OSC host cannot be empty".to_string());
        }
        if port == 0 {
            return Err("REAPER OSC port must be between 1 and 65535".to_string());
        }
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
        fs::write(&self.path, format!("{host}\n{port}\n")).map_err(|error| error.to_string())?;

        let mut config = self
            .config
            .write()
            .map_err(|_| "REAPER OSC settings are unavailable".to_string())?;
        config.host = host;
        config.receive_port = port;
        Ok(())
    }
}

struct SceneListSelection {
    path: PathBuf,
}

impl SceneListSelection {
    fn load(&self) -> Result<Option<String>, std::io::Error> {
        match fs::read_to_string(&self.path) {
            Ok(name) => {
                let name = name.trim().to_string();
                Ok((!name.is_empty()).then_some(name))
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(error),
        }
    }

    fn save(&self, name: &str) -> Result<(), std::io::Error> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(&self.path, name)
    }

    fn clear(&self) -> Result<(), std::io::Error> {
        match fs::remove_file(&self.path) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(error),
        }
    }
}

struct SceneNames {
    names: Arc<RwLock<Vec<String>>>,
    path: PathBuf,
    scene_lists_dir: PathBuf,
}

impl SceneNames {
    fn load(scene_file: &Path) -> Result<Self, std::io::Error> {
        if let Some(parent) = scene_file.parent() {
            fs::create_dir_all(parent)?;
        }

        if !scene_file.exists() {
            fs::write(scene_file, include_str!("../dLive_Scenes.txt"))?;
        }

        Ok(Self {
            names: Arc::new(RwLock::new(read_scene_names(scene_file)?)),
            path: scene_file.to_path_buf(),
            // Keep the existing app-managed folder so previously saved backups remain available.
            scene_lists_dir: scene_file
                .parent()
                .unwrap_or_else(|| Path::new("."))
                .join("scene_list_backups"),
        })
    }

    fn count(&self) -> Result<usize, String> {
        self.names
            .read()
            .map(|names| names.iter().filter(|name| !name.is_empty()).count())
            .map_err(|_| "scene names are unavailable".to_string())
    }

    fn reload(&self) -> Result<usize, String> {
        let names = read_scene_names(&self.path).map_err(|error| error.to_string())?;
        let mut loaded_names = self
            .names
            .write()
            .map_err(|_| "scene names are unavailable".to_string())?;
        let count = names.iter().filter(|name| !name.is_empty()).count();
        *loaded_names = names;
        Ok(count)
    }

    fn editable_entries(&self) -> Result<Vec<String>, String> {
        let names = self
            .names
            .read()
            .map_err(|_| "scene names are unavailable".to_string())?;
        let mut entries = names
            .iter()
            .take(SCENE_SLOT_COUNT)
            .cloned()
            .collect::<Vec<_>>();
        entries.resize(SCENE_SLOT_COUNT, String::new());
        Ok(entries)
    }

    fn save(&self, names: Vec<String>) -> Result<usize, String> {
        if names.len() != SCENE_SLOT_COUNT {
            return Err(format!(
                "exactly {SCENE_SLOT_COUNT} scene names are required"
            ));
        }
        if names
            .iter()
            .any(|name| name.contains('\n') || name.contains('\r'))
        {
            return Err("scene names cannot contain line breaks".to_string());
        }

        fs::write(&self.path, format!("{}\n", names.join("\n")))
            .map_err(|error| error.to_string())?;
        self.reload()
    }

    fn export_to(&self, destination: &Path) -> Result<(), String> {
        let contents = fs::read_to_string(&self.path).map_err(|error| error.to_string())?;
        fs::write(destination, contents).map_err(|error| error.to_string())
    }

    fn import_from(&self, source: &Path) -> Result<usize, String> {
        let mut names = read_scene_names(source).map_err(|error| error.to_string())?;
        names.truncate(SCENE_SLOT_COUNT);
        names.resize(SCENE_SLOT_COUNT, String::new());
        self.save(names)
    }

    fn list_saved_scene_lists(&self) -> Result<Vec<String>, String> {
        let entries = match fs::read_dir(&self.scene_lists_dir) {
            Ok(entries) => entries,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(error) => return Err(error.to_string()),
        };

        let mut scene_lists = entries
            .filter_map(Result::ok)
            .filter_map(|entry| {
                let path = entry.path();
                if !path.is_file() || !path.extension().is_some_and(|extension| extension == "txt")
                {
                    return None;
                }

                path.file_stem()
                    .map(|name| name.to_string_lossy().into_owned())
            })
            .filter(|name| self.scene_list_path(name).is_ok())
            .collect::<Vec<_>>();
        scene_lists.sort_by(|left, right| right.cmp(left));
        Ok(scene_lists)
    }

    fn save_as_scene_list(&self, name: &str, names: Option<Vec<String>>) -> Result<String, String> {
        fs::create_dir_all(&self.scene_lists_dir).map_err(|error| error.to_string())?;

        let base_name = sanitize_scene_list_name(name);
        if base_name.is_empty() {
            return Err("scene list name is required".to_string());
        }
        if self.scene_list_path(&base_name)?.exists() {
            return Err(
                "a scene list with that name already exists; choose a different name".to_string(),
            );
        }

        match names {
            Some(names) => self.write_scene_list(&base_name, &names)?,
            None => self.save_current_to_scene_list(&base_name)?,
        }
        Ok(base_name)
    }

    fn create_scene_list(&self, name: &str) -> Result<String, String> {
        self.save_as_scene_list(name, Some(vec![String::new(); SCENE_SLOT_COUNT]))
    }

    fn load_scene_list(&self, scene_list_name: &str) -> Result<usize, String> {
        self.import_from(&self.scene_list_path(scene_list_name)?)
    }

    fn rename_scene_list(&self, scene_list_name: &str, new_name: &str) -> Result<String, String> {
        let source_path = self.scene_list_path(scene_list_name)?;
        if !source_path.is_file() {
            return Err("scene list does not exist".to_string());
        }

        let renamed_scene_list = sanitize_scene_list_name(new_name);
        if renamed_scene_list.is_empty() {
            return Err("scene list name is required".to_string());
        }
        if renamed_scene_list == scene_list_name {
            return Err("scene list name is unchanged".to_string());
        }

        let destination_path = self.scene_list_path(&renamed_scene_list)?;
        if destination_path.exists() {
            return Err(
                "a scene list with that name already exists; choose a different name".to_string(),
            );
        }

        fs::rename(source_path, destination_path).map_err(|error| error.to_string())?;
        Ok(renamed_scene_list)
    }

    fn delete_scene_list(&self, scene_list_name: &str) -> Result<(), String> {
        let scene_list_path = self.scene_list_path(scene_list_name)?;
        if !scene_list_path.is_file() {
            return Err("scene list does not exist".to_string());
        }

        fs::remove_file(scene_list_path).map_err(|error| error.to_string())
    }

    fn import_as_scene_list(&self, source: &Path) -> Result<(String, usize), String> {
        let source_name = source
            .file_stem()
            .and_then(|name| name.to_str())
            .unwrap_or("Imported Scene List");
        let scene_list_name = self.save_as_scene_list_name(source_name)?;

        let mut names = read_scene_names(source).map_err(|error| error.to_string())?;
        names.truncate(SCENE_SLOT_COUNT);
        names.resize(SCENE_SLOT_COUNT, String::new());
        self.write_scene_list(&scene_list_name, &names)?;
        let count = self.load_scene_list(&scene_list_name)?;
        Ok((scene_list_name, count))
    }

    fn save_current_to_scene_list(&self, scene_list_name: &str) -> Result<(), String> {
        let mut names = read_scene_names(&self.path).map_err(|error| error.to_string())?;
        names.truncate(SCENE_SLOT_COUNT);
        names.resize(SCENE_SLOT_COUNT, String::new());
        self.write_scene_list(scene_list_name, &names)
    }

    fn save_loaded_scene_list(&self, scene_list_name: &str) -> Result<(), String> {
        self.save_current_to_scene_list(scene_list_name)
    }

    fn save_as_scene_list_name(&self, name: &str) -> Result<String, String> {
        fs::create_dir_all(&self.scene_lists_dir).map_err(|error| error.to_string())?;

        let base_name = sanitize_scene_list_name(name);
        let base_name = (!base_name.is_empty())
            .then_some(base_name)
            .unwrap_or_else(|| {
                format!(
                    "Imported Scene List {}",
                    SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs()
                )
            });
        let mut scene_list_name = base_name.clone();
        let mut suffix = 2;
        while self.scene_list_path(&scene_list_name)?.exists() {
            scene_list_name = format!("{base_name}-{suffix}");
            suffix += 1;
        }

        Ok(scene_list_name)
    }

    fn write_scene_list(&self, scene_list_name: &str, names: &[String]) -> Result<(), String> {
        if names.len() != SCENE_SLOT_COUNT {
            return Err(format!(
                "exactly {SCENE_SLOT_COUNT} scene names are required"
            ));
        }
        if names
            .iter()
            .any(|name| name.contains('\n') || name.contains('\r'))
        {
            return Err("scene names cannot contain line breaks".to_string());
        }

        fs::write(
            self.scene_list_path(scene_list_name)?,
            format!("{}\n", names.join("\n")),
        )
        .map_err(|error| error.to_string())
    }

    fn scene_list_path(&self, scene_list_name: &str) -> Result<PathBuf, String> {
        if scene_list_name.is_empty()
            || sanitize_scene_list_name(scene_list_name) != scene_list_name
        {
            return Err("invalid scene list name".to_string());
        }

        Ok(self.scene_lists_dir.join(format!("{scene_list_name}.txt")))
    }

    fn has_scene_list(&self, scene_list_name: &str) -> bool {
        self.scene_list_path(scene_list_name)
            .is_ok_and(|path| path.is_file())
    }
}

fn sanitize_scene_list_name(name: &str) -> String {
    name.trim()
        .chars()
        .map(|character| match character {
            'a'..='z' | 'A'..='Z' | '0'..='9' | ' ' | '_' | '-' => character,
            _ => '_',
        })
        .take(80)
        .collect::<String>()
        .trim_matches('_')
        .trim()
        .to_string()
}

fn read_scene_names(scene_file: &Path) -> Result<Vec<String>, std::io::Error> {
    fs::read_to_string(scene_file).map(|contents| {
        if contents.is_empty() {
            return Vec::new();
        }

        contents
            .strip_suffix('\n')
            .unwrap_or(&contents)
            .split('\n')
            .map(|line| line.trim().to_string())
            .collect()
    })
}

fn scene_name(scene_names: &RwLock<Vec<String>>, scene_number: u16) -> String {
    let Ok(scene_names) = scene_names.read() else {
        return format!("Unnamed scene {scene_number}");
    };

    scene_names
        .get(usize::from(scene_number - 1))
        .filter(|name| !name.is_empty())
        .cloned()
        .unwrap_or_else(|| format!("Unnamed scene {scene_number}"))
}

// Keep line/slot numbers, including blank slots; never use the REAPER marker ID.
fn scene_number_for_marker(names: &[String], marker: &str) -> Result<Option<u16>, String> {
    if marker.is_empty() {
        return Ok(None);
    }
    let mut matches = names
        .iter()
        .take(SCENE_SLOT_COUNT)
        .enumerate()
        .filter(|(_, name)| name.as_str() == marker);
    let first = matches.next().map(|(index, _)| (index + 1) as u16);
    if matches.next().is_some() {
        return Err("Ambiguous marker name: multiple scene slots match".to_string());
    }
    Ok(first)
}

struct PlaybackRecall {
    state: Mutex<RecallState>,
}

struct RecallState {
    session: Option<bridge::SendSession>,
    last_scene: Option<u16>,
    last_sent: Option<(u16, Instant)>,
    disabled_at: Option<Instant>,
    message: String,
}

impl Default for RecallState {
    fn default() -> Self {
        Self {
            session: None,
            last_scene: None,
            last_sent: None,
            disabled_at: None,
            message: "Off — playback preview only".into(),
        }
    }
}

impl RecallState {
    fn disable(&mut self, message: &str, now: Instant) {
        self.session = None;
        self.disabled_at = Some(now);
        self.message = message.into();
    }

    fn blocks_marker(&self, scene: u16, received_at: Instant) -> bool {
        self.session.is_some()
            || self.disabled_at.is_some_and(|at| received_at <= at)
            // Protect a delayed desk echo after switching recall off.
            || self.last_sent.is_some_and(|(sent_scene, at)|
                sent_scene == scene && received_at >= at
                && received_at.duration_since(at) < Duration::from_secs(2))
    }

    fn candidate(
        &self,
        feedback: &reaper_osc::PlaybackFeedback,
        names: &[String],
        now: Instant,
    ) -> Result<Option<u16>, String> {
        if self.session.is_none()
            || feedback.preview_status() != "playing"
            || !feedback.settled(now)
        {
            return Ok(None);
        }
        let scene = scene_number_for_marker(names, feedback.marker_name.as_deref().unwrap_or(""))?;
        Ok(scene.filter(|scene| Some(*scene) != self.last_scene))
    }
}

impl PlaybackRecall {
    fn new() -> Self {
        Self {
            state: Mutex::new(RecallState::default()),
        }
    }
    fn suspend_for_edit(&self) -> std::sync::MutexGuard<'_, RecallState> {
        let mut state = self.state.lock().unwrap();
        state.disable(
            "Off — settings or scene list changed; re-enable playback recall",
            Instant::now(),
        );
        state
    }
}

#[tauri::command]
fn set_playback_recall(
    enabled: bool,
    recall: tauri::State<'_, PlaybackRecall>,
    bridge: tauri::State<'_, bridge::Bridge>,
    feedback: tauri::State<'_, reaper_osc::ReaperOscFeedback>,
    scenes: tauri::State<'_, SceneNames>,
    selection: tauri::State<'_, SceneListSelection>,
) -> Result<(), String> {
    let mut state = recall
        .state
        .lock()
        .map_err(|_| "Playback recall unavailable")?;
    if !enabled {
        state.disable("Off — playback preview only", Instant::now());
        return Ok(());
    }
    if state.session.is_some() {
        return Ok(());
    }
    if selection
        .load()
        .map_err(|error| error.to_string())?
        .is_none()
    {
        return Err("Load a scene list before enabling playback recall".into());
    }
    let names = scenes.names.read().map_err(|_| "Scene names unavailable")?;
    let mut unique = std::collections::HashSet::new();
    if names
        .iter()
        .take(SCENE_SLOT_COUNT)
        .filter(|name| !name.is_empty())
        .any(|name| !unique.insert(name))
    {
        return Err("Playback recall requires unique scene names".into());
    }
    let observed = feedback.playback()?;
    if observed.preview_status() == "unknown" || observed.marker_name.is_none() {
        return Err(
            "Refresh all surfaces in REAPER and wait for transport and marker feedback".into(),
        );
    }
    state.session = Some(bridge.send_session()?);
    state.last_scene = None;
    state.message = "On — waiting for mapped playback; marker creation is suspended".into();
    Ok(())
}

fn process_playback_recall(
    state: &mut RecallState,
    bridge: &bridge::Bridge,
    feedback: &reaper_osc::PlaybackFeedback,
    names: &[String],
    now: Instant,
) {
    let Some(session) = state.session else {
        return;
    };
    if bridge.send_session().ok() != Some(session) {
        state.disable(
            "Off — desk connection changed; re-enable playback recall",
            now,
        );
        return;
    }
    // While playing REAPER emits /time continuously. A silent feed must not
    // leave a stale playing/marker snapshot capable of sending.
    if feedback.preview_status() == "playing"
        && !feedback
            .received_at
            .is_some_and(|at| now.saturating_duration_since(at) < Duration::from_secs(2))
    {
        state.disable(
            "Off — REAPER feedback lost; refresh and re-enable recall",
            now,
        );
        return;
    }
    match state.candidate(feedback, names, now) {
        Ok(Some(scene)) => match bridge.send_scene(session, scene) {
            Ok(()) => {
                state.last_scene = Some(scene);
                state.last_sent = Some((scene, now));
                state.message = format!(
                    "Scene {scene}: {} — command sent (desk recall unconfirmed)",
                    feedback.marker_name.as_deref().unwrap_or("")
                );
            }
            Err(error) => state.disable(&format!("Off — {error}; re-enable recall"), now),
        },
        Err(error) => state.disable(&format!("Off — {error}"), now),
        Ok(None) => {}
    }
}

// This worker lives in Rust, so hiding the window cannot stop scene recall.
fn start_playback_recall(app: tauri::AppHandle) {
    std::thread::spawn(move || loop {
        {
            let recall = app.state::<PlaybackRecall>();
            let mut state = recall.state.lock().unwrap();
            if state.session.is_some() {
                let bridge = app.state::<bridge::Bridge>();
                if let Ok(feedback) = app.state::<reaper_osc::ReaperOscFeedback>().playback() {
                    let scenes = app.state::<SceneNames>();
                    let names = scenes.names.read().unwrap();
                    process_playback_recall(&mut state, &bridge, &feedback, &names, Instant::now());
                }
            }
        }
        std::thread::sleep(Duration::from_millis(20));
    });
}

#[derive(serde::Serialize)]
struct PlaybackPreview {
    feedback: reaper_osc::PlaybackFeedback,
    transport: String,
    scene_number: Option<u16>,
    message: String,
    recall_enabled: bool,
    recall_message: String,
}

#[tauri::command]
fn get_playback_preview(
    recall: tauri::State<'_, PlaybackRecall>,
    feedback: tauri::State<'_, reaper_osc::ReaperOscFeedback>,
    scenes: tauri::State<'_, SceneNames>,
    selection: tauri::State<'_, SceneListSelection>,
) -> Result<PlaybackPreview, String> {
    let state = recall
        .state
        .lock()
        .map_err(|_| "Playback recall unavailable")?;
    let feedback = feedback.playback()?;
    let transport = feedback.preview_status().to_string();
    let names = scenes
        .names
        .read()
        .map_err(|_| "Scene names unavailable".to_string())?;
    let mapping = if selection
        .load()
        .map_err(|error| error.to_string())?
        .is_none()
    {
        Err("Load a scene list to match markers".to_string())
    } else {
        scene_number_for_marker(&names, feedback.marker_name.as_deref().unwrap_or(""))
    };
    let scene_number = mapping.as_ref().ok().copied().flatten();
    let message = match transport.as_str() {
        "recording" => "Recording — no playback recall".to_string(),
        "paused" => "Paused — no playback recall".to_string(),
        "stopped" => "Stopped — no playback recall".to_string(),
        "unknown" => "Waiting for complete transport feedback".to_string(),
        _ => match mapping {
            Err(error) => error,
            Ok(Some(number)) => format!(
                "{} scene {number}: {}",
                if state.session.is_some() {
                    "Mapped"
                } else {
                    "Would recall"
                },
                feedback.marker_name.as_deref().unwrap_or("")
            ),
            Ok(None) if feedback.marker_name.is_none() => {
                "Waiting for current marker feedback".to_string()
            }
            Ok(None) => "No scene matches the current marker name".to_string(),
        },
    };
    Ok(PlaybackPreview {
        feedback,
        transport,
        scene_number,
        message,
        recall_enabled: state.session.is_some(),
        recall_message: state.message.clone(),
    })
}

fn decode_scene_number(bank_number: u8, program: u8) -> Option<u16> {
    let scene_number = u16::from(bank_number) * 128 + u16::from(program) + 1;
    (usize::from(scene_number) <= SCENE_SLOT_COUNT).then_some(scene_number)
}

fn should_create_marker(
    recent_scene_recall: &Mutex<Option<(u16, Instant)>>,
    scene_number: u16,
    now: Instant,
) -> bool {
    let Ok(mut last_scene_recall) = recent_scene_recall.lock() else {
        return true;
    };

    if matches!(
        *last_scene_recall,
        Some((last_scene_number, timestamp))
            if last_scene_number == scene_number
                && now.duration_since(timestamp) < DUPLICATE_SCENE_RECALL_WINDOW
    ) {
        return false;
    }

    *last_scene_recall = Some((scene_number, now));
    true
}

fn create_reaper_marker(
    reaper_osc_config: &RwLock<reaper_osc::ReaperOscConfig>,
    reaper_osc_feedback: &reaper_osc::ReaperOscFeedback,
    name: &str,
) -> Result<String, String> {
    let config = reaper_osc_config
        .read()
        .map_err(|_| "REAPER OSC settings are unavailable".to_string())?
        .clone();

    reaper_osc::ReaperOscClient::new(config)
        .create_marker(reaper_osc_feedback, name)
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn get_marker_mode(settings: tauri::State<'_, MarkerModeState>) -> Result<String, String> {
    Ok(settings.get()?.as_str().to_string())
}

#[tauri::command]
fn save_marker_mode(
    mode: String,
    settings: tauri::State<'_, MarkerModeState>,
) -> Result<(), String> {
    let mode = MarkerMode::from_str(&mode).ok_or_else(|| "invalid marker mode".to_string())?;
    settings.save(mode)
}

#[tauri::command]
fn list_midi_inputs() -> Result<Vec<String>, String> {
    bridge::midi_inputs()
}

#[tauri::command]
fn get_saved_midi_input(
    selection: tauri::State<'_, MidiInputSelection>,
) -> Result<Option<String>, String> {
    selection.load().map_err(|error| error.to_string())
}

#[tauri::command]
fn save_midi_input(
    port_name: String,
    selection: tauri::State<'_, MidiInputSelection>,
) -> Result<(), String> {
    selection
        .save(&port_name)
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn get_reaper_osc_settings(
    settings: tauri::State<'_, ReaperOscSettings>,
) -> Result<(String, u16), String> {
    settings.get()
}

#[tauri::command]
fn save_reaper_osc_settings(
    host: String,
    port: u16,
    settings: tauri::State<'_, ReaperOscSettings>,
    recall: tauri::State<'_, PlaybackRecall>,
    feedback: tauri::State<'_, reaper_osc::ReaperOscFeedback>,
) -> Result<(), String> {
    let _recall_edit = recall.suspend_for_edit();
    settings.save(host, port)?;
    feedback.reset_target(&settings.get()?.0);
    Ok(())
}

#[tauri::command]
fn get_scene_file_path(scene_names: tauri::State<'_, SceneNames>) -> String {
    scene_names.path.display().to_string()
}

#[tauri::command]
fn reveal_scene_file(
    app: tauri::AppHandle,
    scene_names: tauri::State<'_, SceneNames>,
) -> Result<(), String> {
    fs::create_dir_all(&scene_names.scene_lists_dir).map_err(|error| error.to_string())?;
    app.opener()
        .open_path(
            scene_names.scene_lists_dir.to_string_lossy().into_owned(),
            None::<&str>,
        )
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn get_scene_names_count(scene_names: tauri::State<'_, SceneNames>) -> Result<usize, String> {
    scene_names.count()
}

#[tauri::command]
fn reload_scene_names(
    scene_names: tauri::State<'_, SceneNames>,
    recall: tauri::State<'_, PlaybackRecall>,
) -> Result<usize, String> {
    let _recall_edit = recall.suspend_for_edit();
    scene_names.reload()
}

#[tauri::command]
fn get_scene_name_entries(
    scene_names: tauri::State<'_, SceneNames>,
) -> Result<Vec<String>, String> {
    scene_names.editable_entries()
}

#[tauri::command]
fn save_scene_names(
    names: Vec<String>,
    scene_names: tauri::State<'_, SceneNames>,
    scene_list_selection: tauri::State<'_, SceneListSelection>,
    recall: tauri::State<'_, PlaybackRecall>,
) -> Result<usize, String> {
    let _recall_edit = recall.suspend_for_edit();
    let count = scene_names.save(names)?;
    if let Some(scene_list_name) = scene_list_selection
        .load()
        .map_err(|error| error.to_string())?
    {
        scene_names.save_loaded_scene_list(&scene_list_name)?;
    }
    Ok(count)
}

#[tauri::command]
fn export_scene_list(
    destination_path: String,
    scene_names: tauri::State<'_, SceneNames>,
) -> Result<(), String> {
    scene_names.export_to(Path::new(&destination_path))
}

#[tauri::command]
fn import_scene_list(
    source_path: String,
    scene_names: tauri::State<'_, SceneNames>,
    scene_list_selection: tauri::State<'_, SceneListSelection>,
    recall: tauri::State<'_, PlaybackRecall>,
) -> Result<(String, usize), String> {
    let _recall_edit = recall.suspend_for_edit();
    let (scene_list_name, count) = scene_names.import_as_scene_list(Path::new(&source_path))?;
    scene_list_selection
        .save(&scene_list_name)
        .map_err(|error| error.to_string())?;
    Ok((scene_list_name, count))
}

#[tauri::command]
fn get_loaded_scene_list(
    scene_names: tauri::State<'_, SceneNames>,
    scene_list_selection: tauri::State<'_, SceneListSelection>,
) -> Result<Option<String>, String> {
    let selected_scene_list = scene_list_selection
        .load()
        .map_err(|error| error.to_string())?;
    if selected_scene_list
        .as_deref()
        .is_some_and(|name| scene_names.has_scene_list(name))
    {
        Ok(selected_scene_list)
    } else {
        scene_list_selection
            .clear()
            .map_err(|error| error.to_string())?;
        Ok(None)
    }
}

#[tauri::command]
fn list_saved_scene_lists(
    scene_names: tauri::State<'_, SceneNames>,
) -> Result<Vec<String>, String> {
    scene_names.list_saved_scene_lists()
}

#[tauri::command]
fn save_scene_list_as(
    name: String,
    names: Option<Vec<String>>,
    scene_names: tauri::State<'_, SceneNames>,
    scene_list_selection: tauri::State<'_, SceneListSelection>,
    recall: tauri::State<'_, PlaybackRecall>,
) -> Result<String, String> {
    let _recall_edit = recall.suspend_for_edit();
    let scene_list_name = scene_names.save_as_scene_list(&name, names)?;
    scene_names.load_scene_list(&scene_list_name)?;
    scene_list_selection
        .save(&scene_list_name)
        .map_err(|error| error.to_string())?;
    Ok(scene_list_name)
}

#[tauri::command]
fn create_scene_list(
    name: String,
    scene_names: tauri::State<'_, SceneNames>,
    scene_list_selection: tauri::State<'_, SceneListSelection>,
    recall: tauri::State<'_, PlaybackRecall>,
) -> Result<(String, usize), String> {
    let _recall_edit = recall.suspend_for_edit();
    let scene_list_name = scene_names.create_scene_list(&name)?;
    let count = scene_names.load_scene_list(&scene_list_name)?;
    scene_list_selection
        .save(&scene_list_name)
        .map_err(|error| error.to_string())?;
    Ok((scene_list_name, count))
}

#[tauri::command]
fn load_scene_list(
    scene_list_name: String,
    scene_names: tauri::State<'_, SceneNames>,
    scene_list_selection: tauri::State<'_, SceneListSelection>,
    recall: tauri::State<'_, PlaybackRecall>,
) -> Result<usize, String> {
    let _recall_edit = recall.suspend_for_edit();
    let count = scene_names.load_scene_list(&scene_list_name)?;
    scene_list_selection
        .save(&scene_list_name)
        .map_err(|error| error.to_string())?;
    Ok(count)
}

#[tauri::command]
fn rename_scene_list(
    scene_list_name: String,
    new_name: String,
    scene_names: tauri::State<'_, SceneNames>,
    scene_list_selection: tauri::State<'_, SceneListSelection>,
) -> Result<String, String> {
    let renamed_scene_list = scene_names.rename_scene_list(&scene_list_name, &new_name)?;
    if scene_list_selection
        .load()
        .map_err(|error| error.to_string())?
        .as_deref()
        == Some(scene_list_name.as_str())
    {
        scene_list_selection
            .save(&renamed_scene_list)
            .map_err(|error| error.to_string())?;
    }
    Ok(renamed_scene_list)
}

#[tauri::command]
fn delete_scene_list(
    scene_list_name: String,
    scene_names: tauri::State<'_, SceneNames>,
    scene_list_selection: tauri::State<'_, SceneListSelection>,
) -> Result<(), String> {
    if scene_list_selection
        .load()
        .map_err(|error| error.to_string())?
        .as_deref()
        == Some(scene_list_name.as_str())
    {
        return Err("cannot delete the loaded scene list; load another list first".to_string());
    }

    scene_names.delete_scene_list(&scene_list_name)
}

#[tauri::command]
fn get_bridge_status(bridge: tauri::State<'_, bridge::Bridge>) -> bridge::Snapshot {
    bridge.snapshot()
}
#[tauri::command]
fn save_bridge_settings(
    config: bridge::Config,
    bridge: tauri::State<'_, bridge::Bridge>,
) -> Result<(), String> {
    bridge.save(config)
}
#[tauri::command]
fn set_bridge_listening(requested: bool, bridge: tauri::State<'_, bridge::Bridge>) {
    bridge.listen(requested);
}
#[tauri::command]
fn reconnect_bridge(bridge: tauri::State<'_, bridge::Bridge>) {
    bridge.reconnect();
}

struct MenuBar {
    status: tauri::menu::MenuItem<tauri::Wry>,
    toggle: tauri::menu::MenuItem<tauri::Wry>,
    reconnect: tauri::menu::MenuItem<tauri::Wry>,
}
fn status_icon(status: &str) -> tauri::image::Image<'static> {
    let color = match status {
        "connected" => [42, 178, 83, 255],
        "connecting" | "waiting" => [230, 163, 35, 255],
        "error" => [219, 76, 65, 255],
        _ => [143, 143, 147, 255],
    };
    // macOS displays this at 30 × 18 points. Render at 2× with coverage
    // antialiasing so the connector ring and pins stay clear on Retina screens.
    const WIDTH: usize = 60;
    const HEIGHT: usize = 36;
    const SAMPLES: usize = 4;
    let pins: [(f64, f64); 6] = [
        (9.0, 4.8),
        (5.2, 7.0),
        (12.8, 7.0),
        (5.8, 11.0),
        (12.2, 11.0),
        (9.0, 13.0),
    ];
    let mut rgba = vec![0; WIDTH * HEIGHT * 4];
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let mut covered = 0;
            for sy in 0..SAMPLES {
                for sx in 0..SAMPLES {
                    let px = (x as f64 + (sx as f64 + 0.5) / SAMPLES as f64) / 2.0;
                    let py = (y as f64 + (sy as f64 + 0.5) / SAMPLES as f64) / 2.0;
                    let radius = (px - 9.0).powi(2) + (py - 9.0).powi(2);
                    let ring = (5.7_f64.powi(2)..=7.3_f64.powi(2)).contains(&radius);
                    let pin = pins
                        .iter()
                        .any(|&(cx, cy)| (px - cx).powi(2) + (py - cy).powi(2) <= 1.0);
                    let play = (20.0..=28.5).contains(&px) && (py - 9.0).abs() <= (28.5 - px) * 0.7;
                    if ring || pin || play {
                        covered += 1;
                    }
                }
            }
            if covered > 0 {
                let index = (y * WIDTH + x) * 4;
                rgba[index..index + 3].copy_from_slice(&color[..3]);
                rgba[index + 3] = (covered * 255 / (SAMPLES * SAMPLES)) as u8;
            }
        }
    }
    tauri::image::Image::new_owned(rgba, WIDTH as u32, HEIGHT as u32)
}
fn refresh_menu_bar(app: &tauri::AppHandle) {
    let (Some(bridge), Some(menu)) = (
        app.try_state::<bridge::Bridge>(),
        app.try_state::<MenuBar>(),
    ) else {
        return;
    };
    let snapshot = bridge.snapshot();
    let source = if snapshot.config.transport == "tcp" {
        "MixRack TCP"
    } else {
        "MIDI input"
    };
    let label = match snapshot.status.as_str() {
        "connected" => "Open",
        "connecting" => "Connecting",
        "waiting" => "Waiting",
        "error" => "Error",
        _ => "Stopped",
    };
    let _ = menu.status.set_text(format!("{source}: {label}"));
    let _ = menu.toggle.set_text(if snapshot.requested {
        "Stop Listening"
    } else {
        "Start Listening"
    });
    let _ = menu.reconnect.set_enabled(snapshot.requested);
    if let Some(tray) = app.tray_by_id("bridge") {
        let _ = tray.set_icon(Some(status_icon(&snapshot.status)));
        let _ = tray.set_tooltip(Some(format!("dLive REAPER Link — {source}: {label}")));
    }
}
#[tauri::command]
fn open_settings(app: tauri::AppHandle, view: String) -> Result<(), String> {
    if !matches!(view.as_str(), "scene-lists" | "setup" | "diagnostics") {
        return Err("Unknown settings view".into());
    }
    let window = app
        .get_webview_window("settings")
        .ok_or("Settings window unavailable")?;
    if let Some(live) = app.get_webview_window("live") {
        let _ = live.hide();
    }
    #[cfg(target_os = "macos")]
    app.set_activation_policy(tauri::ActivationPolicy::Regular)
        .map_err(|e| e.to_string())?;
    window
        .emit("open-settings", view)
        .map_err(|e| e.to_string())?;
    window.unminimize().map_err(|e| e.to_string())?;
    window.show().map_err(|e| e.to_string())?;
    window.set_focus().map_err(|e| e.to_string())
}

#[tauri::command]
fn hide_live(app: tauri::AppHandle) {
    if let Some(live) = app.get_webview_window("live") {
        let _ = live.hide();
    }
}

#[tauri::command]
fn resize_live(window: tauri::WebviewWindow, height: u32) -> Result<(), String> {
    if window.label() != "live" {
        return Err("Only the Live dropdown can resize to its content".into());
    }
    window
        .set_size(tauri::LogicalSize::new(320.0, f64::from(height.max(200))))
        .map_err(|error| error.to_string())
}

fn toggle_live(app: &tauri::AppHandle, rect: tauri::Rect) -> tauri::Result<()> {
    let Some(window) = app.get_webview_window("live") else {
        return Ok(());
    };
    if window.is_visible()? {
        return window.hide();
    }
    // Tray coordinates are physical. Clamp to the tray's monitor, including Retina displays.
    let position = rect.position.to_physical::<f64>(1.0);
    let size = rect.size.to_physical::<f64>(1.0);
    let monitor = app.monitor_from_point(position.x, position.y)?;
    let scale = monitor.as_ref().map_or(1.0, |m| m.scale_factor());
    let width = 320.0 * scale;
    let height = f64::from(window.outer_size()?.height);
    let mut x = position.x + (size.width - width) / 2.0;
    let mut y = position.y + size.height;
    if let Some(monitor) = monitor {
        let area = monitor.work_area();
        let left = f64::from(area.position.x);
        let top = f64::from(area.position.y);
        x = x.clamp(left, (left + f64::from(area.size.width) - width).max(left));
        y = y.clamp(top, (top + f64::from(area.size.height) - height).max(top));
    }
    window.set_position(tauri::PhysicalPosition::new(x as i32, y as i32))?;
    window.show()?;
    window.set_focus()
}

fn setup_menu_bar(app: &mut tauri::App) -> tauri::Result<()> {
    use tauri::{
        menu::{Menu, MenuItem, PredefinedMenuItem},
        tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    };
    let status = MenuItem::with_id(app, "status", "Scene source: Stopped", false, None::<&str>)?;
    let note = MenuItem::with_id(
        app,
        "note",
        "REAPER marker delivery is separate",
        false,
        None::<&str>,
    )?;
    let toggle = MenuItem::with_id(app, "toggle", "Start Listening", true, None::<&str>)?;
    let reconnect = MenuItem::with_id(app, "reconnect", "Reconnect", false, None::<&str>)?;
    let preferences = MenuItem::with_id(app, "preferences", "Preferences…", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app)?;
    let menu = Menu::with_items(
        app,
        &[
            &status,
            &note,
            &separator,
            &toggle,
            &reconnect,
            &preferences,
            &quit,
        ],
    )?;
    TrayIconBuilder::with_id("bridge")
        .icon(status_icon("stopped"))
        .icon_as_template(false)
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                rect,
                ..
            } = event
            {
                if let Err(error) = toggle_live(tray.app_handle(), rect) {
                    eprintln!("Could not open Live: {error}");
                }
            }
        })
        .on_menu_event(|app, event| match event.id.as_ref() {
            "toggle" => {
                let bridge = app.state::<bridge::Bridge>();
                bridge.listen(!bridge.snapshot().requested);
                refresh_menu_bar(app);
            }
            "reconnect" => {
                app.state::<bridge::Bridge>().reconnect();
                refresh_menu_bar(app);
            }
            "preferences" => {
                let _ = open_settings(app.clone(), "setup".into());
            }
            "quit" => {
                app.state::<bridge::Bridge>().listen(false);
                app.exit(0);
            }
            _ => {}
        })
        .build(app)?;
    app.manage(MenuBar {
        status,
        toggle,
        reconnect,
    });
    refresh_menu_bar(app.handle());
    #[cfg(target_os = "macos")]
    app.set_activation_policy(tauri::ActivationPolicy::Accessory);
    Ok(())
}

fn start_bridge(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let config_dir = app.path().app_config_dir()?;
    let path = config_dir.join("bridge_settings.json");
    let saved_midi = app
        .state::<MidiInputSelection>()
        .load()?
        .unwrap_or_default();
    let config = bridge::Bridge::load(path.clone(), saved_midi).map_err(std::io::Error::other)?;
    let scene_names = Arc::clone(&app.state::<SceneNames>().names);
    let reaper_osc_config = Arc::clone(&app.state::<ReaperOscSettings>().config);
    let reaper_osc_feedback = app.state::<reaper_osc::ReaperOscFeedback>().inner().clone();
    let marker_mode = Arc::clone(&app.state::<MarkerModeState>().mode);
    let recent_scene_recall = Mutex::new(None);
    let app_handle = app.handle().clone();
    let bridge = bridge::Bridge::new(config, path, Some(app.handle().clone()), move |recall| {
        let app = &app_handle;
        let scene_number = recall.scene_number;
        let program = recall.program;
        let channel = recall.channel;
        let bank_number = recall.bank;
        let bank_lsb = recall.lsb;
        let scene_name = scene_name(&scene_names, scene_number);

        let _ = app.emit(
            "midi-scene-recall",
            (
                scene_number,
                scene_name.clone(),
                program,
                channel,
                bank_number,
                bank_lsb,
            ),
        );

        let playback_recall = app.state::<PlaybackRecall>();
        let playback_state = playback_recall.state.lock().unwrap();
        if playback_state.blocks_marker(scene_number, recall.received_at) {
            let _ = app.emit(
                "marker-decision",
                "Marker creation suspended for playback recall",
            );
            return;
        }
        if should_create_marker(&recent_scene_recall, scene_number, recall.received_at) {
            let decision = match marker_mode
                            .read()
                            .map(|mode| *mode)
                            .unwrap_or(MarkerMode::RecordingOnly)
                        {
                            MarkerMode::Always => match create_reaper_marker(
                                &reaper_osc_config,
                                &reaper_osc_feedback,
                                &scene_name,
                            ) {
                                Ok(_) => format!("Scene {scene_number} \"{scene_name}\" - marker created. Marker Mode: Always."),
                                Err(error) => format!("Scene {scene_number} \"{scene_name}\" - marker creation failed: {error}. Marker Mode: Always."),
                            },
                            MarkerMode::RecordingOnly => match reaper_osc_feedback.recording() {
                                Some(true) => match create_reaper_marker(
                                    &reaper_osc_config,
                                    &reaper_osc_feedback,
                                    &scene_name,
                                ) {
                                    Ok(_) => format!("Scene {scene_number} \"{scene_name}\" - marker created. Marker Mode: Recording Only, REAPER was recording."),
                                    Err(error) => format!("Scene {scene_number} \"{scene_name}\" - marker creation failed: {error}. Marker Mode: Recording Only, REAPER was recording."),
                                },
                                Some(false) => format!("Scene {scene_number} \"{scene_name}\" - marker skipped. Marker Mode: Recording Only, REAPER was not recording."),
                                None => format!("Scene {scene_number} \"{scene_name}\" - marker skipped. Marker Mode: Recording Only, REAPER transport state unknown."),
                            }
                        };
            let _ = app.emit("marker-decision", decision);
        }
    });
    app.manage(bridge);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        bridge, decode_scene_number, process_playback_recall, reaper_osc, sanitize_scene_list_name,
        scene_number_for_marker, Duration, Instant, RecallState,
    };

    #[test]
    fn playback_recall_simulation_covers_dedup_recording_loops_and_disconnect() {
        use std::{io::Read, net::TcpListener, path::PathBuf, thread};
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let bridge = bridge::Bridge::new(
            bridge::Config {
                transport: "tcp".into(),
                host: "127.0.0.1".into(),
                port: listener.local_addr().unwrap().port(),
                channel: 12,
                midi_input: String::new(),
            },
            PathBuf::new(),
            None,
            |_| {},
        );
        let deadline = Instant::now() + Duration::from_secs(3);
        let session = loop {
            if let Ok(session) = bridge.send_session() {
                break session;
            }
            assert!(Instant::now() < deadline);
            thread::sleep(Duration::from_millis(10));
        };
        let (mut socket, _) = listener.accept().unwrap();
        socket
            .set_read_timeout(Some(Duration::from_millis(100)))
            .unwrap();
        let names = vec!["Song A".into(), "".into(), "Song B".into()];
        let now = Instant::now();
        let mut feedback = reaper_osc::PlaybackFeedback {
            playing: Some(true),
            paused: Some(false),
            stopped: Some(false),
            recording: Some(false),
            marker_name: Some("Song A".into()),
            marker_number: Some("99".into()),
            position: Some(5.0),
            changed_at: Some(now - Duration::from_millis(200)),
            received_at: Some(now),
            ..Default::default()
        };
        let mut state = RecallState::default();
        process_playback_recall(&mut state, &bridge, &feedback, &names, now);
        assert_eq!(state.last_scene, None, "off by default");

        state.session = Some(session);
        process_playback_recall(&mut state, &bridge, &feedback, &names, now);
        let mut bytes = [0; 5];
        socket.read_exact(&mut bytes).unwrap();
        assert_eq!(
            bytes,
            [0xbb, 0, 0, 0xcb, 0],
            "starting mid-song uses the name, not marker 99"
        );

        feedback.playing = Some(false);
        feedback.stopped = Some(true);
        process_playback_recall(&mut state, &bridge, &feedback, &names, now);
        feedback.playing = Some(true);
        feedback.stopped = Some(false);
        process_playback_recall(&mut state, &bridge, &feedback, &names, now);
        assert_eq!(state.last_scene, Some(1), "Stop/Play retains scene");

        feedback.paused = Some(true);
        process_playback_recall(&mut state, &bridge, &feedback, &names, now);
        feedback.paused = Some(false);
        feedback.position = Some(2.0); // same-song loop
        process_playback_recall(&mut state, &bridge, &feedback, &names, now);
        let error = socket.read(&mut bytes).unwrap_err();
        assert!(
            matches!(
                error.kind(),
                std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
            ),
            "Stop/Play, pause/resume and same-song loops must not repeat recalls"
        );

        feedback.marker_name = Some("Song B".into());
        feedback.recording = Some(true);
        process_playback_recall(&mut state, &bridge, &feedback, &names, now);
        assert_eq!(state.last_scene, Some(1), "recording must not send");
        assert!(socket.read(&mut bytes).is_err());
        feedback.recording = Some(false);
        feedback.changed_at = Some(now);
        process_playback_recall(&mut state, &bridge, &feedback, &names, now);
        assert_eq!(
            state.last_scene,
            Some(1),
            "separate feedback messages must settle"
        );
        feedback.changed_at = Some(now - Duration::from_millis(200));
        process_playback_recall(&mut state, &bridge, &feedback, &names, now);
        socket.read_exact(&mut bytes).unwrap();
        assert_eq!(bytes, [0xbb, 0, 0, 0xcb, 2], "blank slot 2 is preserved");

        feedback.marker_name = Some("Song A".into()); // cross-song loop/seek destination
        process_playback_recall(&mut state, &bridge, &feedback, &names, now);
        socket.read_exact(&mut bytes).unwrap();
        assert_eq!(bytes, [0xbb, 0, 0, 0xcb, 0]);
        feedback.marker_name = Some("Unknown".into());
        process_playback_recall(&mut state, &bridge, &feedback, &names, now);
        assert!(socket.read(&mut bytes).is_err());

        assert!(
            state.blocks_marker(500, now),
            "all incoming recalls blocked while enabled, even Always mode"
        );
        state.disable("Off", now);
        assert!(
            state.blocks_marker(1, now + Duration::from_millis(100)),
            "late sent-scene echo blocked"
        );
        assert!(
            state.blocks_marker(3, now - Duration::from_millis(1)),
            "queued recalls from before disable blocked"
        );
        assert!(
            !state.blocks_marker(3, now + Duration::from_millis(100)),
            "fresh desk recalls allowed when off"
        );

        state.session = Some(session);
        bridge.reconnect();
        process_playback_recall(&mut state, &bridge, &feedback, &names, now);
        assert!(state.session.is_none(), "connection change disarms");
        let deadline = Instant::now() + Duration::from_secs(3);
        while bridge.send_session().is_err() {
            assert!(Instant::now() < deadline);
            thread::sleep(Duration::from_millis(10));
        }
        let (mut recovered, _) = listener.accept().unwrap();
        recovered
            .set_read_timeout(Some(Duration::from_millis(100)))
            .unwrap();
        feedback.marker_name = Some("Song B".into());
        process_playback_recall(&mut state, &bridge, &feedback, &names, now);
        assert!(
            recovered.read(&mut bytes).is_err(),
            "reconnect must not rearm or replay"
        );
        state.session = Some(bridge.send_session().unwrap());
        feedback.received_at = Some(now - Duration::from_secs(3));
        process_playback_recall(&mut state, &bridge, &feedback, &names, now);
        assert!(state.session.is_none(), "silent REAPER feedback disarms");
        bridge.listen(false);
    }

    #[test]
    fn playback_mapping_preserves_slots_and_rejects_ambiguity() {
        let names = vec!["Song A".into(), "".into(), "Song B".into()];
        assert_eq!(scene_number_for_marker(&names, "Song B"), Ok(Some(3)));
        assert_eq!(scene_number_for_marker(&names, "song b"), Ok(None));
        assert_eq!(scene_number_for_marker(&names, ""), Ok(None));
        assert!(scene_number_for_marker(&["Song A".into(), "Song A".into()], "Song A").is_err());
    }

    #[test]
    fn decodes_banked_dlive_scene_numbers() {
        assert_eq!(decode_scene_number(0, 0), Some(1));
        assert_eq!(decode_scene_number(0, 127), Some(128));
        assert_eq!(decode_scene_number(1, 0), Some(129));
        assert_eq!(decode_scene_number(1, 1), Some(130));
        assert_eq!(decode_scene_number(1, 127), Some(256));
        assert_eq!(decode_scene_number(2, 0), Some(257));
    }

    #[test]
    fn ignores_scene_numbers_beyond_the_scene_list() {
        assert_eq!(decode_scene_number(3, 116), None);
    }

    #[test]
    fn sanitizes_scene_list_names() {
        assert_eq!(sanitize_scene_list_name("  Friday / Set  "), "Friday _ Set");
        assert_eq!(sanitize_scene_list_name("../../"), "");
    }
}

#[tauri::command]
fn test_reaper_osc_marker(
    reaper_osc_settings: tauri::State<'_, ReaperOscSettings>,
    reaper_osc_feedback: tauri::State<'_, reaper_osc::ReaperOscFeedback>,
) -> Result<String, String> {
    create_reaper_marker(
        &reaper_osc_settings.config,
        reaper_osc_feedback.inner(),
        "REAPER CONNECTION TEST",
    )
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(
            tauri_plugin_window_state::Builder::default()
                .with_denylist(&["live"])
                .with_state_flags(
                    tauri_plugin_window_state::StateFlags::SIZE
                        | tauri_plugin_window_state::StateFlags::POSITION,
                )
                .build(),
        )
        .setup(|app| {
            let scene_file = app.path().app_data_dir()?.join("dLive_Scenes.txt");
            let app_config_dir = app.path().app_config_dir()?;
            let midi_input_selection = app_config_dir.join("selected_midi_input.txt");
            let reaper_osc_settings = app_config_dir.join("reaper_osc_settings.txt");
            let scene_list_selection = app_config_dir.join("loaded_scene_list.txt");
            app.manage(SceneNames::load(&scene_file)?);
            app.manage(MidiInputSelection {
                path: midi_input_selection,
            });
            let reaper_osc_settings = ReaperOscSettings::load(reaper_osc_settings)?;
            let reaper_osc_feedback = reaper_osc::ReaperOscFeedback::start(
                &reaper_osc_settings
                    .feedback_config()
                    .map_err(std::io::Error::other)?,
            )?;
            app.manage(reaper_osc_settings);
            app.manage(reaper_osc_feedback);
            app.manage(MarkerModeState::new());
            app.manage(SceneListSelection {
                path: scene_list_selection,
            });
            app.manage(PlaybackRecall::new());
            start_bridge(app)?;
            start_playback_recall(app.handle().clone());
            setup_menu_bar(app)?;
            Ok(())
        })
        .on_window_event(|window, event| {
            if window.label() == "live" {
                if matches!(event, tauri::WindowEvent::Focused(false)) {
                    let _ = window.hide();
                }
            }
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                if window.hide().is_ok() && window.label() == "settings" {
                    #[cfg(target_os = "macos")]
                    let _ = window
                        .app_handle()
                        .set_activation_policy(tauri::ActivationPolicy::Accessory);
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            open_settings,
            hide_live,
            resize_live,
            list_midi_inputs,
            get_saved_midi_input,
            save_midi_input,
            get_reaper_osc_settings,
            save_reaper_osc_settings,
            get_marker_mode,
            save_marker_mode,
            get_scene_file_path,
            reveal_scene_file,
            get_scene_names_count,
            reload_scene_names,
            get_scene_name_entries,
            save_scene_names,
            export_scene_list,
            import_scene_list,
            get_loaded_scene_list,
            list_saved_scene_lists,
            save_scene_list_as,
            create_scene_list,
            load_scene_list,
            rename_scene_list,
            delete_scene_list,
            set_playback_recall,
            get_playback_preview,
            get_bridge_status,
            save_bridge_settings,
            set_bridge_listening,
            reconnect_bridge,
            test_reaper_osc_marker
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
