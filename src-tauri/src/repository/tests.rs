//! Repository Integration Tests
//!
//! Tests for ItemRepository with in-memory SQLite database.

#[cfg(test)]
mod tests {
    use crate::domain::{Item, ItemType};
    use crate::repository::{Repository, ItemRepository, init_db};
    use std::sync::Arc;
    use tokio::sync::Mutex;
    use std::path::PathBuf;

    async fn setup_test_db() -> ItemRepository {
        // Use in-memory database for tests
        let db_path = PathBuf::from(":memory:");
        let db_state = init_db(&db_path).await.expect("Failed to init test DB");
        let conn = db_state.get_connection().await.expect("Failed to get connection");
        ItemRepository::new(Arc::new(Mutex::new(conn)))
    }

    #[tokio::test]
    async fn test_create_item() {
        let repo = setup_test_db().await;
        
        let item = Item::new(0, "Test item".to_string(), ItemType::Daily);
        let created = repo.create(&item).await.expect("Failed to create");
        
        assert!(created.id > 0);
        assert_eq!(created.text, "Test item");
        assert!(!created.completed);
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
        
        created.text = "Updated".to_string();
        created.completed = true;
        
        let updated = repo.update(&created).await.expect("Update failed");
        assert_eq!(updated.text, "Updated");
        assert!(updated.completed);
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
}
