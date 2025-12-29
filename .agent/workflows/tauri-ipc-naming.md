# Tauri IPC 命名规范

## 概述
当从前端 (Leptos/WASM) 调用 Tauri 后端命令时，参数命名必须使用 **camelCase**。

## 规则

### 后端参数命名 (Rust)
后端 Tauri 命令使用 snake_case：
```rust
#[tauri::command]
pub async fn update_item(
    state: State<'_, AppState>,
    id: u32,
    text: Option<String>,
    item_type: Option<String>,  // snake_case in Rust
) -> Result<Item, String>
```

### 前端参数命名 (JSON)
前端调用时必须使用 camelCase：
```rust
// 正确 ✅
json.push_str(&format!(r#","itemType":"{}""#, it));

// 错误 ❌ - Tauri 不会识别
json.push_str(&format!(r#","item_type":"{}""#, it));
```

## 为什么

Tauri 2.0 的 IPC 机制自动将 JavaScript/JSON 的 camelCase 参数转换为 Rust 的 snake_case 参数。这意味着：

- 前端 JSON: `{"itemType": "daily"}` → 后端 Rust: `item_type = Some("daily")`
- 前端 JSON: `{"item_type": "daily"}` → 后端 Rust: `item_type = None` (不匹配!)

## 常见字段映射

| 前端 JSON (camelCase) | 后端 Rust (snake_case) |
|-----------------------|------------------------|
| `itemType`            | `item_type`            |
| `targetCount`         | `target_count`         |
| `currentCount`        | `current_count`        |
| `parentId`            | `parent_id`            |
| `workspaceId`         | `workspace_id`         |
| `newParentId`         | `new_parent_id`        |

## 调试技巧

如果后端收到 `None` 但前端确实发送了值：
1. 检查前端 JSON 字段名是否为 camelCase
2. 在后端添加 `println!` 日志确认收到的参数
3. 在前端添加 `console.log` 显示发送的 JSON
