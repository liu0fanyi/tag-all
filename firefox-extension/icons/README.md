# Firefox扩展图标占位符

该目录需要放置以下图标文件：

- icon-16.png (16x16px)
- icon-48.png (48x48px)  
- icon-128.png (128x128px)

## 创建图标

### 方式1：使用在线工具
访问 https://www.favicon-generator.org/ 上传logo生成多种尺寸

### 方式2：使用设计工具
- Figma、Photoshop、GIMP等创建
- 导出PNG格式，透明背景

### 方式3：使用命令行工具（ImageMagick）
```bash
# 从SVG生成多种尺寸
convert logo.svg -resize 16x16 icon-16.png
convert logo.svg -resize 48x48 icon-48.png
convert logo.svg -resize 128x128 icon-128.png
```

## 设计建议

- 使用简洁的图标设计
- 透明背景
- 主色调可以是紫色/蓝色（与tag-all品牌一致）
- 在白色和深色背景下都要清晰可见

## 临时方案

在创建正式图标前，可以使用纯色方块作为占位符：
- 16x16: 蓝色方块，中间白色"T"
- 48x48: 同上
- 128x128: 同上

可以使用在线工具快速生成：https://placeholder.com/
