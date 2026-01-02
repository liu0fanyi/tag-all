# 开发者文档

## 项目结构

```
firefox-extension/
├── manifest.json              # 扩展配置文件
├── background.js              # 后台脚本（核心逻辑）
├── content-extract.js         # 内容提取脚本
├── config/                    # 配置页面
│   ├── config.html
│   ├── config.js
│   └── config.css
├── sidebar/                   # 侧边栏UI
│   ├── sidebar.html
│   ├── sidebar.js
│   └── sidebar.css
├── lib/                       # 第三方库
│   ├── libsql-client.js       # Turso客户端
│   ├── readability.js         # Mozilla内容提取
│   └── DEPENDENCIES.md
└── icons/                     # 图标资源
```

## 核心模块

### 1. background.js
后台脚本，负责：
- 监听工具栏按钮点击
- 调用content script提取页面内容
- 保存数据到Turso
- 显示通知
- 管理workspace和tags

### 2. content-extract.js
内容脚本，在页面上下文中运行：
- 使用Readability提取正文
- 生成页面摘要
- 通过消息传递发送回background

### 3. config/
配置管理：
- 保存Turso URL和Token到browser.storage.local
- 测试数据库连接
- 配置验证

### 4. sidebar/
侧边栏UI：
- 加载web-bookmark列表
- 实时搜索
- 点击打开原始URL

## 数据流

```
用户点击图标
    ↓
background.js 监听
    ↓
发送消息到 content-extract.js
    ↓
提取页面内容（Readability）
    ↓
返回 {title, url, summary}
    ↓
background.js 保存到Turso
    ↓
更新sidebar
```

## API说明

### Background Script

**saveToTurso(tab, content)**
```javascript
// 保存页面到Turso
const result = await saveToTurso(tab, {
  title: '页面标题',
  summary: '摘要...'
});
```

**ensureWorkspace(client, name)**
```javascript
// 确保workspace存在
const workspaceId = await ensureWorkspace(client, 'web-bookmark');
```

**ensureTag(client, name, color)**
```javascript
// 确保tag存在
const tagId = await ensureTag(client, '待整理', '#FFA500');
```

### Content Script

**extractContent()**
```javascript
// 提取页面内容
const content = extractContent();
// 返回 {title, summary, textContent}
```

## 消息传递

### Background → Content Script
```javascript
browser.tabs.sendMessage(tabId, {
  type: 'extract-content'
});
```

### Content Script → Background
```javascript
browser.runtime.sendMessage({
  type: 'content-extracted',
  data: { title, summary, textContent }
});
```

### Background → Sidebar
```javascript
browser.runtime.sendMessage({
  type: 'refresh-bookmarks'
});
```

## 存储结构

### browser.storage.local
```javascript
{
  tursoUrl: 'libsql://xxx.turso.io',
  tursoToken: 'eyJ...'
}
```

## 数据库Schema

### Items表（需要的字段）
```sql
CREATE TABLE items (
  id INTEGER PRIMARY KEY,
  text TEXT NOT NULL,
  url TEXT,                  -- 新增
  summary TEXT,              -- 新增
  item_type TEXT,
  workspace_id INTEGER,
  position INTEGER,
  created_at TEXT
);
```

## 调试

### 开启调试日志
所有错误已输出到控制台：
```javascript
console.error('保存失败:', error);
```

### 查看日志
1. 打开 `about:debugging`
2. 找到扩展
3. 点击"检查"
4. 查看Console标签

### 调试Content Script
1. 打开任意网页
2. 按F12打开开发者工具
3. 在Console中输入`extractContent()`

## 已知限制

1. **Manifest V2**: Firefox仍完全支持，但未来可能需要迁移到V3
2. **Content Script权限**: 需要`<all_urls>`权限
3. **CORS**: libsql-client需要Turso支持CORS
4. **文件大小**: 扩展总大小应控制在5MB以下

## 优化建议

### 性能
- [ ] 使用IndexedDB缓存书签列表
- [ ] 实现增量同步而非全量加载
- [ ] 添加loading状态

### 功能
- [ ] 支持批量导入书签
- [ ] 添加标签选择器
- [ ] 支持自定义workspace
- [ ] 添加快捷键

### UI/UX
- [ ] 添加深色模式
- [ ] 优化移动端适配
- [ ] 添加动画过渡

## 发布清单

- [ ] 下载并包含所有依赖库
- [ ] 创建/优化图标
- [ ] 测试所有功能
- [ ] 编写用户文档
- [ ] 创建截图
- [ ] 提交到Firefox Addons

## 许可证

MIT
