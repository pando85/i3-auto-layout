use anyhow::Error;
use async_trait::async_trait;
use tokio_i3ipc::{
    I3,
    event::{Event, Subscribe, WindowChange},
    msg::Msg,
    reply::{Node, NodeLayout, Rect},
};
use tokio_stream::StreamExt;

pub struct I3Adapter;
pub struct I3Conn(pub I3);

#[async_trait]
impl super::WMAdapter for I3Adapter {
    type Node = Node;
    type Rect = Rect;
    type Event = Event;
    type Id = usize;
    type Connection = I3Conn;

    fn is_tabbed_layout(node: &Self::Node) -> bool {
        matches!(node.layout, NodeLayout::Tabbed | NodeLayout::Stacked)
    }
    fn get_id(node: &Self::Node) -> Self::Id {
        node.id
    }
    fn get_rect(node: &Self::Node) -> Self::Rect {
        node.window_rect.clone()
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
        if let Ok(mut i3) = I3::connect().await {
            if i3.get_tree().await.is_ok() {
                return Ok(true);
            }
        }
        Ok(false)
    }
    async fn new_connection() -> Result<Self::Connection, Error> {
        Ok(I3Conn(I3::connect().await?))
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
        let mut owned = std::mem::replace(conn, I3Conn(I3::connect().await?));
        owned.0.subscribe([Subscribe::Window]).await?;
        Ok(Box::new(owned.0.listen().map(|e| e.map_err(Error::from))))
    }
    fn extract_window_event(ev: &Self::Event) -> Option<&Self::Node> {
        if let Event::Window(win_ev) = ev {
            Some(&win_ev.container)
        } else {
            None
        }
    }
    fn window_change_is_focus(ev: &Self::Event) -> bool {
        if let Event::Window(win_ev) = ev {
            win_ev.change == WindowChange::Focus
        } else {
            false
        }
    }
    async fn send_command(conn: &mut Self::Connection, cmd: &str) -> Result<(), Error> {
        conn.0.send_msg_body(Msg::RunCommand, cmd).await?;
        Ok(())
    }
}
