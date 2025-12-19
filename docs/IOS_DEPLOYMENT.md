# iOS Deployment Guide for Dioxus Apps

This guide documents how to deploy a Dioxus Rust app to iOS devices using Xcode.

## Prerequisites

1. **macOS** with Xcode installed
2. **Rust** with iOS targets:
   ```bash
   rustup target add aarch64-apple-ios        # For real devices
   rustup target add aarch64-apple-ios-sim    # For simulator
   ```
3. **Dioxus CLI**:
   ```bash
   cargo install dioxus-cli
   ```
4. **Apple Developer Account** (free or paid) for device deployment

## Project Requirements

Your `Cargo.toml` should have:
```toml
[features]
default = ["mobile"]
mobile = ["dioxus/mobile"]

[package.metadata.bundle]
identifier = "com.yourcompany.yourapp"
```

Your `Dioxus.toml` should have:
```toml
[application]
asset_dir = "./assets"

[bundle]
publisher = "Your Company"
identifier = "com.yourcompany.yourapp"
icon = ["./assets/icon.png"]
```

## The Problem with `dx bundle --platform ios`

The Dioxus CLI's `dx bundle --platform ios` command has a limitation: it always builds for the **iOS Simulator** target (`aarch64-apple-ios-sim`), not for real devices (`aarch64-apple-ios`). The `--target` flag is ignored.

## The Solution

We create an Xcode project that:
1. Uses `cargo build --target aarch64-apple-ios` for device builds
2. Uses `dx bundle` only for asset processing (CSS hashing, etc.)
3. Copies the correct binary and processed assets to the app bundle
4. Handles code signing through Xcode

## Setup Steps

### 1. Generate the iOS Xcode Project

Run the setup script from your project root:
```bash
./scripts/setup-ios.sh
```

This creates:
```
ios/
├── Blackbird.xcodeproj/
│   └── project.pbxproj
└── Blackbird/
    ├── Info.plist
    ├── LaunchScreen.storyboard
    └── Assets.xcassets/
        ├── AppIcon.appiconset/
        └── AccentColor.colorset/
```

### 2. Open in Xcode

```bash
open ios/YourApp.xcodeproj
```

### 3. Configure Signing

1. Select your project in the navigator
2. Go to **Signing & Capabilities**
3. Check **"Automatically manage signing"**
4. Select your **Team** (Apple ID)

### 4. Build and Run

1. Connect your iPhone via USB
2. Select your device from the dropdown
3. Click **Run** (Cmd+R)

First run may require trusting the developer on your iPhone:
**Settings → General → VPN & Device Management → [Your Apple ID] → Trust**

## How the Build Works

The Xcode project has two custom build phases:

### Build Phase 1: "Build Rust"
```bash
# Determine target based on platform
if [ "$PLATFORM_NAME" = "iphonesimulator" ]; then
    RUST_TARGET="aarch64-apple-ios-sim"
else
    RUST_TARGET="aarch64-apple-ios"
fi

# Build with cargo (correct platform)
cargo build --target $RUST_TARGET --release --features mobile

# Run dx bundle for processed assets only
dx bundle --platform ios --release

# Copy correct binary from cargo
cp "target/$RUST_TARGET/release/yourapp" "$BUILT_PRODUCTS_DIR/$PRODUCT_NAME.app/yourapp"

# Copy processed assets from dx
cp -R "target/dx/yourapp/release/ios/YourApp.app/assets" "$BUILT_PRODUCTS_DIR/$PRODUCT_NAME.app/"
```

### Build Phase 2: "Sign Executable"
Re-signs the app bundle with the correct entitlements after our binary is copied.

## Troubleshooting

### "Library not loaded: Security.framework (wrong platform)"
The binary was built for simulator. Ensure the build script uses `cargo build --target aarch64-apple-ios` for device builds.

### "ad-hoc signed" or signature errors
The Sign Executable build phase isn't running. Check that it exists and runs after Resources.

### WebView shows raw text / UI doesn't render
Assets aren't being loaded. Ensure the `assets/` folder is copied to the app bundle and contains processed files from `dx bundle`.

### "Could not create sandbox extension"
This is usually harmless. If the app works, ignore it.

## Key Insights

1. **`dx bundle` always targets simulator** - We must use `cargo build` with explicit `--target` for device builds

2. **Dioxus mobile uses WKWebView** - The UI is rendered in a WebView, so assets must be accessible

3. **Asset hashing** - `dx bundle` creates hashed versions of assets (e.g., `style-abc123.css`). The binary expects these hashed names.

4. **Code signing order matters** - The binary must be signed after it's copied to the bundle, and the whole bundle must be re-sealed

## Files Reference

| File | Purpose |
|------|---------|
| `ios/YourApp.xcodeproj/project.pbxproj` | Xcode project configuration |
| `ios/YourApp/Info.plist` | iOS app metadata |
| `ios/YourApp/LaunchScreen.storyboard` | Launch screen |
| `ios/YourApp/Assets.xcassets/` | App icons and colors |
| `scripts/setup-ios.sh` | Regenerates the iOS project |
