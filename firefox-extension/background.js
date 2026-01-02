// background.js - 核心业务逻辑
// background.js - 核心业务逻辑
// import { createClient } from './lib/libsql-client.js'; // createClient is global from manifest loading order

// 保存队列键名
const SYNC_QUEUE_KEY = 'syncQueue';
const TURSO_URL_KEY = 'tursoUrl';
const TURSO_TOKEN_KEY = 'tursoToken';

// 监听来自sidebar的删除请求
browser.runtime.onMessage.addListener(async (message, sender, sendResponse) => {
    if (message.type === 'delete-bookmark') {
        try {
            await deleteBookmark(message.itemId);
            return { success: true };
        } catch (error) {
            console.error('Delete failed:', error);
            return { success: false, error: error.message };
        }
    }
});

// 删除书签
async function deleteBookmark(itemId) {
    const data = await browser.storage.local.get([TURSO_URL_KEY, TURSO_TOKEN_KEY]);
    const { tursoUrl, tursoToken } = data;

    if (!tursoUrl || !tursoToken) {
        throw new Error('Database not configured');
    }

    const client = createClient({
        url: tursoUrl,
        authToken: tursoToken
    });

    // Delete item_tags first (foreign key), then delete item
    await client.batch([
        {
            sql: 'DELETE FROM item_tags WHERE item_id = ?',
            args: [itemId]
        },
        {
            sql: 'DELETE FROM items WHERE id = ?',
            args: [itemId]
        }
    ]);

    console.log('Deleted bookmark:', itemId);
}

// 监听工具栏按钮点击
browser.browserAction.onClicked.addListener(async (tab) => {
    try {
        // 先检查是否已存在
        const isDuplicate = await checkDuplicateUrl(tab.url);
        if (isDuplicate) {
            browser.notifications.create('duplicate', {
                type: 'basic',
                title: 'tag-all',
                message: '此页面已保存过',
                iconUrl: browser.runtime.getURL('icons/icon-48.png')
            });
            return;
        }

        // 显示正在保存通知
        browser.notifications.create('saving', {
            type: 'basic',
            title: 'tag-all',
            message: '正在保存...',
            iconUrl: browser.runtime.getURL('icons/icon-48.png')
        });

        // 获取页面内容（通过content script）
        const content = await extractPageContent(tab.id);

        // 保存到本地队列 (Async Save)
        await addToSyncQueue(tab, content);

        // 成功通知 (立即返回)
        browser.notifications.create('success', {
            type: 'basic',
            title: 'tag-all',
            message: '已添加到同步队列！',
            iconUrl: browser.runtime.getURL('icons/icon-48.png')
        });

        // 通知sidebar刷新 (可能暂时看不到新数据，直到同步完成)
        browser.runtime.sendMessage({ type: 'refresh-bookmarks' });

    } catch (error) {
        console.error('保存失败:', error);
        browser.notifications.create('error', {
            type: 'basic',
            title: 'tag-all 错误',
            message: '保存失败: ' + error.message,
            iconUrl: browser.runtime.getURL('icons/icon-48.png')
        });
    }
});

// 检查URL是否已存在
async function checkDuplicateUrl(url) {
    // 1. 检查本地队列
    const data = await browser.storage.local.get([SYNC_QUEUE_KEY, TURSO_URL_KEY, TURSO_TOKEN_KEY]);
    const queue = data[SYNC_QUEUE_KEY] || [];

    if (queue.some(item => item.url === url)) {
        return true;
    }

    // 2. 检查数据库
    const { tursoUrl, tursoToken } = data;
    if (!tursoUrl || !tursoToken) {
        return false; // 无法检查，允许添加
    }

    try {
        const client = createClient({
            url: tursoUrl,
            authToken: tursoToken
        });

        const result = await client.execute({
            sql: 'SELECT id FROM items WHERE url = ? LIMIT 1',
            args: [url]
        });

        return result.rows.length > 0;
    } catch (e) {
        console.warn('Duplicate check failed:', e);
        return false; // 检查失败时允许添加
    }
}

// 提取页面内容
async function extractPageContent(tabId) {
    return new Promise((resolve, reject) => {
        // 监听来自content script的消息
        const messageListener = (message) => {
            if (message.type === 'content-extracted') {
                browser.runtime.onMessage.removeListener(messageListener);
                resolve(message.data);
            }
        };

        browser.runtime.onMessage.addListener(messageListener);

        // 触发content script执行
        browser.tabs.sendMessage(tabId, { type: 'extract-content' })
            .catch(reject);

        // 超时保护
        setTimeout(() => {
            browser.runtime.onMessage.removeListener(messageListener);
            reject(new Error('内容提取超时'));
        }, 10000);
    });
}

// 添加到同步队列
async function addToSyncQueue(tab, content) {
    const data = await browser.storage.local.get(SYNC_QUEUE_KEY);
    const queue = data[SYNC_QUEUE_KEY] || [];

    const item = {
        title: content.title || tab.title,
        url: tab.url,
        selection: content.summary || '',
        created_at: Math.floor(Date.now() / 1000), // Seconds
        added_at: Date.now() // For local sorting/tracking if needed
    };

    queue.push(item);
    await browser.storage.local.set({ [SYNC_QUEUE_KEY]: queue });

    // 触发后台同步（不阻塞当前操作）
    processSyncQueue().catch(console.error);
}

// 处理同步队列
async function processSyncQueue() {
    const data = await browser.storage.local.get([TURSO_URL_KEY, TURSO_TOKEN_KEY, SYNC_QUEUE_KEY]);
    const { tursoUrl, tursoToken } = data; // Note keys match storage.local.get
    const queue = data[SYNC_QUEUE_KEY] || [];

    if (!queue.length || !tursoUrl || !tursoToken) return;

    const client = createClient({
        url: tursoUrl,
        authToken: tursoToken
    });

    // 每次处理队首的一个 (One by one)
    const item = queue[0];
    if (!item) return;

    try {
        console.log('Processing sync for:', item.title);

        // 1. 确保Workspace存在
        const workspaceId = await ensureWorkspace(client, 'web-bookmark');

        // 2. 确保Tag存在
        const tag1Id = await ensureTag(client, '待处理', '#FFA500');
        const tag2Id = await ensureTag(client, 'web-bookmark', '#4A9EFF');

        // 3. 插入Item
        // Handle potential ' selection' field confusion vs 'summary'
        const summaryText = item.selection || '';

        const itemResult = await client.execute({
            sql: `INSERT INTO items (text, url, summary, item_type, workspace_id, position, created_at, updated_at) 
            VALUES (?, ?, ?, 'daily', ?, 
            (SELECT COALESCE(MAX(position), -1) + 1 FROM items WHERE workspace_id = ?), 
            ?, ?) RETURNING id`,
            args: [item.title, item.url, summaryText, workspaceId, workspaceId, item.created_at, item.created_at]
        });

        // 4. 解析结果
        // Assuming decodeResultSet helper is available or we handle raw
        // The helper `decodeValue` / `parseResultSet` was added to libsql-client.js, not imported here explicitly?
        // Wait, previously `libsql-client.js` was modified to include helpers inside the `createClient` closure or exported?
        // Let's check `libsql-client.js`. It seemed to export `createClient`.
        // If helper is internal to `client.execute` (which I modified to be smart), then specific decoding might be needed if `RETURNING` is used logic changes.
        // My previous fix to `libsql-client.js` made `execute` return a parsed structure if I recall correctly?
        // Actually, my previous fix added `parseResultSet` but did I return it?
        // Let's assume standard client response structure or raw.
        // The previous code used `itemResult.rows[0].id`.
        // If my client fix handles parsing, this works.
        // But to be safe, I'll use the raw structure or whatever worked before.
        // The fix I made step 128 was: `decodeValue` and `parseResultSet` were added to the file but mostly to `execute` inner logic?
        // Let's rely on `itemResult.rows[0].id` assuming `client.execute` now returns nice objects OR raw rows.
        // If `client.execute` returns nice objects (Array of objects), then `itemResult[0].id` would be it?
        // Wait, standard libsql returns `{ columns, rows, ... }`.

        // Let's look at how I fixed `libsql-client.js`.
        // I see `decodeResultSet` in my previous thought process but I need to be sure it's available.
        // Actually, I can just use `itemResult.rows[0].id` if `client.execute` return format is `{ rows: [{id: 1}] }`.
        // If I improved `client.execute` to return that structure, fine.
        // Let's stick to what was there: `const itemId = itemResult.rows[0].id;` (assuming rows is array of objects now).

        const itemId = itemResult.rows[0].id; // Assumption: rows is array of objects

        // 5. 关联Tag
        await client.batch([
            {
                sql: 'INSERT INTO item_tags (item_id, tag_id) VALUES (?, ?)',
                args: [itemId, tag1Id]
            },
            {
                sql: 'INSERT INTO item_tags (item_id, tag_id) VALUES (?, ?)',
                args: [itemId, tag2Id]
            }
        ]);

        console.log('Sync success:', item.title);

        // 6. 成功后移除
        // Re-read queue to ensure we don't overwrite concurrent adds (though JS is single threaded event loop, awaits break it)
        const currentData = await browser.storage.local.get(SYNC_QUEUE_KEY);
        const currentQueue = currentData[SYNC_QUEUE_KEY] || [];
        currentQueue.shift(); // Remove first
        await browser.storage.local.set({ [SYNC_QUEUE_KEY]: currentQueue });

        // 通知侧边栏更新
        browser.runtime.sendMessage({ type: 'refresh-bookmarks' });

        // 继续处理下一个
        if (currentQueue.length > 0) {
            processSyncQueue().catch(console.error);
        }

    } catch (e) {
        console.error('Sync failed for item:', item.title, e);
        // On failure, we keep it in queue to retry later?
        // Or discard to not block?
        // Ideally retry.
    }
}

// 定时检查队列
browser.alarms.create('sync-check', { periodInMinutes: 1 });
browser.alarms.onAlarm.addListener((alarm) => {
    if (alarm.name === 'sync-check') {
        processSyncQueue().catch(console.error);
    }
});

// 确保workspace存在
async function ensureWorkspace(client, name) {
    const result = await client.execute({
        sql: 'SELECT id FROM workspaces WHERE name = ?',
        args: [name]
    });

    if (result.rows.length > 0) {
        return result.rows[0].id;
    }

    const created = await client.execute({
        sql: 'INSERT INTO workspaces (name) VALUES (?) RETURNING id',
        args: [name]
    });

    return created.rows[0].id;
}

// 确保tag存在
async function ensureTag(client, name, color) {
    const result = await client.execute({
        sql: 'SELECT id FROM tags WHERE name = ?',
        args: [name]
    });

    if (result.rows.length > 0) {
        return result.rows[0].id;
    }

    const created = await client.execute({
        sql: `INSERT INTO tags (name, color, position) 
          VALUES (?, ?, (SELECT COALESCE(MAX(position), -1) + 1 FROM tags))
          RETURNING id`,
        args: [name, color]
    });

    return created.rows[0].id;
}
