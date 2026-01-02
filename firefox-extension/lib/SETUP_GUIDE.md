# 使用Clone的完整版库文件

如果你已经clone了官方仓库，按以下步骤提取文件：

## 1. @libsql/client

### 仓库位置
假设你clone到了某个目录，比如 `D:\libs\libsql-client-ts`

### 方式A：直接复制构建后的文件（推荐）

```bash
# 首先在仓库目录中构建
cd D:\libs\libsql-client-ts
npm install
npm run build

# 复制构建后的文件到扩展目录
copy dist\index.js D:\tag_all\project\tauri-apps\tag-all\firefox-extension\lib\libsql-client.js
```

### 方式B：如果没有构建版本，使用源代码

```bash
# 复制整个src目录
xcopy /E /I D:\libs\libsql-client-ts\src D:\tag_all\project\tauri-apps\tag-all\firefox-extension\lib\libsql

# 然后修改background.js的导入方式
```

---

## 2. @mozilla/readability

### 仓库位置
假设clone到 `D:\libs\readability`

### 直接复制（非常简单）

```bash
# Readability只有一个文件
copy D:\libs\readability\Readability.js D:\tag_all\project\tauri-apps\tag-all\firefox-extension\lib\readability.js
```

---

## 完整步骤示例

假设你的clone目录结构是：
```
D:\libs\
  ├── libsql-client-ts/
  └── readability/
```

### 在PowerShell中执行：

```powershell
# 设置路径变量
$libsqlPath = "D:\libs\libsql-client-ts"
$readabilityPath = "D:\libs\readability"
$targetPath = "D:\tag_all\project\tauri-apps\tag-all\firefox-extension\lib"

# 1. 构建libsql-client
cd $libsqlPath
npm install
npm run build

# 2. 复制文件
Copy-Item "$libsqlPath\dist\index.js" "$targetPath\libsql-client.js"
Copy-Item "$readabilityPath\Readability.js" "$targetPath\readability.js"

# 3. 验证
Get-ChildItem $targetPath
```

---

## 如果npm build失败

如果`@libsql/client`构建失败或太复杂，**继续使用我提供的简化版**就可以：
- 简化版已经包含核心功能
- 足够支持Firefox扩展使用
- 无需构建步骤

只复制Readability即可：
```bash
copy D:\libs\readability\Readability.js lib\readability.js
```

---

## 验证是否正确

在Firefox中加载扩展后，打开调试控制台：

```javascript
// 应该都返回 "function"
console.log(typeof createClient);
console.log(typeof Readability);

// 测试createClient
const client = createClient({
  url: 'libsql://test.turso.io',
  authToken: 'test'
});
console.log(client); // 应该是一个对象
```

---

## 我的建议

**推荐组合**：
- ✅ **libsql-client.js**: 使用我提供的简化版（已在lib目录）
- ✅ **Readability.js**: 使用官方完整版（从clone复制）

这样既简单又稳定！只需要：
```bash
copy D:\libs\readability\Readability.js firefox-extension\lib\readability.js
```

然后就可以直接在Firefox中测试扩展了！
