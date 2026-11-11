#!/usr/bin/env bash
# Download the Linux x64 rdlnative library and the sample reports from a
# majorsilence/Reporting release, and export the env vars the wrapper tests need.
#
#   usage: fetch-rdlnative.sh stable|prerelease|<tag>
#
# Requires: gh (with GH_TOKEN), unzip, git.  Appends RDLNATIVE_LIB,
# REPORTING_REPO_ROOT and RDLNATIVE_TAG to $GITHUB_ENV when it is set.
set -euo pipefail

repo=majorsilence/Reporting
channel="${1:?usage: fetch-rdlnative.sh stable|prerelease|<tag>}"
dest="${RUNNER_TEMP:-/tmp}/rdlnative"

case "$channel" in
  stable)     tag=$(gh release view --repo "$repo" --json tagName -q .tagName) ;;
  prerelease) tag=$(gh release list --repo "$repo" --limit 30 --json tagName,isPrerelease \
                      -q '[.[] | select(.isPrerelease)][0].tagName') ;;
  *)          tag="$channel" ;;
esac
[ -n "$tag" ] && [ "$tag" != null ] || { echo "could not resolve a '$channel' release" >&2; exit 1; }
echo "Using $repo release $tag"

rm -rf "$dest" && mkdir -p "$dest"
gh release download "$tag" --repo "$repo" -D "$dest" -p '*-majorsilence-reporting-rdlnative-linux-x64.zip'
unzip -q "$dest"/*.zip -d "$dest/native"
lib=$(find "$dest/native" -name rdlnative.so | head -n 1)
[ -f "$lib" ] || { echo "rdlnative.so not found in release $tag" >&2; exit 1; }

git clone -q --depth 1 --branch "$tag" --filter=blob:none --sparse "https://github.com/$repo.git" "$dest/Reporting"
git -C "$dest/Reporting" sparse-checkout set Examples
for f in Examples/SqliteExamples/SimpleTest1.rdl Examples/northwindEF.db Examples/SetDataFromCode/SalesReport.rdl; do
  [ -f "$dest/Reporting/$f" ] || { echo "missing $f in $tag" >&2; exit 1; }
done

echo "RDLNATIVE_LIB=$lib  REPORTING_REPO_ROOT=$dest/Reporting  RDLNATIVE_TAG=$tag"
if [ -n "${GITHUB_ENV:-}" ]; then
  {
    echo "RDLNATIVE_LIB=$lib"
    echo "REPORTING_REPO_ROOT=$dest/Reporting"
    echo "RDLNATIVE_TAG=$tag"
    echo "DOTNET_SYSTEM_GLOBALIZATION_INVARIANT=1"
  } >> "$GITHUB_ENV"
fi
