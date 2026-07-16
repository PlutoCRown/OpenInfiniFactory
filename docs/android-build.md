# Android 构建

使用 **cargo-ndk + Gradle** 打包 APK，这是 [Bevy 官方推荐](https://github.com/bevyengine/bevy/blob/main/examples/README.md#android)的方案。

## 工具链准备（新电脑）

### 1. Android Studio

安装 [Android Studio](https://developer.android.com/studio)，安装时勾选：

- Android SDK
- Android SDK Platform 30 / 34
- NDK (Side by side) → 选 **27.x**

安装后确认以下路径存在：

```
~/Library/Android/sdk          # macOS
~/Library/Android/sdk/ndk/27.0.12077973
~/Library/Android/sdk/platforms/android-30
~/Library/Android/sdk/platforms/android-34
```

### 2. Java (JBR)

Gradle 需要 JDK 17+。Android Studio 自带 JBR，无需额外安装：

```
/Applications/Android Studio.app/Contents/jbr/Contents/Home
```

### 3. Rust 工具链

```bash
# Android target
rustup target add aarch64-linux-android

# cargo-ndk
cargo install cargo-ndk
```

### 4. 环境变量

构建脚本会自动设置以下变量，也可在 `~/.zshrc` / `~/.bashrc` 中持久配置：

```bash
export ANDROID_HOME="$HOME/Library/Android/sdk"
export ANDROID_NDK_HOME="$ANDROID_HOME/ndk/27.0.12077973"
export JAVA_HOME="/Applications/Android Studio.app/Contents/jbr/Contents/Home"
```

## 构建命令

```bash
# 一键打包（编译 .so + Gradle 打包 APK）
./scripts/package_android.sh

# 产物
# dist/android/OpenInfiniFactory.apk
```

### 分步执行

```bash
# 1. 编译 .so 到 jniLibs/
cargo ndk -t aarch64-linux-android -P 26 \
  -o mobile/android/app/src/main/jniLibs/arm64-v8a \
  build --release

# 2. Gradle 打包 APK
cd mobile/android
./gradlew assembleRelease --no-daemon
```

## 项目结构

```
mobile/android/                  # Android Gradle 项目
├── settings.gradle
├── gradle.properties
├── gradle/
│   ├── libs.versions.toml       # 依赖版本目录
│   └── wrapper/
├── gradlew / gradlew.bat
├── local.properties             # 本地 SDK 路径（gitignore）
└── app/
    ├── build.gradle             # APK 配置 + 签名
    ├── release.keystore         # 签名密钥（gitignore）
    └── src/main/
        ├── AndroidManifest.xml
        ├── java/.../MainActivity.java
        └── jniLibs/arm64-v8a/   # cargo-ndk 输出的 .so（gitignore）
```

## 签名

APK 使用 `mobile/android/app/release.keystore` 签名，密码 `openinfinifactory`。

生成新密钥：

```bash
keytool -genkeypair -v \
  -keystore mobile/android/app/release.keystore \
  -alias release -keyalg RSA -keysize 2048 \
  -validity 10000 \
  -storepass openinfinifactory -keypass openinfinifactory \
  -dname "CN=OpenInfiniFactory, O=OpenInfiniFactory, C=CN"
```

## 安装到设备

```bash
adb install dist/android/OpenInfiniFactory.apk

# 查看日志
adb logcat | grep 'RustStdoutStderr\|bevy\|wgpu'
```

## 常见问题

**`unable to find library -laaudio`** — minSdkVersion 低于 26，需在 `app/build.gradle` 中设置 `minSdk 26`。

**`Android SDK has no platforms installed`** — 通过 `sdkmanager` 安装对应 platform：`sdkmanager "platforms;android-30"`。

**`Did not expect a [workspace]`** — 使用了旧的 cargo-apk，本项目已迁移到 cargo-ndk，不会出现此问题。
