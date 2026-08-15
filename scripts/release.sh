#!/usr/bin/env bash
# just release <app|site> <major|minor|patch>
# Computes the next version from existing git tags, creates the tag, pushes it.
# The tag push triggers the matching release workflow (build + push to GHCR).
set -euo pipefail

target="${1:?usage: release.sh <app|site> <major|minor|patch>}"
bump="${2:?usage: release.sh <app|site> <major|minor|patch>}"

case "$target" in
  app) prefix="v" ;;
  site) prefix="site-v" ;;
  *) echo "target must be 'app' or 'site'" >&2; exit 1 ;;
esac

git fetch --tags --quiet

last=$(git tag --list "${prefix}[0-9]*" --sort=-v:refname | head -1)
if [ -z "$last" ]; then
  version="0.0.0"
else
  version="${last#"$prefix"}"
fi

IFS=. read -r major minor patch <<<"$version"
case "$bump" in
  major) major=$((major + 1)); minor=0; patch=0 ;;
  minor) minor=$((minor + 1)); patch=0 ;;
  patch) patch=$((patch + 1)) ;;
  *) echo "bump must be major, minor or patch" >&2; exit 1 ;;
esac

tag="${prefix}${major}.${minor}.${patch}"
echo "tagging ${tag} (previous: ${last:-none})"
git tag -a "$tag" -m "release ${tag}"
git push origin "$tag"
echo "pushed — the release workflow takes it from here"
