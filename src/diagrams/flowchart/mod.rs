// TODO error handling - loads of places currently panic where they should error gracefully

mod parse;
use anyhow::{anyhow, Result};
use petgraph::graphmap::GraphMap;
use std::{collections::HashMap, fmt};

/// A flowchart
///
/// If any of the mutating methods return an error, the flowchart state is undefined and should be
/// discarded. This doesn't affect memory safety (no `unsafe` is used).
pub struct Flowchart<'input> {
    /// The direction this flowchart should be rendered in.
    pub direction: Direction,
    /// The graph of nodes and edges that make up the flowchart.
    ///
    /// Node IDs are used as keys. Associated node information is stored in `self.nodes`.
    pub graph: GraphMap<&'input str, Connector<'input>, petgraph::Directed>,
    /// Assocated information for the nodes (label, style etc.)
    pub nodes: HashMap<&'input str, Node<'input>>,
}

impl<'input> Flowchart<'input> {
    fn new(direction: Direction) -> Self {
        Flowchart {
            direction,
            graph: GraphMap::new(),
            nodes: HashMap::new(),
        }
    }

    /// Take textual input conforming to the mermaid spec and parse it into a [`Flowchart`].
    pub fn parse<'a>(input: &'a str) -> Result<Flowchart<'a>> {
        parse::parse_flowchart(input).map_err(|e| anyhow!("{}", e))
    }

    fn add_node(&mut self, node: &Node<'input>) -> &'input str {
        let id = node.id;
        if node.is_id() {
            // only insert the node if it's not there
            self.nodes.entry(id).or_insert(node.clone());
        } else {
            // insert the node and panic if one is already there
            if self.nodes.insert(id, node.clone()).is_some() {
                panic!("node with given name already exists");
            }
        }
        id
    }

    fn add_edge(&mut self, from: &'input str, to: &'input str, edge: Connector<'input>) {
        assert!(self.nodes.contains_key(from) && self.nodes.contains_key(to));
        if self.graph.add_edge(from, to, edge).is_some() {
            panic!("edge already exists")
        }
    }
}

/// The direction the flowchart should be drawn in.
#[derive(Debug, Copy, Clone)]
pub enum Direction {
    /// Flow from top to bottom.
    TopBottom,
    /// Flow from bottom to top.
    BottomTop,
    /// Flow from left to right.
    LeftRight,
    /// Flow from right to left.
    RightLeft,
}

/// A node of the flowchart
#[derive(Debug, Copy, Clone)]
pub struct Node<'input> {
    /// The node's id (mandatory)
    pub id: &'input str,
    /// The node's label.
    ///
    /// The empty string and no string are not disambiguated, for now. If this is empty, use the id
    /// (see [`Node::label_or_id`])
    pub label: &'input str,
    /// The shape that should be used for the node.
    pub style: NodeStyle,
}

impl<'input> Node<'input> {
    /// Whether the node is just and ID and nothing else
    fn is_id(&self) -> bool {
        self.label.is_empty()
    }

    /// Get the label for the node, falling back to the ID if there is no label set.
    pub fn label_or_id(&self) -> &'input str {
        if self.label.is_empty() {
            self.id
        } else {
            self.label
        }
    }
}

/// The shape that the node should be drawn inside.
#[derive(Debug, Copy, Clone)]
pub enum NodeStyle {
    /// A square node
    ///
    /// Default
    Square,
    /// Slightly rounded edges.
    Round,
    /// Fully rounded edges (like a chariot stadium)
    Stadium,
    /// Square with lines down each side.
    ///
    /// Used to represent a subroutine.
    Subroutine,
    /// The shape of a 3D cylinder.
    ///
    /// Used to represent a database.
    Cylinder,
    /// A circle
    Circle,
    /// A flag shape.
    Asymmetric,
    /// A rhombus (diamond).
    Rhombus,
    /// A hexagon.
    Hexagon,
    /// A parallelogram leaning forward.
    Parallelogram,
    /// A parallelogram leaning backward.
    ParallelogramRev,
    /// A trapezoid bigger at the bottom.
    Trapezoid,
    /// A trapezoid bigger at the top.
    TrapezoidRev,
    /// A circle with an extra line round the edge.
    DoubleCircle,
}

/// Information associated with a connection between nodes (an edge).
#[derive(Debug, Copy, Clone)]
pub struct Connector<'input> {
    /// The style of the line.
    pub line_style: LineStyle,
    /// What style (if any) should be used for the "from" arrow
    pub arrow_start: Option<ArrowStyle>,
    /// What style (if any) should be used for the "to" arrow
    pub arrow_end: Option<ArrowStyle>,
    /// An optional label
    pub label: &'input str,
    /// The rank of the connection.
    ///
    /// This is used to hint to the layout engine which connections should be longer.
    pub rank: u16,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum LineStyle {
    /// A normal solid line.
    ///
    /// Default
    Normal,
    /// A thicker solid line.
    Thick,
    /// A dotted line.
    Dotted,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum ArrowStyle {
    /// An arrowhead that looks like an arrow.
    Arrow,
    /// A circle shaped arrowhead.
    Circle,
    /// A cross shaped arrowhead.
    Cross,
}

impl fmt::Debug for Flowchart<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // is there a better way of doing this? I wish there was. Sigh.
        struct Nodes<'a>(&'a Flowchart<'a>);
        impl fmt::Debug for Nodes<'_> {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.debug_set().entries(self.0.nodes.values()).finish()
            }
        }

        struct Edges<'a>(&'a Flowchart<'a>);
        impl fmt::Debug for Edges<'_> {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                let mut dbg = f.debug_set();
                for (from, to, conn) in self.0.graph.all_edges() {
                    dbg.entry(&Edge(from, to, conn));
                }
                dbg.finish()
            }
        }

        struct Edge<'a>(&'a str, &'a str, &'a Connector<'a>);
        impl fmt::Debug for Edge<'_> {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.debug_struct("Edge")
                    .field("from", &self.0)
                    .field("to", &self.1)
                    .field("conn", &self.2)
                    .finish()
            }
        }

        f.debug_struct("Flowchart")
            .field("direction", &self.direction)
            .field("nodes", &Nodes(self))
            .field("edges", &Edges(self))
            .finish()
    }
}
