//! Repository Integration Tests
//!
//! Tests for ItemRepository with in-memory SQLite database.

#[cfg(test)]
mod tests {
    use crate::domain::{Item, ItemType};
    use crate::repository::{Repository, HierarchyRepository, ItemRepository, init_db};
    use crate::repository::item::ItemHierarchyOperations;
    use std::path::PathBuf;

    async fn setup_test_db() -> ItemRepository {
        // Use in-memory database for tests
        // Note: init_db implementation in db.rs handles :memory: path handling if supported by tauri-sync-db-backend
        // If not, we might need a temporary file. 
        // But assuming rusqlite accepts :memory:
        let db_path = PathBuf::from(":memory:");
        let db_state = init_db(&db_path).await.expect("Failed to init test DB");
        
        // Return repository using the shared connection
        ItemRepository::new(db_state.conn.clone())
    }

    // ========================
    // Level 1: Basic CRUD Tests
    // ========================

    #[tokio::test]
    async fn test_create_item() {
        let repo = setup_test_db().await;
        
        // Use default ID 0, repository handles assignment
        let item = Item::new(0, "Test item".to_string(), ItemType::Daily);
        let created = repo.create(&item).await.expect("Failed to create");
        
        assert!(created.id > 0);
        assert_eq!(created.text, "Test item");
        assert!(!created.completed);
        
        // Verify timestamps
        assert!(created.created_at.is_some());
        assert!(created.updated_at.is_some());
    }

    #[tokio::test]
    async fn test_find_by_id() {
        let repo = setup_test_db().await;
        
        let item = Item::new(0, "Find me".to_string(), ItemType::Once);
        let created = repo.create(&item).await.expect("Failed to create");
        
        let found = repo.find_by_id(created.id).await.expect("Find failed");
        assert!(found.is_some());
        assert_eq!(found.unwrap().text, "Find me");
    }

    #[tokio::test]
    async fn test_list_items() {
        let repo = setup_test_db().await;
        
        repo.create(&Item::new(0, "Item 1".to_string(), ItemType::Daily)).await.unwrap();
        repo.create(&Item::new(0, "Item 2".to_string(), ItemType::Daily)).await.unwrap();
        
        let items = repo.list().await.expect("List failed");
        assert_eq!(items.len(), 2);
    }

    #[tokio::test]
    async fn test_update_item() {
        let repo = setup_test_db().await;
        
        let item = Item::new(0, "Original".to_string(), ItemType::Daily);
        let mut created = repo.create(&item).await.unwrap();
        
        // Wait to ensure updated_at changes
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        
        let created_ts = created.updated_at.unwrap_or(0);

        created.text = "Updated".to_string();
        created.completed = true;
        
        let updated = repo.update(&created).await.expect("Update failed");
        assert_eq!(updated.text, "Updated");
        assert!(updated.completed);
        assert!(updated.updated_at.unwrap_or(0) > created_ts, "Updated timestamp should increase");
    }

    #[tokio::test]
    async fn test_delete_item() {
        let repo = setup_test_db().await;
        
        let item = Item::new(0, "To delete".to_string(), ItemType::Daily);
        let created = repo.create(&item).await.unwrap();
        
        repo.delete(created.id).await.expect("Delete failed");
        
        let found = repo.find_by_id(created.id).await.expect("Find failed");
        assert!(found.is_none());
    }

    #[tokio::test]
    async fn test_item_type_persistence() {
        let repo = setup_test_db().await;
        
        let item = Item::new(0, "Countdown".to_string(), ItemType::Countdown);
        let created = repo.create(&item).await.unwrap();
        
        let found = repo.find_by_id(created.id).await.unwrap().unwrap();
        assert_eq!(found.item_type, ItemType::Countdown);
    }

    // ========================
    // Level 2: Hierarchy Tests
    // ========================

    #[tokio::test]
    async fn test_create_child_item() {
        let repo = setup_test_db().await;
        
        // Create parent
        let parent = repo.create(&Item::new(0, "Parent".to_string(), ItemType::Daily)).await.unwrap();
        
        // Create child
        let child = Item::new_child(0, "Child".to_string(), ItemType::Daily, parent.id, 0);
        let created = repo.create(&child).await.unwrap();
        
        assert_eq!(created.parent_id, Some(parent.id));
    }

    #[tokio::test]
    async fn test_get_children() {
        let repo = setup_test_db().await;
        
        // Create parent
        let parent = repo.create(&Item::new(0, "Parent".to_string(), ItemType::Daily)).await.unwrap();
        
        // Create children
        repo.create(&Item::new_child(0, "Child 1".to_string(), ItemType::Daily, parent.id, 0)).await.unwrap();
        repo.create(&Item::new_child(0, "Child 2".to_string(), ItemType::Daily, parent.id, 1)).await.unwrap();
        
        let children = repo.get_children(Some(parent.id)).await.expect("Get children failed");
        assert_eq!(children.len(), 2);
        assert_eq!(children[0].text, "Child 1");
        assert_eq!(children[1].text, "Child 2");
    }

    #[tokio::test]
    async fn test_get_root_items() {
        let repo = setup_test_db().await;
        
        // Create root items
        repo.create(&Item::new(0, "Root 1".to_string(), ItemType::Daily)).await.unwrap();
        let parent = repo.create(&Item::new(0, "Root 2".to_string(), ItemType::Daily)).await.unwrap();
        
        // Create child (should not appear in root)
        repo.create(&Item::new_child(0, "Child".to_string(), ItemType::Daily, parent.id, 0)).await.unwrap();
        
        let roots = repo.get_children(None).await.expect("Get roots failed");
        assert_eq!(roots.len(), 2);
    }

    #[tokio::test]
    async fn test_move_item() {
        let repo = setup_test_db().await;
        
        // Create two parents
        let parent1 = repo.create(&Item::new(0, "Parent 1".to_string(), ItemType::Daily)).await.unwrap();
        let parent2 = repo.create(&Item::new(0, "Parent 2".to_string(), ItemType::Daily)).await.unwrap();
        
        // Create child under parent1
        let child = repo.create(&Item::new_child(0, "Child".to_string(), ItemType::Daily, parent1.id, 0)).await.unwrap();
        
        // Move to parent2
        repo.move_to(child.id, Some(parent2.id), 0).await.expect("Move failed");
        
        // Verify
        let moved = repo.find_by_id(child.id).await.unwrap().unwrap();
        assert_eq!(moved.parent_id, Some(parent2.id));
        
        // Verify updated_at
        assert!(moved.updated_at.is_some());
    }

    #[tokio::test]
    async fn test_toggle_collapsed() {
        let repo = setup_test_db().await;
        
        let item = repo.create(&Item::new(0, "Parent".to_string(), ItemType::Daily)).await.unwrap();
        assert!(!item.collapsed);
        
        let new_state = repo.toggle_collapsed(item.id).await.expect("Toggle failed");
        assert!(new_state);
        
        let new_state2 = repo.toggle_collapsed(item.id).await.expect("Toggle failed");
        assert!(!new_state2);
    }

    #[tokio::test]
    async fn test_get_descendants() {
        let repo = setup_test_db().await;
        
        // Create hierarchy: Parent -> Child -> Grandchild
        let parent = repo.create(&Item::new(0, "Parent".to_string(), ItemType::Daily)).await.unwrap();
        let child = repo.create(&Item::new_child(0, "Child".to_string(), ItemType::Daily, parent.id, 0)).await.unwrap();
        repo.create(&Item::new_child(0, "Grandchild".to_string(), ItemType::Daily, child.id, 0)).await.unwrap();
        
        let descendants = repo.get_descendants(parent.id).await.expect("Get descendants failed");
        assert_eq!(descendants.len(), 2); // Child + Grandchild
    }

    #[tokio::test]
    async fn test_delete_cascade() {
        let repo = setup_test_db().await;
        
        // Create parent with child
        let parent = repo.create(&Item::new(0, "Parent".to_string(), ItemType::Daily)).await.unwrap();
        let child = repo.create(&Item::new_child(0, "Child".to_string(), ItemType::Daily, parent.id, 0)).await.unwrap();
        
        // Delete parent
        repo.delete(parent.id).await.expect("Delete failed");
        
        // Child should also be deleted (CASCADE)
        let found = repo.find_by_id(child.id).await.expect("Find failed");
        assert!(found.is_none());
    }
}
