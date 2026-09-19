pub mod reaper_osc;

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};
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

    fn count(&self) -> Result<usize, String> {
        self.names
            .read()
            .map(|names| names.len())
            .map_err(|_| "scene names are unavailable".to_string())
    }

    fn reload(&self) -> Result<usize, String> {
        let names = read_scene_names(&self.path).map_err(|error| error.to_string())?;
        let mut loaded_names = self
            .names
            .write()
            .map_err(|_| "scene names are unavailable".to_string())?;
        *loaded_names = names;
        Ok(loaded_names.len())
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
    name: &str,
) -> Result<String, String> {
    let config = reaper_osc_config
        .read()
        .map_err(|_| "REAPER OSC settings are unavailable".to_string())?
        .clone();

    reaper_osc::ReaperOscClient::new(config)
        .create_marker(name)
        .map_err(|error| error.to_string())
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
) -> Result<usize, String> {
    scene_names.save(names)
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
) -> Result<usize, String> {
    scene_names.import_from(Path::new(&source_path))
}

#[tauri::command]
fn start_midi_log(
    port_name: String,
    app: tauri::AppHandle,
    state: tauri::State<'_, MidiLogState>,
    scene_names: tauri::State<'_, SceneNames>,
    reaper_osc_settings: tauri::State<'_, ReaperOscSettings>,
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
                        let _ = create_reaper_marker(&reaper_osc_config, &scene_name);
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
    use super::decode_scene_number;

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
) -> Result<String, String> {
    create_reaper_marker(&reaper_osc_settings.config, "REAPER CONNECTION TEST")
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
            app.manage(SceneNames::load(&scene_file)?);
            app.manage(MidiInputSelection {
                path: midi_input_selection,
            });
            app.manage(ReaperOscSettings::load(reaper_osc_settings)?);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_midi_inputs,
            get_saved_midi_input,
            save_midi_input,
            get_reaper_osc_settings,
            save_reaper_osc_settings,
            get_scene_file_path,
            reveal_scene_file,
            get_scene_names_count,
            reload_scene_names,
            get_scene_name_entries,
            save_scene_names,
            export_scene_list,
            import_scene_list,
            start_midi_log,
            stop_midi_log,
            test_reaper_osc_marker
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
