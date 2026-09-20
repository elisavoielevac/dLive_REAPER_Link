pub mod reaper_osc;

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tauri::{Emitter, Manager};
use tauri_plugin_opener::OpenerExt;

const DUPLICATE_SCENE_RECALL_WINDOW: Duration = Duration::from_millis(250);
const SCENE_SLOT_COUNT: usize = 500;
const DLIVE_SCENE_RECALL_STATUS: u8 = 0xCB;
const DLIVE_BANK_SELECT_STATUS: u8 = 0xBB;
const MIDI_BANK_SELECT_MSB: u8 = 0;
const MIDI_BANK_SELECT_LSB: u8 = 32;

#[derive(Default)]
struct BankSelectState {
    msb: u8,
    lsb: u8,
}

impl BankSelectState {
    fn update(&mut self, controller: u8, value: u8) {
        match controller {
            MIDI_BANK_SELECT_MSB => self.msb = value,
            MIDI_BANK_SELECT_LSB => self.lsb = value,
            _ => {}
        }
    }

    fn d_live_bank(&self) -> (u8, u8) {
        // dLive uses the Bank Select MSB as its 128-scene bank number.
        (self.msb, self.lsb)
    }
}

#[derive(Default)]
struct MidiLogState {
    connection: Mutex<Option<midir::MidiInputConnection<()>>>,
    recent_scene_recall: Arc<Mutex<Option<(u16, Instant)>>>,
    bank_select: Arc<Mutex<BankSelectState>>,
}

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

fn decode_scene_number(bank_number: u8, program: u8) -> Option<u16> {
    let scene_number = u16::from(bank_number) * 128 + u16::from(program) + 1;
    (usize::from(scene_number) <= SCENE_SLOT_COUNT).then_some(scene_number)
}

fn should_create_marker(
    recent_scene_recall: &Mutex<Option<(u16, Instant)>>,
    scene_number: u16,
) -> bool {
    let Ok(mut last_scene_recall) = recent_scene_recall.lock() else {
        return true;
    };

    let now = Instant::now();
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
    let midi_input =
        midir::MidiInput::new("dLive REAPER Link").map_err(|error| error.to_string())?;

    midi_input
        .ports()
        .iter()
        .map(|port| {
            midi_input
                .port_name(port)
                .map_err(|error| error.to_string())
        })
        .collect()
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
) -> Result<(), String> {
    settings.save(host, port)
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
    app.opener()
        .reveal_item_in_dir(&scene_names.path)
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn get_scene_names_count(scene_names: tauri::State<'_, SceneNames>) -> Result<usize, String> {
    scene_names.count()
}

#[tauri::command]
fn reload_scene_names(scene_names: tauri::State<'_, SceneNames>) -> Result<usize, String> {
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
) -> Result<usize, String> {
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
) -> Result<(String, usize), String> {
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
) -> Result<String, String> {
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
) -> Result<(String, usize), String> {
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
) -> Result<usize, String> {
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
fn start_midi_log(
    port_name: String,
    app: tauri::AppHandle,
    state: tauri::State<'_, MidiLogState>,
    scene_names: tauri::State<'_, SceneNames>,
    reaper_osc_settings: tauri::State<'_, ReaperOscSettings>,
    reaper_osc_feedback: tauri::State<'_, reaper_osc::ReaperOscFeedback>,
    marker_mode_settings: tauri::State<'_, MarkerModeState>,
) -> Result<(), String> {
    let midi_input =
        midir::MidiInput::new("dLive REAPER Link").map_err(|error| error.to_string())?;
    let port = midi_input
        .ports()
        .into_iter()
        .find(|port| midi_input.port_name(port).ok().as_deref() == Some(port_name.as_str()))
        .ok_or_else(|| format!("MIDI input not found: {port_name}"))?;

    let mut active_connection = state
        .connection
        .lock()
        .map_err(|_| "MIDI log state is unavailable".to_string())?;
    if active_connection.is_some() {
        return Err("dLive Link is already running".to_string());
    }

    *state
        .bank_select
        .lock()
        .map_err(|_| "MIDI bank state is unavailable".to_string())? = BankSelectState::default();

    let recent_scene_recall = Arc::clone(&state.recent_scene_recall);
    let bank_select = Arc::clone(&state.bank_select);
    let scene_names = Arc::clone(&scene_names.names);
    let reaper_osc_config = Arc::clone(&reaper_osc_settings.config);
    let reaper_osc_feedback = reaper_osc_feedback.inner().clone();
    let marker_mode = Arc::clone(&marker_mode_settings.mode);
    let connection = midi_input
        .connect(
            &port,
            "dLive REAPER Link MIDI Log",
            move |_timestamp, message, _| {
                let raw_message = message
                    .iter()
                    .map(|byte| format!("{byte:02X}"))
                    .collect::<Vec<_>>()
                    .join(" ");

                let _ = app.emit("midi-message", raw_message);

                if let [DLIVE_BANK_SELECT_STATUS, controller, value, ..] = message {
                    if matches!(*controller, MIDI_BANK_SELECT_MSB | MIDI_BANK_SELECT_LSB) {
                        if let Ok(mut bank_select) = bank_select.lock() {
                            bank_select.update(*controller, *value);
                        }
                    }
                }

                if let [DLIVE_SCENE_RECALL_STATUS, program, ..] = message {
                    let (bank_number, bank_lsb) = bank_select
                        .lock()
                        .map(|bank_select| bank_select.d_live_bank())
                        .unwrap_or_default();
                    let Some(scene_number) = decode_scene_number(bank_number, *program) else {
                        return;
                    };
                    let scene_name = scene_name(&scene_names, scene_number);
                    let channel = (DLIVE_SCENE_RECALL_STATUS & 0x0F) + 1;
                    let _ = app.emit(
                        "midi-scene-recall",
                        (
                            scene_number,
                            scene_name.clone(),
                            *program,
                            channel,
                            bank_number,
                            bank_lsb,
                        ),
                    );

                    if should_create_marker(&recent_scene_recall, scene_number) {
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
                }
            },
            (),
        )
        .map_err(|error| error.to_string())?;

    *active_connection = Some(connection);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{decode_scene_number, sanitize_scene_list_name};

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
fn stop_midi_log(state: tauri::State<'_, MidiLogState>) -> Result<(), String> {
    let connection = state
        .connection
        .lock()
        .map_err(|_| "MIDI log state is unavailable".to_string())?
        .take()
        .ok_or_else(|| "dLive Link is not running".to_string())?;
    drop(connection);

    Ok(())
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
        .manage(MidiLogState::default())
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
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
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
            start_midi_log,
            stop_midi_log,
            test_reaper_osc_marker
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
