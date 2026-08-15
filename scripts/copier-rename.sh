#!/bin/sh
# Rename the freshly-copied template from "charpente" to the project's slug.
# Runs as a copier task (cwd = the generated project), and again on every
# `copier update` — idempotent because upstream files always say "charpente".
set -eu

name="${1:?usage: copier-rename.sh <project-slug> <github-owner>}"
owner="${2:?usage: copier-rename.sh <project-slug> <github-owner>}"

# Content replacement, skipping VCS/build dirs and binaries.
# .copier-answers.yml stays untouched: its _src_path must keep pointing at the
# template repo or `copier update` breaks.
find . \
  -path ./.git -prune -o \
  -path ./node_modules -prune -o \
  -path ./target -prune -o \
  -name ".copier-answers.yml" -prune -o \
  -type f \( \
    -name "*.rs" -o -name "*.toml" -o -name "*.ts" -o -name "*.vue" -o \
    -name "*.json" -o -name "*.yml" -o -name "*.yaml" -o -name "*.md" -o \
    -name "*.sql" -o -name "*.css" -o -name "*.html" -o -name "*.mjs" -o \
    -name "*.sh" -o -name "justfile" -o -name "Caddyfile" -o -name "*.Dockerfile" -o \
    -name ".env.example" -o -name "*.svg" -o -name "*.snap" \
  \) -print | while IFS= read -r file; do
    sed -i.bak \
      -e "s/AloisH\\/charpente/${owner}\\/${name}/g" \
      -e "s/charpente/${name}/g" \
      "$file" && rm -f "${file}.bak"
done

echo "renamed template to '${name}' (owner: ${owner})"
