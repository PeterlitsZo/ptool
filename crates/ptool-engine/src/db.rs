use crate::{Error, ErrorKind, Result};
use sqlx::any::{AnyArguments, AnyRow, AnyTransactionManager, AnyTypeInfoKind};
use sqlx::{Any, AnyConnection};
use sqlx::{Arguments, Column, Connection, Row, TransactionManager, ValueRef};
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::rc::Rc;
use std::sync::{Arc, Once};
use tokio::runtime::Runtime;

static DB_DRIVERS: Once = Once::new();

#[derive(Clone)]
pub struct DbConnection {
    runtime: Arc<Runtime>,
    state: Rc<RefCell<ConnectionState>>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum DbParams {
    Positional(Vec<DbBindValue>),
    Named(BTreeMap<String, DbBindValue>),
}

#[derive(Clone, Debug, PartialEq)]
pub enum DbBindValue {
    Boolean(bool),
    Integer(i64),
    Number(f64),
    String(String),
}

#[derive(Clone, Debug, PartialEq)]
pub enum DbValue {
    Nil,
    Boolean(bool),
    Integer(i64),
    Number(f64),
    String(Vec<u8>),
}

pub type DbRow = BTreeMap<String, DbValue>;

#[derive(Clone, Debug, PartialEq)]
pub struct DbQueryResult {
    pub rows: Vec<DbRow>,
    pub columns: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DbExecuteResult {
    pub rows_affected: u64,
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

pub(crate) fn connect(
    runtime: Arc<Runtime>,
    url: &str,
    current_dir: &Path,
) -> Result<DbConnection> {
    DB_DRIVERS.call_once(sqlx::any::install_default_drivers);

    let backend = detect_backend(url)?;
    let url = normalize_database_url(url, backend, current_dir)?;
    let conn = runtime
        .block_on(AnyConnection::connect(&url))
        .map_err(|err| db_error("failed to connect", err))?;

    Ok(DbConnection {
        runtime,
        state: Rc::new(RefCell::new(ConnectionState {
            conn: Some(conn),
            backend,
            tx_active: false,
        })),
    })
}

impl DbConnection {
    pub fn query(&self, sql: &str, params: Option<DbParams>) -> Result<DbQueryResult> {
        self.with_connection_mut(false, "query", |runtime, backend, conn| {
            let query = build_query(sql, params, backend)?;
            let rows: Vec<AnyRow> = runtime
                .block_on(async move { query.fetch_all(conn).await })
                .map_err(|err| db_error("failed to execute query", err))?;
            query_result(rows)
        })
    }

    pub fn query_one(&self, sql: &str, params: Option<DbParams>) -> Result<Option<DbRow>> {
        self.with_connection_mut(false, "query_one", |runtime, backend, conn| {
            let query = build_query(sql, params, backend)?;
            let row: Option<AnyRow> = runtime
                .block_on(async move { query.fetch_optional(conn).await })
                .map_err(|err| db_error("failed to execute query", err))?;
            row.map(|row| row_to_db(&row)).transpose()
        })
    }

    pub fn scalar(&self, sql: &str, params: Option<DbParams>) -> Result<Option<DbValue>> {
        self.with_connection_mut(false, "scalar", |runtime, backend, conn| {
            let query = build_query(sql, params, backend)?;
            let row: Option<AnyRow> = runtime
                .block_on(async move { query.fetch_optional(conn).await })
                .map_err(|err| db_error("failed to execute scalar query", err))?;

            let Some(row) = row else {
                return Ok(None);
            };
            if row.columns().is_empty() {
                return Err(Error::new(
                    ErrorKind::Db,
                    "scalar query expected at least 1 column",
                ));
            }
            Ok(Some(row_value(&row, 0)?))
        })
    }

    pub fn execute(&self, sql: &str, params: Option<DbParams>) -> Result<DbExecuteResult> {
        self.with_connection_mut(false, "execute", |runtime, backend, conn| {
            let query = build_query(sql, params, backend)?;
            let result: sqlx::any::AnyQueryResult = runtime
                .block_on(async move { query.execute(conn).await })
                .map_err(|err| db_error("failed to execute statement", err))?;
            Ok(DbExecuteResult {
                rows_affected: result.rows_affected(),
            })
        })
    }

    pub fn begin_transaction(&self) -> Result<()> {
        let mut state = self.state.borrow_mut();
        ensure_connection_open(&state, "begin_transaction")?;
        if state.tx_active {
            return Err(Error::new(
                ErrorKind::Db,
                "nested transactions are not supported",
            ));
        }
        let conn = state.conn.as_mut().expect("checked above");
        self.runtime
            .block_on(AnyTransactionManager::begin(conn, None))
            .map_err(|err| db_error("failed to begin transaction", err))?;
        state.tx_active = true;
        Ok(())
    }

    pub fn transaction_query(&self, sql: &str, params: Option<DbParams>) -> Result<DbQueryResult> {
        self.with_transaction_connection_mut("query", |runtime, backend, conn| {
            let query = build_query(sql, params, backend)?;
            let rows: Vec<AnyRow> = runtime
                .block_on(async move { query.fetch_all(conn).await })
                .map_err(|err| db_error("failed to execute query", err))?;
            query_result(rows)
        })
    }

    pub fn transaction_query_one(
        &self,
        sql: &str,
        params: Option<DbParams>,
    ) -> Result<Option<DbRow>> {
        self.with_transaction_connection_mut("query_one", |runtime, backend, conn| {
            let query = build_query(sql, params, backend)?;
            let row: Option<AnyRow> = runtime
                .block_on(async move { query.fetch_optional(conn).await })
                .map_err(|err| db_error("failed to execute query", err))?;
            row.map(|row| row_to_db(&row)).transpose()
        })
    }

    pub fn transaction_scalar(
        &self,
        sql: &str,
        params: Option<DbParams>,
    ) -> Result<Option<DbValue>> {
        self.with_transaction_connection_mut("scalar", |runtime, backend, conn| {
            let query = build_query(sql, params, backend)?;
            let row: Option<AnyRow> = runtime
                .block_on(async move { query.fetch_optional(conn).await })
                .map_err(|err| db_error("failed to execute scalar query", err))?;

            let Some(row) = row else {
                return Ok(None);
            };
            if row.columns().is_empty() {
                return Err(Error::new(
                    ErrorKind::Db,
                    "scalar query expected at least 1 column",
                ));
            }
            Ok(Some(row_value(&row, 0)?))
        })
    }

    pub fn transaction_execute(
        &self,
        sql: &str,
        params: Option<DbParams>,
    ) -> Result<DbExecuteResult> {
        self.with_transaction_connection_mut("execute", |runtime, backend, conn| {
            let query = build_query(sql, params, backend)?;
            let result: sqlx::any::AnyQueryResult = runtime
                .block_on(async move { query.execute(conn).await })
                .map_err(|err| db_error("failed to execute statement", err))?;
            Ok(DbExecuteResult {
                rows_affected: result.rows_affected(),
            })
        })
    }

    pub fn commit_transaction(&self) -> Result<()> {
        self.finish_transaction(true)
    }

    pub fn rollback_transaction(&self) -> Result<()> {
        self.finish_transaction(false)
    }

    pub fn close(&self) -> Result<()> {
        let conn = {
            let mut state = self.state.borrow_mut();
            if state.tx_active {
                return Err(Error::new(
                    ErrorKind::Db,
                    "cannot close while a transaction is active",
                ));
            }
            state.conn.take()
        };

        if let Some(conn) = conn {
            self.runtime
                .block_on(async move { AnyConnection::close(conn).await })
                .map_err(|err| db_error("failed to close connection", err))?;
        }

        Ok(())
    }

    fn finish_transaction(&self, commit: bool) -> Result<()> {
        let mut state = self.state.borrow_mut();
        ensure_connection_open(
            &state,
            if commit {
                "commit_transaction"
            } else {
                "rollback_transaction"
            },
        )?;
        if !state.tx_active {
            return Err(Error::new(
                ErrorKind::Db,
                "no active transaction to finalize",
            ));
        }

        let conn = state.conn.as_mut().expect("checked above");
        let result = if commit {
            self.runtime
                .block_on(AnyTransactionManager::commit(conn))
                .map_err(|err| db_error("failed to commit transaction", err))
        } else {
            self.runtime
                .block_on(AnyTransactionManager::rollback(conn))
                .map_err(|err| db_error("failed to rollback transaction", err))
        };
        state.tx_active = false;
        result
    }

    fn with_connection_mut<R>(
        &self,
        allow_transaction: bool,
        op_name: &str,
        f: impl FnOnce(&Runtime, DbBackend, &mut AnyConnection) -> Result<R>,
    ) -> Result<R> {
        let mut state = self.state.borrow_mut();
        ensure_connection_open(&state, op_name)?;
        if state.tx_active && !allow_transaction {
            return Err(Error::new(
                ErrorKind::Db,
                format!("{op_name} cannot be used while a transaction is active"),
            ));
        }
        let backend = state.backend;
        let conn = state.conn.as_mut().expect("checked above");
        f(&self.runtime, backend, conn)
    }

    fn with_transaction_connection_mut<R>(
        &self,
        op_name: &str,
        f: impl FnOnce(&Runtime, DbBackend, &mut AnyConnection) -> Result<R>,
    ) -> Result<R> {
        let mut state = self.state.borrow_mut();
        ensure_connection_open(&state, op_name)?;
        if !state.tx_active {
            return Err(Error::new(
                ErrorKind::Db,
                "transaction handle is no longer active",
            ));
        }
        let backend = state.backend;
        let conn = state.conn.as_mut().expect("checked above");
        f(&self.runtime, backend, conn)
    }
}

fn detect_backend(url: &str) -> Result<DbBackend> {
    if url.starts_with("sqlite:") {
        Ok(DbBackend::Sqlite)
    } else if url.starts_with("postgres://") || url.starts_with("postgresql://") {
        Ok(DbBackend::Postgres)
    } else if url.starts_with("mysql://") {
        Ok(DbBackend::MySql)
    } else {
        Err(Error::new(
            ErrorKind::Db,
            format!("unsupported database url {url:?}"),
        ))
    }
}

fn normalize_database_url(url: &str, backend: DbBackend, current_dir: &Path) -> Result<String> {
    if backend != DbBackend::Sqlite {
        return Ok(url.to_string());
    }
    if url == "sqlite::memory:" || url == "sqlite://:memory:" || url.starts_with("sqlite://?") {
        return Ok(url.to_string());
    }

    let rest = if let Some(rest) = url.strip_prefix("sqlite://") {
        rest
    } else if let Some(rest) = url.strip_prefix("sqlite:") {
        rest
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
        let absolute = pathbuf_to_string(&absolute)?;
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
    params: Option<DbParams>,
    backend: DbBackend,
) -> Result<sqlx::query::Query<'static, Any, AnyArguments<'static>>> {
    let (rewritten_sql, args) = rewrite_sql_and_build_args(sql, params, backend)?;
    Ok(sqlx::query_with(
        Box::leak(rewritten_sql.into_boxed_str()),
        args,
    ))
}

fn rewrite_sql_and_build_args(
    sql: &str,
    params: Option<DbParams>,
    backend: DbBackend,
) -> Result<(String, AnyArguments<'static>)> {
    let mut rewritten = String::with_capacity(sql.len() + 16);
    let mut args = AnyArguments::default();
    let mut positional_index = 0usize;
    let mut used_named = BTreeSet::new();
    let named = match &params {
        Some(DbParams::Named(values)) => Some(values),
        _ => None,
    };
    let positional = match &params {
        Some(DbParams::Positional(values)) => Some(values),
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
            Some(DbParams::Positional(values)) if ch == '?' => {
                let value = values.get(positional_index).ok_or_else(|| {
                    Error::new(
                        ErrorKind::Db,
                        format!(
                            "expected at least {} positional params",
                            positional_index + 1
                        ),
                    )
                })?;
                positional_index += 1;
                bind_index += 1;
                rewritten.push_str(&placeholder_text(backend, bind_index));
                push_bind_value(&mut args, value)?;
                i += 1;
            }
            Some(DbParams::Named(values))
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
                    Error::new(ErrorKind::Db, format!("missing named param :{name}"))
                })?;
                used_named.insert(name);
                bind_index += 1;
                rewritten.push_str(&placeholder_text(backend, bind_index));
                push_bind_value(&mut args, value)?;
                i = end;
            }
            None if ch == '?' => {
                return Err(Error::new(
                    ErrorKind::Db,
                    "SQL contains placeholders but no params were provided",
                ));
            }
            None if ch == ':'
                && i + 1 < chars.len()
                && chars[i + 1] != ':'
                && is_name_start(chars[i + 1]) =>
            {
                return Err(Error::new(
                    ErrorKind::Db,
                    "SQL contains named placeholders but no params were provided",
                ));
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
        return Err(Error::new(
            ErrorKind::Db,
            format!(
                "received {} positional params but SQL used {}",
                values.len(),
                positional_index
            ),
        ));
    }

    if let Some(values) = named {
        let unused: Vec<&str> = values
            .keys()
            .filter(|key| !used_named.contains(*key))
            .map(String::as_str)
            .collect();
        if !unused.is_empty() {
            return Err(Error::new(
                ErrorKind::Db,
                format!("unused named params: {}", unused.join(", ")),
            ));
        }
    }

    Ok((rewritten, args))
}

fn push_bind_value(args: &mut AnyArguments<'static>, value: &DbBindValue) -> Result<()> {
    let result = match value {
        DbBindValue::Boolean(value) => args.add(*value),
        DbBindValue::Integer(value) => args.add(*value),
        DbBindValue::Number(value) => args.add(*value),
        DbBindValue::String(value) => args.add(value.clone()),
    };
    result.map_err(|err| db_error("failed to bind parameter", err))
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

fn copy_block_comment(chars: &[char], start: usize, output: &mut String) -> Result<usize> {
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
    Err(Error::new(
        ErrorKind::Db,
        "SQL parse failed: unterminated block comment",
    ))
}

fn is_name_start(ch: char) -> bool {
    ch == '_' || ch.is_ascii_alphabetic()
}

fn is_name_continue(ch: char) -> bool {
    ch == '_' || ch.is_ascii_alphanumeric()
}

fn query_result(rows: Vec<AnyRow>) -> Result<DbQueryResult> {
    let columns = rows
        .first()
        .map(|row| {
            row.columns()
                .iter()
                .map(|column| column.name().to_string())
                .collect()
        })
        .unwrap_or_default();
    let rows = rows.iter().map(row_to_db).collect::<Result<Vec<_>>>()?;
    Ok(DbQueryResult { rows, columns })
}

fn row_to_db(row: &AnyRow) -> Result<DbRow> {
    let mut values = BTreeMap::new();
    for (index, column) in row.columns().iter().enumerate() {
        let name = column.name();
        if values.contains_key(name) {
            return Err(Error::new(
                ErrorKind::Db,
                format!("query result contains duplicate column name {name:?}"),
            ));
        }
        values.insert(name.to_string(), row_value(row, index)?);
    }
    Ok(values)
}

fn row_value(row: &AnyRow, index: usize) -> Result<DbValue> {
    let raw = row
        .try_get_raw(index)
        .map_err(|err| db_error(&format!("failed to read column {index}"), err))?;

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

fn row_decode_err(index: usize) -> impl FnOnce(sqlx::Error) -> Error {
    move |err| db_error(&format!("failed to decode column {index}"), err)
}

fn ensure_connection_open(state: &ConnectionState, op_name: &str) -> Result<()> {
    if state.conn.is_some() {
        Ok(())
    } else {
        Err(Error::new(
            ErrorKind::Db,
            format!("{op_name} cannot be used after the connection is closed"),
        ))
    }
}

fn db_error(context: &str, err: impl std::fmt::Display) -> Error {
    Error::new(ErrorKind::Db, format!("{context}: {err}"))
}

fn pathbuf_to_string(path: &Path) -> Result<String> {
    path.to_str()
        .map(std::string::ToString::to_string)
        .ok_or_else(|| Error::new(ErrorKind::Db, "path contains invalid UTF-8"))
}
