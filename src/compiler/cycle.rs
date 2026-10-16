use crate::error::{OrcrustratorError, Result};
use crate::oop::RelationshipDeclaration;
use std::collections::{HashMap, HashSet, VecDeque};

/// Validates that a set of relationship declarations forms a Directed Acyclic Graph (DAG) without circular cycles.
///
/// Uses Kahn's algorithm for topological sorting to guarantee cycle-freedom.
pub fn validate_dag(declarations: &[RelationshipDeclaration]) -> Result<()> {
    // 1. Build adjacency list and compute in-degrees
    let mut in_degree: HashMap<String, usize> = HashMap::new();
    let mut adj_list: HashMap<String, Vec<String>> = HashMap::new();
    let mut all_nodes: HashSet<String> = HashSet::new();

    for decl in declarations {
        all_nodes.insert(decl.source.clone());
        all_nodes.insert(decl.target.clone());

        adj_list
            .entry(decl.source.clone())
            .or_default()
            .push(decl.target.clone());

        in_degree.entry(decl.source.clone()).or_insert(0);
        *in_degree.entry(decl.target.clone()).or_insert(0) += 1;
    }

    // 2. Collect all nodes with in-degree 0 (roots)
    let mut queue: VecDeque<String> = in_degree
        .iter()
        .filter(|(_, deg)| **deg == 0)
        .map(|(node, _)| node.clone())
        .collect();

    let mut visited_count = 0;

    // 3. Process BFS topological order
    while let Some(node) = queue.pop_front() {
        visited_count += 1;
        if let Some(neighbors) = adj_list.get(&node) {
            for neighbor in neighbors {
                if let Some(deg) = in_degree.get_mut(neighbor) {
                    *deg -= 1;
                    if *deg == 0 {
                        queue.push_back(neighbor.clone());
                    }
                }
            }
        }
    }

    // 4. If visited count does not equal total nodes, a cycle exists
    if visited_count != all_nodes.len() {
        let cyclic_nodes: Vec<String> = in_degree
            .into_iter()
            .filter(|(_, deg)| *deg > 0)
            .map(|(node, _)| node)
            .collect();
        return Err(OrcrustratorError::CircularRelation(format!(
            "Cyclic dependency detected among endpoints: {}",
            cyclic_nodes.join(" -> ")
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_dag() {
        let decls = vec![
            RelationshipDeclaration::new("r1", "/users", "/posts", "HasMany", "user_id", "posts"),
            RelationshipDeclaration::new("r2", "/posts", "/comments", "HasMany", "post_id", "comments"),
        ];
        assert!(validate_dag(&decls).is_ok());
    }

    #[test]
    fn test_cycle_detection() {
        let decls = vec![
            RelationshipDeclaration::new("r1", "/a", "/b", "HasOne", "id", "b"),
            RelationshipDeclaration::new("r2", "/b", "/c", "HasOne", "id", "c"),
            RelationshipDeclaration::new("r3", "/c", "/a", "HasOne", "id", "a"),
        ];
        assert!(validate_dag(&decls).is_err());
    }
}
