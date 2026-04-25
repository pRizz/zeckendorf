#!/usr/bin/env bash
# bright-builds-rules-managed-file: scripts/bright-builds-auto-update.sh
# Managed upstream by bright-builds-rules.
# If this helper needs a fix, open an upstream PR or issue instead of editing the downstream managed copy.
set -euo pipefail

audit_path="bright-builds-rules.audit.md"
legacy_audit_path="coding-and-architecture-requirements.audit.md"
update_branch="bright-builds/auto-update"
commit_message="chore: update Bright Builds Rules"
github_actions_name="github-actions[bot]"
github_actions_email="41898282+github-actions[bot]@users.noreply.github.com"

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
tmp_dir="$(mktemp -d "${TMPDIR:-/tmp}/bright-builds-auto-update.XXXXXX")"

cleanup() {
	rm -rf "$tmp_dir"
}

note() {
	printf '%s\n' "$*"
}

die() {
	printf 'error: %s\n' "$*" >&2
	exit 1
}

extract_markdown_value() {
	local file_path="$1"
	local label="$2"

	awk -v label="$label" '
    BEGIN {
      prefix = "- " label ": `"
    }

    index($0, prefix) == 1 {
      value = substr($0, length(prefix) + 1)
      sub(/`$/, "", value)
      print value
      exit
    }
  ' "$file_path"
}

extract_repo_slug_from_url() {
	local input_url="$1"

	printf '%s' "$input_url" | sed -n 's#^https://github.com/\(.*\)$#\1#p' | sed 's#/$##'
}

path_exists_or_is_tracked() {
	local relative_path="$1"

	[[ -e "$relative_path" ]] || git ls-files --error-unmatch "$relative_path" >/dev/null 2>&1
}

managed_path_has_changes() {
	local relative_path="$1"

	if ! path_exists_or_is_tracked "$relative_path"; then
		return 1
	fi

	[[ -n "$(git status --short --untracked-files=all -- "$relative_path")" ]]
}

resolve_default_branch() {
	local maybe_branch=""

	maybe_branch="$(git symbolic-ref --short refs/remotes/origin/HEAD 2>/dev/null | sed 's#^origin/##' || true)"
	if [[ -n "$maybe_branch" ]]; then
		printf '%s\n' "$maybe_branch"
		return
	fi

	maybe_branch="$(git branch --show-current 2>/dev/null || true)"
	if [[ -n "$maybe_branch" ]] && [[ "$maybe_branch" != "HEAD" ]]; then
		printf '%s\n' "$maybe_branch"
		return
	fi

	if command -v gh >/dev/null 2>&1; then
		maybe_branch="$(gh repo view --json defaultBranchRef --jq .defaultBranchRef.name 2>/dev/null || true)"
		if [[ -n "$maybe_branch" ]]; then
			printf '%s\n' "$maybe_branch"
			return
		fi
	fi

	die "could not resolve the default branch"
}

stage_managed_paths() {
	local relative_path=""

	for relative_path in \
		AGENTS.md \
		AGENTS.bright-builds.md \
		CONTRIBUTING.md \
		README.md \
		bright-builds-rules.audit.md \
		coding-and-architecture-requirements.audit.md \
		.github/pull_request_template.md \
		.github/workflows/bright-builds-auto-update.yml \
		scripts/bright-builds-auto-update.sh; do
		if [[ -e "$relative_path" ]] || git ls-files --error-unmatch "$relative_path" >/dev/null 2>&1; then
			git add -A -- "$relative_path"
		fi
	done
}

restore_audit_if_only_runtime_changed() {
	local audit_before_path="${tmp_dir}/audit.before"
	local sanitized_before_path="${tmp_dir}/audit.before.sanitized"
	local sanitized_after_path="${tmp_dir}/audit.after.sanitized"
	local relative_path=""

	[[ -f "$audit_before_path" ]] || return
	[[ -f "$audit_path" ]] || return

	grep -vE '^- Last operation: `|^- Last updated \(UTC\): `' "$audit_before_path" >"$sanitized_before_path"
	grep -vE '^- Last operation: `|^- Last updated \(UTC\): `' "$audit_path" >"$sanitized_after_path"

	if ! cmp -s "$sanitized_before_path" "$sanitized_after_path"; then
		return
	fi

	for relative_path in \
		AGENTS.md \
		AGENTS.bright-builds.md \
		CONTRIBUTING.md \
		README.md \
		.github/pull_request_template.md \
		.github/workflows/bright-builds-auto-update.yml \
		scripts/bright-builds-auto-update.sh; do
		if managed_path_has_changes "$relative_path"; then
			return
		fi
	done

	cp "$audit_before_path" "$audit_path"
	note "Ignored audit-only runtime metadata changes."
}

create_or_reuse_pr() {
	local default_branch="$1"
	local maybe_existing_pr=""

	command -v gh >/dev/null 2>&1 || die "gh is required to open the fallback pull request"

	maybe_existing_pr="$(gh pr list --head "$update_branch" --base "$default_branch" --json number --jq '.[0].number' 2>/dev/null || true)"

	if [[ "$maybe_existing_pr" == "[]" || "$maybe_existing_pr" == "null" ]]; then
		maybe_existing_pr=""
	fi

	if [[ -n "$maybe_existing_pr" ]]; then
		note "Reused existing pull request #${maybe_existing_pr}"
		return
	fi

	gh pr create \
		--base "$default_branch" \
		--head "$update_branch" \
		--title "$commit_message" \
		--body "Automated Bright Builds Rules update." >/dev/null

	note "Opened pull request from ${update_branch} to ${default_branch}"
}

trap cleanup EXIT

cd "$repo_root"

if [[ ! -f "$audit_path" && -f "$legacy_audit_path" ]]; then
	audit_path="$legacy_audit_path"
fi

[[ -f "$audit_path" ]] || die "missing audit manifest: ${audit_path}"

source_url="$(extract_markdown_value "$audit_path" "Source repository")"
ref="$(extract_markdown_value "$audit_path" "Version pin")"
auto_update="$(extract_markdown_value "$audit_path" "Auto-update")"

[[ -n "$source_url" ]] || die "missing source repository in ${audit_path}"
[[ -n "$ref" ]] || die "missing version pin in ${audit_path}"

if [[ "$auto_update" != "enabled" ]]; then
	note "Auto-update is disabled; nothing to do."
	exit 0
fi

repo_slug="$(extract_repo_slug_from_url "$source_url")"
[[ -n "$repo_slug" ]] || die "could not parse GitHub source repository from ${source_url}"

installer_path="${tmp_dir}/manage-downstream.sh"

curl -fsSL "https://raw.githubusercontent.com/${repo_slug}/${ref}/scripts/manage-downstream.sh" -o "$installer_path"
chmod +x "$installer_path"

set +e
status_output="$(bash "$installer_path" status --repo-root "$repo_root" 2>&1)"
status_code=$?
set -e

printf '%s\n' "$status_output"

if [[ "$status_code" -ne 0 ]]; then
	die "status failed"
fi

if [[ "$status_output" != *"Repo state: installed"* ]]; then
	die "auto-update requires the repo state to remain installed"
fi

if [[ -f "$audit_path" ]]; then
	cp "$audit_path" "${tmp_dir}/audit.before"
fi

set +e
update_output="$(bash "$installer_path" update --repo-root "$repo_root" 2>&1)"
update_code=$?
set -e

printf '%s\n' "$update_output"

if [[ "$update_code" -ne 0 ]]; then
	die "update failed"
fi

restore_audit_if_only_runtime_changed

stage_managed_paths

if git diff --cached --quiet --exit-code; then
	note "No managed-file changes detected."
	exit 0
fi

default_branch="$(resolve_default_branch)"

git -c user.name="$github_actions_name" -c user.email="$github_actions_email" commit -m "$commit_message" >/dev/null

if git push origin HEAD:"${default_branch}" >/dev/null 2>&1; then
	note "Pushed managed updates directly to ${default_branch}"
	exit 0
fi

note "Direct push to ${default_branch} failed; falling back to ${update_branch}"

git push --force-with-lease origin HEAD:"${update_branch}" >/dev/null
create_or_reuse_pr "$default_branch"
