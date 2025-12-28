//! Tree Utilities
//!
//! Helper functions for tree rendering.

use crate::models::Item;
use std::collections::HashMap;

/// Render items as indented tree using recursive DFS
/// Returns (Item, depth) pairs in display order
pub fn flatten_tree(items: &[Item]) -> Vec<(Item, usize)> {
    // Build parent -> children map
    let mut children_map: HashMap<Option<u32>, Vec<&Item>> = HashMap::new();
    for item in items {
        children_map.entry(item.parent_id).or_default().push(item);
    }
    
    // Sort children by position
    for children in children_map.values_mut() {
        children.sort_by_key(|i| i.position);
    }
    
    // Recursive helper
    fn collect(
        parent_id: Option<u32>,
        depth: usize,
        children_map: &HashMap<Option<u32>, Vec<&Item>>,
        result: &mut Vec<(Item, usize)>,
    ) {
        if let Some(children) = children_map.get(&parent_id) {
            for item in children {
                // Add this item
                result.push(((*item).clone(), depth));
                // If not collapsed, add its children
                if !item.collapsed {
                    collect(Some(item.id), depth + 1, children_map, result);
                }
            }
        }
    }
    
    let mut result = Vec::new();
    collect(None, 0, &children_map, &mut result);
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Item;

    fn make_item(id: u32, parent_id: Option<u32>, position: i32) -> Item {
        Item {
            id,
            text: format!("Item {}", id),
            completed: false,
            item_type: "daily".to_string(),
            memo: None,
            target_count: None,
            current_count: 0,
            parent_id,
            position,
            collapsed: false,
        }
    }

    #[test]
    fn test_flatten_tree() {
        let items = vec![
            make_item(1, None, 0),    // Root 1
            make_item(2, None, 1),    // Root 2
            make_item(3, Some(1), 0), // Child of 1
            make_item(4, Some(1), 1), // Child of 1
            make_item(5, Some(3), 0), // Child of 3 (grandchild of 1)
        ];
        
        let tree = flatten_tree(&items);
        
        // Should be: 1 (depth 0), 3 (depth 1), 5 (depth 2), 4 (depth 1), 2 (depth 0)
        assert_eq!(tree.len(), 5);
        assert_eq!(tree[0].0.id, 1); assert_eq!(tree[0].1, 0);
        assert_eq!(tree[1].0.id, 3); assert_eq!(tree[1].1, 1);
        assert_eq!(tree[2].0.id, 5); assert_eq!(tree[2].1, 2);
        assert_eq!(tree[3].0.id, 4); assert_eq!(tree[3].1, 1);
        assert_eq!(tree[4].0.id, 2); assert_eq!(tree[4].1, 0);
    }
}
