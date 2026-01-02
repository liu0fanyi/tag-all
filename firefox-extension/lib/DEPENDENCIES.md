# 依赖库说明

## 状态：✅ 已包含

所有必要的库文件已经包含在 `lib/` 目录中，**无需额外下载**。

### 包含的文件

1. **libsql-client.js** - Turso数据库客户端（简化版）
   - 直接使用Turso HTTP API
   - 支持execute、batch、sync操作
   
2. **readability.js** - 页面内容提取（简化版）
   - 基于Mozilla Readability核心算法
   - 自动提取标题和正文
   - 生成摘要

## 验证安装

在Firefox扩展调试控制台中测试：

```javascript
// 测试libsql客户端
console.log(typeof createClient); // 应显示 "function"

// 测试Readability
console.log(typeof Readability); // 应显示 "function"
```

## 原始库信息（供参考）

如果需要使用官方完整版本：

- **@libsql/client**: https://github.com/tursodatabase/libsql-client-ts
- **@mozilla/readability**: https://github.com/mozilla/readability

---

## 故障排除

### createClient未定义

确保manifest.json正确加载了库：
```json
"background": {
  "scripts": ["lib/libsql-client.js", "background.js"]
}
```

### Readability未定义

确保content_scripts正确配置：
```json
"content_scripts": [{
  "js": ["lib/readability.js", "content-extract.js"]
}]
```

## 升级到官方库（可选）

如果以后想使用官方完整版：

```bash
npm install @libsql/client @mozilla/readability
# 然后使用bundler打包
```
