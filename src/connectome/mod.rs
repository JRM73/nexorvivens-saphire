// =============================================================================
// connectome/mod.rs — Dynamic neural connections graph
//
// Role: Models connections between concepts, emotions, brain modules,
//       senses and memories as a dynamic graph. Connections strengthen
//       according to Hebb's rule ("neurons that fire together wire
//       together") and weaken through disuse (pruning).
//       The system continuously reorganizes (autopoiesis).
//
// Place in the architecture:
//   Called in pipeline.rs after each thought cycle.
//   Active nodes (dominant emotion, stimulated modules, active senses)
//   reinforce their mutual connections on each pass.
//   Periodic pruning to eliminate dead connections.
// =============================================================================

use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use std::collections::{HashMap, HashSet, BinaryHeap, VecDeque};
use std::cmp::Reverse;

// =============================================================================
// Node and edge types
// =============================================================================

/// Type of node in the connectome.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NodeType {
    /// Abstract concept (curiosity, danger, music...)
    Concept,
    /// Emotional state (joy, sadness, anxiety...)
    Emotion,
    /// Brain module (reptilian, limbic, neocortex)
    Module,
    /// LTM memory
    Memory,
    /// Sense (reading, listening, touch...)
    Sense,
    /// Primary need (hunger, thirst)
    Need,
}

/// Type of connection between two nodes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EdgeType {
    /// Activates the target node when the source is active
    Excitatory,
    /// Inhibits the target node
    Inhibitory,
    /// Modulates the sensitivity of the target node
    Modulatory,
}

// =============================================================================
// Connectome structures
// =============================================================================

/// A conceptual node in the connectome.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConceptNode {
    pub id: u64,
    pub label: String,
    pub node_type: NodeType,
    /// Current activation (0.0 to 1.0) — reset to zero each cycle
    pub activation: f64,
    /// Total number of activations since creation
    pub total_activations: u64,
    /// Node creation date
    pub created_at: DateTime<Utc>,
    /// Optional semantic embedding (768-dim if available).
    /// Used as heuristic for semantic A*:
    /// h(n) = 1.0 - cosine_similarity(n.embedding, goal.embedding)
    #[serde(default)]
    pub embedding: Option<Vec<f64>>,
}

/// A connection between two nodes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeuralEdge {
    pub from: u64,
    pub to: u64,
    /// Connection strength (0.0 to 1.0)
    pub strength: f64,
    pub edge_type: EdgeType,
    /// Last activation of this connection
    pub last_activated: DateTime<Utc>,
    /// Number of times this connection has been activated
    pub activation_count: u64,
}

/// Connectome metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectomeMetrics {
    pub total_nodes: usize,
    pub total_edges: usize,
    pub average_strength: f64,
    pub strongest_edge: Option<(String, String, f64)>,
    pub most_connected_node: Option<(String, usize)>,
    pub total_synaptic_strength: f64,
    pub plasticity: f64,
}

/// The complete connectome: graph of nodes and connections.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connectome {
    pub nodes: Vec<ConceptNode>,
    pub edges: Vec<NeuralEdge>,
    /// Global adaptation capacity (decreases slowly = maturation)
    pub plasticity: f64,
    /// Threshold below which connections are pruned
    pub pruning_threshold: f64,
    /// Maximum number of edges
    pub max_edges: usize,
    /// Hebbian learning rate
    pub learning_rate: f64,
    /// Internal cycle counter for pruning
    pub cycle_counter: u64,
    /// Pruning interval (in cycles)
    pub pruning_interval: u64,
    /// Node counter (for generating IDs)
    next_id: u64,
    /// Reverse index: label -> id (O(1) lookup by label)
    #[serde(skip)]
    label_index: HashMap<String, u64>,
    /// Reverse index: id -> index in Vec<ConceptNode> (O(1) lookup by ID)
    #[serde(skip)]
    id_index: HashMap<u64, usize>,
    /// Adjacency list: node_id -> [(neighbor_id, edge_index, bidirectional)]
    /// Allows O(degree) to find neighbors instead of O(E)
    #[serde(skip)]
    adjacency: HashMap<u64, Vec<(u64, usize)>>,
}

// =============================================================================
// Implementation
// =============================================================================

impl Connectome {
    /// Creates a new connectome with base nodes.
    pub fn new(learning_rate: f64, pruning_threshold: f64, max_edges: usize, pruning_interval: u64) -> Self {
        let mut connectome = Self {
            nodes: Vec::new(),
            edges: Vec::new(),
            plasticity: 1.0,
            pruning_threshold,
            max_edges,
            learning_rate,
            cycle_counter: 0,
            pruning_interval,
            next_id: 0,
            label_index: HashMap::new(),
            id_index: HashMap::new(),
            adjacency: HashMap::new(),
        };

        // Base nodes: brain modules
        connectome.add_node("reptilien", NodeType::Module);
        connectome.add_node("limbique", NodeType::Module);
        connectome.add_node("neocortex", NodeType::Module);

        // Base nodes: primary emotions
        for e in &["joie", "tristesse", "colere", "peur", "degout",
                    "surprise", "curiosite", "serenite", "anxiete", "ennui"] {
            connectome.add_node(e, NodeType::Emotion);
        }

        // Base nodes: 5 senses
        for s in &["lecture", "ecoute", "contact", "saveur", "ambiance"] {
            connectome.add_node(s, NodeType::Sense);
        }

        // Base nodes: needs
        connectome.add_node("faim", NodeType::Need);
        connectome.add_node("soif", NodeType::Need);

        // Initial connections: interconnected modules
        connectome.add_edge_by_label("reptilien", "limbique", 0.3, EdgeType::Excitatory);
        connectome.add_edge_by_label("limbique", "neocortex", 0.3, EdgeType::Excitatory);
        connectome.add_edge_by_label("neocortex", "limbique", 0.2, EdgeType::Modulatory);
        connectome.add_edge_by_label("reptilien", "neocortex", 0.1, EdgeType::Inhibitory);

        // Emotion-module connections
        connectome.add_edge_by_label("peur", "reptilien", 0.4, EdgeType::Excitatory);
        connectome.add_edge_by_label("colere", "reptilien", 0.3, EdgeType::Excitatory);
        connectome.add_edge_by_label("joie", "limbique", 0.3, EdgeType::Excitatory);
        connectome.add_edge_by_label("curiosite", "neocortex", 0.4, EdgeType::Excitatory);
        connectome.add_edge_by_label("serenite", "limbique", 0.2, EdgeType::Modulatory);

        connectome
    }

    /// Adds a node to the connectome. Returns its ID.
    pub fn add_node(&mut self, label: &str, node_type: NodeType) -> u64 {
        self.add_node_inner(label, node_type, None)
    }

    /// Adds a node with a semantic embedding. Returns its ID.
    pub fn add_node_with_embedding(&mut self, label: &str, node_type: NodeType, embedding: Vec<f64>) -> u64 {
        self.add_node_inner(label, node_type, Some(embedding))
    }

    fn add_node_inner(&mut self, label: &str, node_type: NodeType, embedding: Option<Vec<f64>>) -> u64 {
        if let Some(&id) = self.label_index.get(label) {
            // If the node exists but has no embedding, update it
            if embedding.is_some() {
                if let Some(node) = self.node_by_id_mut(id) {
                    if node.embedding.is_none() {
                        node.embedding = embedding;
                    }
                }
            }
            return id;
        }
        let id = self.next_id;
        self.next_id += 1;
        let idx = self.nodes.len();
        self.nodes.push(ConceptNode {
            id,
            label: label.to_string(),
            node_type,
            activation: 0.0,
            total_activations: 0,
            created_at: Utc::now(),
            embedding,
        });
        self.label_index.insert(label.to_string(), id);
        self.id_index.insert(id, idx);
        id
    }

    /// Adds an edge between two nodes by ID.
    pub fn add_edge(&mut self, from: u64, to: u64, strength: f64, edge_type: EdgeType) {
        // Check that the edge doesn't already exist via adjacency list (O(degree))
        if let Some(neighbors) = self.adjacency.get(&from) {
            if neighbors.iter().any(|&(n, _)| n == to) {
                return;
            }
        }
        if self.edges.len() >= self.max_edges {
            return;
        }
        let edge_idx = self.edges.len();
        self.edges.push(NeuralEdge {
            from,
            to,
            strength: strength.clamp(0.0, 1.0),
            edge_type,
            last_activated: Utc::now(),
            activation_count: 0,
        });
        // Update the adjacency list (bidirectional)
        self.adjacency.entry(from).or_default().push((to, edge_idx));
        self.adjacency.entry(to).or_default().push((from, edge_idx));
    }

    /// Adds an edge between two nodes by label.
    fn add_edge_by_label(&mut self, from: &str, to: &str, strength: f64, edge_type: EdgeType) {
        let from_id = self.label_index.get(from).copied();
        let to_id = self.label_index.get(to).copied();
        if let (Some(f), Some(t)) = (from_id, to_id) {
            self.add_edge(f, t, strength, edge_type);
        }
    }

    /// Returns a reference to the node by ID (O(1) via id_index).
    fn node_by_id(&self, id: u64) -> Option<&ConceptNode> {
        self.id_index.get(&id).and_then(|&idx| self.nodes.get(idx))
    }

    /// Returns a mutable reference to the node by ID (O(1) via id_index).
    fn node_by_id_mut(&mut self, id: u64) -> Option<&mut ConceptNode> {
        self.id_index.get(&id).copied().and_then(|idx| self.nodes.get_mut(idx))
    }

    /// Returns the label of a node by ID (O(1)).
    fn label_of(&self, id: u64) -> String {
        self.node_by_id(id).map(|n| n.label.clone()).unwrap_or_default()
    }

    /// Activates a node by label. Returns the ID if found.
    pub fn activate(&mut self, label: &str) -> Option<u64> {
        if let Some(&id) = self.label_index.get(label) {
            if let Some(node) = self.node_by_id_mut(id) {
                node.activation = 1.0;
                node.total_activations += 1;
            }
            Some(id)
        } else {
            None
        }
    }

    /// Resets all activations to zero (start of cycle).
    pub fn reset_activations(&mut self) {
        for node in &mut self.nodes {
            node.activation = 0.0;
        }
    }

    /// Applies Hebb's rule: strengthens connections between co-active nodes.
    /// Slightly weakens connections when only one endpoint is active.
    pub fn hebbian_update(&mut self) {
        let active_ids: Vec<u64> = self.nodes.iter()
            .filter(|n| n.activation > 0.5)
            .map(|n| n.id)
            .collect();

        let lr = self.learning_rate * self.plasticity;
        let now = Utc::now();

        for edge in &mut self.edges {
            let from_active = active_ids.contains(&edge.from);
            let to_active = active_ids.contains(&edge.to);

            if from_active && to_active {
                // LTP (Long-Term Potentiation): strengthening
                edge.strength = (edge.strength + lr).min(1.0);
                edge.activation_count += 1;
                edge.last_activated = now;
            } else if from_active || to_active {
                // LTD (Long-Term Depression): slight weakening
                edge.strength = (edge.strength - lr * 0.1).max(0.0);
            }
        }

        // Synaptogenesis: create connections between co-active nodes
        // without existing connection (uses adjacency O(degree) instead of O(E))
        if active_ids.len() >= 2 {
            for i in 0..active_ids.len() {
                for j in (i+1)..active_ids.len() {
                    let a = active_ids[i];
                    let b = active_ids[j];
                    // add_edge already checks existence via adjacency
                    self.add_edge(a, b, lr * 2.0, EdgeType::Excitatory);
                }
            }
        }
    }

    /// Prunes connections that are too weak (synaptic pruning).
    pub fn prune(&mut self) {
        let threshold = self.pruning_threshold;
        let before = self.edges.len();
        self.edges.retain(|e| e.strength > threshold);
        if self.edges.len() != before {
            self.rebuild_adjacency();
        }
    }

    /// Synaptic homeostasis: if too many strong connections,
    /// reduce globally to avoid saturation.
    pub fn synaptic_homeostasis(&mut self) {
        let total: f64 = self.edges.iter().map(|e| e.strength).sum();
        let count = self.edges.len() as f64;
        if count > 0.0 {
            let avg = total / count;
            if avg > 0.7 {
                // Proportional reduction
                let factor = 0.7 / avg;
                for edge in &mut self.edges {
                    edge.strength *= factor;
                }
            }
        }
    }

    /// Called at each thought cycle.
    /// Activates relevant nodes, applies Hebb, periodic pruning.
    pub fn tick(&mut self, active_labels: &[&str]) {
        self.reset_activations();

        // Activate corresponding nodes
        for label in active_labels {
            // Try the direct label
            if self.activate(label).is_none() {
                // Try creating a Concept node if it doesn't exist
                let id = self.add_node(label, NodeType::Concept);
                if let Some(node) = self.node_by_id_mut(id) {
                    node.activation = 1.0;
                    node.total_activations += 1;
                }
            }
        }

        // Hebbian reinforcement
        self.hebbian_update();

        // Periodic pruning
        self.cycle_counter += 1;
        if self.pruning_interval > 0 && self.cycle_counter % self.pruning_interval == 0 {
            self.prune();
            self.synaptic_homeostasis();
        }

        // Slow plasticity decay (maturation)
        self.plasticity = (self.plasticity - 0.0001).max(0.3);
    }

    /// Computes connectome metrics.
    pub fn metrics(&self) -> ConnectomeMetrics {
        let total_edges = self.edges.len();
        let total_strength: f64 = self.edges.iter().map(|e| e.strength).sum();
        let avg_strength = if total_edges > 0 {
            total_strength / total_edges as f64
        } else {
            0.0
        };

        // Strongest edge (O(E) scan + O(1) node lookup)
        let strongest = self.edges.iter()
            .max_by(|a, b| a.strength.partial_cmp(&b.strength).unwrap_or(std::cmp::Ordering::Equal))
            .map(|e| {
                let from_label = self.label_of(e.from);
                let to_label = self.label_of(e.to);
                (from_label, to_label, e.strength)
            });

        // Most connected node (via adjacency O(V))
        let most_connected = self.adjacency.iter()
            .max_by_key(|(_, neighbors)| neighbors.len())
            .and_then(|(&id, neighbors)| {
                self.node_by_id(id).map(|n| (n.label.clone(), neighbors.len()))
            });

        ConnectomeMetrics {
            total_nodes: self.nodes.len(),
            total_edges,
            average_strength: avg_strength,
            strongest_edge: strongest,
            most_connected_node: most_connected,
            total_synaptic_strength: total_strength,
            plasticity: self.plasticity,
        }
    }

    /// Serializes the connectome to JSON for the API.
    pub fn to_json(&self) -> serde_json::Value {
        let metrics = self.metrics();
        serde_json::json!({
            "nodes": self.nodes.len(),
            "edges": self.edges.len(),
            "plasticity": self.plasticity,
            "metrics": metrics,
            "top_nodes": self.nodes.iter()
                .filter(|n| n.total_activations > 0)
                .take(20)
                .map(|n| serde_json::json!({
                    "label": n.label,
                    "type": format!("{:?}", n.node_type),
                    "activations": n.total_activations,
                }))
                .collect::<Vec<_>>(),
            "top_edges": self.edges.iter()
                .filter(|e| e.strength > 0.1)
                .take(30)
                .map(|e| {
                    let from = self.node_by_id(e.from)
                        .map(|n| n.label.as_str()).unwrap_or("?");
                    let to = self.node_by_id(e.to)
                        .map(|n| n.label.as_str()).unwrap_or("?");
                    serde_json::json!({
                        "from": from,
                        "to": to,
                        "strength": format!("{:.3}", e.strength),
                        "type": format!("{:?}", e.edge_type),
                        "activations": e.activation_count,
                    })
                })
                .collect::<Vec<_>>(),
        })
    }

    // =========================================================================
    // A* Pathfinding + Spreading Activation + Associative chains
    // =========================================================================

    /// A* pathfinding between two nodes by label.
    /// Uses cosine similarity between embeddings as heuristic
    /// when nodes have embeddings (true A*). Otherwise Dijkstra (h=0).
    /// Edge cost = (1.0 - strength) * type_factor
    /// Returns the path as a list of labels, or None if unreachable.
    pub fn find_path(&self, from_label: &str, to_label: &str) -> Option<Vec<String>> {
        let from_id = *self.label_index.get(from_label)?;
        let to_id = *self.label_index.get(to_label)?;
        self.astar_path(from_id, to_id, None)
    }

    /// A* with explicit semantic heuristic (goal embedding provided).
    /// Useful for memory search: finds the path to the node
    /// semantically closest to the current stimulus.
    pub fn find_path_semantic(
        &self,
        from_label: &str,
        goal_embedding: &[f64],
        max_hops: usize,
    ) -> Vec<(String, f64)> {
        let from_id = match self.label_index.get(from_label) {
            Some(&id) => id,
            None => return Vec::new(),
        };

        // Exploratory A*: we don't seek a specific node but the path
        // to nodes semantically close to the goal
        let mut heap: BinaryHeap<Reverse<(u64, u64)>> = BinaryHeap::new();
        let mut g_score: HashMap<u64, f64> = HashMap::new();
        let mut visited: HashSet<u64> = HashSet::new();
        let mut results: Vec<(String, f64)> = Vec::new();

        g_score.insert(from_id, 0.0);
        heap.push(Reverse((0u64, from_id)));

        while let Some(Reverse((_, current))) = heap.pop() {
            if !visited.insert(current) {
                continue;
            }

            // Compute similarity with the goal
            if let Some(node) = self.node_by_id(current) {
                if let Some(ref emb) = node.embedding {
                    let sim = crate::vectorstore::cosine_similarity(emb, goal_embedding);
                    if sim > 0.3 {
                        results.push((node.label.clone(), sim));
                    }
                }
            }

            let current_g = g_score.get(&current).copied().unwrap_or(f64::MAX);

            // Limit depth
            if current_g > max_hops as f64 {
                continue;
            }

            if let Some(adj) = self.adjacency.get(&current) {
                for &(neighbor, edge_idx) in adj {
                    if visited.contains(&neighbor) {
                        continue;
                    }
                    let edge = &self.edges[edge_idx];
                    let edge_cost = self.edge_cost(edge);
                    let tentative_g = current_g + edge_cost;
                    let prev_g = g_score.get(&neighbor).copied().unwrap_or(f64::MAX);
                    if tentative_g < prev_g {
                        g_score.insert(neighbor, tentative_g);
                        // Heuristic: 1.0 - cosine_similarity with the goal
                        let h = self.node_by_id(neighbor)
                            .and_then(|n| n.embedding.as_ref())
                            .map(|emb| 1.0 - crate::vectorstore::cosine_similarity(emb, goal_embedding).max(0.0))
                            .unwrap_or(0.5); // Moderate default if no embedding
                        let f = ((tentative_g + h) * 10000.0) as u64;
                        heap.push(Reverse((f, neighbor)));
                    }
                }
            }
        }

        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        results
    }

    /// Computes the cost of an edge for A*.
    /// Integrates synaptic strength and connection type.
    fn edge_cost(&self, edge: &NeuralEdge) -> f64 {
        let base_cost = (1.0 - edge.strength).max(0.01);
        let type_factor = match edge.edge_type {
            EdgeType::Excitatory => 1.0,
            EdgeType::Inhibitory => 3.0, // Inhibitory connections are "expensive" to traverse
            EdgeType::Modulatory => 1.5,
        };
        base_cost * type_factor
    }

    /// Internal A* implementation between two nodes by ID.
    fn astar_path(&self, from_id: u64, to_id: u64, _goal_embedding: Option<&[f64]>) -> Option<Vec<String>> {
        if from_id == to_id {
            return Some(vec![self.label_of(from_id)]);
        }

        // Retrieve the goal's embedding for the heuristic
        let goal_emb = self.node_by_id(to_id).and_then(|n| n.embedding.as_ref());

        let mut heap: BinaryHeap<Reverse<(u64, u64)>> = BinaryHeap::new();
        let mut g_score: HashMap<u64, f64> = HashMap::new();
        let mut parents: HashMap<u64, u64> = HashMap::new();
        let mut visited: HashSet<u64> = HashSet::new();

        g_score.insert(from_id, 0.0);
        heap.push(Reverse((0u64, from_id)));

        while let Some(Reverse((_, current))) = heap.pop() {
            if current == to_id {
                let mut path = Vec::new();
                let mut node = to_id;
                loop {
                    path.push(self.label_of(node));
                    if node == from_id { break; }
                    node = match parents.get(&node) {
                        Some(&p) => p,
                        None => break,
                    };
                }
                path.reverse();
                return Some(path);
            }

            if !visited.insert(current) {
                continue;
            }

            let current_g = g_score.get(&current).copied().unwrap_or(f64::MAX);

            if let Some(adj) = self.adjacency.get(&current) {
                for &(neighbor, edge_idx) in adj {
                    if visited.contains(&neighbor) {
                        continue;
                    }
                    let edge = &self.edges[edge_idx];
                    let edge_cost = self.edge_cost(edge);
                    let tentative_g = current_g + edge_cost;
                    let prev_g = g_score.get(&neighbor).copied().unwrap_or(f64::MAX);
                    if tentative_g < prev_g {
                        g_score.insert(neighbor, tentative_g);
                        parents.insert(neighbor, current);
                        // Semantic heuristic if embeddings exist
                        let h = match goal_emb {
                            Some(ge) => self.node_by_id(neighbor)
                                .and_then(|n| n.embedding.as_ref())
                                .map(|emb| 1.0 - crate::vectorstore::cosine_similarity(emb, ge).max(0.0))
                                .unwrap_or(0.0),
                            None => 0.0, // Dijkstra if no embeddings
                        };
                        let f = ((tentative_g + h) * 10000.0) as u64;
                        heap.push(Reverse((f, neighbor)));
                    }
                }
            }
        }

        None
    }

    /// Finds connectome nodes semantically closest to a given embedding.
    /// Useful when the searched word is not a connectome node.
    /// Returns top_k nodes sorted by descending similarity (> threshold 0.3).
    pub fn find_similar_by_embedding(&self, target_embedding: &[f64], top_k: usize) -> Vec<(String, f64)> {
        let mut results: Vec<(String, f64)> = Vec::new();
        for node in &self.nodes {
            if let Some(ref emb) = node.embedding {
                let sim = crate::vectorstore::cosine_similarity(emb, target_embedding);
                if sim > 0.3 {
                    results.push((node.label.clone(), sim));
                }
            }
        }
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(top_k);
        results
    }

    /// BFS activation propagation from a source node.
    /// Activation propagates over `depth` levels, decreasing
    /// proportionally to edge strength.
    /// Returns reached nodes sorted by descending activation.
    pub fn spreading_activation(&self, source_label: &str, depth: usize) -> Vec<(String, f64)> {
        let source_id = match self.label_index.get(source_label) {
            Some(&id) => id,
            None => return Vec::new(),
        };

        let mut activations: HashMap<u64, f64> = HashMap::new();
        activations.insert(source_id, 1.0);

        // BFS by level (uses the persistent adjacency list)
        let mut current_layer: VecDeque<(u64, f64)> = VecDeque::new();
        current_layer.push_back((source_id, 1.0));

        let mut visited: HashSet<u64> = HashSet::new();
        visited.insert(source_id);

        for _ in 0..depth {
            let mut next_layer: VecDeque<(u64, f64)> = VecDeque::new();
            while let Some((node, node_activation)) = current_layer.pop_front() {
                if let Some(adj) = self.adjacency.get(&node) {
                    for &(neighbor, edge_idx) in adj {
                        let edge = &self.edges[edge_idx];
                        let propagated = match edge.edge_type {
                            // Excitatory: propagates activation proportionally
                            EdgeType::Excitatory => node_activation * edge.strength,
                            // Inhibitory: REDUCES the neighbor's existing activation
                            EdgeType::Inhibitory => -(node_activation * edge.strength),
                            // Modulatory: MULTIPLIES existing activation (amplifies or attenuates)
                            EdgeType::Modulatory => {
                                let existing = activations.get(&neighbor).copied().unwrap_or(0.0);
                                existing * edge.strength - existing // Delta only
                            }
                        };
                        if propagated.abs() < 0.01 {
                            continue;
                        }
                        let entry = activations.entry(neighbor).or_insert(0.0);
                        if propagated > 0.0 {
                            // Excitatory: keep the max
                            if propagated > *entry {
                                *entry = propagated;
                            }
                        } else {
                            // Inhibitory/Modulatory: subtract (clamp to 0)
                            *entry = (*entry + propagated).max(0.0);
                        }
                        if visited.insert(neighbor) {
                            next_layer.push_back((neighbor, propagated.max(0.0)));
                        }
                    }
                }
            }
            current_layer = next_layer;
        }

        // Remove source node and sort by descending activation
        activations.remove(&source_id);
        let mut result: Vec<(String, f64)> = activations.into_iter()
            .filter_map(|(id, act)| {
                self.node_by_id(id).map(|n| (n.label.clone(), act))
            })
            .collect();
        result.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        result
    }

    /// Associative chain between two concepts: finds the strongest path
    /// with a maximum number of hops, returns nodes with their scores.
    /// Combines A* pathfinding with activation strengths.
    pub fn associative_chain(
        &self,
        from_label: &str,
        to_label: &str,
        max_hops: usize,
    ) -> Option<Vec<(String, f64)>> {
        let path = self.find_path(from_label, to_label)?;
        if path.len() > max_hops + 1 {
            return None; // Too many hops
        }

        // Compute the score of each node in the chain
        let mut result: Vec<(String, f64)> = Vec::new();
        let mut cumulative_strength = 1.0;

        for (i, label) in path.iter().enumerate() {
            if i > 0 {
                // Find edge strength via adjacency (O(degree))
                let prev_label = &path[i - 1];
                let prev_id = self.label_index.get(prev_label.as_str()).copied();
                let curr_id = self.label_index.get(label.as_str()).copied();
                if let (Some(pid), Some(cid)) = (prev_id, curr_id) {
                    let edge_strength = self.adjacency.get(&pid)
                        .and_then(|adj| adj.iter()
                            .find(|&&(n, _)| n == cid)
                            .map(|&(_, ei)| self.edges[ei].strength))
                        .unwrap_or(0.1);
                    cumulative_strength *= edge_strength;
                }
            }
            result.push((label.clone(), cumulative_strength));
        }

        Some(result)
    }

    /// Rebuilds the adjacency list from edges.
    fn rebuild_adjacency(&mut self) {
        self.adjacency.clear();
        for (edge_idx, edge) in self.edges.iter().enumerate() {
            self.adjacency.entry(edge.from).or_default().push((edge.to, edge_idx));
            self.adjacency.entry(edge.to).or_default().push((edge.from, edge_idx));
        }
    }

    /// Rebuilds all indexes (after deserialization or pruning).
    pub fn rebuild_index(&mut self) {
        // Index label -> id
        self.label_index.clear();
        for node in &self.nodes {
            self.label_index.insert(node.label.clone(), node.id);
        }
        // Index id -> vec index
        self.id_index.clear();
        for (idx, node) in self.nodes.iter().enumerate() {
            self.id_index.insert(node.id, idx);
        }
        // Adjacency list
        self.rebuild_adjacency();
        // Ensure next_id is correct
        if let Some(max_id) = self.nodes.iter().map(|n| n.id).max() {
            self.next_id = max_id + 1;
        }
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_nodes() {
        let c = Connectome::new(0.01, 0.05, 1000, 100);
        // 3 modules + 10 emotions + 5 sens + 2 besoins = 20 noeuds
        assert_eq!(c.nodes.len(), 20);
        assert!(c.edges.len() >= 8); // Connexions initiales
    }

    #[test]
    fn test_hebbian_reinforcement() {
        let mut c = Connectome::new(0.05, 0.01, 1000, 100);
        let initial = c.edges.iter()
            .find(|e| {
                let from = c.nodes.iter().find(|n| n.id == e.from).unwrap();
                let to = c.nodes.iter().find(|n| n.id == e.to).unwrap();
                from.label == "curiosite" && to.label == "neocortex"
            })
            .unwrap().strength;

        // Activer curiosite et neocortex ensemble
        c.tick(&["curiosite", "neocortex"]);

        let after = c.edges.iter()
            .find(|e| {
                let from = c.nodes.iter().find(|n| n.id == e.from).unwrap();
                let to = c.nodes.iter().find(|n| n.id == e.to).unwrap();
                from.label == "curiosite" && to.label == "neocortex"
            })
            .unwrap().strength;

        assert!(after > initial, "La connexion doit se renforcer apres co-activation");
    }

    #[test]
    fn test_synaptogenesis() {
        let mut c = Connectome::new(0.05, 0.01, 1000, 100);
        let edges_before = c.edges.len();

        // Activer deux noeuds qui n'ont pas de connexion directe
        c.tick(&["faim", "colere"]);

        assert!(c.edges.len() > edges_before, "Une nouvelle connexion doit etre creee");
    }

    #[test]
    fn test_pruning() {
        let mut c = Connectome::new(0.01, 0.5, 1000, 1); // Seuil eleve pour forcer le pruning
        let edges_before = c.edges.len();
        // Tick without activating connected nodes → weakening
        for _ in 0..5 {
            c.tick(&[]);
        }
        assert!(c.edges.len() < edges_before, "Le pruning doit retirer les connexions faibles");
    }

    #[test]
    fn test_new_concept_creation() {
        let mut c = Connectome::new(0.01, 0.05, 1000, 100);
        let nodes_before = c.nodes.len();
        c.tick(&["philosophie", "art"]); // Concepts that don't exist yet
        assert_eq!(c.nodes.len(), nodes_before + 2);
    }

    #[test]
    fn test_metrics() {
        let c = Connectome::new(0.01, 0.05, 1000, 100);
        let m = c.metrics();
        assert_eq!(m.total_nodes, 20);
        assert!(m.total_edges > 0);
        assert!(m.average_strength > 0.0);
    }

    #[test]
    fn test_find_path_direct() {
        let c = Connectome::new(0.01, 0.05, 1000, 100);
        // peur → reptilien est une connexion directe
        let path = c.find_path("peur", "reptilien");
        assert!(path.is_some(), "Doit trouver un chemin peur → reptilien");
        let p = path.unwrap();
        assert_eq!(p.first().unwrap(), "peur");
        assert_eq!(p.last().unwrap(), "reptilien");
    }

    #[test]
    fn test_find_path_indirect() {
        let c = Connectome::new(0.01, 0.05, 1000, 100);
        // peur → reptilien → limbique → neocortex (chemin indirect)
        let path = c.find_path("peur", "neocortex");
        assert!(path.is_some(), "Doit trouver un chemin indirect peur → neocortex");
    }

    #[test]
    fn test_find_path_same_node() {
        let c = Connectome::new(0.01, 0.05, 1000, 100);
        let path = c.find_path("joie", "joie");
        assert_eq!(path, Some(vec!["joie".to_string()]));
    }

    #[test]
    fn test_find_path_nonexistent() {
        let c = Connectome::new(0.01, 0.05, 1000, 100);
        assert!(c.find_path("inexistant", "joie").is_none());
    }

    #[test]
    fn test_spreading_activation() {
        let c = Connectome::new(0.01, 0.05, 1000, 100);
        let activated = c.spreading_activation("curiosite", 2);
        assert!(!activated.is_empty(), "Doit propager l'activation");
        // Le premier resultat doit etre le voisin le plus fort (neocortex)
        assert!(activated[0].1 > 0.0);
    }

    #[test]
    fn test_spreading_activation_nonexistent() {
        let c = Connectome::new(0.01, 0.05, 1000, 100);
        let activated = c.spreading_activation("inexistant", 3);
        assert!(activated.is_empty());
    }

    #[test]
    fn test_associative_chain() {
        let c = Connectome::new(0.01, 0.05, 1000, 100);
        let chain = c.associative_chain("peur", "limbique", 5);
        assert!(chain.is_some(), "Doit trouver une chaine associative");
        let ch = chain.unwrap();
        assert!(ch.len() >= 2);
        // Les scores doivent etre decroissants
        for i in 1..ch.len() {
            assert!(ch[i].1 <= ch[i-1].1, "Score cumulatif doit etre decroissant");
        }
    }

    #[test]
    fn test_associative_chain_too_far() {
        let c = Connectome::new(0.01, 0.05, 1000, 100);
        // max_hops = 1, mais le chemin est plus long
        let chain = c.associative_chain("peur", "neocortex", 1);
        // Pourrait etre None si le chemin fait > 2 noeuds
        if let Some(ch) = chain {
            assert!(ch.len() <= 2);
        }
    }
}
