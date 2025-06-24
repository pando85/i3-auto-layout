use anyhow::Error;
use async_trait::async_trait;
use futures::StreamExt;
use tokio::sync::mpsc;

pub mod i3;
pub mod sway;

/// Abstraction for WM backend operations
#[async_trait]
pub trait WMAdapter: Send + Sync + 'static {
    type Node;
    type Rect;
    type Event;
    type Id;
    type Connection: Send + Sync;

    fn is_tabbed_layout(node: &Self::Node) -> bool;
    fn get_id(node: &Self::Node) -> Self::Id;
    fn get_rect(node: &Self::Node) -> Self::Rect;
    fn get_name(node: &Self::Node) -> Option<String>;
    fn split_rect(rect: &Self::Rect) -> &'static str;
    fn has_tabbed_parent(node: &Self::Node, window_id: &Self::Id, tabbed: bool) -> bool;

    async fn get_tree(conn: &mut Self::Connection) -> Result<Self::Node, Error>;
    async fn subscribe_window_events(
        conn: &mut Self::Connection,
    ) -> Result<Box<dyn futures::Stream<Item = Result<Self::Event, Error>> + Send + Unpin>, Error>;
    fn extract_window_event(ev: &Self::Event) -> Option<&Self::Node>;
    fn window_change_is_focus(ev: &Self::Event) -> bool;
    async fn send_command(conn: &mut Self::Connection, cmd: &str) -> Result<(), Error>;
    async fn try_connection() -> anyhow::Result<bool>;
    async fn new_connection() -> Result<Self::Connection, Error>;
}

pub async fn generic_event_loop<T: WMAdapter>(
    mut conn: T::Connection,
    send: mpsc::Sender<&'static str>,
) -> Result<(), Error> {
    let mut events = T::subscribe_window_events(&mut conn).await?;
    while let Some(event) = events.next().await {
        if let Ok(ev) = event {
            if T::window_change_is_focus(&ev) {
                if let Some(container) = T::extract_window_event(&ev) {
                    let is_tabbed = T::is_tabbed_layout(container);
                    let tabbed_parent = T::has_tabbed_parent(
                        &T::get_tree(&mut conn).await?,
                        &T::get_id(container),
                        is_tabbed,
                    );
                    log::debug!(
                        "name={:?}, tabbed_parent={}",
                        T::get_name(container),
                        tabbed_parent
                    );
                    if !tabbed_parent {
                        send.send(T::split_rect(&T::get_rect(container))).await?;
                    }
                }
            }
        }
    }
    Ok(())
}

pub async fn generic_command_loop<T: WMAdapter>(
    mut conn: T::Connection,
    mut recv: mpsc::Receiver<&'static str>,
) -> Result<(), Error> {
    while let Some(cmd) = recv.recv().await {
        T::send_command(&mut conn, cmd).await?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    struct DummyNode {
        id: u32,
        rect: (i32, i32, i32, i32),
        name: Option<String>,
        layout: &'static str,
        parent: Option<Box<DummyNode>>,
    }

    impl DummyNode {
        fn new(id: u32, layout: &'static str, parent: Option<Box<DummyNode>>) -> Self {
            Self {
                id,
                rect: (0, 0, 100, 100),
                name: Some(format!("node{id}")),
                layout,
                parent,
            }
        }
    }

    struct DummyAdapter;

    #[async_trait::async_trait]
    impl WMAdapter for DummyAdapter {
        type Node = DummyNode;
        type Rect = (i32, i32, i32, i32);
        type Event = DummyNode;
        type Id = u32;
        type Connection = ();

        fn is_tabbed_layout(node: &Self::Node) -> bool {
            node.layout == "tabbed"
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
        fn split_rect(_rect: &Self::Rect) -> &'static str {
            "split"
        }
        fn has_tabbed_parent(node: &Self::Node, _window_id: &Self::Id, tabbed: bool) -> bool {
            // Traverse up the parent chain
            let mut current = node.parent.as_ref();
            while let Some(parent) = current {
                if parent.layout == "tabbed" {
                    return true;
                }
                current = parent.parent.as_ref();
            }
            tabbed
        }
        async fn get_tree(_conn: &mut Self::Connection) -> Result<Self::Node, anyhow::Error> {
            Ok(DummyNode::new(1, "split", None))
        }
        async fn subscribe_window_events(
            _conn: &mut Self::Connection,
        ) -> Result<
            Box<dyn futures::Stream<Item = Result<Self::Event, anyhow::Error>> + Send + Unpin>,
            anyhow::Error,
        > {
            unimplemented!()
        }
        fn extract_window_event(ev: &Self::Event) -> Option<&Self::Node> {
            Some(ev)
        }
        fn window_change_is_focus(_ev: &Self::Event) -> bool {
            true
        }
        async fn send_command(
            _conn: &mut Self::Connection,
            _cmd: &str,
        ) -> Result<(), anyhow::Error> {
            Ok(())
        }
        async fn try_connection() -> anyhow::Result<bool> {
            Ok(true)
        }
        async fn new_connection() -> Result<Self::Connection, anyhow::Error> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_split_rect() {
        let rect = (0, 0, 100, 100);
        assert_eq!(DummyAdapter::split_rect(&rect), "split");
    }

    #[test]
    fn test_has_tabbed_parent() {
        let leaf = DummyNode::new(3, "split", None);
        let tabbed = DummyNode::new(2, "tabbed", Some(Box::new(leaf)));
        let root = DummyNode::new(1, "split", Some(Box::new(tabbed)));
        // Should find tabbed parent
        assert!(DummyAdapter::has_tabbed_parent(&root, &1, false));
        // No tabbed parent, but tabbed is true
        let node = DummyNode::new(4, "split", None);
        assert!(DummyAdapter::has_tabbed_parent(&node, &4, true));
        // No tabbed parent, tabbed is false
        assert!(!DummyAdapter::has_tabbed_parent(&node, &4, false));
    }

    #[test]
    fn test_is_tabbed_layout() {
        let tabbed = DummyNode::new(1, "tabbed", None);
        let split = DummyNode::new(2, "split", None);
        assert!(DummyAdapter::is_tabbed_layout(&tabbed));
        assert!(!DummyAdapter::is_tabbed_layout(&split));
    }

    #[test]
    fn test_get_id_and_get_name() {
        let node = DummyNode::new(42, "split", None);
        assert_eq!(DummyAdapter::get_id(&node), 42);
        assert_eq!(DummyAdapter::get_name(&node), Some("node42".to_string()));
    }

    #[test]
    fn test_get_rect() {
        let node = DummyNode::new(1, "split", None);
        assert_eq!(DummyAdapter::get_rect(&node), (0, 0, 100, 100));
    }

    #[tokio::test]
    async fn test_get_tree_and_send_command() {
        let tree = DummyAdapter::get_tree(&mut ()).await.unwrap();
        assert_eq!(tree.id, 1);
        DummyAdapter::send_command(&mut (), "split").await.unwrap();
    }

    #[test]
    fn test_extract_window_event_and_window_change_is_focus() {
        let node = DummyNode::new(1, "split", None);
        let ev = DummyAdapter::extract_window_event(&node);
        assert!(ev.is_some());
        assert!(DummyAdapter::window_change_is_focus(&node));
    }
}
