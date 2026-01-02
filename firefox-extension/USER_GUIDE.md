# Firefox Web Clipper 使用指南

## 快速开始

### 1. 安装扩展

1. 下载依赖库（见 lib/DEPENDENCIES.md）
2. 打开Firefox，输入 `about:debugging`
3. 点击"此Firefox"
4. 点击"临时载入附加组件"
5. 选择 `manifest.json` 文件

### 2. 配置数据库

1. 右键点击工具栏的扩展图标
2. 选择"选项"或直接点击图标
3. 输入你的Turso配置：
   - **Database URL**: `libsql://your-db.turso.io`
   - **Auth Token**: 从Turso控制台获取
4. 点击"测试连接"确认配置正确
5. 点击"保存配置"

### 3. 保存网页

- 浏览任意网页
- 点击工具栏的tag-all图标
- 自动保存到云端，显示通知"已保存到云端！"

### 4. 查看已保存内容

**两种方式**：

1. **侧边栏**（推荐）：
   - 按 `Ctrl+B` 或点击 Firefox菜单 → 侧边栏 → Web Bookmarks
   - 可搜索、刷新

2. **tag-all桌面应用**：
   - 打开tag-all app
   - 查看"web-bookmark" workspace
   - 所有带"web-bookmark"标签的内容

---

## 功能特性

### 自动化
- ✅ 自动创建"web-bookmark" workspace
- ✅ 自动添加"待整理"和"web-bookmark"标签
- ✅ 自动提取页面标题
- ✅ 自动生成摘要（前200字）
- ✅ 保存原始URL

### 智能提取
- 使用Mozilla Readability提取正文
- 过滤广告和无关内容
- 生成简洁摘要

### 实时同步
- 直接保存到Turso云端
- tag-all桌面应用实时可见
- 支持多设备访问

---

## 故障排除

### 无法保存
**症状**: 点击图标没反应或报错

**解决**：
1. 检查是否已配置Turso连接
2. 打开浏览器控制台（F12）查看错误
3. 确认网络连接正常
4. 验证Token权限正确

### 侧边栏无法加载
**症状**: 侧边栏显示"加载失败"

**解决**：
1. 确认已配置Turso URL和Token
2. 点击"设置"重新配置
3. 检查tag-all数据库是否包含url和summary字段

### 摘要提取失败
**症状**: 保存的item没有摘要

**解决**：
- 某些网页（如纯图片页面）可能无法提取正文
- 扩展会使用降级方案，保存基本信息
- 不影响URL和标题的保存

---

## 高级功能

### 快捷键（Firefox设置）

可以在Firefox中自定义快捷键：
1. 输入 `about:addons`
2. 找到 tag-all Web Clipper
3. 点击设置图标 → 管理扩展快捷键
4. 设置保存快捷键（如 `Ctrl+Shift+S`）

### 数据格式

保存到数据库的格式：
```json
{
  "text": "页面标题",
  "url": "https://example.com",
  "summary": "页面摘要...",
  "item_type": "daily",
  "workspace": "web-bookmark",
  "tags": ["待整理", "web-bookmark"]
}
```

---

## 隐私和安全

- ✅ 所有数据直接保存到你的Turso数据库
- ✅ Token仅存储在本地浏览器
- ✅ 不会发送到第三方服务器
- ✅ 开源代码，可审查

---

## 技术支持

如有问题：
1. 查看浏览器控制台错误信息
2. 检查tag-all桌面应用是否最新版本
3. 确认数据库schema包含url和summary字段

## 更新日志

### v1.0.0 (2026-01-02)
- 🎉 首次发布
- ✅ 基础保存功能
- ✅ 自动摘要提取
- ✅ 侧边栏显示
- ✅ 搜索功能
