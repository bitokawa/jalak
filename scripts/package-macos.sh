#!/bin/zsh
set -euo pipefail

script_dir=${0:A:h}
repo_root=${script_dir:h}
bundle="$repo_root/target/Jalak.app"
iconset="$repo_root/target/AppIcon.iconset"
binary="$bundle/Contents/MacOS/jalak-desktop"

cd "$repo_root"
version=$(cargo pkgid -p jalak-desktop | sed 's/.*[#@]//')

# JALAK_UNIVERSAL=1 builds an arm64 + x86_64 binary (needs both rustup targets).
if [[ -n ${JALAK_UNIVERSAL:-} ]]; then
  cargo build --workspace --release --target aarch64-apple-darwin
  cargo build --workspace --release --target x86_64-apple-darwin
else
  cargo build --workspace --release
fi

rm -rf "$bundle" "$iconset"
mkdir -p "$bundle/Contents/MacOS" "$bundle/Contents/Resources" "$iconset"
if [[ -n ${JALAK_UNIVERSAL:-} ]]; then
  lipo -create -output "$binary" \
    "$repo_root/target/aarch64-apple-darwin/release/jalak-desktop" \
    "$repo_root/target/x86_64-apple-darwin/release/jalak-desktop"
else
  cp "$repo_root/target/release/jalak-desktop" "$binary"
fi
cp "$repo_root/apps/desktop/resources/Info.plist" "$bundle/Contents/Info.plist"
/usr/libexec/PlistBuddy -c "Set :CFBundleShortVersionString $version" "$bundle/Contents/Info.plist"
/usr/libexec/PlistBuddy -c "Set :CFBundleVersion $version" "$bundle/Contents/Info.plist"
printf 'APPL????' > "$bundle/Contents/PkgInfo"

icon_source="$repo_root/apps/desktop/resources/AppIcon.png"
sips -z 16 16 "$icon_source" --out "$iconset/icon_16x16.png" >/dev/null
sips -z 32 32 "$icon_source" --out "$iconset/icon_16x16@2x.png" >/dev/null
sips -z 32 32 "$icon_source" --out "$iconset/icon_32x32.png" >/dev/null
sips -z 64 64 "$icon_source" --out "$iconset/icon_32x32@2x.png" >/dev/null
sips -z 128 128 "$icon_source" --out "$iconset/icon_128x128.png" >/dev/null
sips -z 256 256 "$icon_source" --out "$iconset/icon_128x128@2x.png" >/dev/null
sips -z 256 256 "$icon_source" --out "$iconset/icon_256x256.png" >/dev/null
sips -z 512 512 "$icon_source" --out "$iconset/icon_256x256@2x.png" >/dev/null
sips -z 512 512 "$icon_source" --out "$iconset/icon_512x512.png" >/dev/null
sips -z 1024 1024 "$icon_source" --out "$iconset/icon_512x512@2x.png" >/dev/null
iconutil -c icns "$iconset" -o "$bundle/Contents/Resources/AppIcon.icns"

codesign --force --deep --sign - "$bundle"
plutil -lint "$bundle/Contents/Info.plist"
codesign --verify --deep --strict "$bundle"
print "Built $bundle ($version)"
