use std::collections::HashMap;
use zbus::fdo::{self, RequestNameFlags};
use zbus::interface;
use zbus::object_server::SignalEmitter;
use zbus::zvariant::{SerializeDict, Type, Value};

use super::Start;

pub struct Introspect {
    to_srwm: calloop::channel::Sender<IntrospectToSrwm>,
    from_srwm: async_channel::Receiver<SrwmToIntrospect>,
}

pub enum IntrospectToSrwm {
    GetWindows,
}

pub enum SrwmToIntrospect {
    Windows(HashMap<u64, WindowProperties>),
}

#[derive(Debug, SerializeDict, Type, Value)]
#[zvariant(signature = "dict")]
pub struct WindowProperties {
    pub title: String,
    #[zvariant(rename = "app-id")]
    pub app_id: String,
}

#[interface(name = "org.gnome.Shell.Introspect")]
impl Introspect {
    async fn get_windows(&self) -> fdo::Result<HashMap<u64, WindowProperties>> {
        if let Err(err) = self.to_srwm.send(IntrospectToSrwm::GetWindows) {
            tracing::warn!("error sending message to srwm: {err:?}");
            return Err(fdo::Error::Failed("internal error".to_owned()));
        }
        match self.from_srwm.recv().await {
            Ok(SrwmToIntrospect::Windows(windows)) => Ok(windows),
            Err(err) => {
                tracing::warn!("error receiving from srwm: {err:?}");
                Err(fdo::Error::Failed("internal error".to_owned()))
            }
        }
    }

    #[zbus(signal)]
    pub async fn windows_changed(ctxt: &SignalEmitter<'_>) -> zbus::Result<()>;
}

impl Introspect {
    pub fn new(
        to_srwm: calloop::channel::Sender<IntrospectToSrwm>,
        from_srwm: async_channel::Receiver<SrwmToIntrospect>,
    ) -> Self {
        Self { to_srwm, from_srwm }
    }
}

impl Start for Introspect {
    fn start(self) -> anyhow::Result<zbus::blocking::Connection> {
        let conn = zbus::blocking::Connection::session()?;
        let flags = RequestNameFlags::AllowReplacement
            | RequestNameFlags::ReplaceExisting
            | RequestNameFlags::DoNotQueue;
        conn.object_server()
            .at("/org/gnome/Shell/Introspect", self)?;
        conn.request_name_with_flags("org.gnome.Shell.Introspect", flags)?;
        Ok(conn)
    }
}
