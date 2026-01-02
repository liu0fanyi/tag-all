// config.js - 配置页面逻辑

document.addEventListener('DOMContentLoaded', async () => {
    // 加载已保存的配置
    const config = await browser.storage.local.get(['tursoUrl', 'tursoToken']);

    if (config.tursoUrl) {
        document.getElementById('turso-url').value = config.tursoUrl;
    }

    if (config.tursoToken) {
        document.getElementById('turso-token').value = config.tursoToken;
    }

    // 保存配置
    document.getElementById('save').addEventListener('click', async () => {
        const url = document.getElementById('turso-url').value.trim();
        const token = document.getElementById('turso-token').value.trim();

        if (!url || !token) {
            showStatus('请填写完整的URL和Token', 'error');
            return;
        }

        try {
            await browser.storage.local.set({
                tursoUrl: url,
                tursoToken: token
            });

            showStatus('配置已保存！', 'success');
        } catch (error) {
            showStatus('保存失败: ' + error.message, 'error');
        }
    });

    // 测试连接
    document.getElementById('test').addEventListener('click', async () => {
        const url = document.getElementById('turso-url').value.trim();
        const token = document.getElementById('turso-token').value.trim();

        if (!url || !token) {
            showStatus('请先填写URL和Token', 'error');
            return;
        }

        showStatus('正在测试连接...', 'info');

        try {
            const client = createClient({
                url: url,
                authToken: token
            });

            // 先确保schema正确
            showStatus('正在检查数据库结构...', 'info');
            await ensureSchema(client);

            // 测试查询
            const result = await client.execute({
                sql: 'SELECT 1 as test',
                args: []
            });

            if (result) {
                // 测试成功，自动保存配置
                await browser.storage.local.set({
                    tursoUrl: url,
                    tursoToken: token
                });

                showStatus('✅ 连接成功！配置已自动保存', 'success');
            } else {
                showStatus('连接失败：无法执行查询', 'error');
            }
        } catch (error) {
            console.error('连接测试失败:', error);
            showStatus('❌ 连接失败: ' + error.message, 'error');
        }
    });
});

// 确保数据库有url和summary字段
async function ensureSchema(client) {
    try {
        // 尝试添加url字段
        await client.execute({
            sql: 'ALTER TABLE items ADD COLUMN url TEXT',
            args: []
        });
    } catch (e) {
        // 字段已存在，忽略错误
    }

    try {
        // 尝试添加summary字段
        await client.execute({
            sql: 'ALTER TABLE items ADD COLUMN summary TEXT',
            args: []
        });
    } catch (e) {
        // 字段已存在，忽略错误
    }
}

function showStatus(message, type) {
    const statusEl = document.getElementById('status');
    statusEl.textContent = message;
    statusEl.className = 'status ' + type;
    statusEl.classList.remove('hidden');

    if (type === 'success' || type === 'error') {
        setTimeout(() => {
            statusEl.classList.add('hidden');
        }, 5000);
    }
}
