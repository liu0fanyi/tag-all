// lib/libsql-client.js - 简化版Turso客户端
// 如果无法从CDN下载官方客户端，使用此简化版本

function createClient({ url, authToken }) {
    // 将libsql://转换为https://
    const baseUrl = url.replace('libsql://', 'https://');

    function encodeValue(val) {
        if (val === null || val === undefined) {
            return { type: 'null' };
        }
        if (typeof val === 'number') {
            if (!Number.isFinite(val)) {
                // NaN or Infinity cannot be sent as JSON numbers correctly for SQL
                // Treat as null or 0? 0 is safer for integers, null for others.
                // Let's return null to be safe, or throw error.
                return { type: 'null' };
            }
            if (Number.isInteger(val)) {
                return { type: 'integer', value: String(val) };
            }
            return { type: 'float', value: val };
        }
        if (typeof val === 'boolean') {
            return { type: 'integer', value: val ? "1" : "0" };
        }
        if (typeof val === 'string') {
            return { type: 'text', value: val };
        }
        if (val instanceof Uint8Array) {
            // base64 encoding needed for blobs if we supported them, 
            // but for now simple fallback or error. 
            // Ideally implement base64 conversion here.
            // For this lightweight client, let's assume text for now or implement a simple encoder if needed.
            // But the user use case (web clipper) mostly uses text.
            // Let's safe-guard:
            return { type: 'blob', base64: btoa(String.fromCharCode(...val)) };
        }
        // Fallback for objects/arrays -> JSON string or text representation
        return { type: 'text', value: String(val) };
    }

    function decodeValue(val) {
        if (!val) return null;
        if (val.type === 'null') return null;
        if (val.type === 'integer') return Number(val.value);
        if (val.type === 'float') return Number(val.value);
        if (val.type === 'blob') {
            // simplified: return base64 string or we could decode it. 
            // For now, raw base64 string is safer for simple usage.
            return val.base64;
        }
        if (val.type === 'text') return val.value;
        return val.value; // fallback
    }

    function parseResultSet(result) {
        const columns = result.cols || [];
        const rows = result.rows || [];

        return {
            rows: rows.map(row => {
                const obj = {};
                row.forEach((val, index) => {
                    const colName = columns[index]?.name;
                    if (colName) {
                        obj[colName] = decodeValue(val);
                    }
                });
                return obj;
            }),
            rowsAffected: result.affected_row_count || 0,
            lastInsertRowid: result.last_insert_rowid
        };
    }

    async function request(endpoint, body) {
        const response = await fetch(`${baseUrl}${endpoint}`, {
            method: 'POST',
            headers: {
                'Authorization': `Bearer ${authToken}`,
                'Content-Type': 'application/json'
            },
            body: JSON.stringify(body)
        });

        if (!response.ok) {
            const errorText = await response.text();
            throw new Error(`HTTP ${response.status}: ${errorText}`);
        }

        return await response.json();
    }

    return {
        async execute({ sql, args = [] }) {
            const encodedArgs = args.map(encodeValue);
            const data = await request('/v2/pipeline', {
                requests: [{
                    type: 'execute',
                    stmt: { sql, args: encodedArgs }
                }]
            });

            // Turso API返回格式: { results: [{ type: "ok", response: { type: "execute", result: {...} } }] }
            if (!data.results || data.results.length === 0) {
                throw new Error('Invalid response from Turso');
            }

            const firstResult = data.results[0];

            // 检查是否有错误
            if (firstResult.type === 'error') {
                throw new Error(firstResult.error?.message || 'Unknown error');
            }

            // 提取结果
            const result = firstResult.response?.result || {};
            return parseResultSet(result);
        },

        async batch(requests) {
            const data = await request('/v2/pipeline', {
                requests: requests.map(r => ({
                    type: 'execute',
                    stmt: {
                        sql: r.sql,
                        args: (r.args || []).map(encodeValue)
                    }
                }))
            });

            if (!data.results) {
                throw new Error('Invalid batch response from Turso');
            }

            return data.results.map(r => {
                if (r.type === 'error') {
                    throw new Error(r.error?.message || 'Batch operation failed');
                }

                const result = r.response?.result || {};
                return parseResultSet(result);
            });
        },

        async sync() {
            // Turso自动处理同步，这里只需返回成功
            return Promise.resolve();
        }
    };
}

// 导出为全局变量（用于非模块环境）
if (typeof window !== 'undefined') {
    window.createClient = createClient;
}
