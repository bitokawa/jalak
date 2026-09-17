#!/bin/zsh
set -euo pipefail

script_dir=${0:A:h}
repo_root=${script_dir:h}
bundle="$repo_root/target/Jalak.app"
icon_work="$repo_root/target/jalak-icon"
iconset="$repo_root/target/AppIcon.iconset"

cd "$repo_root"
cargo build --workspace --release

rm -rf "$bundle" "$icon_work" "$iconset"
mkdir -p "$bundle/Contents/MacOS" "$bundle/Contents/Resources" "$icon_work" "$iconset"
cp "$repo_root/target/release/jalak-desktop" "$bundle/Contents/MacOS/jalak-desktop"
cp "$repo_root/apps/desktop/resources/Info.plist" "$bundle/Contents/Info.plist"
printf 'APPL????' > "$bundle/Contents/PkgInfo"

qlmanage -t -s 1024 -o "$icon_work" "$repo_root/apps/desktop/resources/AppIcon.svg" >/dev/null 2>&1
icon_source="$icon_work/AppIcon.svg.png"
sips -z 16 16 "$icon_source" --out "$iconset/icon_16x16.png" >/dev/null
sips -z 32 32 "$icon_source" --out "$iconset/icon_16x16@2x.png" >/dev/null
sips -z 32 32 "$icon_source" --out "$iconset/icon_32x32.png" >/dev/null
sips -z 64 64 "$icon_source" --out "$iconset/icon_32x32@2x.png" >/dev/null
sips -z 128 128 "$icon_source" --out "$iconset/icon_128x128.png" >/dev/null
sips -z 256 256 "$icon_source" --out "$iconset/icon_128x128@2x.png" >/dev/null
sips -z 256 256 "$icon_source" --out "$iconset/icon_256x256.png" >/dev/null
sips -z 512 512 "$icon_source" --out "$iconset/icon_256x256@2x.png" >/dev/null
sips -z 512 512 "$icon_source" --out "$iconset/icon_512x512.png" >/dev/null
cp "$icon_source" "$iconset/icon_512x512@2x.png"
iconutil -c icns "$iconset" -o "$bundle/Contents/Resources/AppIcon.icns"

codesign --force --deep --sign - "$bundle"
plutil -lint "$bundle/Contents/Info.plist"
codesign --verify --deep --strict "$bundle"
print "Built $bundle"
