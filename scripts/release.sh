#!/bin/zsh
# Usage: scripts/release.sh <X.Y.Z> [--dry-run]
# Validates, runs checks, packages, then bumps the version, cuts the
# CHANGELOG "Unreleased" section, commits, tags vX.Y.Z and pushes.
# The Release workflow builds the universal app and publishes the GitHub release.
set -euo pipefail

version=${1:-}
dry_run=${2:-}
die() { print -u2 "error: $*"; exit 1 }

[[ $version =~ '^[0-9]+\.[0-9]+\.[0-9]+$' ]] || die "usage: scripts/release.sh <X.Y.Z> [--dry-run]"
[[ -z $dry_run || $dry_run == --dry-run ]] || die "unknown option: $dry_run"
tag="v$version"

cd "${0:A:h:h}"

[[ $(git branch --show-current) == main ]] || die "not on main"
[[ -z $(git status --porcelain) ]] || die "working tree not clean"
git fetch --quiet origin main --tags
[[ $(git rev-parse HEAD) == $(git rev-parse origin/main) ]] || die "main differs from origin/main"
git rev-parse -q --verify "refs/tags/$tag" >/dev/null && die "tag $tag already exists"

current=$(cargo pkgid -p jalak-desktop | sed 's/.*[#@]//')
[[ $(printf '%s\n' "$current" "$version" | sort -V | tail -1) == "$version" ]] \
  || die "$version is older than current $current"

notes=$(awk '/^## Unreleased/{f=1; next} /^## /{f=0} f' CHANGELOG.md)
[[ -n ${notes//[[:space:]]/} ]] || die "CHANGELOG.md has no Unreleased entries"

cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
./scripts/package-macos.sh

if [[ -n $dry_run ]]; then
  print "Dry run OK: would release $current -> $tag with notes:\n$notes"
  exit 0
fi

sed -i '' "s/^version = \"$current\"/version = \"$version\"/" apps/desktop/Cargo.toml
cargo update --workspace --offline --quiet
awk -v heading="## $version - $(date +%F)" \
  '!done && /^## Unreleased/{print; print ""; print heading; done=1; next} {print}' \
  CHANGELOG.md > CHANGELOG.md.tmp
mv CHANGELOG.md.tmp CHANGELOG.md

git add apps/desktop/Cargo.toml Cargo.lock CHANGELOG.md
git commit --quiet -m "chore: release $tag"
git tag -a "$tag" -m "Jalak $tag"
git push --atomic origin main "$tag"

print "Pushed $tag. Watch: gh run watch \$(gh run list --workflow release.yml -L1 --json databaseId -q '.[0].databaseId')"
