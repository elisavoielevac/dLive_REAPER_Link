pub mod reaper_osc;

use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tauri::{Emitter, Manager};

const DUPLICATE_SCENE_RECALL_WINDOW: Duration = Duration::from_millis(250);

#[derive(Default)]
struct MidiLogState {
    connection: Mutex<Option<midir::MidiInputConnection<()>>>,
    recent_scene_recall: Arc<Mutex<Option<(u16, Instant)>>>,
}

struct SceneNames(Arc<Vec<String>>);

impl SceneNames {
    fn load(scene_file: &Path) -> Result<Self, std::io::Error> {
        if let Some(parent) = scene_file.parent() {
            fs::create_dir_all(parent)?;
        }

        if !scene_file.exists() {
            fs::write(scene_file, include_str!("../dLive_Scenes.txt"))?;
        }

        let names = fs::read_to_string(scene_file)?
            .lines()
            .map(|line| line.trim().to_string())
            .collect();

        Ok(Self(Arc::new(names)))
    }
}

fn scene_name(scene_names: &[String], scene_number: u16) -> String {
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

    let previous_connection = state
        .connection
        .lock()
        .map_err(|_| "MIDI log state is unavailable".to_string())?
        .take();
    drop(previous_connection);

    let recent_scene_recall = Arc::clone(&state.recent_scene_recall);
    let scene_names = Arc::clone(&scene_names.0);
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

    let mut active_connection = state
        .connection
        .lock()
        .map_err(|_| "MIDI log state is unavailable".to_string())?;
    *active_connection = Some(connection);

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
            app.manage(SceneNames::load(&scene_file)?);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_midi_inputs,
            start_midi_log,
            test_reaper_osc_marker
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
