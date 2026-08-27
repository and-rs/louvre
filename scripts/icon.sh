#!/usr/bin/env bash
set -euo pipefail

usage() {
  echo "usage: $0 <phosphor-name> [--force]" >&2
  exit 1
}

name=""
force=false
for arg in "$@"; do
  case "$arg" in
  --force) force=true ;;
  -*) usage ;;
  *)
    if [[ -n "$name" ]]; then usage; fi
    name="$arg"
    ;;
  esac
done

if [[ -z "$name" ]]; then usage; fi
if [[ ! "$name" =~ ^[a-z0-9-]+$ ]]; then
  echo "error: invalid icon name '$name' (expected lowercase letters, digits, dashes)" >&2
  exit 1
fi

module="${name//-/_}"
icons_dir="louvre-site/src/templates/components/icons"
target="$icons_dir/$module.rs"
mod_file="$icons_dir/mod.rs"

if [[ -f "$target" && "$force" != true ]]; then
  echo "error: $target already exists (use --force to overwrite)" >&2
  exit 1
fi

url="https://raw.githubusercontent.com/phosphor-icons/core/main/assets/regular/$name.svg"
tmp="$(mktemp)"
trap 'rm -f "$tmp"' EXIT

if ! curl -fsSL "$url" -o "$tmp"; then
  echo "error: could not download $url" >&2
  exit 1
fi

head="$(tr -d '\n' <"$tmp" | sed -E 's/<path .*//')"

if ! grep -q 'viewBox="0 0 256 256"' <<<"$head"; then
  echo "error: unexpected SVG root (missing viewBox=\"0 0 256 256\"); upstream format may have changed" >&2
  exit 1
fi

if grep -Eq '<(circle|ellipse|rect|line|polyline|polygon|g|defs|mask|linearGradient|radialGradient)' "$tmp"; then
  echo "error: $name contains non-path elements; this script only supports path-only regular-weight icons" >&2
  exit 1
fi

mapfile -t paths < <(tr -d '\n' <"$tmp" | grep -o 'd="[^"]*"' | sed -E 's/^d="//; s/"$//')

if [[ "${#paths[@]}" -eq 0 ]]; then
  echo "error: no <path d=\"...\"> elements found in $name.svg" >&2
  exit 1
fi

mkdir -p "$icons_dir"

{
  echo "use maud::{Markup, html};"
  echo ""
  echo "pub fn $module(class: &str) -> Markup {"
  echo "    html! {"
  echo "        svg class=(format!(\"shrink-0 fill-current {class}\")) viewBox=\"0 0 256 256\" aria-hidden=\"true\" {"
  for d in "${paths[@]}"; do
    echo "            path d=\"$d\";"
  done
  echo "        }"
  echo "    }"
  echo "}"
} >"$target"

if [[ ! -f "$mod_file" ]]; then
  cat >"$mod_file" <<'EOF'
EOF
fi

if ! grep -q "^mod $module;$" "$mod_file"; then
  printf 'mod %s;\n' "$module" >>"$mod_file"
fi
if ! grep -q "^pub use $module::$module;$" "$mod_file"; then
  printf 'pub use %s::%s;\n' "$module" "$module" >>"$mod_file"
fi

components_mod="louvre-site/src/templates/components/mod.rs"
if [[ ! -f "$components_mod" ]] || ! grep -q "^mod icons;$" "$components_mod"; then
  printf 'mod icons;\n' >>"$components_mod"
fi
if ! grep -q "^pub use icons::\*;$" "$components_mod"; then
  printf 'pub use icons::*;\n' >>"$components_mod"
fi

echo "added icon: $module ($name, regular) -> $target"
