// The "path" attribute is needed to avoid this error:
// Error writing files: failed to resolve mod `common`: garden/tests does not exist
// https://github.com/rust-lang/rustfmt/issues/4510
#[path = "common/mod.rs"]
pub mod common;
use common::{
    assert_cmd, assert_cmd_capture, assert_ref, assert_ref_missing, exec_garden, garden_capture,
    BareRepoFixture,
};

use garden::git;
use garden::model;

use anyhow::Result;
use function_name::named;

/// `garden grow` clones repositories
#[test]
#[named]
fn grow_clone() -> Result<()> {
    let fixture = BareRepoFixture::new(function_name!());
    // garden grow examples/tree
    exec_garden(&[
        "--verbose",
        "--verbose",
        "--chdir",
        &fixture.root(),
        "--config",
        "tests/data/garden.yaml",
        "grow",
        "example/tree",
    ])?;

    // A repository was created.
    let worktree = fixture.worktree("example/tree/repo");
    // The repository has all branches.
    assert_ref(&worktree, "origin/default");
    assert_ref(&worktree, "origin/dev");

    Ok(())
}

/// `garden grow` can create shallow clones with depth: 1.
#[test]
#[named]
fn grow_clone_shallow() -> Result<()> {
    let fixture = BareRepoFixture::new(function_name!());
    // garden grow examples/shallow
    exec_garden(&[
        "--verbose",
        "--verbose",
        "--chdir",
        &fixture.root(),
        "--config",
        "tests/data/garden.yaml",
        "grow",
        "example/shallow",
    ])?;

    // A repository was created.
    let worktree = fixture.worktree("example/tree/shallow");
    // The repository has all branches.
    assert_ref(&worktree, "origin/default");
    assert_ref(&worktree, "origin/dev");

    // Only one commit must be cloned because of "depth: 1".
    let cmd = ["git", "rev-list", "HEAD"];
    let output = assert_cmd_capture(&cmd, &worktree);
    let lines = output.split("\n").collect::<Vec<&str>>();
    assert_eq!(lines.len(), 1); // One commit only!

    Ok(())
}

/// `garden grow` clones a single branch with "single-branch: true".
#[test]
#[named]
fn grow_clone_single_branch() -> Result<()> {
    let fixture = BareRepoFixture::new(function_name!());
    // garden grow examples/single-branch
    exec_garden(&[
        "--verbose",
        "--verbose",
        "--chdir",
        &fixture.root(),
        "--config",
        "tests/data/garden.yaml",
        "grow",
        "example/single-branch",
    ])?;

    // A repository was created.
    let worktree = fixture.worktree("example/tree/single-branch");

    // The repository must have the default branch.
    assert_ref(&worktree, "origin/default");
    // The dev branch must *not* exist because we cloned with --single-branch.
    assert_ref_missing(&worktree, "origin/dev");

    // Only one commit must be cloned because of "depth: 1".
    let cmd = ["git", "rev-list", "HEAD"];
    let output = assert_cmd_capture(&cmd, &worktree);
    let lines = output.split("\n").collect::<Vec<&str>>();
    assert_eq!(lines.len(), 1); // One commit only!

    Ok(())
}

#[test]
#[named]
fn grow_branch_default() -> Result<()> {
    let fixture = BareRepoFixture::new(function_name!());
    // garden grow default dev
    exec_garden(&[
        "--verbose",
        "--verbose",
        "--chdir",
        &fixture.root(),
        "--config",
        "tests/data/branches.yaml",
        "grow",
        "default",
        "dev",
    ])?;

    // Ensure the repositories were created.
    let worktree_default = fixture.worktree("default");
    let worktree_dev = fixture.worktree("dev");

    // The "default" repository must have a branch called "default" checked-out.
    let cmd = ["git", "symbolic-ref", "--short", "HEAD"];
    let output = assert_cmd_capture(&cmd, &worktree_default);
    let lines = output.split("\n").collect::<Vec<&str>>();
    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0], "default");

    // The "dev" repository must have a branch called "dev" checked-out.
    let cmd = ["git", "symbolic-ref", "--short", "HEAD"];
    let output = assert_cmd_capture(&cmd, &worktree_dev);
    let lines = output.split("\n").collect::<Vec<&str>>();
    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0], "dev");

    // The origin/dev and origin/default branches must exist because we cloned with
    // --no-single-branch.
    assert_ref(&worktree_default, "origin/default");
    assert_ref(&worktree_default, "origin/dev");

    assert_ref(&worktree_dev, "origin/default");
    assert_ref(&worktree_dev, "origin/dev");

    Ok(())
}

/// This creates bare repositories based on the "bare.git" naming convention.
/// The configuration does not specify "bare: true".
#[test]
#[named]
fn grow_bare_repo() -> Result<()> {
    let fixture = BareRepoFixture::new(function_name!());
    // garden grow bare.git
    exec_garden(&[
        "--verbose",
        "--verbose",
        "--chdir",
        &fixture.root(),
        "--config",
        "tests/data/bare.yaml",
        "grow",
        "bare.git",
    ])?;

    // A repository was created.
    let bare_repo = fixture.path("bare.git");

    // The all branches must exist because we cloned with --no-single-branch.
    assert_ref(&bare_repo, "default");
    assert_ref(&bare_repo, "dev");

    // The repository must be bare.
    let cmd = ["git", "config", "--bool", "core.bare"];
    let output = assert_cmd_capture(&cmd, &bare_repo);
    assert_eq!(output, String::from("true"));

    Ok(())
}

/// This creates bare repositories using the "bare: true" configuration.
#[test]
#[named]
fn grow_bare_repo_with_config() -> Result<()> {
    let fixture = BareRepoFixture::new(function_name!());
    // garden grow bare.git
    exec_garden(&[
        "--verbose",
        "--verbose",
        "--chdir",
        &fixture.root(),
        "--config",
        "tests/data/bare.yaml",
        "grow",
        "bare",
    ])?;

    // Ensure the repository was created
    // tests/tmp/grow-bare-repo-config/bare
    let bare_repo = fixture.path("bare");
    let repo = std::path::PathBuf::from(&bare_repo);
    assert!(repo.exists());

    // We cloned with --no-single-branch so "default" and "dev" must exist.
    assert_ref(&bare_repo, "default");
    assert_ref(&bare_repo, "dev");

    // The repository must be bare.
    let cmd = ["git", "config", "core.bare"];
    let output = assert_cmd_capture(&cmd, &bare_repo);
    assert_eq!(output, String::from("true"));

    Ok(())
}

/// `garden grow` sets up remotes
#[test]
#[named]
fn grow_remotes() -> Result<()> {
    let fixture = BareRepoFixture::new(function_name!());
    // garden grow examples/tree
    exec_garden(&[
        "--verbose",
        "--verbose",
        "--chdir",
        &fixture.root(),
        "--config",
        "tests/data/garden.yaml",
        "grow",
        "example/tree",
    ])?;

    // remote.origin.url is a read-only https:// URL
    let worktree = fixture.path("example/tree/repo");
    let cmd = ["git", "config", "remote.origin.url"];
    let output = assert_cmd_capture(&cmd, &worktree);
    assert!(
        output.ends_with("/repos/example.git"),
        "{} does not end with {}",
        output,
        "/repos/example.git"
    );

    // remote.publish.url is a ssh push URL
    let cmd = ["git", "config", "remote.publish.url"];
    let output = assert_cmd_capture(&cmd, &worktree);
    assert_eq!("git@github.com:user/example.git", output);

    Ok(())
}

/// `garden grow` creates symlinks
#[test]
#[named]
fn grow_symlinks() -> Result<()> {
    let fixture = BareRepoFixture::new(function_name!());
    // garden grow examples/tree examples/link
    exec_garden(&[
        "--verbose",
        "--verbose",
        "--chdir",
        &fixture.root(),
        "--config",
        "tests/data/garden.yaml",
        "grow",
        "example/tree",
        "link",
        "example/link",
    ])?;

    let repo = std::path::PathBuf::from(fixture.path("example/tree/repo/.git"));
    assert!(repo.exists());

    // tests/tmp/symlinks/link is a symlink pointing to example/tree/repo
    let link_str = fixture.path("link");
    let link = std::path::PathBuf::from(&link_str);
    assert!(link.exists(), "{} does not exist", link_str);
    assert!(link.read_link().is_ok());

    let target = link.read_link().unwrap();
    assert_eq!("example/tree/repo", target.to_string_lossy());

    // tests/tmp/symlinks/example/link is a symlink pointing to tree/repo
    let link_str = fixture.path("example/link");
    let link = std::path::PathBuf::from(&link_str);
    assert!(link.exists(), "{} does not exist", link_str);
    assert!(link.read_link().is_ok());

    let target = link.read_link().unwrap();
    assert_eq!("tree/repo", target.to_string_lossy());

    Ok(())
}

/// `garden grow` sets up git config settings
#[test]
#[named]
fn grow_gitconfig() -> Result<()> {
    let fixture = BareRepoFixture::new(function_name!());
    // garden grow examples/tree
    exec_garden(&[
        "--verbose",
        "--verbose",
        "--chdir",
        &fixture.root(),
        "--config",
        "tests/data/garden.yaml",
        "grow",
        "example/tree",
    ])?;

    // remote.origin.annex-ignore is true
    let worktree = fixture.path("example/tree/repo");
    let cmd = ["git", "config", "remote.origin.annex-ignore"];
    let output = assert_cmd_capture(&cmd, &worktree);
    assert_eq!("true", output);

    // user.name is "A U Thor"
    let cmd = ["git", "config", "user.name"];
    let output = assert_cmd_capture(&cmd, &worktree);
    assert_eq!("A U Thor", output);

    // user.email is "author@example.com"
    let cmd = ["git", "config", "user.email"];
    let output = assert_cmd_capture(&cmd, &worktree);
    assert_eq!("author@example.com", output);

    Ok(())
}

/// This creates a worktree
#[test]
#[named]
fn grow_worktree_and_parent() -> Result<()> {
    let fixture = BareRepoFixture::new(function_name!());
    // garden grow dev
    exec_garden(&[
        "--verbose",
        "--verbose",
        "--chdir",
        &fixture.root(),
        "--config",
        "tests/data/worktree.yaml",
        "grow",
        "dev",
    ])?;

    // Ensure the repository was created
    let worktree_default = fixture.worktree("default");
    let worktree_dev = fixture.worktree("dev");

    assert_ref(&worktree_default, "default");
    assert_ref(&worktree_dev, "dev");

    // Ensure that the "echo" command is available from the child worktree.
    let output = garden_capture(&[
        "--chdir",
        &fixture.root(),
        "--config",
        "tests/data/worktree.yaml",
        "echo",
        "dev",
        "--",
        "hello",
    ]);
    // The "echo" command is: echo ${TREE_NAME} "$@"
    assert_eq!("dev hello", output);

    // Ensure that the "echo" command is available from the parent worktree.
    let output = garden_capture(&[
        "--chdir",
        &fixture.root(),
        "--config",
        "tests/data/worktree.yaml",
        "echo",
        "default",
        "--",
        "hello",
    ]);
    // The "echo" command is: echo ${TREE_NAME} "$@"
    assert_eq!("default hello", output);

    Ok(())
}

/// `garden eval` evaluates ${GARDEN_CONFIG_DIR}
#[test]
#[named]
fn eval_garden_config_dir() -> Result<()> {
    let fixture = BareRepoFixture::new(function_name!());
    // garden eval ${GARDEN_CONFIG_DIR}
    let output = garden_capture(&[
        "--chdir",
        &fixture.root(),
        "--config",
        "tests/data/garden.yaml",
        "eval",
        "${GARDEN_CONFIG_DIR}",
    ]);
    assert!(
        output.ends_with("/tests/data"),
        "{} does not end with /tests/data",
        output
    );

    Ok(())
}

/// `garden::git::worktree_details(path)` returns a struct with branches and a
/// GitTreeType (Tree, Bare, Parent, Worktree) for this worktree.
#[test]
#[named]
fn git_worktree_details() -> Result<()> {
    let fixture = BareRepoFixture::new(function_name!());

    // repos/example.git is a bare repository.
    let details = git::worktree_details(&fixture.pathbuf("repos/example.git"))?;
    assert_eq!(details.branch, ""); // Bare repository has no branch.
    assert_eq!(details.tree_type, model::GitTreeType::Bare);

    // Create a plain git worktree called "tree" with a branch called "branch".
    let cmd = ["git", "init", "--quiet", "tree"];
    assert_cmd(&cmd, &fixture.root());

    let cmd = ["git", "symbolic-ref", "HEAD", "refs/heads/branch"];
    assert_cmd(&cmd, &fixture.path("tree"));

    let details = git::worktree_details(&fixture.pathbuf("tree"))?;
    assert_eq!(details.branch, "branch");
    assert_eq!(details.tree_type, model::GitTreeType::Tree);

    // Create a parent worktree called "parent" branched off of "default".
    let cmd = ["git", "clone", "--quiet", "repos/example.git", "parent"];
    assert_cmd(&cmd, &fixture.root());

    // The initial query will be a Tree because there are no child worktrees.
    let details = git::worktree_details(&fixture.pathbuf("parent"))?;
    assert_eq!(details.branch, "default");
    assert_eq!(details.tree_type, model::GitTreeType::Tree);

    // Create a child worktree called "child" branched off of "dev".
    let cmd = [
        "git",
        "worktree",
        "add",
        "--track",
        "-B",
        "dev",
        "../child",
        "origin/dev",
    ];
    assert_cmd(&cmd, &fixture.path("parent"));

    // The "parent" repository is a GitTreeType::Parent.
    let details = git::worktree_details(&fixture.pathbuf("parent"))?;
    assert_eq!(details.branch, "default");
    assert_eq!(details.tree_type, model::GitTreeType::Parent);

    // The "child" repository is a GitTreeType::Worktree(parent_path).
    let parent_path_relative = &fixture.pathbuf("parent");
    let parent_path = parent_path_relative
        .to_path_buf()
        .canonicalize()
        .unwrap_or(parent_path_relative.to_path_buf())
        .to_string_lossy()
        .to_string();
    let details = git::worktree_details(&fixture.pathbuf("child"))?;
    assert_eq!(details.branch, "dev");
    assert_eq!(details.tree_type, model::GitTreeType::Worktree(parent_path));

    Ok(())
}

/// Test eval behavior around the "--root" option
#[test]
#[named]
fn eval_root_with_root() {
    // garden eval ${GARDEN_ROOT}
    let output = garden_capture(&[
        "--config",
        "tests/data/garden.yaml",
        "--root",
        "tests/tmp",
        "eval",
        "${GARDEN_ROOT}",
    ]);
    assert!(output.ends_with("/tests/tmp"));

    let path = std::path::PathBuf::from(&output);
    assert!(path.exists());
    assert!(path.is_absolute());
}

/// Test eval ${GARDEN_CONFIG_DIR} behavior with both "--root" and "--chdir"
#[test]
fn eval_config_dir_with_chdir_and_root() {
    let output = garden_capture(&[
        "--chdir",
        "tests/tmp",
        "--config",
        "tests/data/garden.yaml",
        "--root",
        "tests/tmp",
        "eval",
        "${GARDEN_CONFIG_DIR}",
    ]);
    assert!(output.ends_with("/tests/data"));

    let path = std::path::PathBuf::from(&output);
    assert!(path.exists());
    assert!(path.is_absolute());
}

/// Test pwd with both "--root" and "--chdir"
#[test]
fn eval_exec_pwd_with_root_and_chdir() {
    let output = garden_capture(&[
        "--chdir",
        "tests/tmp",
        "--config",
        "tests/data/garden.yaml",
        "--root",
        "tests/tmp",
        "eval",
        "$ pwd",
    ]);
    assert!(output.ends_with("/tests/tmp"));

    let path = std::path::PathBuf::from(&output);
    assert!(path.exists());
    assert!(path.is_absolute());
}

/// Test ${GARDEN_ROOT} with both "--root" and "--chdir"
#[test]
fn eval_root_with_root_and_chdir() {
    let output = garden_capture(&[
        "--chdir",
        "tests/tmp",
        "--config",
        "tests/data/garden.yaml",
        "--root",
        "tests/tmp",
        "eval",
        "${GARDEN_ROOT}",
    ]);
    assert!(output.ends_with("/tests/tmp"));

    let path = std::path::PathBuf::from(&output);
    assert!(path.exists());
    assert!(path.is_absolute());
}

/// Test dash-dash arguments in custom commands via "garden cmd ..."
#[test]
fn cmd_dash_dash_arguments() {
    let output = garden_capture(&[
        "--chdir",
        "tests/data",
        "--quiet",
        "cmd",
        ".",
        "echo-dir",
        "echo-args",
        "echo-dir",
        "echo-args",
        "--",
        "d",
        "e",
        "f",
        "--",
        "g",
        "h",
        "i",
    ]);
    // Repeated command names were used to operate on the tree twice.
    let msg = format!(
        "data\ngarden\n{}",
        "arguments -- a b c -- d e f -- g h i -- x y z"
    );
    assert_eq!(output, format!("{}\n{}", msg, msg));
}

/// Test dash-dash arguments in custom commands via "garden <custom> ..."
#[test]
fn cmd_dash_dash_arguments_custom() {
    let output = garden_capture(&[
        "--chdir",
        "tests/data",
        "--quiet",
        "echo-args",
        ".",
        ".",
        "--",
        "d",
        "e",
        "f",
        "--",
        "g",
        "h",
        "i",
    ]);
    // `. .` was used to operate on the tree twice.
    let msg = "garden\narguments -- a b c -- d e f -- g h i -- x y z";
    assert_eq!(format!("{}\n{}", msg, msg), output);
}

/// Test "." default for custom "garden <command>" with no arguments
#[test]
fn cmd_dot_default_no_args() {
    let output = garden_capture(&["--quiet", "--chdir", "tests/data", "echo-dir"]);
    assert_eq!("data", output);
}

/// Test "." default for "garden <command>" with no arguments and echo
#[test]
fn cmd_dot_default_no_args_echo() {
    let output = garden_capture(&["--quiet", "--chdir", "tests/data", "echo-args"]);
    let msg = "garden\narguments -- a b c -- -- x y z";
    assert_eq!(msg, output);
}

/// Test "." default for "garden <command>" with double-dash
#[test]
fn cmd_dot_default_double_dash() {
    let output = garden_capture(&["--quiet", "--chdir", "tests/data", "echo-args", "--"]);
    let msg = "garden\narguments -- a b c -- -- x y z";
    assert_eq!(msg, output);
}

/// Test "." default for "garden <command>" with extra arguments
#[test]
fn cmd_dot_default_double_dash_args() {
    let output = garden_capture(&[
        "--quiet",
        "--chdir",
        "tests/data",
        "echo-args",
        "--",
        "d",
        "e",
        "f",
        "--",
        "g",
        "h",
        "i",
    ]);
    let msg = "garden\narguments -- a b c -- d e f -- g h i -- x y z";
    assert_eq!(msg, output);
}
