// sidebar.js - 侧边栏逻辑

let client = null;
let searchTimeout = null;

// 初始化
document.addEventListener('DOMContentLoaded', () => {
    loadBookmarks();

    // 搜索
    document.getElementById('search').addEventListener('input', (e) => {
        clearTimeout(searchTimeout);
        searchTimeout = setTimeout(() => {
            loadBookmarks(e.target.value.trim());
        }, 300);
    });

    // 刷新
    document.getElementById('refresh').addEventListener('click', () => {
        loadBookmarks();
    });

    // 设置
    document.getElementById('settings').addEventListener('click', () => {
        browser.runtime.openOptionsPage();
    });

    // 监听background的刷新通知
    browser.runtime.onMessage.addListener((message) => {
        if (message.type === 'refresh-bookmarks') {
            loadBookmarks();
        }
    });
});

async function loadBookmarks(searchQuery = '') {
    const listEl = document.getElementById('list');

    try {
        // 获取配置
        const config = await browser.storage.local.get(['tursoUrl', 'tursoToken']);

        if (!config.tursoUrl || !config.tursoToken) {
            listEl.innerHTML = `
        <div class="empty-state">
          <p>⚙️ 请先配置数据库连接</p>
          <button id="open-settings-btn">打开设置</button>
        </div>
      `;
            document.getElementById('open-settings-btn').addEventListener('click', () => {
                browser.runtime.openOptionsPage();
            });
            return;
        }

        // 创建客户端
        client = createClient({
            url: config.tursoUrl,
            authToken: config.tursoToken
        });

        console.log('Sidebar: Ensuring schema...');
        // 确保schema正确（添加url和summary字段如果不存在）
        try {
            await ensureSchema(client);
            console.log('Sidebar: Schema ensured');
        } catch (error) {
            console.warn('Sidebar: Schema migration failed, will try query anyway:', error);
        }

        console.log('Sidebar: Querying bookmarks...');
        // 查询数据
        const sql = searchQuery ? `
      SELECT DISTINCT i.id, i.text, i.url, i.summary, i.created_at
      FROM items i
      INNER JOIN item_tags it ON i.id = it.item_id
      INNER JOIN tags t ON it.tag_id = t.id
      WHERE t.name = 'web-bookmark' AND i.text LIKE ?
      ORDER BY i.created_at DESC
      LIMIT 100
    ` : `
      SELECT i.id, i.text, i.url, i.summary, i.created_at
      FROM items i
      INNER JOIN item_tags it ON i.id = it.item_id
      INNER JOIN tags t ON it.tag_id = t.id
      WHERE t.name = 'web-bookmark'
      ORDER BY i.created_at DESC
      LIMIT 100
    `;

        const result = await client.execute({
            sql: sql,
            args: searchQuery ? [`%${searchQuery}%`] : []
        });

        renderBookmarks(result.rows);

    } catch (error) {
        console.error('加载失败:', error);
        listEl.innerHTML = `
      <div class="error">
        <p>❌ 加载失败</p>
        <small>${escapeHtml(error.message)}</small>
      </div>
    `;
    }
}

function renderBookmarks(items) {
    const listEl = document.getElementById('list');

    if (items.length === 0) {
        listEl.innerHTML = `
      <div class="empty-state">
        <p>📭 还没有保存的书签</p>
        <small>点击工具栏图标保存当前页面</small>
      </div>
    `;
        return;
    }

    listEl.innerHTML = items.map(item => `
    <div class="item" data-url="${escapeHtml(item.url || '')}">
      <div class="title">${escapeHtml(item.text)}</div>
      ${item.summary ? `<div class="summary">${escapeHtml(item.summary)}</div>` : ''}
      <div class="meta">${formatDate(item.created_at)}</div>
    </div>
  `).join('');

    // 添加点击事件
    listEl.querySelectorAll('.item').forEach(el => {
        el.addEventListener('click', () => {
            const url = el.dataset.url;
            if (url) {
                browser.tabs.create({ url });
            }
        });
    });
}

function formatDate(timestamp) {
    const date = new Date(timestamp);
    const now = new Date();
    const diff = now - date;

    const seconds = Math.floor(diff / 1000);
    const minutes = Math.floor(seconds / 60);
    const hours = Math.floor(minutes / 60);
    const days = Math.floor(hours / 24);

    if (days > 7) {
        return date.toLocaleDateString('zh-CN');
    } else if (days > 0) {
        return `${days}天前`;
    } else if (hours > 0) {
        return `${hours}小时前`;
    } else if (minutes > 0) {
        return `${minutes}分钟前`;
    } else {
        return '刚刚';
    }
}

// 确保数据库有url和summary字段
async function ensureSchema(client) {
    console.log('ensureSchema: Adding url column...');
    try {
        await client.execute({
            sql: 'ALTER TABLE items ADD COLUMN url TEXT',
            args: []
        });
        console.log('ensureSchema: url column added');
    } catch (e) {
        console.log('ensureSchema: url column exists or error:', e.message);
    }

    console.log('ensureSchema: Adding summary column...');
    try {
        await client.execute({
            sql: 'ALTER TABLE items ADD COLUMN summary TEXT',
            args: []
        });
        console.log('ensureSchema: summary column added');
    } catch (e) {
        console.log('ensureSchema: summary column exists or error:', e.message);
    }

    console.log('ensureSchema: Adding created_at column...');
    try {
        await client.execute({
            sql: 'ALTER TABLE items ADD COLUMN created_at INTEGER DEFAULT 0',
            args: []
        });
        console.log('ensureSchema: created_at column added');
    } catch (e) {
        console.log('ensureSchema: created_at column exists or error:', e.message);
    }

    console.log('ensureSchema: Adding updated_at column...');
    try {
        await client.execute({
            sql: 'ALTER TABLE items ADD COLUMN updated_at INTEGER DEFAULT 0',
            args: []
        });
        console.log('ensureSchema: updated_at column added');
    } catch (e) {
        console.log('ensureSchema: updated_at column exists or error:', e.message);
    }
}

function escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
}
