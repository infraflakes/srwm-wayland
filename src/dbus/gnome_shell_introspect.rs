use std::collections::HashMap;
use zbus::fdo::{self, RequestNameFlags};
use zbus::interface;
use zbus::object_server::SignalEmitter;
use zbus::zvariant::{SerializeDict, Type, Value};

use super::Start;

pub struct Introspect {
    to_srwc: calloop::channel::Sender<IntrospectToSrwc>,
    from_srwc: async_channel::Receiver<SrwcToIntrospect>,
}

pub enum IntrospectToSrwc {
    GetWindows,
}

pub enum SrwcToIntrospect {
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
        if let Err(err) = self.to_srwc.send(IntrospectToSrwc::GetWindows) {
            tracing::warn!("error sending message to srwc: {err:?}");
            return Err(fdo::Error::Failed("internal error".to_owned()));
        }
        match self.from_srwc.recv().await {
            Ok(SrwcToIntrospect::Windows(windows)) => Ok(windows),
            Err(err) => {
                tracing::warn!("error receiving from srwc: {err:?}");
                Err(fdo::Error::Failed("internal error".to_owned()))
            }
        }
    }

    #[zbus(signal)]
    pub async fn windows_changed(ctxt: &SignalEmitter<'_>) -> zbus::Result<()>;
}

impl Introspect {
    pub fn new(
        to_srwc: calloop::channel::Sender<IntrospectToSrwc>,
        from_srwc: async_channel::Receiver<SrwcToIntrospect>,
    ) -> Self {
        Self { to_srwc, from_srwc }
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
