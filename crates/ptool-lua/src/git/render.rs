use super::{FETCH_SIGNATURE, STATUS_SIGNATURE};
use mlua::{Lua, Table};
use ptool_engine::{
    GitConflictEntry, GitFetchStats, GitIntegrateResult, GitPushResult, GitRebaseResult,
    GitSignature, GitStashInfo, GitStatusEntry, GitStatusSummary, GitTagInfo,
};

pub(super) fn git_head_to_lua(lua: &Lua, head: ptool_engine::GitHeadInfo) -> mlua::Result<Table> {
    let table = lua.create_table()?;
    table.set("oid", head.oid)?;
    table.set("shorthand", head.shorthand)?;
    table.set("detached", head.detached)?;
    table.set("unborn", head.unborn)?;
    Ok(table)
}

pub(super) fn git_status_to_lua(lua: &Lua, status: GitStatusSummary) -> mlua::Result<Table> {
    let table = lua.create_table()?;
    table.set("root", status.root)?;
    table.set("branch", status.branch)?;
    table.set("head", git_head_to_lua(lua, status.head)?)?;
    table.set("upstream", status.upstream)?;
    table.set(
        "ahead",
        i64::try_from(status.ahead).map_err(|_| {
            crate::lua_error::invalid_argument(STATUS_SIGNATURE, "`ahead` is too large")
        })?,
    )?;
    table.set(
        "behind",
        i64::try_from(status.behind).map_err(|_| {
            crate::lua_error::invalid_argument(STATUS_SIGNATURE, "`behind` is too large")
        })?,
    )?;
    table.set("clean", status.clean)?;

    let entries = lua.create_table()?;
    for (index, entry) in status.entries.into_iter().enumerate() {
        entries.set(index + 1, git_status_entry_to_lua(lua, entry)?)?;
    }
    table.set("entries", entries)?;
    Ok(table)
}

pub(super) fn git_status_entry_to_lua(lua: &Lua, entry: GitStatusEntry) -> mlua::Result<Table> {
    let table = lua.create_table()?;
    table.set("path", entry.path)?;
    table.set("index_status", entry.index_status)?;
    table.set("worktree_status", entry.worktree_status)?;
    table.set("conflicted", entry.conflicted)?;
    table.set("ignored", entry.ignored)?;
    Ok(table)
}

pub(super) fn git_fetch_stats_to_lua(lua: &Lua, stats: GitFetchStats) -> mlua::Result<Table> {
    let table = lua.create_table()?;
    table.set(
        "received_objects",
        i64::try_from(stats.received_objects).map_err(|_| {
            crate::lua_error::invalid_argument(FETCH_SIGNATURE, "`received_objects` is too large")
        })?,
    )?;
    table.set(
        "indexed_objects",
        i64::try_from(stats.indexed_objects).map_err(|_| {
            crate::lua_error::invalid_argument(FETCH_SIGNATURE, "`indexed_objects` is too large")
        })?,
    )?;
    table.set(
        "local_objects",
        i64::try_from(stats.local_objects).map_err(|_| {
            crate::lua_error::invalid_argument(FETCH_SIGNATURE, "`local_objects` is too large")
        })?,
    )?;
    table.set(
        "total_objects",
        i64::try_from(stats.total_objects).map_err(|_| {
            crate::lua_error::invalid_argument(FETCH_SIGNATURE, "`total_objects` is too large")
        })?,
    )?;
    table.set(
        "received_bytes",
        i64::try_from(stats.received_bytes).map_err(|_| {
            crate::lua_error::invalid_argument(FETCH_SIGNATURE, "`received_bytes` is too large")
        })?,
    )?;
    let updated_refs = lua.create_table()?;
    for (index, reference) in stats.updated_refs.into_iter().enumerate() {
        updated_refs.set(index + 1, reference)?;
    }
    table.set("updated_refs", updated_refs)?;
    Ok(table)
}

pub(super) fn git_push_result_to_lua(lua: &Lua, result: GitPushResult) -> mlua::Result<Table> {
    let table = lua.create_table()?;
    let refspecs = lua.create_table()?;
    for (index, refspec) in result.refspecs.into_iter().enumerate() {
        refspecs.set(index + 1, refspec)?;
    }
    table.set("refspecs", refspecs)?;
    let rejected = lua.create_table()?;
    for (index, rejection) in result.rejected.into_iter().enumerate() {
        let item = lua.create_table()?;
        item.set("reference", rejection.reference)?;
        item.set("message", rejection.message)?;
        rejected.set(index + 1, item)?;
    }
    let ok = rejected.raw_len() == 0;
    table.set("rejected", rejected)?;
    table.set("ok", ok)?;
    Ok(table)
}

pub(super) fn git_tags_to_lua(lua: &Lua, tags: Vec<GitTagInfo>) -> mlua::Result<Table> {
    let table = lua.create_table()?;
    for (index, tag) in tags.into_iter().enumerate() {
        table.set(index + 1, git_tag_to_lua(lua, tag)?)?;
    }
    Ok(table)
}

pub(super) fn git_tag_to_lua(lua: &Lua, tag: GitTagInfo) -> mlua::Result<Table> {
    let table = lua.create_table()?;
    table.set("name", tag.name)?;
    table.set("oid", tag.oid)?;
    table.set("target_oid", tag.target_oid)?;
    table.set("target_kind", tag.target_kind)?;
    table.set("annotated", tag.annotated)?;
    table.set("message", tag.message)?;
    table.set(
        "tagger",
        match tag.tagger {
            Some(tagger) => Some(git_signature_to_lua(lua, tagger)?),
            None => None,
        },
    )?;
    Ok(table)
}

fn git_signature_to_lua(lua: &Lua, signature: GitSignature) -> mlua::Result<Table> {
    let table = lua.create_table()?;
    table.set("name", signature.name)?;
    table.set("email", signature.email)?;
    table.set("time_seconds", signature.time_seconds)?;
    table.set("offset_minutes", signature.offset_minutes)?;
    Ok(table)
}

pub(super) fn git_integrate_result_to_lua(
    lua: &Lua,
    result: GitIntegrateResult,
) -> mlua::Result<Table> {
    let table = lua.create_table()?;
    table.set("outcome", result.outcome)?;
    table.set("oid", result.oid)?;
    table.set("conflicts", git_conflicts_to_lua(lua, result.conflicts)?)?;
    Ok(table)
}

pub(super) fn git_conflicts_to_lua(
    lua: &Lua,
    conflicts: Vec<GitConflictEntry>,
) -> mlua::Result<Table> {
    let table = lua.create_table()?;
    for (index, conflict) in conflicts.into_iter().enumerate() {
        let item = lua.create_table()?;
        item.set("path", conflict.path)?;
        item.set("ancestor", conflict.ancestor)?;
        item.set("ours", conflict.ours)?;
        item.set("theirs", conflict.theirs)?;
        table.set(index + 1, item)?;
    }
    Ok(table)
}

pub(super) fn git_stashes_to_lua(lua: &Lua, stashes: Vec<GitStashInfo>) -> mlua::Result<Table> {
    let table = lua.create_table()?;
    for (index, stash) in stashes.into_iter().enumerate() {
        let item = lua.create_table()?;
        item.set(
            "index",
            i64::try_from(stash.index).map_err(|_| {
                crate::lua_error::invalid_argument(
                    "ptool.git.Repo:stashes()",
                    "stash index is too large",
                )
            })?,
        )?;
        item.set("message", stash.message)?;
        item.set("oid", stash.oid)?;
        table.set(index + 1, item)?;
    }
    Ok(table)
}

pub(super) fn git_rebase_result_to_lua(lua: &Lua, result: GitRebaseResult) -> mlua::Result<Table> {
    let table = lua.create_table()?;
    table.set("outcome", result.outcome)?;
    table.set("oid", result.oid)?;
    table.set("conflicts", git_conflicts_to_lua(lua, result.conflicts)?)?;
    table.set(
        "current",
        result
            .current
            .map(|value| {
                i64::try_from(value).map_err(|_| {
                    crate::lua_error::invalid_argument(
                        "ptool.git.Repo:rebase(options)",
                        "operation index is too large",
                    )
                })
            })
            .transpose()?,
    )?;
    table.set(
        "total",
        i64::try_from(result.total).map_err(|_| {
            crate::lua_error::invalid_argument(
                "ptool.git.Repo:rebase(options)",
                "operation count is too large",
            )
        })?,
    )?;
    Ok(table)
}
