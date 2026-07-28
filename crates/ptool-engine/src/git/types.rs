#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GitInitOptions {
    pub bare: bool,
    pub initial_head: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GitCloneOptions {
    pub branch: Option<String>,
    pub bare: bool,
    pub depth: Option<i32>,
    pub checkout: bool,
    pub remote: Option<String>,
    pub tags: GitTagDownload,
    pub auth: GitRemoteAuth,
}

impl Default for GitCloneOptions {
    fn default() -> Self {
        Self {
            branch: None,
            bare: false,
            depth: None,
            checkout: true,
            remote: None,
            tags: GitTagDownload::Auto,
            auth: GitRemoteAuth::Default,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum GitTagDownload {
    #[default]
    Auto,
    All,
    None,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GitFetchOptions {
    pub refspecs: Vec<String>,
    pub auth: GitRemoteAuth,
    pub depth: Option<i32>,
    pub prune: bool,
    pub tags: GitTagDownload,
    pub update_fetchhead: bool,
}

impl Default for GitFetchOptions {
    fn default() -> Self {
        Self {
            refspecs: Vec::new(),
            auth: GitRemoteAuth::Default,
            depth: None,
            prune: false,
            tags: GitTagDownload::Auto,
            update_fetchhead: true,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GitPushOptions {
    pub auth: GitRemoteAuth,
    pub force: bool,
    pub set_upstream: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GitStatusOptions {
    pub include_untracked: bool,
    pub include_ignored: bool,
    pub recurse_untracked_dirs: bool,
    pub paths: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GitAddOptions {
    pub update: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GitCommitOptions {
    pub author: Option<GitSignature>,
    pub committer: Option<GitSignature>,
    pub amend: bool,
    pub allow_empty: bool,
}

impl Default for GitCommitOptions {
    fn default() -> Self {
        Self {
            author: None,
            committer: None,
            amend: false,
            // Preserve the existing API behavior, which already permitted empty commits.
            allow_empty: true,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GitCheckoutOptions {
    pub force: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GitSwitchOptions {
    pub create: bool,
    pub force: bool,
    pub start_point: Option<String>,
    pub track: Option<String>,
    pub orphan: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GitSignature {
    pub name: String,
    pub email: String,
    pub time_seconds: Option<i64>,
    pub offset_minutes: Option<i32>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum GitRemoteAuth {
    #[default]
    Default,
    SshAgent {
        username: Option<String>,
    },
    SshKey {
        username: String,
        public_key: Option<String>,
        private_key: String,
        passphrase: Option<String>,
    },
    UserPass {
        username: String,
        password: String,
    },
    CredentialHelper {
        username: Option<String>,
    },
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GitHeadInfo {
    pub oid: Option<String>,
    pub shorthand: Option<String>,
    pub detached: bool,
    pub unborn: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GitStatusEntry {
    pub path: String,
    pub index_status: Option<String>,
    pub worktree_status: Option<String>,
    pub conflicted: bool,
    pub ignored: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GitStatusSummary {
    pub root: Option<String>,
    pub branch: Option<String>,
    pub head: GitHeadInfo,
    pub upstream: Option<String>,
    pub ahead: usize,
    pub behind: usize,
    pub clean: bool,
    pub entries: Vec<GitStatusEntry>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GitTagCreateOptions {
    pub message: Option<String>,
    pub tagger: Option<GitSignature>,
    pub force: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GitTagInfo {
    pub name: String,
    pub oid: String,
    pub target_oid: String,
    pub target_kind: String,
    pub annotated: bool,
    pub message: Option<String>,
    pub tagger: Option<GitSignature>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GitFetchStats {
    pub received_objects: usize,
    pub indexed_objects: usize,
    pub local_objects: usize,
    pub total_objects: usize,
    pub received_bytes: usize,
    pub updated_refs: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GitPushResult {
    pub refspecs: Vec<String>,
    pub rejected: Vec<GitPushRejection>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GitPushRejection {
    pub reference: String,
    pub message: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GitRemoteInfo {
    pub name: String,
    pub url: Option<String>,
    pub push_url: Option<String>,
    pub fetch_refspecs: Vec<String>,
    pub push_refspecs: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GitRemoteAddOptions {
    pub push_url: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GitObjectInfo {
    pub oid: String,
    pub kind: String,
    pub shorthand: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GitSignatureInfo {
    pub name: Option<String>,
    pub email: Option<String>,
    pub time_seconds: i64,
    pub offset_minutes: i32,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GitCommitInfo {
    pub oid: String,
    pub summary: Option<String>,
    pub message: Option<String>,
    pub author: GitSignatureInfo,
    pub committer: GitSignatureInfo,
    pub parent_oids: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GitLogOptions {
    pub rev: String,
    pub max_count: usize,
    pub skip: usize,
    pub first_parent: bool,
    pub reverse: bool,
    pub paths: Vec<String>,
}

impl Default for GitLogOptions {
    fn default() -> Self {
        Self {
            rev: "HEAD".to_string(),
            max_count: 100,
            skip: 0,
            first_parent: false,
            reverse: false,
            paths: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GitDiffOptions {
    pub from: Option<String>,
    pub to: Option<String>,
    pub cached: bool,
    pub paths: Vec<String>,
    pub context_lines: u32,
    pub find_renames: bool,
}

impl Default for GitDiffOptions {
    fn default() -> Self {
        Self {
            from: None,
            to: None,
            cached: false,
            paths: Vec::new(),
            context_lines: 3,
            find_renames: true,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GitDiffDelta {
    pub status: String,
    pub old_path: Option<String>,
    pub new_path: Option<String>,
    pub binary: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GitDiffSummary {
    pub patch: String,
    pub files_changed: usize,
    pub insertions: usize,
    pub deletions: usize,
    pub deltas: Vec<GitDiffDelta>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GitDescribeOptions {
    pub rev: Option<String>,
    pub pattern: Option<String>,
    pub always: bool,
    pub abbrev: usize,
    pub dirty_suffix: Option<String>,
}

impl Default for GitDescribeOptions {
    fn default() -> Self {
        Self {
            rev: None,
            pattern: None,
            always: false,
            abbrev: 7,
            dirty_suffix: None,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum GitBranchKind {
    #[default]
    Local,
    Remote,
    All,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GitBranchListOptions {
    pub kind: GitBranchKind,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GitBranchCreateOptions {
    pub start_point: Option<String>,
    pub force: bool,
    pub checkout: bool,
    pub upstream: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GitBranchDeleteOptions {
    pub force: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GitBranchInfo {
    pub name: String,
    pub kind: String,
    pub oid: String,
    pub head: bool,
    pub upstream: Option<String>,
    pub ahead: usize,
    pub behind: usize,
}

impl Default for GitStatusOptions {
    fn default() -> Self {
        Self {
            include_untracked: true,
            include_ignored: false,
            recurse_untracked_dirs: true,
            paths: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GitRestoreOptions {
    pub source: String,
    pub staged: bool,
    pub worktree: bool,
}

impl Default for GitRestoreOptions {
    fn default() -> Self {
        Self {
            source: "HEAD".to_string(),
            staged: false,
            worktree: true,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum GitResetMode {
    Soft,
    #[default]
    Mixed,
    Hard,
}

impl GitResetMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Soft => "soft",
            Self::Mixed => "mixed",
            Self::Hard => "hard",
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GitResetOptions {
    pub mode: GitResetMode,
    pub force: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GitRemoveOptions {
    pub cached: bool,
    pub force: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GitCleanOptions {
    pub dry_run: bool,
    pub force: bool,
    pub dirs: bool,
    pub ignored: bool,
    pub paths: Vec<String>,
}

impl Default for GitCleanOptions {
    fn default() -> Self {
        Self {
            dry_run: true,
            force: false,
            dirs: false,
            ignored: false,
            paths: Vec::new(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum GitConfigScope {
    #[default]
    Local,
    Global,
    System,
}

impl GitConfigScope {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Local => "local",
            Self::Global => "global",
            Self::System => "system",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GitConfigValue {
    String(String),
    Boolean(bool),
    Integer(i64),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GitConfigEntry {
    pub name: String,
    pub value: GitConfigValue,
    pub scope: String,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum GitFastForwardMode {
    #[default]
    Allow,
    Only,
    Never,
}

impl GitFastForwardMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Allow => "allow",
            Self::Only => "only",
            Self::Never => "never",
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GitMergeOptions {
    pub ff: GitFastForwardMode,
    pub signature: Option<GitSignature>,
    pub message: Option<String>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum GitPullStrategy {
    #[default]
    FastForwardOnly,
    Merge,
    Rebase,
}

impl GitPullStrategy {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::FastForwardOnly => "ff_only",
            Self::Merge => "merge",
            Self::Rebase => "rebase",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GitPullOptions {
    pub auth: GitRemoteAuth,
    pub depth: Option<i32>,
    pub prune: bool,
    pub tags: GitTagDownload,
    pub update_fetchhead: bool,
    pub strategy: GitPullStrategy,
    pub signature: Option<GitSignature>,
    pub message: Option<String>,
}

impl Default for GitPullOptions {
    fn default() -> Self {
        Self {
            auth: GitRemoteAuth::Default,
            depth: None,
            prune: false,
            tags: GitTagDownload::Auto,
            update_fetchhead: true,
            strategy: GitPullStrategy::FastForwardOnly,
            signature: None,
            message: None,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GitConflictEntry {
    pub path: String,
    pub ancestor: bool,
    pub ours: bool,
    pub theirs: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GitIntegrateResult {
    pub outcome: String,
    pub oid: Option<String>,
    pub conflicts: Vec<GitConflictEntry>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GitApplyCommitOptions {
    pub commit: bool,
    pub signature: Option<GitSignature>,
    pub message: Option<String>,
    pub mainline: Option<u32>,
}

impl Default for GitApplyCommitOptions {
    fn default() -> Self {
        Self {
            commit: true,
            signature: None,
            message: None,
            mainline: None,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GitStashSaveOptions {
    pub include_untracked: bool,
    pub include_ignored: bool,
    pub keep_index: bool,
    pub signature: Option<GitSignature>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GitStashApplyOptions {
    pub reinstate_index: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GitStashInfo {
    pub index: usize,
    pub message: String,
    pub oid: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GitRebaseOptions {
    pub upstream: String,
    pub onto: Option<String>,
    pub branch: String,
    pub signature: Option<GitSignature>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GitRebaseContinueOptions {
    pub signature: Option<GitSignature>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GitRebaseResult {
    pub outcome: String,
    pub oid: Option<String>,
    pub conflicts: Vec<GitConflictEntry>,
    pub current: Option<usize>,
    pub total: usize,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GitWorktreeAddOptions {
    pub reference: Option<String>,
    pub lock: bool,
    pub checkout_existing: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GitWorktreePruneOptions {
    pub valid: bool,
    pub locked: bool,
    pub working_tree: bool,
    pub force: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GitWorktreeInfo {
    pub name: String,
    pub path: String,
    pub locked: bool,
    pub lock_reason: Option<String>,
    pub valid: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GitSubmoduleInitOptions {
    pub overwrite: bool,
    pub recursive: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GitSubmoduleUpdateOptions {
    pub init: bool,
    pub recursive: bool,
    pub allow_fetch: bool,
    pub auth: GitRemoteAuth,
}

impl Default for GitSubmoduleUpdateOptions {
    fn default() -> Self {
        Self {
            init: true,
            recursive: false,
            allow_fetch: true,
            auth: GitRemoteAuth::Default,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GitSubmoduleSyncOptions {
    pub recursive: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GitSubmoduleInfo {
    pub name: String,
    pub path: String,
    pub url: Option<String>,
    pub branch: Option<String>,
    pub head_oid: Option<String>,
    pub index_oid: Option<String>,
    pub workdir_oid: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GitBlameOptions {
    pub newest: Option<String>,
    pub oldest: Option<String>,
    pub min_line: Option<usize>,
    pub max_line: Option<usize>,
    pub first_parent: bool,
    pub track_copies_same_file: bool,
    pub track_copies_same_commit_moves: bool,
    pub track_copies_same_commit_copies: bool,
    pub track_copies_any_commit_copies: bool,
    pub ignore_whitespace: bool,
    pub use_mailmap: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GitBlameHunk {
    pub final_start_line: usize,
    pub original_start_line: usize,
    pub line_count: usize,
    pub commit_oid: String,
    pub author: GitSignatureInfo,
    pub origin_path: Option<String>,
    pub boundary: bool,
}
