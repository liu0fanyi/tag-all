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

    // 监听标签页激活，高亮当前标签页对应的书签
    browser.tabs.onActivated.addListener(async (activeInfo) => {
        const tab = await browser.tabs.get(activeInfo.tabId);
        highlightCurrentTab(tab.url);
    });

    // 监听标签页URL变化
    browser.tabs.onUpdated.addListener((tabId, changeInfo, tab) => {
        if (changeInfo.url && tab.active) {
            highlightCurrentTab(changeInfo.url);
        }
    });

    // 初始化时高亮当前标签
    browser.tabs.query({ active: true, currentWindow: true }).then(tabs => {
        if (tabs[0]) highlightCurrentTab(tabs[0].url);
    });
});

let currentTagFilter = null;

async function loadBookmarks(searchQuery = '') {
    const listEl = document.getElementById('list');

    try {
        // 获取配置和本地队列
        const data = await browser.storage.local.get(['tursoUrl', 'tursoToken', 'syncQueue']);
        const config = { tursoUrl: data.tursoUrl, tursoToken: data.tursoToken };
        const syncQueue = data.syncQueue || [];

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

        // 1. 先立即显示本地队列项 (不阻塞)
        const queuedItems = syncQueue.filter(item => {
            const matchSearch = !searchQuery || (item.title && item.title.toLowerCase().includes(searchQuery.toLowerCase()));
            const matchTag = !currentTagFilter;
            return matchSearch && matchTag;
        }).map(item => ({
            id: 'pending-' + (item.added_at || Date.now()),
            text: item.title,
            url: item.url,
            summary: item.selection,
            created_at: item.created_at,
            pending: true
        })).reverse();

        // 如果有本地队列项，先显示它们（加上"加载中"提示）
        if (queuedItems.length > 0) {
            renderBookmarks(queuedItems, true); // true = isLoading
        } else {
            listEl.innerHTML = `<div class="loading">⏳ 加载中...</div>`;
        }

        // 2. 创建客户端并异步加载DB数据
        client = createClient({
            url: config.tursoUrl,
            authToken: config.tursoToken
        });

        // 加载Tags (不阻塞主列表)
        loadTags(client).catch(console.warn);

        // 3. 查询DB数据
        let dbItems = [];
        try {
            let sql = '';
            let args = [];

            if (currentTagFilter) {
                sql = `
                  SELECT DISTINCT i.id, i.text, i.url, i.summary, i.created_at
                  FROM items i
                  JOIN item_tags it1 ON i.id = it1.item_id
                  JOIN tags t1 ON it1.tag_id = t1.id
                  JOIN item_tags it2 ON i.id = it2.item_id
                  JOIN tags t2 ON it2.tag_id = t2.id
                  WHERE t1.name = 'web-bookmark' 
                  AND t2.id = ? 
                  ${searchQuery ? 'AND i.text LIKE ?' : ''}
                  ORDER BY i.created_at DESC
                  LIMIT 100
               `;
                args = [currentTagFilter];
                if (searchQuery) args.push(`%${searchQuery}%`);

            } else {
                sql = `
                  SELECT DISTINCT i.id, i.text, i.url, i.summary, i.created_at
                  FROM items i
                  JOIN item_tags it ON i.id = it.item_id
                  JOIN tags t ON it.tag_id = t.id
                  WHERE t.name = 'web-bookmark'
                  ${searchQuery ? 'AND i.text LIKE ?' : ''}
                  ORDER BY i.created_at DESC
                  LIMIT 100
                `;
                if (searchQuery) args.push(`%${searchQuery}%`);
            }

            const result = await client.execute({ sql, args });
            dbItems = result.rows;

        } catch (error) {
            console.warn('DB Query failed:', error);
        }

        // 4. 合并并最终渲染
        const finalItems = [...queuedItems, ...dbItems];
        renderBookmarks(finalItems);

    } catch (error) {
        console.error('加载失败:', error);
        getErrorHtml(error.message);
    }
}

async function loadTags(client) {
    const tagsEl = document.getElementById('tags');
    if (!tagsEl) return;

    // 获取所有在该workspace下使用过的Tags
    // (关联了 web-bookmark 里的items 的 tags)
    // SQL: Find tags used by items that also have 'web-bookmark' tag.
    try {
        const sql = `
            SELECT DISTINCT t.id, t.name, t.color
            FROM tags t
            JOIN item_tags it ON t.id = it.tag_id
            JOIN items i ON it.item_id = i.id
            JOIN item_tags it_wb ON i.id = it_wb.item_id
            JOIN tags t_wb ON it_wb.tag_id = t_wb.id
            WHERE t_wb.name = 'web-bookmark'
            AND t.name != 'web-bookmark' -- Exclude itself
            ORDER BY t.name
        `;

        const result = await client.execute({ sql, args: [] });
        const tags = result.rows;

        // Render
        const allClass = !currentTagFilter ? 'active' : '';
        let html = `<span class="tag-pill ${allClass}" data-id="">全部</span>`;

        tags.forEach(tag => {
            const activeClass = currentTagFilter === tag.id ? 'active' : '';
            const colorStyle = tag.color ? `style="border-color:${tag.color}; color:${tag.color}"` : '';
            html += `<span class="tag-pill ${activeClass}" data-id="${tag.id}" ${colorStyle}>${escapeHtml(tag.name)}</span>`;
        });

        tagsEl.innerHTML = html;

        // Events
        tagsEl.querySelectorAll('.tag-pill').forEach(el => {
            el.addEventListener('click', () => {
                const id = el.dataset.id;
                currentTagFilter = id ? parseInt(id) : null;
                loadBookmarks(); // Refresh list with filter
            });
        });

    } catch (e) {
        console.warn('Tag fetch failed:', e);
    }
}

function renderBookmarks(items, isLoading = false) {
    const listEl = document.getElementById('list');

    if (items.length === 0) {
        if (isLoading) {
            listEl.innerHTML = `<div class="loading">⏳ 加载中...</div>`;
            return;
        }

        listEl.innerHTML = `
      <div class="empty-state">
        <p>📭 还没有保存的书签</p>
        <small>点击工具栏图标保存当前页面</small>
      </div>
    `;
        return;
    }

    let html = items.map(item => {
        const pendingClass = item.pending ? 'pending' : '';
        const pendingBadge = item.pending ? '<span class="badge">⏳</span>' : '';

        // Extract domain from URL
        let domain = '';
        try {
            if (item.url) {
                domain = new URL(item.url).hostname.replace(/^www\./, '');
            }
        } catch (e) {
            domain = '';
        }

        // Favicon URL (using DuckDuckGo's reliable favicon service)
        const faviconUrl = domain ? `https://icons.duckduckgo.com/ip3/${domain}.ico` : '';

        // Item ID for delete (pending items use url as identifier)
        const itemId = item.pending ? '' : item.id;
        const isPending = item.pending ? 'true' : 'false';

        return `
    <div class="item ${pendingClass}" data-url="${escapeHtml(item.url || '')}" data-id="${itemId}" data-pending="${isPending}">
      ${faviconUrl ? `<img class="favicon" src="${faviconUrl}" alt="">` : '<span class="favicon">📄</span>'}
      <div class="item-info">
        ${pendingBadge}
        <span class="title">${escapeHtml(item.text)}</span>
        ${domain ? `<span class="domain">${escapeHtml(domain)}</span>` : ''}
      </div>
      <button class="delete-btn" title="删除">×</button>
    </div>
  `}).join('');

    if (isLoading) {
        html += `<div class="loading-mini" style="text-align:center; padding:10px; color:#999;">⏳ 同步中...</div>`;
    }

    listEl.innerHTML = html;

    // 添加点击事件
    listEl.querySelectorAll('.item').forEach(el => {
        // Click on item to open
        el.addEventListener('click', async (e) => {
            // Ignore if clicking delete button
            if (e.target.classList.contains('delete-btn')) return;

            const url = el.dataset.url;
            if (url) {
                // 查找是否已有打开的标签页
                const tabs = await browser.tabs.query({ url: url });
                if (tabs.length > 0) {
                    // 切换到已有标签页
                    await browser.tabs.update(tabs[0].id, { active: true });
                    await browser.windows.update(tabs[0].windowId, { focused: true });
                } else {
                    // 没有则新开
                    browser.tabs.create({ url });
                }
            }
        });

        // Delete button click
        el.querySelector('.delete-btn').addEventListener('click', async (e) => {
            e.stopPropagation();

            const isPending = el.dataset.pending === 'true';
            const url = el.dataset.url;
            const itemId = el.dataset.id;

            if (isPending) {
                // Remove from local sync queue
                await removeFromSyncQueue(url);
            } else if (itemId) {
                // Delete from database via background
                await browser.runtime.sendMessage({
                    type: 'delete-bookmark',
                    itemId: parseInt(itemId)
                });
            }

            // Remove from UI immediately
            el.remove();
        });
    });
}

// Remove item from sync queue by URL
async function removeFromSyncQueue(url) {
    const data = await browser.storage.local.get('syncQueue');
    const queue = data.syncQueue || [];

    const newQueue = queue.filter(item => item.url !== url);
    await browser.storage.local.set({ syncQueue: newQueue });
}

// 高亮当前标签页对应的书签
function highlightCurrentTab(currentUrl) {
    if (!currentUrl) return;

    const listEl = document.getElementById('list');
    if (!listEl) return;

    // 移除之前的高亮
    listEl.querySelectorAll('.item.active').forEach(el => {
        el.classList.remove('active');
    });

    // 查找匹配的书签并高亮
    listEl.querySelectorAll('.item').forEach(el => {
        const itemUrl = el.dataset.url;
        if (itemUrl && itemUrl === currentUrl) {
            el.classList.add('active');
            // 滚动到可见区域
            el.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
        }
    });
}
// Helper to just return error HTML if needed or keep existing logic logic
function getErrorHtml(msg) {
    const listEl = document.getElementById('list');
    listEl.innerHTML = `<div class="error"><p>❌ 加载失败</p><small>${escapeHtml(msg)}</small></div>`;
}

// ensureSchema ... existing ...

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
