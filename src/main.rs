use anyhow::{Error, Result};
use flexi_logger::DeferredNow;
use flexi_logger::TS_DASHES_BLANK_COLONS_DOT_BLANK;
use log::Record;
use tokio::sync::mpsc;
use tokio_i3ipc::{
    I3,
    event::{Event, Subscribe, WindowChange},
    msg::Msg,
    reply::{Node, NodeLayout, Rect},
};
use tokio_stream::StreamExt;

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

fn split_rect(r: Rect) -> &'static str {
    if r.width > r.height {
        "split h"
    } else {
        "split v"
    }
}

// walk the tree and determine if `window_id` has tabbed parent
fn has_tabbed_parent(node: &Node, window_id: usize, tabbed: bool) -> bool {
    if node.id == window_id {
        tabbed
    } else {
        node.nodes.iter().any(|child| {
            has_tabbed_parent(
                child,
                window_id,
                matches!(node.layout, NodeLayout::Tabbed | NodeLayout::Stacked),
            )
        })
    }
}

async fn run() -> Result<(), anyhow::Error> {
    let (send, mut recv) = mpsc::channel::<&'static str>(10);
    let s_handle = tokio::spawn(async move {
        let mut i3 = I3::connect().await?;
        i3.subscribe([Subscribe::Window]).await?;

        let mut event_listener = i3.listen();
        let mut i3_for_ops = I3::connect().await?;

        while let Some(event) = event_listener.next().await {
            match event {
                Ok(Event::Window(ev)) => {
                    if ev.change == WindowChange::Focus {
                        let is_tabbed = matches!(
                            ev.container.layout,
                            NodeLayout::Tabbed | NodeLayout::Stacked
                        );
                        let (name, tabbed_parent) = (
                            ev.container.name,
                            has_tabbed_parent(
                                &i3_for_ops.get_tree().await?,
                                ev.container.id,
                                is_tabbed,
                            ),
                        );
                        log::debug!("name={:?}, tabbed_parent={}", &name, tabbed_parent);
                        if !tabbed_parent {
                            send.send(split_rect(ev.container.window_rect)).await?;
                        }
                    }
                }
                Err(e) => {
                    log::error!("Error receiving window event: {:?}", e);
                }
                _ => {}
            }
        }
        log::debug!("Sender loop ended");
        Ok::<_, Error>(())
    });

    let r_handle = tokio::spawn(async move {
        let mut i3 = I3::connect().await?;
        while let Some(cmd) = recv.recv().await {
            i3.send_msg_body(Msg::RunCommand, cmd).await?;
        }
        log::debug!("Receiver loop ended");
        Ok::<_, Error>(())
    });

    let (send, recv) = tokio::try_join!(s_handle, r_handle)?;
    send.and(recv)?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    flexi_logger::Logger::try_with_env()?
        .format_for_stderr(ts_log_format)
        .use_utc()
        .start()?;

    loop {
        if let Err(e) = run().await {
            log::error!("Error: {}", e);
        }
    }
}
