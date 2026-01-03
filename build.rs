use std::process::Command;

fn main() {
    // Get the git commit SHA
    let maybe_git_commit = Command::new("git")
        .args(&["rev-parse", "HEAD"])
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                String::from_utf8(output.stdout)
                    .ok()
                    .map(|s| s.trim().to_string())
            } else {
                None
            }
        });

    // Check if the workspace is dirty (has uncommitted changes)
    let is_dirty = Command::new("git")
        .args(&["status", "--porcelain"])
        .output()
        .ok()
        .map(|output| {
            // If there's any output, the workspace is dirty
            !output.stdout.is_empty()
        })
        .unwrap_or(false);

    if let Some(mut git_commit) = maybe_git_commit {
        if is_dirty {
            git_commit.push_str("-dirty");
        }
        println!("cargo:rustc-env=GIT_COMMIT_SHA={}", git_commit);
    } else {
        // If no git commit is available, set an empty string
        println!("cargo:rustc-env=GIT_COMMIT_SHA=");
    }

    // Re-run if git information might change
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/refs/heads");
}
