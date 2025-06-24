use crate::backend::WMAdapter;
use crate::backend::i3::I3Adapter;
use crate::backend::sway::SwayAdapter;
use crate::backend::{generic_command_loop, generic_event_loop};

use anyhow::Result;
use flexi_logger::DeferredNow;
use flexi_logger::TS_DASHES_BLANK_COLONS_DOT_BLANK;
use log::Record;
use tokio::sync::mpsc;

mod backend;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WMBackend {
    I3,
    Sway,
}

pub struct WM {
    backend: WMBackend,
}

impl WM {
    /// Detects the running window manager backend (i3 or sway) by attempting a real IPC operation.
    pub async fn detect() -> Result<Self> {
        log::debug!("detecting window manager backend...");
        if I3Adapter::try_connection().await? {
            log::info!("detected i3 backend");
            Ok(Self {
                backend: WMBackend::I3,
            })
        } else if SwayAdapter::try_connection().await? {
            log::info!("detected sway backend");
            Ok(Self {
                backend: WMBackend::Sway,
            })
        } else {
            log::error!("neither i3 nor sway detected or both failed get_tree()");
            Err(anyhow::anyhow!(
                "Neither i3 nor sway detected or both failed get_tree()"
            ))
        }
    }
    pub fn backend(&self) -> WMBackend {
        self.backend
    }
}

pub fn ts_log_format(
    w: &mut dyn std::io::Write,
    now: &mut DeferredNow,
    record: &Record,
) -> Result<(), std::io::Error> {
    write!(
        w,
        "[{}] [{}:{}] ",
        now.format(TS_DASHES_BLANK_COLONS_DOT_BLANK),
        record.level(),
        record.line().unwrap_or(0),
    )?;

    write!(w, "{}", &record.args())
}

/// Main event and command loop, spawns backend-specific tasks.
async fn run() -> Result<()> {
    let (send, recv) = mpsc::channel::<&'static str>(10);
    log::debug!("starting backend event and command loops");
    let wm = WM::detect().await?;
    let backend = wm.backend();
    log::debug!("spawning event and command loops for {:?}", backend);
    match backend {
        WMBackend::I3 => {
            let event_conn = I3Adapter::new_connection().await?;
            let cmd_conn = I3Adapter::new_connection().await?;
            let s_handle = tokio::spawn(generic_event_loop::<I3Adapter>(event_conn, send));
            let r_handle = tokio::spawn(generic_command_loop::<I3Adapter>(cmd_conn, recv));
            let (send, recv) = tokio::try_join!(s_handle, r_handle)?;
            send.and(recv)?;
        }
        WMBackend::Sway => {
            let event_conn = SwayAdapter::new_connection().await?;
            let cmd_conn = SwayAdapter::new_connection().await?;
            let s_handle = tokio::spawn(generic_event_loop::<SwayAdapter>(event_conn, send));
            let r_handle = tokio::spawn(generic_command_loop::<SwayAdapter>(cmd_conn, recv));
            let (send, recv) = tokio::try_join!(s_handle, r_handle)?;
            send.and(recv)?;
        }
    }
    Ok(())
}

#[tokio::main]
pub async fn main() -> Result<()> {
    flexi_logger::Logger::try_with_env()?
        .format_for_stderr(ts_log_format)
        .use_utc()
        .start()?;
    const INITIAL_BACKOFF: f64 = 0.2;
    const MAX_BACKOFF: f64 = 5.0;
    let mut backoff = INITIAL_BACKOFF;

    loop {
        match run().await {
            Ok(_) => backoff = INITIAL_BACKOFF,
            Err(e) => {
                log::error!("{}. Retrying in {:.1}s", e, backoff);
                std::thread::sleep(std::time::Duration::from_secs_f64(backoff));
                backoff = (backoff * 2.0).min(MAX_BACKOFF);
            }
        }
    }
}
