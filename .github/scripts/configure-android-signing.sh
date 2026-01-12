#!/bin/bash
# 配置 Android Gradle 签名

set -e

GRADLE_FILE="src-tauri/gen/android/app/build.gradle.kts"

echo "Configuring Android signing in $GRADLE_FILE"

# 1. 在文件开头添加 import（如果还没有）
if ! grep -q "import java.io.FileInputStream" "$GRADLE_FILE"; then
  # 在第一行插入
  sed -i '1i import java.io.FileInputStream' "$GRADLE_FILE"
  echo "Added FileInputStream import"
fi

# 2. 在 buildTypes 之前插入 signingConfigs
if ! grep -q "signingConfigs" "$GRADLE_FILE"; then
  # 找到 buildTypes 的行号
  LINE_NUM=$(grep -n "buildTypes {" "$GRADLE_FILE" | head -1 | cut -d: -f1)
  
  if [ -n "$LINE_NUM" ]; then
    # 在 buildTypes 之前插入配置
    sed -i "${LINE_NUM}i\\
    signingConfigs {\\
        create(\"release\") {\\
            val keystorePropertiesFile = rootProject.file(\"keystore.properties\")\\
            val keystoreProperties = Properties()\\
            if (keystorePropertiesFile.exists()) {\\
                keystoreProperties.load(FileInputStream(keystorePropertiesFile))\\
            }\\
            keyAlias = keystoreProperties[\"keyAlias\"] as String\\
            keyPassword = keystoreProperties[\"password\"] as String\\
            storeFile = file(keystoreProperties[\"storeFile\"] as String)\\
            storePassword = keystoreProperties[\"password\"] as String\\
        }\\
    }\\
\\
" "$GRADLE_FILE"
    echo "Added signingConfigs block"
  fi
fi

# 3. 在 release buildType 中添加 signingConfig
if ! grep -q "signingConfig = signingConfigs" "$GRADLE_FILE"; then
  # 找到 getByName("release") 的行号
  LINE_NUM=$(grep -n 'getByName("release")' "$GRADLE_FILE" | head -1 | cut -d: -f1)
  
  if [ -n "$LINE_NUM" ]; then
    # 在下一行（{ 后面）插入
    NEXT_LINE=$((LINE_NUM + 1))
    sed -i "${NEXT_LINE}i\\            signingConfig = signingConfigs.getByName(\"release\")" "$GRADLE_FILE"
    echo "Added signingConfig to release buildType"
  fi
fi

echo "Android signing configuration completed!"
