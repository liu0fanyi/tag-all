// background.js - 核心业务逻辑

// 监听工具栏按钮点击
browser.browserAction.onClicked.addListener(async (tab) => {
    try {
        // 显示正在保存通知
        browser.notifications.create('saving', {
            type: 'basic',
            title: 'tag-all',
            message: '正在保存...',
            iconUrl: browser.runtime.getURL('icons/icon-48.png')
        });

        // 获取页面内容（通过content script）
        const content = await extractPageContent(tab.id);

        // 保存到Turso
        const result = await saveToTurso(tab, content);

        if (result.success) {
            // 成功通知
            browser.notifications.create('success', {
                type: 'basic',
                title: 'tag-all',
                message: '已保存到云端！',
                iconUrl: browser.runtime.getURL('icons/icon-48.png')
            });

            // 通知sidebar刷新
            browser.runtime.sendMessage({ type: 'refresh-bookmarks' });
        } else {
            throw new Error(result.error);
        }
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

// 保存到Turso数据库
async function saveToTurso(tab, content) {
    try {
        // 1. 获取配置
        const config = await browser.storage.local.get(['tursoUrl', 'tursoToken']);

        if (!config.tursoUrl || !config.tursoToken) {
            throw new Error('请先配置Turso数据库连接（右键图标 → 选项）');
        }

        // 2. 创建libsql客户端
        const client = createClient({
            url: config.tursoUrl,
            authToken: config.tursoToken
        });

        // 3. 确保workspace存在
        let workspaceId = await ensureWorkspace(client, 'web-bookmark');

        // 4. 确保tags存在
        const tag1Id = await ensureTag(client, '待处理', '#FFA500');
        const tag2Id = await ensureTag(client, 'web-bookmark', '#4A9EFF');

        // 5. 创建item
        const now = Math.floor(Date.now() / 1000);
        const itemResult = await client.execute({
            sql: `INSERT INTO items (text, url, summary, item_type, workspace_id, position, created_at, updated_at) 
            VALUES (?, ?, ?, 'daily', ?, 
              (SELECT COALESCE(MAX(position), -1) + 1 FROM items WHERE workspace_id = ?), ?, ?)
            RETURNING id`,
            args: [
                content.title || tab.title,
                tab.url,
                content.summary || '',
                workspaceId,
                workspaceId,
                now,
                now
            ]
        });

        const itemId = itemResult.rows[0].id;

        // 6. 添加tags
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

        // 7. 同步
        await client.sync();

        return { success: true, itemId };

    } catch (error) {
        console.error('Turso保存错误:', error);
        return { success: false, error: error.message };
    }
}

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
