#!/bin/bash
#
# setup-ios.sh - Generate iOS Xcode project for Dioxus apps
#
# Usage: ./scripts/setup-ios.sh
#
# This script creates an Xcode project that properly builds Dioxus apps
# for real iOS devices (not just simulator).
#

set -e

# Configuration - modify these for your app
APP_NAME="Blackbird"
APP_NAME_LOWER="blackbird"
TEAM_ID='""'  # Leave empty quotes for automatic, or set your Team ID like 'ABCD1234'
MIN_IOS_VERSION="15.0"
APP_VERSION="0.1.0"

# Paths
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
IOS_DIR="$PROJECT_ROOT/ios"

# Load bundle ID from .env if available, otherwise use default
if [ -f "$PROJECT_ROOT/.env" ]; then
    source "$PROJECT_ROOT/.env" 2>/dev/null || true
fi
BUNDLE_ID="${IOS_BUNDLE_ID:-com.example.blackbird}"

echo "Setting up iOS Xcode project for $APP_NAME..."
echo "Project root: $PROJECT_ROOT"

# Check prerequisites
if ! command -v rustup &> /dev/null; then
    echo "Error: rustup not found. Please install Rust."
    exit 1
fi

if ! command -v dx &> /dev/null; then
    echo "Error: dx (Dioxus CLI) not found. Install with: cargo install dioxus-cli"
    exit 1
fi

if ! xcode-select -p &> /dev/null; then
    echo "Error: Xcode not found. Please install Xcode."
    exit 1
fi

# Ensure iOS targets are installed
echo "Checking Rust iOS targets..."
rustup target add aarch64-apple-ios 2>/dev/null || true
rustup target add aarch64-apple-ios-sim 2>/dev/null || true

# Clean existing iOS directory
if [ -d "$IOS_DIR" ]; then
    echo "Removing existing iOS directory..."
    rm -rf "$IOS_DIR"
fi

# Create directory structure
echo "Creating directory structure..."
mkdir -p "$IOS_DIR/$APP_NAME.xcodeproj"
mkdir -p "$IOS_DIR/$APP_NAME/Assets.xcassets/AppIcon.appiconset"
mkdir -p "$IOS_DIR/$APP_NAME/Assets.xcassets/AccentColor.colorset"

# Create project.pbxproj
echo "Creating Xcode project..."
cat > "$IOS_DIR/$APP_NAME.xcodeproj/project.pbxproj" << 'PBXPROJ_EOF'
// !$*UTF8*$!
{
	archiveVersion = 1;
	classes = {
	};
	objectVersion = 56;
	objects = {

/* Begin PBXBuildFile section */
		E91234570001 /* Assets.xcassets in Resources */ = {isa = PBXBuildFile; fileRef = E91234540001 /* Assets.xcassets */; };
		E91234580001 /* LaunchScreen.storyboard in Resources */ = {isa = PBXBuildFile; fileRef = E91234530001 /* LaunchScreen.storyboard */; };
/* End PBXBuildFile section */

/* Begin PBXFileReference section */
		E91234500001 /* __APP_NAME__.app */ = {isa = PBXFileReference; explicitFileType = wrapper.application; includeInIndex = 0; path = __APP_NAME__.app; sourceTree = BUILT_PRODUCTS_DIR; };
		E91234510001 /* Info.plist */ = {isa = PBXFileReference; lastKnownFileType = text.plist.xml; path = Info.plist; sourceTree = "<group>"; };
		E91234530001 /* LaunchScreen.storyboard */ = {isa = PBXFileReference; lastKnownFileType = file.storyboard; path = LaunchScreen.storyboard; sourceTree = "<group>"; };
		E91234540001 /* Assets.xcassets */ = {isa = PBXFileReference; lastKnownFileType = folder.assetcatalog; path = Assets.xcassets; sourceTree = "<group>"; };
/* End PBXFileReference section */

/* Begin PBXFrameworksBuildPhase section */
		E9123460001 /* Frameworks */ = {
			isa = PBXFrameworksBuildPhase;
			buildActionMask = 2147483647;
			files = (
			);
			runOnlyForDeploymentPostprocessing = 0;
		};
/* End PBXFrameworksBuildPhase section */

/* Begin PBXGroup section */
		E91234610001 = {
			isa = PBXGroup;
			children = (
				E91234620001 /* __APP_NAME__ */,
				E91234630001 /* Products */,
			);
			sourceTree = "<group>";
		};
		E91234620001 /* __APP_NAME__ */ = {
			isa = PBXGroup;
			children = (
				E91234540001 /* Assets.xcassets */,
				E91234530001 /* LaunchScreen.storyboard */,
				E91234510001 /* Info.plist */,
			);
			path = __APP_NAME__;
			sourceTree = "<group>";
		};
		E91234630001 /* Products */ = {
			isa = PBXGroup;
			children = (
				E91234500001 /* __APP_NAME__.app */,
			);
			name = Products;
			sourceTree = "<group>";
		};
/* End PBXGroup section */

/* Begin PBXNativeTarget section */
		E91234640001 /* __APP_NAME__ */ = {
			isa = PBXNativeTarget;
			buildConfigurationList = E91234650001 /* Build configuration list for PBXNativeTarget "__APP_NAME__" */;
			buildPhases = (
				E91234660001 /* Build Rust */,
				E9123460001 /* Frameworks */,
				E91234670001 /* Resources */,
				E912346E0001 /* Sign Executable */,
			);
			buildRules = (
			);
			dependencies = (
			);
			name = __APP_NAME__;
			productName = __APP_NAME__;
			productReference = E91234500001 /* __APP_NAME__.app */;
			productType = "com.apple.product-type.application";
		};
/* End PBXNativeTarget section */

/* Begin PBXProject section */
		E91234680001 /* Project object */ = {
			isa = PBXProject;
			attributes = {
				BuildIndependentTargetsInParallel = 1;
				LastUpgradeCheck = 1500;
				TargetAttributes = {
					E91234640001 = {
						CreatedOnToolsVersion = 15.0;
					};
				};
			};
			buildConfigurationList = E91234690001 /* Build configuration list for PBXProject "__APP_NAME__" */;
			compatibilityVersion = "Xcode 14.0";
			developmentRegion = en;
			hasScannedForEncodings = 0;
			knownRegions = (
				en,
				Base,
			);
			mainGroup = E91234610001;
			productRefGroup = E91234630001 /* Products */;
			projectDirPath = "";
			projectRoot = "";
			targets = (
				E91234640001 /* __APP_NAME__ */,
			);
		};
/* End PBXProject section */

/* Begin PBXResourcesBuildPhase section */
		E91234670001 /* Resources */ = {
			isa = PBXResourcesBuildPhase;
			buildActionMask = 2147483647;
			files = (
				E91234570001 /* Assets.xcassets in Resources */,
				E91234580001 /* LaunchScreen.storyboard in Resources */,
			);
			runOnlyForDeploymentPostprocessing = 0;
		};
/* End PBXResourcesBuildPhase section */

/* Begin PBXShellScriptBuildPhase section */
		E91234660001 /* Build Rust */ = {
			isa = PBXShellScriptBuildPhase;
			alwaysOutOfDate = 1;
			buildActionMask = 2147483647;
			files = (
			);
			inputFileListPaths = (
			);
			inputPaths = (
			);
			name = "Build Rust";
			outputFileListPaths = (
			);
			outputPaths = (
			);
			runOnlyForDeploymentPostprocessing = 0;
			shellPath = /bin/bash;
			shellScript = "set -e\n\nPROJECT_ROOT=\"$SRCROOT/..\"\ncd \"$PROJECT_ROOT\"\n\necho \"Building from: $(pwd)\"\necho \"PLATFORM_NAME: $PLATFORM_NAME\"\n\nif [ \"$PLATFORM_NAME\" = \"iphonesimulator\" ]; then\n    RUST_TARGET=\"aarch64-apple-ios-sim\"\nelse\n    RUST_TARGET=\"aarch64-apple-ios\"\nfi\n\nif [ \"$CONFIGURATION\" = \"Release\" ]; then\n    PROFILE=\"release\"\n    RELEASE_FLAG=\"--release\"\nelse\n    PROFILE=\"release\"\n    RELEASE_FLAG=\"--release\"\nfi\n\nexport PATH=\"$HOME/.cargo/bin:$PATH\"\n\necho \"Building with cargo for target: $RUST_TARGET ($PROFILE)...\"\n\n# Build with cargo for correct platform\ncargo build --target $RUST_TARGET $RELEASE_FLAG --features mobile\n\n# Also run dx bundle to get processed assets (it builds for wrong target but we just need assets)\ndx bundle --platform ios $RELEASE_FLAG 2>/dev/null || true\n\nDX_APP=\"$PROJECT_ROOT/target/dx/__APP_NAME_LOWER__/$PROFILE/ios/__APP_NAME__.app\"\nCARGO_BIN=\"$PROJECT_ROOT/target/$RUST_TARGET/$PROFILE/__APP_NAME_LOWER__\"\n\necho \"Using binary from: $CARGO_BIN\"\necho \"Using assets from: $DX_APP/assets\"\n\n# Create the app bundle directory\nmkdir -p \"$BUILT_PRODUCTS_DIR/$PRODUCT_NAME.app\"\n\n# Copy the cargo-built binary (correct platform)\ncp \"$CARGO_BIN\" \"$BUILT_PRODUCTS_DIR/$PRODUCT_NAME.app/__APP_NAME_LOWER__\"\n\n# Copy the assets folder from dx (processed assets)\nif [ -d \"$DX_APP/assets\" ]; then\n    rm -rf \"$BUILT_PRODUCTS_DIR/$PRODUCT_NAME.app/assets\"\n    cp -R \"$DX_APP/assets\" \"$BUILT_PRODUCTS_DIR/$PRODUCT_NAME.app/\"\nelse\n    # Fallback to raw assets\n    mkdir -p \"$BUILT_PRODUCTS_DIR/$PRODUCT_NAME.app/assets\"\n    cp -R \"$PROJECT_ROOT/assets/\"* \"$BUILT_PRODUCTS_DIR/$PRODUCT_NAME.app/assets/\"\nfi\n\necho \"Build complete!\"\n";
		};
		E912346E0001 /* Sign Executable */ = {
			isa = PBXShellScriptBuildPhase;
			alwaysOutOfDate = 1;
			buildActionMask = 2147483647;
			files = (
			);
			inputFileListPaths = (
			);
			inputPaths = (
			);
			name = "Sign Executable";
			outputFileListPaths = (
			);
			outputPaths = (
			);
			runOnlyForDeploymentPostprocessing = 0;
			shellPath = /bin/bash;
			shellScript = "set -e\n\necho \"Re-signing entire app bundle with identity: $EXPANDED_CODE_SIGN_IDENTITY\"\n\nAPP_BUNDLE=\"$BUILT_PRODUCTS_DIR/$PRODUCT_NAME.app\"\nENTITLEMENTS=\"$TARGET_TEMP_DIR/$PRODUCT_NAME.app.xcent\"\n\nif [ -d \"$APP_BUNDLE\" ]; then\n    if [ -f \"$ENTITLEMENTS\" ]; then\n        /usr/bin/codesign --force --sign \"$EXPANDED_CODE_SIGN_IDENTITY\" --entitlements \"$ENTITLEMENTS\" --timestamp=none \"$APP_BUNDLE\"\n    else\n        /usr/bin/codesign --force --sign \"$EXPANDED_CODE_SIGN_IDENTITY\" --timestamp=none \"$APP_BUNDLE\"\n    fi\n    echo \"Successfully signed app bundle\"\nelse\n    echo \"ERROR: App bundle not found at $APP_BUNDLE\"\n    exit 1\nfi\n";
		};
/* End PBXShellScriptBuildPhase section */

/* Begin XCBuildConfiguration section */
		E912346A0001 /* Debug */ = {
			isa = XCBuildConfiguration;
			buildSettings = {
				ALWAYS_SEARCH_USER_PATHS = NO;
				ASSETCATALOG_COMPILER_GENERATE_SWIFT_ASSET_SYMBOL_EXTENSIONS = YES;
				CLANG_ANALYZER_NONNULL = YES;
				CLANG_ANALYZER_NUMBER_OBJECT_CONVERSION = YES_AGGRESSIVE;
				CLANG_CXX_LANGUAGE_STANDARD = "gnu++20";
				CLANG_ENABLE_MODULES = YES;
				CLANG_ENABLE_OBJC_ARC = YES;
				CLANG_ENABLE_OBJC_WEAK = YES;
				CLANG_WARN_BLOCK_CAPTURE_AUTORELEASING = YES;
				CLANG_WARN_BOOL_CONVERSION = YES;
				CLANG_WARN_COMMA = YES;
				CLANG_WARN_CONSTANT_CONVERSION = YES;
				CLANG_WARN_DEPRECATED_OBJC_IMPLEMENTATIONS = YES;
				CLANG_WARN_DIRECT_OBJC_ISA_USAGE = YES_ERROR;
				CLANG_WARN_DOCUMENTATION_COMMENTS = YES;
				CLANG_WARN_EMPTY_BODY = YES;
				CLANG_WARN_ENUM_CONVERSION = YES;
				CLANG_WARN_INFINITE_RECURSION = YES;
				CLANG_WARN_INT_CONVERSION = YES;
				CLANG_WARN_NON_LITERAL_NULL_CONVERSION = YES;
				CLANG_WARN_OBJC_IMPLICIT_RETAIN_SELF = YES;
				CLANG_WARN_OBJC_LITERAL_CONVERSION = YES;
				CLANG_WARN_OBJC_ROOT_CLASS = YES_ERROR;
				CLANG_WARN_QUOTED_INCLUDE_IN_FRAMEWORK_HEADER = YES;
				CLANG_WARN_RANGE_LOOP_ANALYSIS = YES;
				CLANG_WARN_STRICT_PROTOTYPES = YES;
				CLANG_WARN_SUSPICIOUS_MOVE = YES;
				CLANG_WARN_UNGUARDED_AVAILABILITY = YES_AGGRESSIVE;
				CLANG_WARN_UNREACHABLE_CODE = YES;
				CLANG_WARN__DUPLICATE_METHOD_MATCH = YES;
				COPY_PHASE_STRIP = NO;
				DEBUG_INFORMATION_FORMAT = dwarf;
				DEVELOPMENT_TEAM = __TEAM_ID__;
				ENABLE_STRICT_OBJC_MSGSEND = YES;
				ENABLE_TESTABILITY = YES;
				ENABLE_USER_SCRIPT_SANDBOXING = NO;
				GCC_C_LANGUAGE_STANDARD = gnu17;
				GCC_DYNAMIC_NO_PIC = NO;
				GCC_NO_COMMON_BLOCKS = YES;
				GCC_OPTIMIZATION_LEVEL = 0;
				GCC_PREPROCESSOR_DEFINITIONS = (
					"DEBUG=1",
					"$(inherited)",
				);
				GCC_WARN_64_TO_32_BIT_CONVERSION = YES;
				GCC_WARN_ABOUT_RETURN_TYPE = YES_ERROR;
				GCC_WARN_UNDECLARED_SELECTOR = YES;
				GCC_WARN_UNINITIALIZED_AUTOS = YES_AGGRESSIVE;
				GCC_WARN_UNUSED_FUNCTION = YES;
				GCC_WARN_UNUSED_VARIABLE = YES;
				IPHONEOS_DEPLOYMENT_TARGET = __MIN_IOS__;
				LOCALIZATION_PREFERS_STRING_CATALOGS = YES;
				MTL_ENABLE_DEBUG_INFO = INCLUDE_SOURCE;
				MTL_FAST_MATH = YES;
				ONLY_ACTIVE_ARCH = YES;
				SDKROOT = iphoneos;
			};
			name = Debug;
		};
		E912346B0001 /* Release */ = {
			isa = XCBuildConfiguration;
			buildSettings = {
				ALWAYS_SEARCH_USER_PATHS = NO;
				ASSETCATALOG_COMPILER_GENERATE_SWIFT_ASSET_SYMBOL_EXTENSIONS = YES;
				CLANG_ANALYZER_NONNULL = YES;
				CLANG_ANALYZER_NUMBER_OBJECT_CONVERSION = YES_AGGRESSIVE;
				CLANG_CXX_LANGUAGE_STANDARD = "gnu++20";
				CLANG_ENABLE_MODULES = YES;
				CLANG_ENABLE_OBJC_ARC = YES;
				CLANG_ENABLE_OBJC_WEAK = YES;
				CLANG_WARN_BLOCK_CAPTURE_AUTORELEASING = YES;
				CLANG_WARN_BOOL_CONVERSION = YES;
				CLANG_WARN_COMMA = YES;
				CLANG_WARN_CONSTANT_CONVERSION = YES;
				CLANG_WARN_DEPRECATED_OBJC_IMPLEMENTATIONS = YES;
				CLANG_WARN_DIRECT_OBJC_ISA_USAGE = YES_ERROR;
				CLANG_WARN_DOCUMENTATION_COMMENTS = YES;
				CLANG_WARN_EMPTY_BODY = YES;
				CLANG_WARN_ENUM_CONVERSION = YES;
				CLANG_WARN_INFINITE_RECURSION = YES;
				CLANG_WARN_INT_CONVERSION = YES;
				CLANG_WARN_NON_LITERAL_NULL_CONVERSION = YES;
				CLANG_WARN_OBJC_IMPLICIT_RETAIN_SELF = YES;
				CLANG_WARN_OBJC_LITERAL_CONVERSION = YES;
				CLANG_WARN_OBJC_ROOT_CLASS = YES_ERROR;
				CLANG_WARN_QUOTED_INCLUDE_IN_FRAMEWORK_HEADER = YES;
				CLANG_WARN_RANGE_LOOP_ANALYSIS = YES;
				CLANG_WARN_STRICT_PROTOTYPES = YES;
				CLANG_WARN_SUSPICIOUS_MOVE = YES;
				CLANG_WARN_UNGUARDED_AVAILABILITY = YES_AGGRESSIVE;
				CLANG_WARN_UNREACHABLE_CODE = YES;
				CLANG_WARN__DUPLICATE_METHOD_MATCH = YES;
				COPY_PHASE_STRIP = NO;
				DEBUG_INFORMATION_FORMAT = "dwarf-with-dsym";
				DEVELOPMENT_TEAM = __TEAM_ID__;
				ENABLE_NS_ASSERTIONS = NO;
				ENABLE_STRICT_OBJC_MSGSEND = YES;
				ENABLE_USER_SCRIPT_SANDBOXING = NO;
				GCC_C_LANGUAGE_STANDARD = gnu17;
				GCC_NO_COMMON_BLOCKS = YES;
				GCC_WARN_64_TO_32_BIT_CONVERSION = YES;
				GCC_WARN_ABOUT_RETURN_TYPE = YES_ERROR;
				GCC_WARN_UNDECLARED_SELECTOR = YES;
				GCC_WARN_UNINITIALIZED_AUTOS = YES_AGGRESSIVE;
				GCC_WARN_UNUSED_FUNCTION = YES;
				GCC_WARN_UNUSED_VARIABLE = YES;
				IPHONEOS_DEPLOYMENT_TARGET = __MIN_IOS__;
				LOCALIZATION_PREFERS_STRING_CATALOGS = YES;
				MTL_ENABLE_DEBUG_INFO = NO;
				MTL_FAST_MATH = YES;
				SDKROOT = iphoneos;
				VALIDATE_PRODUCT = YES;
			};
			name = Release;
		};
		E912346C0001 /* Debug */ = {
			isa = XCBuildConfiguration;
			buildSettings = {
				ASSETCATALOG_COMPILER_APPICON_NAME = AppIcon;
				ASSETCATALOG_COMPILER_GLOBAL_ACCENT_COLOR_NAME = AccentColor;
				CODE_SIGN_STYLE = Automatic;
				CURRENT_PROJECT_VERSION = 1;
				GENERATE_INFOPLIST_FILE = NO;
				INFOPLIST_FILE = __APP_NAME__/Info.plist;
				INFOPLIST_KEY_CFBundleDisplayName = __APP_NAME__;
				INFOPLIST_KEY_LSApplicationCategoryType = "public.app-category.productivity";
				INFOPLIST_KEY_UIApplicationSupportsIndirectInputEvents = YES;
				INFOPLIST_KEY_UILaunchStoryboardName = LaunchScreen;
				INFOPLIST_KEY_UIRequiresFullScreen = YES;
				INFOPLIST_KEY_UISupportedInterfaceOrientations = "UIInterfaceOrientationPortrait UIInterfaceOrientationPortraitUpsideDown UIInterfaceOrientationLandscapeLeft UIInterfaceOrientationLandscapeRight";
				LD_RUNPATH_SEARCH_PATHS = (
					"$(inherited)",
					"@executable_path/Frameworks",
				);
				MARKETING_VERSION = __APP_VERSION__;
				PRODUCT_BUNDLE_IDENTIFIER = __BUNDLE_ID__;
				PRODUCT_NAME = "$(TARGET_NAME)";
				SUPPORTED_PLATFORMS = "iphoneos iphonesimulator";
				SUPPORTS_MACCATALYST = NO;
				SWIFT_EMIT_LOC_STRINGS = YES;
				TARGETED_DEVICE_FAMILY = "1,2";
			};
			name = Debug;
		};
		E912346D0001 /* Release */ = {
			isa = XCBuildConfiguration;
			buildSettings = {
				ASSETCATALOG_COMPILER_APPICON_NAME = AppIcon;
				ASSETCATALOG_COMPILER_GLOBAL_ACCENT_COLOR_NAME = AccentColor;
				CODE_SIGN_STYLE = Automatic;
				CURRENT_PROJECT_VERSION = 1;
				GENERATE_INFOPLIST_FILE = NO;
				INFOPLIST_FILE = __APP_NAME__/Info.plist;
				INFOPLIST_KEY_CFBundleDisplayName = __APP_NAME__;
				INFOPLIST_KEY_LSApplicationCategoryType = "public.app-category.productivity";
				INFOPLIST_KEY_UIApplicationSupportsIndirectInputEvents = YES;
				INFOPLIST_KEY_UILaunchStoryboardName = LaunchScreen;
				INFOPLIST_KEY_UIRequiresFullScreen = YES;
				INFOPLIST_KEY_UISupportedInterfaceOrientations = "UIInterfaceOrientationPortrait UIInterfaceOrientationPortraitUpsideDown UIInterfaceOrientationLandscapeLeft UIInterfaceOrientationLandscapeRight";
				LD_RUNPATH_SEARCH_PATHS = (
					"$(inherited)",
					"@executable_path/Frameworks",
				);
				MARKETING_VERSION = __APP_VERSION__;
				PRODUCT_BUNDLE_IDENTIFIER = __BUNDLE_ID__;
				PRODUCT_NAME = "$(TARGET_NAME)";
				SUPPORTED_PLATFORMS = "iphoneos iphonesimulator";
				SUPPORTS_MACCATALYST = NO;
				SWIFT_EMIT_LOC_STRINGS = YES;
				TARGETED_DEVICE_FAMILY = "1,2";
			};
			name = Release;
		};
/* End XCBuildConfiguration section */

/* Begin XCConfigurationList section */
		E91234650001 /* Build configuration list for PBXNativeTarget "__APP_NAME__" */ = {
			isa = XCConfigurationList;
			buildConfigurations = (
				E912346C0001 /* Debug */,
				E912346D0001 /* Release */,
			);
			defaultConfigurationIsVisible = 0;
			defaultConfigurationName = Release;
		};
		E91234690001 /* Build configuration list for PBXProject "__APP_NAME__" */ = {
			isa = XCConfigurationList;
			buildConfigurations = (
				E912346A0001 /* Debug */,
				E912346B0001 /* Release */,
			);
			defaultConfigurationIsVisible = 0;
			defaultConfigurationName = Release;
		};
/* End XCConfigurationList section */
	};
	rootObject = E91234680001 /* Project object */;
}
PBXPROJ_EOF

# Replace placeholders in project.pbxproj
sed -i '' "s/__APP_NAME__/$APP_NAME/g" "$IOS_DIR/$APP_NAME.xcodeproj/project.pbxproj"
sed -i '' "s/__APP_NAME_LOWER__/$APP_NAME_LOWER/g" "$IOS_DIR/$APP_NAME.xcodeproj/project.pbxproj"
sed -i '' "s/__BUNDLE_ID__/$BUNDLE_ID/g" "$IOS_DIR/$APP_NAME.xcodeproj/project.pbxproj"
sed -i '' "s/__TEAM_ID__/$TEAM_ID/g" "$IOS_DIR/$APP_NAME.xcodeproj/project.pbxproj"
sed -i '' "s/__MIN_IOS__/$MIN_IOS_VERSION/g" "$IOS_DIR/$APP_NAME.xcodeproj/project.pbxproj"
sed -i '' "s/__APP_VERSION__/$APP_VERSION/g" "$IOS_DIR/$APP_NAME.xcodeproj/project.pbxproj"

# Create Info.plist
echo "Creating Info.plist..."
cat > "$IOS_DIR/$APP_NAME/Info.plist" << PLIST_EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
	<key>CFBundleDevelopmentRegion</key>
	<string>\$(DEVELOPMENT_LANGUAGE)</string>
	<key>CFBundleDisplayName</key>
	<string>$APP_NAME</string>
	<key>CFBundleExecutable</key>
	<string>$APP_NAME_LOWER</string>
	<key>CFBundleIdentifier</key>
	<string>\$(PRODUCT_BUNDLE_IDENTIFIER)</string>
	<key>CFBundleInfoDictionaryVersion</key>
	<string>6.0</string>
	<key>CFBundleName</key>
	<string>\$(PRODUCT_NAME)</string>
	<key>CFBundlePackageType</key>
	<string>APPL</string>
	<key>CFBundleShortVersionString</key>
	<string>\$(MARKETING_VERSION)</string>
	<key>CFBundleVersion</key>
	<string>\$(CURRENT_PROJECT_VERSION)</string>
	<key>LSRequiresIPhoneOS</key>
	<true/>
	<key>UILaunchStoryboardName</key>
	<string>LaunchScreen</string>
	<key>UIRequiredDeviceCapabilities</key>
	<array>
		<string>arm64</string>
	</array>
	<key>UIRequiresFullScreen</key>
	<true/>
	<key>UISupportedInterfaceOrientations</key>
	<array>
		<string>UIInterfaceOrientationPortrait</string>
		<string>UIInterfaceOrientationLandscapeLeft</string>
		<string>UIInterfaceOrientationLandscapeRight</string>
		<string>UIInterfaceOrientationPortraitUpsideDown</string>
	</array>
	<key>UISupportedInterfaceOrientations~ipad</key>
	<array>
		<string>UIInterfaceOrientationPortrait</string>
		<string>UIInterfaceOrientationPortraitUpsideDown</string>
		<string>UIInterfaceOrientationLandscapeLeft</string>
		<string>UIInterfaceOrientationLandscapeRight</string>
	</array>
</dict>
</plist>
PLIST_EOF

# Create LaunchScreen.storyboard
echo "Creating LaunchScreen.storyboard..."
cat > "$IOS_DIR/$APP_NAME/LaunchScreen.storyboard" << 'STORYBOARD_EOF'
<?xml version="1.0" encoding="UTF-8"?>
<document type="com.apple.InterfaceBuilder3.CocoaTouch.Storyboard.XIB" version="3.0" toolsVersion="22505" targetRuntime="iOS.CocoaTouch" propertyAccessControl="none" useAutolayout="YES" launchScreen="YES" useTraitCollections="YES" useSafeAreas="YES" colorMatched="YES" initialViewController="01J-lp-oVM">
    <device id="retina6_12" orientation="portrait" appearance="dark"/>
    <dependencies>
        <plugIn identifier="com.apple.InterfaceBuilder.IBCocoaTouchPlugin" version="22504"/>
        <capability name="Safe area layout guides" minToolsVersion="9.0"/>
        <capability name="System colors in document resources" minToolsVersion="11.0"/>
        <capability name="documents saved in the Xcode 8 format" minToolsVersion="8.0"/>
    </dependencies>
    <scenes>
        <scene sceneID="EHf-IW-A2E">
            <objects>
                <viewController id="01J-lp-oVM" sceneMemberID="viewController">
                    <view key="view" contentMode="scaleToFill" id="Ze5-6b-2t3">
                        <rect key="frame" x="0.0" y="0.0" width="393" height="852"/>
                        <autoresizingMask key="autoresizingMask" widthSizable="YES" heightSizable="YES"/>
                        <subviews>
                            <label opaque="NO" userInteractionEnabled="NO" contentMode="left" horizontalHuggingPriority="251" verticalHuggingPriority="251" text="__APP_NAME__" textAlignment="center" lineBreakMode="tailTruncation" baselineAdjustment="alignBaselines" adjustsFontSizeToFit="NO" translatesAutoresizingMaskIntoConstraints="NO" id="title-label">
                                <rect key="frame" x="96.666666666666671" y="411" width="200" height="30"/>
                                <fontDescription key="fontDescription" type="boldSystem" pointSize="28"/>
                                <color key="textColor" white="1" alpha="1" colorSpace="custom" customColorSpace="genericGamma22GrayColorSpace"/>
                                <nil key="highlightedColor"/>
                            </label>
                        </subviews>
                        <viewLayoutGuide key="safeArea" id="6Tk-OE-BBY"/>
                        <color key="backgroundColor" red="0.07" green="0.07" blue="0.09" alpha="1" colorSpace="custom" customColorSpace="sRGB"/>
                        <constraints>
                            <constraint firstItem="title-label" firstAttribute="centerX" secondItem="Ze5-6b-2t3" secondAttribute="centerX" id="center-x"/>
                            <constraint firstItem="title-label" firstAttribute="centerY" secondItem="Ze5-6b-2t3" secondAttribute="centerY" id="center-y"/>
                        </constraints>
                    </view>
                </viewController>
                <placeholder placeholderIdentifier="IBFirstResponder" id="iYj-Kq-Ea1" userLabel="First Responder" sceneMemberID="firstResponder"/>
            </objects>
            <point key="canvasLocation" x="52" y="374"/>
        </scene>
    </scenes>
    <resources>
        <systemColor name="systemBackgroundColor">
            <color white="1" alpha="1" colorSpace="custom" customColorSpace="genericGamma22GrayColorSpace"/>
        </systemColor>
    </resources>
</document>
STORYBOARD_EOF

# Replace app name in storyboard
sed -i '' "s/__APP_NAME__/$APP_NAME/g" "$IOS_DIR/$APP_NAME/LaunchScreen.storyboard"

# Create Assets.xcassets/Contents.json
echo "Creating asset catalogs..."
cat > "$IOS_DIR/$APP_NAME/Assets.xcassets/Contents.json" << 'EOF'
{
  "info" : {
    "author" : "xcode",
    "version" : 1
  }
}
EOF

# Create AccentColor.colorset/Contents.json
cat > "$IOS_DIR/$APP_NAME/Assets.xcassets/AccentColor.colorset/Contents.json" << 'EOF'
{
  "colors" : [
    {
      "idiom" : "universal"
    }
  ],
  "info" : {
    "author" : "xcode",
    "version" : 1
  }
}
EOF

# Create AppIcon.appiconset/Contents.json
cat > "$IOS_DIR/$APP_NAME/Assets.xcassets/AppIcon.appiconset/Contents.json" << 'EOF'
{
  "images" : [
    {
      "filename" : "AppIcon.png",
      "idiom" : "universal",
      "platform" : "ios",
      "size" : "1024x1024"
    }
  ],
  "info" : {
    "author" : "xcode",
    "version" : 1
  }
}
EOF

# Copy app icon if it exists
if [ -f "$PROJECT_ROOT/assets/blackbird_logo_1024.png" ]; then
    echo "Copying app icon..."
    cp "$PROJECT_ROOT/assets/blackbird_logo_1024.png" "$IOS_DIR/$APP_NAME/Assets.xcassets/AppIcon.appiconset/AppIcon.png"
elif [ -f "$PROJECT_ROOT/assets/icon.png" ]; then
    cp "$PROJECT_ROOT/assets/icon.png" "$IOS_DIR/$APP_NAME/Assets.xcassets/AppIcon.appiconset/AppIcon.png"
else
    echo "Warning: No app icon found. Add a 1024x1024 PNG to Assets.xcassets/AppIcon.appiconset/AppIcon.png"
fi

echo ""
echo "iOS Xcode project created successfully!"
echo ""
echo "Next steps:"
echo "  1. Open the project:  open $IOS_DIR/$APP_NAME.xcodeproj"
echo "  2. Select your Team in Signing & Capabilities"
echo "  3. Connect your iPhone and select it as the run destination"
echo "  4. Click Run (Cmd+R)"
echo ""
echo "First run on device requires trusting the developer:"
echo "  Settings → General → VPN & Device Management → [Your Apple ID] → Trust"
