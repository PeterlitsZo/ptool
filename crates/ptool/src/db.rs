use mlua::{Function, Lua, Table, UserData, UserDataMethods, Value};
use sqlx::any::{AnyArguments, AnyRow, AnyTransactionManager, AnyTypeInfoKind};
use sqlx::{Any, AnyConnection};
use sqlx::{Arguments, Column, Connection, Row, TransactionManager, ValueRef};
use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::rc::Rc;
use tokio::runtime::Runtime;

const CONNECT_SIGNATURE: &str = "ptool.db.connect(url_or_options)";
const QUERY_SIGNATURE: &str = "ptool.db.Connection:query(sql, params?)";
const QUERY_ONE_SIGNATURE: &str = "ptool.db.Connection:query_one(sql, params?)";
const SCALAR_SIGNATURE: &str = "ptool.db.Connection:scalar(sql, params?)";
const EXECUTE_SIGNATURE: &str = "ptool.db.Connection:execute(sql, params?)";
const TRANSACTION_SIGNATURE: &str = "ptool.db.Connection:transaction(fn)";

#[derive(Clone)]
pub(crate) struct LuaDbConnection {
    runtime: Rc<Runtime>,
    state: Rc<RefCell<ConnectionState>>,
}

#[derive(Clone)]
struct LuaDbTransaction {
    runtime: Rc<Runtime>,
    state: Rc<RefCell<ConnectionState>>,
    active: Rc<Cell<bool>>,
}

struct ConnectionState {
    conn: Option<AnyConnection>,
    backend: DbBackend,
    tx_active: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DbBackend {
    Sqlite,
    Postgres,
    MySql,
}

enum ParsedParams {
    Positional(Vec<DbBindValue>),
    Named(BTreeMap<String, DbBindValue>),
}

enum DbBindValue {
    Boolean(bool),
    Integer(i64),
    Number(f64),
    String(String),
}

enum DbValue {
    Nil,
    Boolean(bool),
    Integer(i64),
    Number(f64),
    String(Vec<u8>),
}

pub(crate) fn connect(
    value: Value,
    current_dir: &Path,
    runtime: Rc<Runtime>,
) -> mlua::Result<LuaDbConnection> {
    let url = parse_connect_value(value)?;
    let backend = detect_backend(&url)?;
    let url = normalize_database_url(&url, backend, current_dir)?;
    let conn = runtime
        .block_on(AnyConnection::connect(&url))
        .map_err(|err| db_error(CONNECT_SIGNATURE, err))?;

    Ok(LuaDbConnection {
        runtime,
        state: Rc::new(RefCell::new(ConnectionState {
            conn: Some(conn),
            backend,
            tx_active: false,
        })),
    })
}

impl UserData for LuaDbConnection {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method(
            "query",
            |lua, this, (sql, params): (String, Option<Value>)| this.query(lua, &sql, params),
        );

        methods.add_method(
            "query_one",
            |lua, this, (sql, params): (String, Option<Value>)| this.query_one(lua, &sql, params),
        );

        methods.add_method(
            "scalar",
            |lua, this, (sql, params): (String, Option<Value>)| this.scalar(lua, &sql, params),
        );

        methods.add_method(
            "execute",
            |lua, this, (sql, params): (String, Option<Value>)| this.execute(lua, &sql, params),
        );

        methods.add_method("transaction", |lua, this, callback: Function| {
            this.transaction(lua, callback)
        });

        methods.add_method("close", |_, this, ()| this.close());
    }
}

impl UserData for LuaDbTransaction {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method(
            "query",
            |lua, this, (sql, params): (String, Option<Value>)| {
                this.ensure_active()?;
                this.query(lua, &sql, params)
            },
        );

        methods.add_method(
            "query_one",
            |lua, this, (sql, params): (String, Option<Value>)| {
                this.ensure_active()?;
                this.query_one(lua, &sql, params)
            },
        );

        methods.add_method(
            "scalar",
            |lua, this, (sql, params): (String, Option<Value>)| {
                this.ensure_active()?;
                this.scalar(lua, &sql, params)
            },
        );

        methods.add_method(
            "execute",
            |lua, this, (sql, params): (String, Option<Value>)| {
                this.ensure_active()?;
                this.execute(lua, &sql, params)
            },
        );
    }
}

impl LuaDbConnection {
    fn query(&self, lua: &Lua, sql: &str, params: Option<Value>) -> mlua::Result<Table> {
        self.with_connection_mut(false, "query", |runtime, backend, conn| {
            let query = build_query(sql, params, backend, QUERY_SIGNATURE)?;
            let rows: Vec<AnyRow> = runtime
                .block_on(async move { query.fetch_all(conn).await })
                .map_err(|err| db_error(QUERY_SIGNATURE, err))?;
            query_rows_to_lua(lua, rows)
        })
    }

    fn query_one(&self, lua: &Lua, sql: &str, params: Option<Value>) -> mlua::Result<Value> {
        self.with_connection_mut(false, "query_one", |runtime, backend, conn| {
            let query = build_query(sql, params, backend, QUERY_ONE_SIGNATURE)?;
            let row: Option<AnyRow> = runtime
                .block_on(async move { query.fetch_optional(conn).await })
                .map_err(|err| db_error(QUERY_ONE_SIGNATURE, err))?;

            match row {
                Some(row) => Ok(Value::Table(row_to_lua(lua, &row)?)),
                None => Ok(Value::Nil),
            }
        })
    }

    fn scalar(&self, lua: &Lua, sql: &str, params: Option<Value>) -> mlua::Result<Value> {
        self.with_connection_mut(false, "scalar", |runtime, backend, conn| {
            let query = build_query(sql, params, backend, SCALAR_SIGNATURE)?;
            let row: Option<AnyRow> = runtime
                .block_on(async move { query.fetch_optional(conn).await })
                .map_err(|err| db_error(SCALAR_SIGNATURE, err))?;

            let Some(row) = row else {
                return Ok(Value::Nil);
            };
            if row.columns().is_empty() {
                return Err(mlua::Error::runtime(format!(
                    "{SCALAR_SIGNATURE} expected at least 1 column"
                )));
            }
            db_value_to_lua(lua, row_value(&row, 0)?)
        })
    }

    fn execute(&self, lua: &Lua, sql: &str, params: Option<Value>) -> mlua::Result<Table> {
        self.with_connection_mut(false, "execute", |runtime, backend, conn| {
            let query = build_query(sql, params, backend, EXECUTE_SIGNATURE)?;
            let result: sqlx::any::AnyQueryResult = runtime
                .block_on(async move { query.execute(conn).await })
                .map_err(|err| db_error(EXECUTE_SIGNATURE, err))?;
            let table = lua.create_table()?;
            table.set(
                "rows_affected",
                u64_to_i64(result.rows_affected(), EXECUTE_SIGNATURE)?,
            )?;
            Ok(table)
        })
    }

    fn transaction(&self, _lua: &Lua, callback: Function) -> mlua::Result<Value> {
        {
            let mut state = self.state.borrow_mut();
            ensure_connection_open(&state, TRANSACTION_SIGNATURE)?;
            if state.tx_active {
                return Err(mlua::Error::runtime(format!(
                    "{TRANSACTION_SIGNATURE} does not support nested transactions"
                )));
            }
            let conn = state.conn.as_mut().expect("checked above");
            self.runtime
                .block_on(AnyTransactionManager::begin(conn, None))
                .map_err(|err| db_error(TRANSACTION_SIGNATURE, err))?;
            state.tx_active = true;
        }

        let active = Rc::new(Cell::new(true));
        let tx = LuaDbTransaction {
            runtime: Rc::clone(&self.runtime),
            state: Rc::clone(&self.state),
            active: Rc::clone(&active),
        };
        let callback_result = callback.call::<Value>(tx);

        active.set(false);

        let finalize_result = {
            let mut state = self.state.borrow_mut();
            let conn = state.conn.as_mut().ok_or_else(|| {
                mlua::Error::runtime(format!(
                    "{TRANSACTION_SIGNATURE} internal error: connection closed during transaction"
                ))
            })?;
            let result = if callback_result.is_ok() {
                self.runtime
                    .block_on(AnyTransactionManager::commit(conn))
                    .map_err(|err| db_error(TRANSACTION_SIGNATURE, err))
            } else {
                self.runtime
                    .block_on(AnyTransactionManager::rollback(conn))
                    .map_err(|err| db_error(TRANSACTION_SIGNATURE, err))
            };
            state.tx_active = false;
            result
        };

        match (callback_result, finalize_result) {
            (Ok(value), Ok(())) => Ok(value),
            (Err(err), Ok(())) => Err(err),
            (Ok(_), Err(err)) => Err(err),
            (Err(callback_err), Err(finalize_err)) => Err(mlua::Error::runtime(format!(
                "{TRANSACTION_SIGNATURE} callback failed: {callback_err}; rollback failed: {finalize_err}"
            ))),
        }
    }

    fn close(&self) -> mlua::Result<()> {
        let conn = {
            let mut state = self.state.borrow_mut();
            if state.tx_active {
                return Err(mlua::Error::runtime(
                    "ptool.db.Connection:close() cannot close during an active transaction",
                ));
            }
            state.conn.take()
        };

        if let Some(conn) = conn {
            self.runtime
                .block_on(async move { AnyConnection::close(conn).await })
                .map_err(|err| {
                    mlua::Error::runtime(format!("ptool.db.Connection:close() failed: {err}"))
                })?;
        }

        Ok(())
    }

    fn with_connection_mut<R>(
        &self,
        allow_transaction: bool,
        op_name: &str,
        f: impl FnOnce(&Runtime, DbBackend, &mut AnyConnection) -> mlua::Result<R>,
    ) -> mlua::Result<R> {
        let mut state = self.state.borrow_mut();
        ensure_connection_open(&state, op_name)?;
        if state.tx_active && !allow_transaction {
            return Err(mlua::Error::runtime(format!(
                "ptool.db.Connection:{op_name}() cannot be used while a transaction callback is active"
            )));
        }
        let backend = state.backend;
        let conn = state.conn.as_mut().expect("checked above");
        f(&self.runtime, backend, conn)
    }
}

impl LuaDbTransaction {
    fn ensure_active(&self) -> mlua::Result<()> {
        if self.active.get() {
            Ok(())
        } else {
            Err(mlua::Error::runtime(
                "ptool.db transaction handle is no longer active",
            ))
        }
    }

    fn query(&self, lua: &Lua, sql: &str, params: Option<Value>) -> mlua::Result<Table> {
        self.with_connection_mut(|runtime, backend, conn| {
            let query = build_query(sql, params, backend, QUERY_SIGNATURE)?;
            let rows: Vec<AnyRow> = runtime
                .block_on(async move { query.fetch_all(conn).await })
                .map_err(|err| db_error(QUERY_SIGNATURE, err))?;
            query_rows_to_lua(lua, rows)
        })
    }

    fn query_one(&self, lua: &Lua, sql: &str, params: Option<Value>) -> mlua::Result<Value> {
        self.with_connection_mut(|runtime, backend, conn| {
            let query = build_query(sql, params, backend, QUERY_ONE_SIGNATURE)?;
            let row: Option<AnyRow> = runtime
                .block_on(async move { query.fetch_optional(conn).await })
                .map_err(|err| db_error(QUERY_ONE_SIGNATURE, err))?;

            match row {
                Some(row) => Ok(Value::Table(row_to_lua(lua, &row)?)),
                None => Ok(Value::Nil),
            }
        })
    }

    fn scalar(&self, lua: &Lua, sql: &str, params: Option<Value>) -> mlua::Result<Value> {
        self.with_connection_mut(|runtime, backend, conn| {
            let query = build_query(sql, params, backend, SCALAR_SIGNATURE)?;
            let row: Option<AnyRow> = runtime
                .block_on(async move { query.fetch_optional(conn).await })
                .map_err(|err| db_error(SCALAR_SIGNATURE, err))?;

            let Some(row) = row else {
                return Ok(Value::Nil);
            };
            if row.columns().is_empty() {
                return Err(mlua::Error::runtime(format!(
                    "{SCALAR_SIGNATURE} expected at least 1 column"
                )));
            }
            db_value_to_lua(lua, row_value(&row, 0)?)
        })
    }

    fn execute(&self, lua: &Lua, sql: &str, params: Option<Value>) -> mlua::Result<Table> {
        self.with_connection_mut(|runtime, backend, conn| {
            let query = build_query(sql, params, backend, EXECUTE_SIGNATURE)?;
            let result: sqlx::any::AnyQueryResult = runtime
                .block_on(async move { query.execute(conn).await })
                .map_err(|err| db_error(EXECUTE_SIGNATURE, err))?;
            let table = lua.create_table()?;
            table.set(
                "rows_affected",
                u64_to_i64(result.rows_affected(), EXECUTE_SIGNATURE)?,
            )?;
            Ok(table)
        })
    }

    fn with_connection_mut<R>(
        &self,
        f: impl FnOnce(&Runtime, DbBackend, &mut AnyConnection) -> mlua::Result<R>,
    ) -> mlua::Result<R> {
        let mut state = self.state.borrow_mut();
        ensure_connection_open(&state, "transaction")?;
        if !state.tx_active {
            return Err(mlua::Error::runtime(
                "ptool.db transaction handle is no longer active",
            ));
        }
        let backend = state.backend;
        let conn = state.conn.as_mut().expect("checked above");
        f(&self.runtime, backend, conn)
    }
}

fn parse_connect_value(value: Value) -> mlua::Result<String> {
    match value {
        Value::String(text) => Ok(text.to_str()?.to_string()),
        Value::Table(options) => {
            let Some(url) = options.get::<Option<String>>("url")? else {
                return Err(mlua::Error::runtime(format!(
                    "{CONNECT_SIGNATURE} requires string `url` when called with a table"
                )));
            };
            Ok(url)
        }
        _ => Err(mlua::Error::runtime(format!(
            "{CONNECT_SIGNATURE} expects a string or a table with `url`"
        ))),
    }
}

fn detect_backend(url: &str) -> mlua::Result<DbBackend> {
    if url.starts_with("sqlite:") {
        Ok(DbBackend::Sqlite)
    } else if url.starts_with("postgres://") || url.starts_with("postgresql://") {
        Ok(DbBackend::Postgres)
    } else if url.starts_with("mysql://") {
        Ok(DbBackend::MySql)
    } else {
        Err(mlua::Error::runtime(format!(
            "{CONNECT_SIGNATURE} unsupported database url `{url}`"
        )))
    }
}

fn normalize_database_url(
    url: &str,
    backend: DbBackend,
    current_dir: &Path,
) -> mlua::Result<String> {
    if backend != DbBackend::Sqlite {
        return Ok(url.to_string());
    }
    if url == "sqlite::memory:" || url == "sqlite://:memory:" || url.starts_with("sqlite://?") {
        return Ok(url.to_string());
    }

    let (prefix, rest) = if let Some(rest) = url.strip_prefix("sqlite://") {
        ("sqlite://", rest)
    } else if let Some(rest) = url.strip_prefix("sqlite:") {
        ("sqlite:", rest)
    } else {
        return Ok(url.to_string());
    };

    if rest.is_empty() || rest.starts_with(':') {
        return Ok(url.to_string());
    }

    let path_end = rest.find(['?', '#']).unwrap_or(rest.len());
    let (path_part, suffix) = rest.split_at(path_end);

    let base_url = if path_part.is_empty() {
        url.to_string()
    } else if path_part.starts_with('/') {
        format!("sqlite://{path_part}{suffix}")
    } else {
        let absolute = current_dir.join(path_part);
        let absolute = pathbuf_to_string(&absolute, CONNECT_SIGNATURE)?;
        let _ = prefix;
        format!("sqlite://{absolute}{suffix}")
    };

    Ok(ensure_sqlite_create_mode(&base_url))
}

fn ensure_sqlite_create_mode(url: &str) -> String {
    if !url.starts_with("sqlite:") {
        return url.to_string();
    }
    if url.contains("?mode=") || url.starts_with("sqlite::memory:") || url.starts_with("sqlite://?")
    {
        return url.to_string();
    }
    if url.contains('?') {
        format!("{url}&mode=rwc")
    } else {
        format!("{url}?mode=rwc")
    }
}

fn build_query(
    sql: &str,
    params: Option<Value>,
    backend: DbBackend,
    signature: &str,
) -> mlua::Result<sqlx::query::Query<'static, Any, AnyArguments<'static>>> {
    let params = parse_params(params, signature)?;
    let (rewritten_sql, args) = rewrite_sql_and_build_args(sql, params, backend, signature)?;
    Ok(sqlx::query_with(
        Box::leak(rewritten_sql.into_boxed_str()),
        args,
    ))
}

fn parse_params(params: Option<Value>, signature: &str) -> mlua::Result<Option<ParsedParams>> {
    let Some(params) = params else {
        return Ok(None);
    };
    match params {
        Value::Nil => Ok(None),
        Value::Table(table) => {
            if is_sequence_table(&table)? {
                let len = table.raw_len();
                let mut values = Vec::with_capacity(len);
                for index in 1..=len {
                    let value = table.raw_get(index)?;
                    values.push(parse_bind_value(value, signature, Some(index))?);
                }
                Ok(Some(ParsedParams::Positional(values)))
            } else {
                let mut values = BTreeMap::new();
                for pair in table.pairs::<Value, Value>() {
                    let (key, value) = pair?;
                    let key = match key {
                        Value::String(key) => key.to_str()?.to_string(),
                        _ => {
                            return Err(mlua::Error::runtime(format!(
                                "{signature} named params keys must be strings"
                            )));
                        }
                    };
                    if key.is_empty() {
                        return Err(mlua::Error::runtime(format!(
                            "{signature} named params keys must not be empty"
                        )));
                    }
                    values.insert(key, parse_bind_value(value, signature, None)?);
                }
                Ok(Some(ParsedParams::Named(values)))
            }
        }
        _ => Err(mlua::Error::runtime(format!(
            "{signature} `params` must be a table"
        ))),
    }
}

fn parse_bind_value(
    value: Value,
    signature: &str,
    index: Option<usize>,
) -> mlua::Result<DbBindValue> {
    match value {
        Value::Boolean(value) => Ok(DbBindValue::Boolean(value)),
        Value::Integer(value) => Ok(DbBindValue::Integer(value)),
        Value::Number(value) => {
            if !value.is_finite() {
                return Err(mlua::Error::runtime(format!(
                    "{signature} parameter must be a finite number"
                )));
            }
            Ok(DbBindValue::Number(value))
        }
        Value::String(value) => Ok(DbBindValue::String(value.to_str()?.to_string())),
        Value::Nil => Err(mlua::Error::runtime(match index {
            Some(index) => format!("{signature} params[{index}] does not support nil"),
            None => format!("{signature} named params do not support nil"),
        })),
        _ => Err(mlua::Error::runtime(match index {
            Some(index) => {
                format!("{signature} params[{index}] only supports boolean/integer/number/string")
            }
            None => format!("{signature} named params only support boolean/integer/number/string"),
        })),
    }
}

fn rewrite_sql_and_build_args(
    sql: &str,
    params: Option<ParsedParams>,
    backend: DbBackend,
    signature: &str,
) -> mlua::Result<(String, AnyArguments<'static>)> {
    let mut rewritten = String::with_capacity(sql.len() + 16);
    let mut args = AnyArguments::default();
    let mut positional_index = 0usize;
    let mut used_named = BTreeSet::new();
    let named = match &params {
        Some(ParsedParams::Named(values)) => Some(values),
        _ => None,
    };
    let positional = match &params {
        Some(ParsedParams::Positional(values)) => Some(values),
        _ => None,
    };

    let chars: Vec<char> = sql.chars().collect();
    let mut i = 0usize;
    let mut bind_index = 0usize;
    while i < chars.len() {
        let ch = chars[i];

        if ch == '\'' {
            i = copy_quoted(&chars, i, '\'', &mut rewritten);
            continue;
        }
        if ch == '"' {
            i = copy_quoted(&chars, i, '"', &mut rewritten);
            continue;
        }
        if ch == '`' {
            i = copy_quoted(&chars, i, '`', &mut rewritten);
            continue;
        }
        if ch == '-' && i + 1 < chars.len() && chars[i + 1] == '-' {
            i = copy_line_comment(&chars, i, &mut rewritten);
            continue;
        }
        if ch == '/' && i + 1 < chars.len() && chars[i + 1] == '*' {
            i = copy_block_comment(&chars, i, &mut rewritten)?;
            continue;
        }

        match &params {
            Some(ParsedParams::Positional(values)) if ch == '?' => {
                let value = values.get(positional_index).ok_or_else(|| {
                    mlua::Error::runtime(format!(
                        "{signature} expected at least {} positional params",
                        positional_index + 1
                    ))
                })?;
                positional_index += 1;
                bind_index += 1;
                rewritten.push_str(&placeholder_text(backend, bind_index));
                push_bind_value(&mut args, value, signature)?;
                i += 1;
            }
            Some(ParsedParams::Named(values))
                if ch == ':'
                    && i + 1 < chars.len()
                    && chars[i + 1] != ':'
                    && is_name_start(chars[i + 1]) =>
            {
                let start = i + 1;
                let mut end = start + 1;
                while end < chars.len() && is_name_continue(chars[end]) {
                    end += 1;
                }
                let name: String = chars[start..end].iter().collect();
                let value = values.get(&name).ok_or_else(|| {
                    mlua::Error::runtime(format!("{signature} missing named param `:{name}`"))
                })?;
                used_named.insert(name);
                bind_index += 1;
                rewritten.push_str(&placeholder_text(backend, bind_index));
                push_bind_value(&mut args, value, signature)?;
                i = end;
            }
            None if ch == '?' => {
                return Err(mlua::Error::runtime(format!(
                    "{signature} SQL contains placeholders but no params were provided"
                )));
            }
            None if ch == ':'
                && i + 1 < chars.len()
                && chars[i + 1] != ':'
                && is_name_start(chars[i + 1]) =>
            {
                return Err(mlua::Error::runtime(format!(
                    "{signature} SQL contains named placeholders but no params were provided"
                )));
            }
            _ => {
                rewritten.push(ch);
                i += 1;
            }
        }
    }

    if let Some(values) = positional
        && positional_index != values.len()
    {
        return Err(mlua::Error::runtime(format!(
            "{signature} received {} positional params but SQL used {}",
            values.len(),
            positional_index
        )));
    }

    if let Some(values) = named {
        let unused: Vec<&str> = values
            .keys()
            .filter(|key| !used_named.contains(*key))
            .map(String::as_str)
            .collect();
        if !unused.is_empty() {
            return Err(mlua::Error::runtime(format!(
                "{signature} has unused named params: {}",
                unused.join(", ")
            )));
        }
    }

    Ok((rewritten, args))
}

fn push_bind_value(
    args: &mut AnyArguments<'static>,
    value: &DbBindValue,
    signature: &str,
) -> mlua::Result<()> {
    let result = match value {
        DbBindValue::Boolean(value) => args.add(*value),
        DbBindValue::Integer(value) => args.add(*value),
        DbBindValue::Number(value) => args.add(*value),
        DbBindValue::String(value) => args.add(value.clone()),
    };
    result
        .map_err(|err| mlua::Error::runtime(format!("{signature} failed to bind parameter: {err}")))
}

fn placeholder_text(backend: DbBackend, bind_index: usize) -> String {
    match backend {
        DbBackend::Sqlite | DbBackend::MySql => "?".to_string(),
        DbBackend::Postgres => format!("${bind_index}"),
    }
}

fn copy_quoted(chars: &[char], start: usize, quote: char, output: &mut String) -> usize {
    output.push(chars[start]);
    let mut i = start + 1;
    while i < chars.len() {
        let ch = chars[i];
        output.push(ch);
        i += 1;
        if ch == quote {
            if i < chars.len() && chars[i] == quote && quote != '`' {
                output.push(chars[i]);
                i += 1;
                continue;
            }
            break;
        }
    }
    i
}

fn copy_line_comment(chars: &[char], start: usize, output: &mut String) -> usize {
    let mut i = start;
    while i < chars.len() {
        let ch = chars[i];
        output.push(ch);
        i += 1;
        if ch == '\n' {
            break;
        }
    }
    i
}

fn copy_block_comment(chars: &[char], start: usize, output: &mut String) -> mlua::Result<usize> {
    let mut i = start;
    while i < chars.len() {
        let ch = chars[i];
        output.push(ch);
        i += 1;
        if ch == '*' && i < chars.len() && chars[i] == '/' {
            output.push(chars[i]);
            return Ok(i + 1);
        }
    }
    Err(mlua::Error::runtime(
        "ptool.db SQL parse failed: unterminated block comment",
    ))
}

fn is_name_start(ch: char) -> bool {
    ch == '_' || ch.is_ascii_alphabetic()
}

fn is_name_continue(ch: char) -> bool {
    ch == '_' || ch.is_ascii_alphanumeric()
}

fn is_sequence_table(table: &Table) -> mlua::Result<bool> {
    let len = table.raw_len();
    let mut count = 0usize;
    for pair in table.pairs::<Value, Value>() {
        let (key, _) = pair?;
        match key {
            Value::Integer(index) if index >= 1 => {
                let Ok(index) = usize::try_from(index) else {
                    return Ok(false);
                };
                if index > len {
                    return Ok(false);
                }
            }
            _ => return Ok(false),
        }
        count += 1;
    }
    Ok(count == len)
}

fn query_rows_to_lua(lua: &Lua, rows: Vec<AnyRow>) -> mlua::Result<Table> {
    let columns = query_columns(lua, rows.first())?;
    let rows_table = lua.create_table()?;
    for (index, row) in rows.iter().enumerate() {
        rows_table.raw_set(index + 1, row_to_lua(lua, row)?)?;
    }

    let result = lua.create_table()?;
    result.set("rows", rows_table)?;
    result.set("columns", columns)?;
    result.set("row_count", usize_to_i64(rows.len(), QUERY_SIGNATURE)?)?;
    Ok(result)
}

fn query_columns(lua: &Lua, row: Option<&AnyRow>) -> mlua::Result<Table> {
    let columns = lua.create_table()?;
    let Some(row) = row else {
        return Ok(columns);
    };
    for (index, column) in row.columns().iter().enumerate() {
        columns.raw_set(index + 1, column.name())?;
    }
    Ok(columns)
}

fn row_to_lua(lua: &Lua, row: &AnyRow) -> mlua::Result<Table> {
    let table = lua.create_table()?;
    let mut seen = BTreeSet::new();
    for (index, column) in row.columns().iter().enumerate() {
        let name = column.name();
        if !seen.insert(name.to_string()) {
            return Err(mlua::Error::runtime(format!(
                "ptool.db query result contains duplicate column name `{name}`; use SQL aliases to disambiguate"
            )));
        }
        table.raw_set(name, db_value_to_lua(lua, row_value(row, index)?)?)?;
    }
    Ok(table)
}

fn row_value(row: &AnyRow, index: usize) -> mlua::Result<DbValue> {
    let raw = row.try_get_raw(index).map_err(|err| {
        mlua::Error::runtime(format!("ptool.db failed to read column {index}: {err}"))
    })?;

    if raw.is_null() {
        return Ok(DbValue::Nil);
    }

    let value = match raw.type_info().kind() {
        AnyTypeInfoKind::Null => DbValue::Nil,
        AnyTypeInfoKind::Bool => DbValue::Boolean(
            row.try_get::<bool, _>(index)
                .map_err(row_decode_err(index))?,
        ),
        AnyTypeInfoKind::SmallInt => DbValue::Integer(i64::from(
            row.try_get::<i16, _>(index)
                .map_err(row_decode_err(index))?,
        )),
        AnyTypeInfoKind::Integer => DbValue::Integer(i64::from(
            row.try_get::<i32, _>(index)
                .map_err(row_decode_err(index))?,
        )),
        AnyTypeInfoKind::BigInt => DbValue::Integer(
            row.try_get::<i64, _>(index)
                .map_err(row_decode_err(index))?,
        ),
        AnyTypeInfoKind::Real => DbValue::Number(f64::from(
            row.try_get::<f32, _>(index)
                .map_err(row_decode_err(index))?,
        )),
        AnyTypeInfoKind::Double => DbValue::Number(
            row.try_get::<f64, _>(index)
                .map_err(row_decode_err(index))?,
        ),
        AnyTypeInfoKind::Text => DbValue::String(
            row.try_get::<String, _>(index)
                .map_err(row_decode_err(index))?
                .into_bytes(),
        ),
        AnyTypeInfoKind::Blob => DbValue::String(
            row.try_get::<Vec<u8>, _>(index)
                .map_err(row_decode_err(index))?,
        ),
    };
    Ok(value)
}

fn row_decode_err(index: usize) -> impl FnOnce(sqlx::Error) -> mlua::Error {
    move |err| mlua::Error::runtime(format!("ptool.db failed to decode column {index}: {err}"))
}

fn db_value_to_lua(lua: &Lua, value: DbValue) -> mlua::Result<Value> {
    match value {
        DbValue::Nil => Ok(Value::Nil),
        DbValue::Boolean(value) => Ok(Value::Boolean(value)),
        DbValue::Integer(value) => Ok(Value::Integer(value)),
        DbValue::Number(value) => Ok(Value::Number(value)),
        DbValue::String(value) => Ok(Value::String(lua.create_string(&value)?)),
    }
}

fn ensure_connection_open(state: &ConnectionState, signature: &str) -> mlua::Result<()> {
    if state.conn.is_some() {
        Ok(())
    } else {
        Err(mlua::Error::runtime(format!(
            "{signature} cannot be used after the connection is closed"
        )))
    }
}

fn db_error(signature: &str, err: sqlx::Error) -> mlua::Error {
    mlua::Error::runtime(format!("{signature} failed: {err}"))
}

fn usize_to_i64(value: usize, signature: &str) -> mlua::Result<i64> {
    i64::try_from(value).map_err(|_| {
        mlua::Error::runtime(format!("{signature} produced a value that is too large"))
    })
}

fn u64_to_i64(value: u64, signature: &str) -> mlua::Result<i64> {
    i64::try_from(value).map_err(|_| {
        mlua::Error::runtime(format!("{signature} produced a value that is too large"))
    })
}

fn pathbuf_to_string(path: &Path, signature: &str) -> mlua::Result<String> {
    path.to_str()
        .map(std::string::ToString::to_string)
        .ok_or_else(|| mlua::Error::runtime(format!("{signature} path contains invalid UTF-8")))
}
