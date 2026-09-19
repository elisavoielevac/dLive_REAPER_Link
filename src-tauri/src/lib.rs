pub mod reaper_osc;

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};
use tauri::{Emitter, Manager};
use tauri_plugin_opener::OpenerExt;

const DUPLICATE_SCENE_RECALL_WINDOW: Duration = Duration::from_millis(250);

#[derive(Default)]
struct MidiLogState {
    connection: Mutex<Option<midir::MidiInputConnection<()>>>,
    recent_scene_recall: Arc<Mutex<Option<(u16, Instant)>>>,
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

struct SceneNames {
    names: Arc<RwLock<Vec<String>>>,
    path: PathBuf,
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
        })
    }

    fn reload(&self) -> Result<(), String> {
        let names = read_scene_names(&self.path).map_err(|error| error.to_string())?;
        let mut loaded_names = self
            .names
            .write()
            .map_err(|_| "scene names are unavailable".to_string())?;
        *loaded_names = names;
        Ok(())
    }
}

fn read_scene_names(scene_file: &Path) -> Result<Vec<String>, std::io::Error> {
    fs::read_to_string(scene_file).map(|contents| {
        contents
            .lines()
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
fn reload_scene_names(scene_names: tauri::State<'_, SceneNames>) -> Result<(), String> {
    scene_names.reload()
}

#[tauri::command]
fn start_midi_log(
    port_name: String,
    app: tauri::AppHandle,
    state: tauri::State<'_, MidiLogState>,
    scene_names: tauri::State<'_, SceneNames>,
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

    let recent_scene_recall = Arc::clone(&state.recent_scene_recall);
    let scene_names = Arc::clone(&scene_names.names);
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

                if let [0xCB, program, ..] = message {
                    let scene_number = u16::from(*program) + 1;
                    let scene_name = scene_name(&scene_names, scene_number);
                    let _ = app.emit("midi-scene-recall", (scene_number, scene_name.clone()));

                    if should_create_marker(&recent_scene_recall, scene_number) {
                        let _ = reaper_osc::ReaperOscClient::default().create_marker(&scene_name);
                    }
                }
            },
            (),
        )
        .map_err(|error| error.to_string())?;

    *active_connection = Some(connection);

    Ok(())
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
fn test_reaper_osc_marker() -> Result<String, String> {
    reaper_osc::ReaperOscClient::default()
        .create_marker("OSC TEST")
        .map_err(|error| error.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(MidiLogState::default())
        .setup(|app| {
            let scene_file = app.path().app_data_dir()?.join("dLive_Scenes.txt");
            let midi_input_selection = app.path().app_config_dir()?.join("selected_midi_input.txt");
            app.manage(SceneNames::load(&scene_file)?);
            app.manage(MidiInputSelection {
                path: midi_input_selection,
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_midi_inputs,
            get_saved_midi_input,
            save_midi_input,
            get_scene_file_path,
            reveal_scene_file,
            reload_scene_names,
            start_midi_log,
            stop_midi_log,
            test_reaper_osc_marker
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
