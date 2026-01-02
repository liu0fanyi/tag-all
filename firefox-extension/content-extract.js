// content-extract.js - 页面内容提取脚本

// 监听来自background的消息
browser.runtime.onMessage.addListener((message) => {
    if (message.type === 'extract-content') {
        extractAndSend();
    }
});

function extractAndSend() {
    try {
        const content = extractContent();
        browser.runtime.sendMessage({
            type: 'content-extracted',
            data: content
        });
    } catch (error) {
        console.error('内容提取失败:', error);
        browser.runtime.sendMessage({
            type: 'content-extracted',
            data: {
                title: document.title,
                summary: '',
                textContent: ''
            }
        });
    }
}

function extractContent() {
    // 使用Readability提取页面内容
    if (typeof Readability !== 'undefined') {
        const documentClone = document.cloneNode(true);
        const reader = new Readability(documentClone);
        const article = reader.parse();

        if (article) {
            return {
                title: article.title || document.title,
                summary: generateSummary(article.textContent),
                textContent: article.textContent || ''
            };
        }
    }

    // 降级方案：使用基础提取
    return {
        title: document.title,
        summary: generateSummary(getPageText()),
        textContent: getPageText()
    };
}

function generateSummary(text) {
    if (!text) return '';

    // 清理文本
    text = text.trim().replace(/\s+/g, ' ');

    // 取前200字符作为摘要
    if (text.length <= 200) {
        return text;
    }

    // 尝试在句号处截断
    const truncated = text.substring(0, 200);
    const lastPeriod = truncated.lastIndexOf('。');

    if (lastPeriod > 100) {
        return truncated.substring(0, lastPeriod + 1);
    }

    return truncated + '...';
}

function getPageText() {
    // 移除script和style标签
    const clone = document.body.cloneNode(true);
    const scripts = clone.querySelectorAll('script, style, nav, footer, aside');
    scripts.forEach(el => el.remove());

    return clone.textContent || clone.innerText || '';
}
