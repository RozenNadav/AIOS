use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Semantic memory object identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(pub Uuid);

impl NodeId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for NodeId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A node in the Knowledge Fabric (not a file).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: NodeId,
    pub title: String,
    pub kind: String,
    pub body: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Node {
    pub fn weave(title: impl Into<String>, kind: impl Into<String>, body: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: NodeId::new(),
            title: title.into(),
            kind: kind.into(),
            body: body.into(),
            created_at: now,
            updated_at: now,
        }
    }
}

/// Directed labeled edge between fabric nodes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relation {
    pub from: NodeId,
    pub to: NodeId,
    pub label: String,
    pub created_at: DateTime<Utc>,
}

/// Persistent semantic memory graph.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Fabric {
    pub nodes: HashMap<NodeId, Node>,
    pub relations: Vec<Relation>,
}

impl Fabric {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn weave(&mut self, title: &str, kind: &str, body: &str) -> Node {
        let node = Node::weave(title, kind, body);
        self.nodes.insert(node.id, node.clone());
        node
    }

    pub fn relate(&mut self, from: NodeId, to: NodeId, label: impl Into<String>) -> Result<Relation, String> {
        if !self.nodes.contains_key(&from) {
            return Err(format!("unknown node {from}"));
        }
        if !self.nodes.contains_key(&to) {
            return Err(format!("unknown node {to}"));
        }
        let rel = Relation {
            from,
            to,
            label: label.into(),
            created_at: Utc::now(),
        };
        self.relations.push(rel.clone());
        Ok(rel)
    }

    /// Lightweight semantic search over titles, kinds, and bodies.
    pub fn query(&self, q: &str) -> Vec<&Node> {
        let needle = q.to_ascii_lowercase();
        let mut hits: Vec<&Node> = self
            .nodes
            .values()
            .filter(|n| {
                n.title.to_ascii_lowercase().contains(&needle)
                    || n.kind.to_ascii_lowercase().contains(&needle)
                    || n.body.to_ascii_lowercase().contains(&needle)
            })
            .collect();
        hits.sort_by(|a, b| a.title.cmp(&b.title));
        hits
    }
}
