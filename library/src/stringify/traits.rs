//! Serialization Abstractions (OCP & DIP)
//!
//! Provides core traits for serializing YAML Node trees into different text formats.

use crate::error::Result;
use crate::io::traits::IDestination;
use crate::nodes::node::Node;

/// Trait defining the contract for serializing a Node AST into a target format.
pub trait NodeSerializer {
    /// Serialize a Node tree to the destination.
    fn serialize(&self, node: &Node, dest: &mut dyn IDestination) -> Result<()>;

    /// Serialize a Node tree to the destination with pretty formatting.
    fn serialize_pretty(&self, node: &Node, dest: &mut dyn IDestination) -> Result<()> {
        self.serialize(node, dest)
    }
}
