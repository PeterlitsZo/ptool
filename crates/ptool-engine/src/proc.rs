use crate::{Error, ErrorKind, Result};
use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsStr;
use std::thread;
use std::time::{Duration, Instant};
use sysinfo::{
    Pid, ProcessRefreshKind, ProcessStatus, ProcessesToUpdate, Signal, System, UpdateKind, Users,
};

const FIND_OP: &str = "ptool.proc.find";
const GET_OP: &str = "ptool.proc.get";
const EXISTS_OP: &str = "ptool.proc.exists";
const KILL_OP: &str = "ptool.proc.kill";
const SELF_OP: &str = "ptool.proc.self";
const WAIT_GONE_OP: &str = "ptool.proc.wait_gone";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProcInfo {
    pub pid: u32,
    pub ppid: Option<u32>,
    pub name: String,
    pub exe: Option<String>,
    pub cwd: Option<String>,
    pub user: Option<String>,
    pub cmdline: Option<String>,
    pub argv: Vec<String>,
    pub state: String,
    pub start_time_unix_ms: u64,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ProcQuery {
    pub pid: Option<u32>,
    pub pids: Vec<u32>,
    pub ppid: Option<u32>,
    pub name: Option<String>,
    pub name_contains: Option<String>,
    pub exe: Option<String>,
    pub exe_contains: Option<String>,
    pub cmdline_contains: Option<String>,
    pub user: Option<String>,
    pub cwd: Option<String>,
    pub include_self: bool,
    pub limit: Option<usize>,
    pub sort_by: ProcSortBy,
    pub reverse: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ProcSortBy {
    #[default]
    Pid,
    StartTime,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProcTarget {
    pub pid: u32,
    pub start_time_unix_ms: Option<u64>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ProcSignal {
    Hangup,
    #[default]
    Term,
    Kill,
    Interrupt,
    Quit,
    Stop,
    Continue,
    User1,
    User2,
}

impl ProcSignal {
    pub fn label(self) -> &'static str {
        match self {
            Self::Hangup => "hup",
            Self::Term => "term",
            Self::Kill => "kill",
            Self::Interrupt => "int",
            Self::Quit => "quit",
            Self::Stop => "stop",
            Self::Continue => "cont",
            Self::User1 => "user1",
            Self::User2 => "user2",
        }
    }

    fn to_sysinfo(self) -> Signal {
        match self {
            Self::Hangup => Signal::Hangup,
            Self::Term => Signal::Term,
            Self::Kill => Signal::Kill,
            Self::Interrupt => Signal::Interrupt,
            Self::Quit => Signal::Quit,
            Self::Stop => Signal::Stop,
            Self::Continue => Signal::Continue,
            Self::User1 => Signal::User1,
            Self::User2 => Signal::User2,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProcKillOptions {
    pub signal: ProcSignal,
    pub missing_ok: bool,
    pub allow_self: bool,
}

impl Default for ProcKillOptions {
    fn default() -> Self {
        Self {
            signal: ProcSignal::default(),
            missing_ok: true,
            allow_self: false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProcKillEntry {
    pub pid: u32,
    pub ok: bool,
    pub existed: bool,
    pub signal: ProcSignal,
    pub message: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProcKillResult {
    pub ok: bool,
    pub signal: ProcSignal,
    pub total: usize,
    pub sent: usize,
    pub missing: usize,
    pub failed: usize,
    pub entries: Vec<ProcKillEntry>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProcWaitGoneOptions {
    pub timeout_ms: Option<u64>,
    pub interval_ms: u64,
}

impl Default for ProcWaitGoneOptions {
    fn default() -> Self {
        Self {
            timeout_ms: None,
            interval_ms: 100,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProcWaitGoneResult {
    pub ok: bool,
    pub timed_out: bool,
    pub total: usize,
    pub gone: Vec<u32>,
    pub remaining: Vec<u32>,
    pub elapsed_ms: u64,
}

pub(crate) fn self_process() -> Result<ProcInfo> {
    let pid = std::process::id();
    get_process(pid)?.ok_or_else(|| {
        Error::new(
            ErrorKind::Io,
            "ptool.proc.self failed: current process is not available",
        )
        .with_op(SELF_OP)
    })
}

pub(crate) fn get_process(pid: u32) -> Result<Option<ProcInfo>> {
    ensure_valid_pid(pid, GET_OP)?;
    let pid = Pid::from_u32(pid);
    let mut system = System::new();
    system.refresh_processes_specifics(
        ProcessesToUpdate::Some(&[pid]),
        true,
        process_refresh_kind(),
    );
    let users = Users::new_with_refreshed_list();
    Ok(system
        .process(pid)
        .map(|process| process_to_info(process, &users)))
}

pub(crate) fn process_exists(pid: u32) -> Result<bool> {
    ensure_valid_pid(pid, EXISTS_OP)?;
    let pid = Pid::from_u32(pid);
    let mut system = System::new();
    system.refresh_processes_specifics(
        ProcessesToUpdate::Some(&[pid]),
        true,
        ProcessRefreshKind::nothing().without_tasks(),
    );
    Ok(system.process(pid).is_some())
}

pub(crate) fn find_processes(query: &ProcQuery) -> Result<Vec<ProcInfo>> {
    validate_query(query)?;
    let mut system = System::new();
    system.refresh_processes_specifics(ProcessesToUpdate::All, true, process_refresh_kind());
    let users = Users::new_with_refreshed_list();
    let current_pid = std::process::id();
    let pid_filter =
        (!query.pids.is_empty()).then(|| query.pids.iter().copied().collect::<BTreeSet<_>>());

    let mut processes = system
        .processes()
        .values()
        .filter_map(|process| {
            let info = process_to_info(process, &users);
            if !query.include_self && info.pid == current_pid {
                return None;
            }
            if let Some(pid) = query.pid
                && info.pid != pid
            {
                return None;
            }
            if let Some(ref pids) = pid_filter
                && !pids.contains(&info.pid)
            {
                return None;
            }
            if let Some(ppid) = query.ppid
                && info.ppid != Some(ppid)
            {
                return None;
            }
            if let Some(ref name) = query.name
                && &info.name != name
            {
                return None;
            }
            if let Some(ref needle) = query.name_contains
                && !info.name.contains(needle)
            {
                return None;
            }
            if let Some(ref exe) = query.exe
                && info.exe.as_deref() != Some(exe.as_str())
            {
                return None;
            }
            if let Some(ref needle) = query.exe_contains
                && !info
                    .exe
                    .as_deref()
                    .is_some_and(|value| value.contains(needle))
            {
                return None;
            }
            if let Some(ref needle) = query.cmdline_contains
                && !info
                    .cmdline
                    .as_deref()
                    .is_some_and(|value| value.contains(needle))
            {
                return None;
            }
            if let Some(ref user) = query.user
                && info.user.as_deref() != Some(user.as_str())
            {
                return None;
            }
            if let Some(ref cwd) = query.cwd
                && info.cwd.as_deref() != Some(cwd.as_str())
            {
                return None;
            }
            Some(info)
        })
        .collect::<Vec<_>>();

    match query.sort_by {
        ProcSortBy::Pid => processes.sort_by_key(|info| info.pid),
        ProcSortBy::StartTime => {
            processes.sort_by_key(|info| (info.start_time_unix_ms, info.pid));
        }
    }
    if query.reverse {
        processes.reverse();
    }
    if let Some(limit) = query.limit {
        processes.truncate(limit);
    }
    Ok(processes)
}

pub(crate) fn kill_processes(
    targets: &[ProcTarget],
    options: &ProcKillOptions,
) -> Result<ProcKillResult> {
    let targets = normalize_targets(targets);
    let total = targets.len();
    if total == 0 {
        return Ok(ProcKillResult {
            ok: true,
            signal: options.signal,
            total: 0,
            sent: 0,
            missing: 0,
            failed: 0,
            entries: Vec::new(),
        });
    }

    let pids = targets
        .iter()
        .map(|target| Pid::from_u32(target.pid))
        .collect::<Vec<_>>();
    let mut system = System::new();
    system.refresh_processes_specifics(
        ProcessesToUpdate::Some(&pids),
        true,
        ProcessRefreshKind::nothing().without_tasks(),
    );

    let current_pid = std::process::id();
    let signal = options.signal.to_sysinfo();
    let mut entries = Vec::with_capacity(total);
    let mut sent = 0usize;
    let mut missing = 0usize;
    let mut failed = 0usize;

    for target in targets {
        let entry = if !options.allow_self && target.pid == current_pid {
            ProcKillEntry {
                pid: target.pid,
                ok: false,
                existed: true,
                signal: options.signal,
                message: Some("refusing to signal current process".to_string()),
            }
        } else if let Some(process) = system.process(Pid::from_u32(target.pid)) {
            if !matches_target(process, &target) {
                build_missing_entry(
                    target.pid,
                    options.signal,
                    options.missing_ok,
                    "process changed",
                )
            } else {
                match process.kill_with(signal) {
                    Some(true) => {
                        sent += 1;
                        ProcKillEntry {
                            pid: target.pid,
                            ok: true,
                            existed: true,
                            signal: options.signal,
                            message: None,
                        }
                    }
                    Some(false) => ProcKillEntry {
                        pid: target.pid,
                        ok: false,
                        existed: true,
                        signal: options.signal,
                        message: Some("failed to send signal".to_string()),
                    },
                    None => {
                        return Err(
                            Error::new(
                                ErrorKind::Unsupported,
                                format!(
                                    "ptool.proc.kill failed: signal `{}` is not supported on this platform",
                                    options.signal.label()
                                ),
                            )
                            .with_op(KILL_OP),
                        );
                    }
                }
            }
        } else {
            build_missing_entry(
                target.pid,
                options.signal,
                options.missing_ok,
                "process not found",
            )
        };

        if !entry.existed {
            missing += 1;
        }
        if !entry.ok {
            failed += 1;
        }
        entries.push(entry);
    }

    Ok(ProcKillResult {
        ok: failed == 0,
        signal: options.signal,
        total,
        sent,
        missing,
        failed,
        entries,
    })
}

pub(crate) fn wait_processes_gone(
    targets: &[ProcTarget],
    options: &ProcWaitGoneOptions,
) -> Result<ProcWaitGoneResult> {
    if options.interval_ms == 0 {
        return Err(Error::new(
            ErrorKind::InvalidArgs,
            "ptool.proc.wait_gone `interval_ms` must be > 0",
        )
        .with_op(WAIT_GONE_OP));
    }

    let targets = normalize_targets(targets);
    let total = targets.len();
    if total == 0 {
        return Ok(ProcWaitGoneResult {
            ok: true,
            timed_out: false,
            total: 0,
            gone: Vec::new(),
            remaining: Vec::new(),
            elapsed_ms: 0,
        });
    }

    let started = Instant::now();
    let timeout = options.timeout_ms.map(Duration::from_millis);
    let interval = Duration::from_millis(options.interval_ms);

    loop {
        let remaining_targets = find_remaining_targets(&targets);
        if remaining_targets.is_empty() {
            let elapsed_ms = saturating_millis(started.elapsed());
            return Ok(ProcWaitGoneResult {
                ok: true,
                timed_out: false,
                total,
                gone: targets.iter().map(|target| target.pid).collect(),
                remaining: Vec::new(),
                elapsed_ms,
            });
        }

        if let Some(timeout) = timeout
            && started.elapsed() >= timeout
        {
            let remaining = remaining_targets
                .iter()
                .map(|target| target.pid)
                .collect::<Vec<_>>();
            let gone = targets
                .iter()
                .filter(|target| !remaining.contains(&target.pid))
                .map(|target| target.pid)
                .collect::<Vec<_>>();
            let elapsed_ms = saturating_millis(started.elapsed());
            return Ok(ProcWaitGoneResult {
                ok: false,
                timed_out: true,
                total,
                gone,
                remaining,
                elapsed_ms,
            });
        }

        thread::sleep(interval);
    }
}

fn find_remaining_targets(targets: &[ProcTarget]) -> Vec<ProcTarget> {
    let pids = targets
        .iter()
        .map(|target| Pid::from_u32(target.pid))
        .collect::<Vec<_>>();
    let mut system = System::new();
    system.refresh_processes_specifics(
        ProcessesToUpdate::Some(&pids),
        true,
        ProcessRefreshKind::nothing().without_tasks(),
    );

    targets
        .iter()
        .filter(|target| {
            system
                .process(Pid::from_u32(target.pid))
                .is_some_and(|process| matches_target(process, target))
        })
        .cloned()
        .collect()
}

fn validate_query(query: &ProcQuery) -> Result<()> {
    if let Some(pid) = query.pid {
        ensure_valid_pid(pid, FIND_OP)?;
    }
    if let Some(ppid) = query.ppid {
        ensure_valid_pid(ppid, FIND_OP)?;
    }
    for pid in &query.pids {
        ensure_valid_pid(*pid, FIND_OP)?;
    }
    if let Some(limit) = query.limit
        && limit == 0
    {
        return Ok(());
    }
    Ok(())
}

fn ensure_valid_pid(pid: u32, op: &str) -> Result<()> {
    if pid == 0 {
        return Err(
            Error::new(ErrorKind::InvalidArgs, format!("{op} `pid` must be > 0")).with_op(op),
        );
    }
    Ok(())
}

fn process_refresh_kind() -> ProcessRefreshKind {
    ProcessRefreshKind::nothing()
        .with_user(UpdateKind::OnlyIfNotSet)
        .with_cwd(UpdateKind::OnlyIfNotSet)
        .with_cmd(UpdateKind::OnlyIfNotSet)
        .with_exe(UpdateKind::OnlyIfNotSet)
        .without_tasks()
}

fn process_to_info(process: &sysinfo::Process, users: &Users) -> ProcInfo {
    let argv = process
        .cmd()
        .iter()
        .map(|value| os_to_string(value.as_os_str()))
        .collect::<Vec<_>>();
    let cmdline = (!argv.is_empty()).then(|| argv.join(" "));
    let user = process
        .user_id()
        .and_then(|user_id| users.get_user_by_id(user_id))
        .map(|user| user.name().to_string());

    ProcInfo {
        pid: process.pid().as_u32(),
        ppid: process.parent().map(|pid| pid.as_u32()),
        name: os_to_string(process.name()),
        exe: process
            .exe()
            .map(|path| path.to_string_lossy().into_owned()),
        cwd: process
            .cwd()
            .map(|path| path.to_string_lossy().into_owned()),
        user,
        cmdline,
        argv,
        state: process_status_name(process.status()).to_string(),
        start_time_unix_ms: process.start_time().saturating_mul(1000),
    }
}

fn process_status_name(status: ProcessStatus) -> &'static str {
    match status {
        ProcessStatus::Idle => "idle",
        ProcessStatus::Run => "running",
        ProcessStatus::Sleep => "sleeping",
        ProcessStatus::Stop => "stopped",
        ProcessStatus::Zombie => "zombie",
        ProcessStatus::Tracing => "tracing",
        ProcessStatus::Dead => "dead",
        ProcessStatus::Wakekill => "wakekill",
        ProcessStatus::Waking => "waking",
        ProcessStatus::Parked => "parked",
        ProcessStatus::LockBlocked => "lock_blocked",
        ProcessStatus::UninterruptibleDiskSleep => "disk_sleep",
        ProcessStatus::Suspended => "suspended",
        ProcessStatus::Unknown(_) => "unknown",
    }
}

fn os_to_string(value: &OsStr) -> String {
    value.to_string_lossy().into_owned()
}

fn normalize_targets(targets: &[ProcTarget]) -> Vec<ProcTarget> {
    let mut map = BTreeMap::<u32, Option<u64>>::new();
    for target in targets {
        let slot = map.entry(target.pid).or_insert(target.start_time_unix_ms);
        if slot.is_none() && target.start_time_unix_ms.is_some() {
            *slot = target.start_time_unix_ms;
        }
    }
    map.into_iter()
        .map(|(pid, start_time_unix_ms)| ProcTarget {
            pid,
            start_time_unix_ms,
        })
        .collect()
}

fn matches_target(process: &sysinfo::Process, target: &ProcTarget) -> bool {
    match target.start_time_unix_ms {
        Some(expected) => process.start_time().saturating_mul(1000) == expected,
        None => true,
    }
}

fn build_missing_entry(
    pid: u32,
    signal: ProcSignal,
    missing_ok: bool,
    message: &str,
) -> ProcKillEntry {
    ProcKillEntry {
        pid,
        ok: missing_ok,
        existed: false,
        signal,
        message: Some(message.to_string()),
    }
}

fn saturating_millis(duration: Duration) -> u64 {
    u64::try_from(duration.as_millis()).unwrap_or(u64::MAX)
}
