use serde::Serialize;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use zbus::fdo::{self, RequestNameFlags};
use zbus::interface;
use zbus::object_server::SignalEmitter;
use zbus::zvariant::{self, OwnedValue, Type};

use super::Start;

// Reuse the existing IpcOutputMap type (Arc<Mutex<HashMap<String, OutputInfo>>>)
// from mutter_screen_cast or define a shared type.

pub struct DisplayConfig {
    ipc_outputs: Arc<Mutex<HashMap<String, super::mutter_screen_cast::OutputInfo>>>,
}

#[derive(Serialize, Type)]
pub struct Monitor {
    names: (String, String, String, String), // (connector, make, model, serial)
    modes: Vec<Mode>,
    properties: HashMap<String, OwnedValue>,
}

#[derive(Serialize, Type)]
pub struct Mode {
    id: String,
    width: i32,
    height: i32,
    refresh_rate: f64,
    preferred_scale: f64,
    supported_scales: Vec<f64>,
    properties: HashMap<String, OwnedValue>,
}

#[derive(Serialize, Type)]
pub struct LogicalMonitor {
    x: i32,
    y: i32,
    scale: f64,
    transform: u32,
    is_primary: bool,
    monitors: Vec<(String, String, String, String)>,
    properties: HashMap<String, OwnedValue>,
}

#[interface(name = "org.gnome.Mutter.DisplayConfig")]
impl DisplayConfig {
    async fn get_current_state(
        &self,
    ) -> fdo::Result<(
        u32,
        Vec<Monitor>,
        Vec<LogicalMonitor>,
        HashMap<String, OwnedValue>,
    )> {
        let mut monitors = Vec::new();
        let mut logical_monitors = Vec::new();

        for output in self.ipc_outputs.lock().unwrap().values() {
            let connector = output.name.clone();
            // srwc doesn't track make/model/serial yet, use placeholders
            let make = String::from("Unknown");
            let model = String::from("Unknown");
            let serial = connector.clone();
            let names = (connector, make, model, serial);

            let mut properties = HashMap::new();
            properties.insert(
                String::from("display-name"),
                OwnedValue::from(zvariant::Str::from(output.name.clone())),
            );
            properties.insert(
                String::from("is-builtin"),
                OwnedValue::from(output.name.starts_with("eDP")),
            );

            // Create a single mode from the output dimensions
            // You'll need to store refresh rate in OutputInfo, or default to 60Hz
            let width = output.width as i32;
            let height = output.height as i32;
            let refresh_rate = 60.0; // TODO: store actual refresh rate in OutputInfo  
            let mode = Mode {
                id: format!("{width}x{height}@{refresh_rate:.3}"),
                width,
                height,
                refresh_rate,
                preferred_scale: 1.,
                supported_scales: vec![1., 1.25, 1.5, 1.75, 2.],
                properties: HashMap::from([
                    (String::from("is-current"), OwnedValue::from(true)),
                    (String::from("is-preferred"), OwnedValue::from(true)),
                ]),
            };

            logical_monitors.push(LogicalMonitor {
                x: output.x,
                y: output.y,
                scale: 1., // TODO: store actual scale in OutputInfo
                transform: 0,
                is_primary: false,
                monitors: vec![names.clone()],
                properties: HashMap::new(),
            });

            monitors.push(Monitor {
                names,
                modes: vec![mode],
                properties,
            });
        }

        let properties = HashMap::from([(String::from("layout-mode"), OwnedValue::from(1u32))]);
        Ok((0, monitors, logical_monitors, properties))
    }

    // apply_monitors_config can be a no-op stub for now
    async fn apply_monitors_config(
        &self,
        _serial: u32,
        _method: u32,
        _logical_monitor_configs: Vec<zvariant::Value<'_>>,
        _properties: HashMap<String, OwnedValue>,
    ) -> fdo::Result<()> {
        Err(fdo::Error::NotSupported("not yet implemented".to_owned()))
    }

    #[zbus(signal)]
    pub async fn monitors_changed(ctxt: &SignalEmitter<'_>) -> zbus::Result<()>;

    #[zbus(property)]
    fn power_save_mode(&self) -> i32 {
        -1
    }

    #[zbus(property)]
    fn set_power_save_mode(&self, _mode: i32) -> zbus::Result<()> {
        Err(zbus::Error::Unsupported)
    }

    #[zbus(property)]
    fn panel_orientation_managed(&self) -> bool {
        false
    }

    #[zbus(property)]
    fn apply_monitors_config_allowed(&self) -> bool {
        true
    }

    #[zbus(property)]
    fn night_light_supported(&self) -> bool {
        false
    }
}

impl DisplayConfig {
    pub fn new(
        ipc_outputs: Arc<Mutex<HashMap<String, super::mutter_screen_cast::OutputInfo>>>,
    ) -> Self {
        Self { ipc_outputs }
    }
}

impl Start for DisplayConfig {
    fn start(self) -> anyhow::Result<zbus::blocking::Connection> {
        let conn = zbus::blocking::Connection::session()?;
        let flags = RequestNameFlags::AllowReplacement
            | RequestNameFlags::ReplaceExisting
            | RequestNameFlags::DoNotQueue;
        conn.object_server()
            .at("/org/gnome/Mutter/DisplayConfig", self)?;
        conn.request_name_with_flags("org.gnome.Mutter.DisplayConfig", flags)?;
        Ok(conn)
    }
}
