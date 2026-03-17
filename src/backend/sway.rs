use anyhow::Error;
use async_trait::async_trait;
use swayipc_async::{
    Connection as SwayConnection, EventType as SwayEventType, Node as SwayNode,
    NodeLayout as SwayNodeLayout, Rect as SwayRect,
};
use swayipc_types::WindowChange;
use tokio_stream::StreamExt;

use super::compute_split_direction;

/// Adapter for Sway window manager IPC.
pub struct SwayAdapter;
pub struct SwayConn(pub SwayConnection);

#[async_trait]
impl super::WMAdapter for SwayAdapter {
    type Node = SwayNode;
    type Rect = SwayRect;
    type Event = swayipc_async::Event;
    type Id = i64;
    type Connection = SwayConn;

    fn is_tabbed_layout(node: &Self::Node) -> bool {
        matches!(
            node.layout,
            SwayNodeLayout::Tabbed | SwayNodeLayout::Stacked
        )
    }

    fn get_id(node: &Self::Node) -> Self::Id {
        node.id
    }

    fn get_rect(node: &Self::Node) -> Self::Rect {
        node.rect
    }

    fn get_name(node: &Self::Node) -> Option<String> {
        node.name.clone()
    }

    fn split_rect(rect: &Self::Rect) -> &'static str {
        compute_split_direction(rect.width, rect.height)
    }

    fn has_tabbed_parent(node: &Self::Node, window_id: &Self::Id, tabbed: bool) -> bool {
        if &node.id == window_id {
            tabbed
        } else {
            node.nodes.iter().any(|child| {
                Self::has_tabbed_parent(child, window_id, Self::is_tabbed_layout(node))
            })
        }
    }

    async fn try_connection() -> anyhow::Result<bool> {
        match SwayConnection::new().await {
            Ok(mut sway) => match sway.get_tree().await {
                Ok(_) => Ok(true),
                Err(e) => {
                    log::debug!("sway connection succeeded but get_tree failed: {e}");
                    Ok(false)
                }
            },
            Err(e) => {
                log::debug!("sway connection failed: {e}");
                Ok(false)
            }
        }
    }

    async fn new_connection() -> Result<Self::Connection, Error> {
        SwayConnection::new()
            .await
            .map(SwayConn)
            .map_err(|e| anyhow::anyhow!("Failed to connect to sway: {}", e))
    }

    async fn get_tree(conn: &mut Self::Connection) -> Result<Self::Node, Error> {
        conn.0.get_tree().await.map_err(Error::from)
    }

    async fn subscribe_window_events(
        conn: &mut Self::Connection,
    ) -> Result<Box<dyn futures::Stream<Item = Result<Self::Event, Error>> + Send + Unpin>, Error>
    {
        let owned = std::mem::replace(conn, SwayConn(SwayConnection::new().await?));
        let stream = owned.0.subscribe(&[SwayEventType::Window]).await?;
        Ok(Box::new(stream.map(|e| e.map_err(Error::from))))
    }

    fn extract_window_event(ev: &Self::Event) -> Option<&Self::Node> {
        match ev {
            swayipc_async::Event::Window(win_ev) => Some(&win_ev.container),
            _ => None,
        }
    }

    fn window_change_is_focus(ev: &Self::Event) -> bool {
        match ev {
            swayipc_async::Event::Window(win_ev) => win_ev.change == WindowChange::Focus,
            _ => false,
        }
    }

    async fn send_command(conn: &mut Self::Connection, cmd: &str) -> Result<(), Error> {
        conn.0.run_command(cmd).await?;
        Ok(())
    }
}
