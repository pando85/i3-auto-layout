use anyhow::Error;
use async_trait::async_trait;
use swayipc_async::{
    Connection as SwayConnection, EventType as SwayEventType, Node as SwayNode,
    NodeLayout as SwayNodeLayout, Rect as SwayRect,
};
use swayipc_types::WindowChange;
use tokio_stream::StreamExt;

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
        if rect.width > rect.height {
            "split h"
        } else {
            "split v"
        }
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
        if let Ok(mut sway) = SwayConnection::new().await {
            if sway.get_tree().await.is_ok() {
                return Ok(true);
            }
        }
        Ok(false)
    }
    async fn new_connection() -> Result<Self::Connection, Error> {
        Ok(SwayConn(SwayConnection::new().await?))
    }
    // --- trait required methods ---
    async fn get_tree(conn: &mut Self::Connection) -> Result<Self::Node, Error> {
        conn.0.get_tree().await.map_err(Error::from)
    }
    async fn subscribe_window_events(
        conn: &mut Self::Connection,
    ) -> Result<Box<dyn futures::Stream<Item = Result<Self::Event, Error>> + Send + Unpin>, Error>
    {
        // Take ownership of the connection for listen
        let owned = std::mem::replace(conn, SwayConn(SwayConnection::new().await?));
        let stream = owned.0.subscribe(&[SwayEventType::Window]).await?;
        Ok(Box::new(stream.map(|e| e.map_err(Error::from))))
    }
    fn extract_window_event(ev: &Self::Event) -> Option<&Self::Node> {
        if let swayipc_async::Event::Window(win_ev) = ev {
            Some(&win_ev.container)
        } else {
            None
        }
    }
    fn window_change_is_focus(ev: &Self::Event) -> bool {
        if let swayipc_async::Event::Window(win_ev) = ev {
            win_ev.change == WindowChange::Focus
        } else {
            false
        }
    }
    async fn send_command(conn: &mut Self::Connection, cmd: &str) -> Result<(), Error> {
        conn.0.run_command(cmd).await?;
        Ok(())
    }
}
