use mlua::{Function, Lua, Table, UserData, UserDataMethods, Value};
use ptool_engine::{
    DbBindValue, DbConnection, DbParams, DbQueryResult, DbRow, DbValue, PtoolEngine,
};
use std::cell::Cell;
use std::collections::BTreeMap;
use std::path::Path;
use std::rc::Rc;

const CONNECT_SIGNATURE: &str = "ptool.db.connect(url_or_options)";
const QUERY_SIGNATURE: &str = "ptool.db.Connection:query(sql, params?)";
const QUERY_ONE_SIGNATURE: &str = "ptool.db.Connection:query_one(sql, params?)";
const SCALAR_SIGNATURE: &str = "ptool.db.Connection:scalar(sql, params?)";
const EXECUTE_SIGNATURE: &str = "ptool.db.Connection:execute(sql, params?)";
const TRANSACTION_SIGNATURE: &str = "ptool.db.Connection:transaction(fn)";

#[derive(Clone)]
pub(crate) struct LuaDbConnection {
    connection: DbConnection,
}

#[derive(Clone)]
struct LuaDbTransaction {
    connection: DbConnection,
    active: Rc<Cell<bool>>,
}

pub(crate) fn connect(
    value: Value,
    current_dir: &Path,
    engine: &PtoolEngine,
) -> mlua::Result<LuaDbConnection> {
    let url = parse_connect_value(value)?;
    let connection = engine
        .db_connect(&url, current_dir)
        .map_err(|err| db_error(CONNECT_SIGNATURE, err))?;
    Ok(LuaDbConnection { connection })
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
        let params = parse_params(params, QUERY_SIGNATURE)?;
        let result = self
            .connection
            .query(sql, params)
            .map_err(|err| db_error(QUERY_SIGNATURE, err))?;
        query_result_to_lua(lua, result)
    }

    fn query_one(&self, lua: &Lua, sql: &str, params: Option<Value>) -> mlua::Result<Value> {
        let params = parse_params(params, QUERY_ONE_SIGNATURE)?;
        let row = self
            .connection
            .query_one(sql, params)
            .map_err(|err| db_error(QUERY_ONE_SIGNATURE, err))?;
        match row {
            Some(row) => Ok(Value::Table(row_to_lua(lua, row)?)),
            None => Ok(Value::Nil),
        }
    }

    fn scalar(&self, lua: &Lua, sql: &str, params: Option<Value>) -> mlua::Result<Value> {
        let params = parse_params(params, SCALAR_SIGNATURE)?;
        let value = self
            .connection
            .scalar(sql, params)
            .map_err(|err| db_error(SCALAR_SIGNATURE, err))?;
        match value {
            Some(value) => db_value_to_lua(lua, value),
            None => Ok(Value::Nil),
        }
    }

    fn execute(&self, lua: &Lua, sql: &str, params: Option<Value>) -> mlua::Result<Table> {
        let params = parse_params(params, EXECUTE_SIGNATURE)?;
        let result = self
            .connection
            .execute(sql, params)
            .map_err(|err| db_error(EXECUTE_SIGNATURE, err))?;
        let table = lua.create_table()?;
        table.set(
            "rows_affected",
            u64_to_i64(result.rows_affected, EXECUTE_SIGNATURE)?,
        )?;
        Ok(table)
    }

    fn transaction(&self, _lua: &Lua, callback: Function) -> mlua::Result<Value> {
        self.connection
            .begin_transaction()
            .map_err(|err| db_error(TRANSACTION_SIGNATURE, err))?;

        let active = Rc::new(Cell::new(true));
        let tx = LuaDbTransaction {
            connection: self.connection.clone(),
            active: Rc::clone(&active),
        };
        let callback_result = callback.call::<Value>(tx);

        active.set(false);

        let finalize_result = if callback_result.is_ok() {
            self.connection.commit_transaction()
        } else {
            self.connection.rollback_transaction()
        }
        .map_err(|err| db_error(TRANSACTION_SIGNATURE, err));

        match (callback_result, finalize_result) {
            (Ok(value), Ok(())) => Ok(value),
            (Err(err), Ok(())) => Err(err),
            (Ok(_), Err(err)) => Err(err),
            (Err(callback_err), Err(finalize_err)) => Err(crate::lua_error::prompt_failed(
                TRANSACTION_SIGNATURE,
                format!("callback failed: {callback_err}; rollback failed: {finalize_err}"),
            )),
        }
    }

    fn close(&self) -> mlua::Result<()> {
        self.connection
            .close()
            .map_err(|err| db_error("ptool.db.Connection:close()", err))
    }
}

impl LuaDbTransaction {
    fn ensure_active(&self) -> mlua::Result<()> {
        if self.active.get() {
            Ok(())
        } else {
            Err(crate::lua_error::invalid_argument(
                TRANSACTION_SIGNATURE,
                "transaction handle is no longer active",
            ))
        }
    }

    fn query(&self, lua: &Lua, sql: &str, params: Option<Value>) -> mlua::Result<Table> {
        let params = parse_params(params, QUERY_SIGNATURE)?;
        let result = self
            .connection
            .transaction_query(sql, params)
            .map_err(|err| db_error(QUERY_SIGNATURE, err))?;
        query_result_to_lua(lua, result)
    }

    fn query_one(&self, lua: &Lua, sql: &str, params: Option<Value>) -> mlua::Result<Value> {
        let params = parse_params(params, QUERY_ONE_SIGNATURE)?;
        let row = self
            .connection
            .transaction_query_one(sql, params)
            .map_err(|err| db_error(QUERY_ONE_SIGNATURE, err))?;
        match row {
            Some(row) => Ok(Value::Table(row_to_lua(lua, row)?)),
            None => Ok(Value::Nil),
        }
    }

    fn scalar(&self, lua: &Lua, sql: &str, params: Option<Value>) -> mlua::Result<Value> {
        let params = parse_params(params, SCALAR_SIGNATURE)?;
        let value = self
            .connection
            .transaction_scalar(sql, params)
            .map_err(|err| db_error(SCALAR_SIGNATURE, err))?;
        match value {
            Some(value) => db_value_to_lua(lua, value),
            None => Ok(Value::Nil),
        }
    }

    fn execute(&self, lua: &Lua, sql: &str, params: Option<Value>) -> mlua::Result<Table> {
        let params = parse_params(params, EXECUTE_SIGNATURE)?;
        let result = self
            .connection
            .transaction_execute(sql, params)
            .map_err(|err| db_error(EXECUTE_SIGNATURE, err))?;
        let table = lua.create_table()?;
        table.set(
            "rows_affected",
            u64_to_i64(result.rows_affected, EXECUTE_SIGNATURE)?,
        )?;
        Ok(table)
    }
}

fn parse_connect_value(value: Value) -> mlua::Result<String> {
    match value {
        Value::String(text) => Ok(text.to_str()?.to_string()),
        Value::Table(options) => {
            let Some(url) = options.get::<Option<String>>("url")? else {
                return Err(crate::lua_error::invalid_argument(
                    CONNECT_SIGNATURE,
                    "requires string `url` when called with a table",
                ));
            };
            Ok(url)
        }
        _ => Err(crate::lua_error::invalid_argument(
            CONNECT_SIGNATURE,
            "expects a string or a table with `url`",
        )),
    }
}

fn parse_params(params: Option<Value>, signature: &str) -> mlua::Result<Option<DbParams>> {
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
                Ok(Some(DbParams::Positional(values)))
            } else {
                let mut values = BTreeMap::new();
                for pair in table.pairs::<Value, Value>() {
                    let (key, value) = pair?;
                    let key = match key {
                        Value::String(key) => key.to_str()?.to_string(),
                        _ => {
                            return Err(crate::lua_error::invalid_argument(
                                signature,
                                "named params keys must be strings",
                            ));
                        }
                    };
                    if key.is_empty() {
                        return Err(crate::lua_error::invalid_argument(
                            signature,
                            "named params keys must not be empty",
                        ));
                    }
                    values.insert(key, parse_bind_value(value, signature, None)?);
                }
                Ok(Some(DbParams::Named(values)))
            }
        }
        _ => Err(crate::lua_error::invalid_argument(
            signature,
            "`params` must be a table",
        )),
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
                return Err(crate::lua_error::invalid_argument(
                    signature,
                    "parameter must be a finite number",
                ));
            }
            Ok(DbBindValue::Number(value))
        }
        Value::String(value) => Ok(DbBindValue::String(value.to_str()?.to_string())),
        Value::Nil => Err(crate::lua_error::invalid_argument(
            signature,
            match index {
                Some(index) => format!("params[{index}] does not support nil"),
                None => "named params do not support nil".to_string(),
            },
        )),
        _ => Err(crate::lua_error::invalid_argument(
            signature,
            match index {
                Some(index) => {
                    format!("params[{index}] only supports boolean/integer/number/string")
                }
                None => "named params only support boolean/integer/number/string".to_string(),
            },
        )),
    }
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

fn query_result_to_lua(lua: &Lua, query: DbQueryResult) -> mlua::Result<Table> {
    let row_count = query.rows.len();
    let rows_table = lua.create_table()?;
    for (index, row) in query.rows.into_iter().enumerate() {
        rows_table.raw_set(index + 1, row_to_lua(lua, row)?)?;
    }

    let columns = lua.create_table()?;
    for (index, column) in query.columns.into_iter().enumerate() {
        columns.raw_set(index + 1, column)?;
    }

    let result = lua.create_table()?;
    result.set("rows", rows_table)?;
    result.set("columns", columns)?;
    result.set("row_count", usize_to_i64(row_count, QUERY_SIGNATURE)?)?;
    Ok(result)
}

fn row_to_lua(lua: &Lua, row: DbRow) -> mlua::Result<Table> {
    let table = lua.create_table()?;
    for (name, value) in row {
        table.raw_set(name, db_value_to_lua(lua, value)?)?;
    }
    Ok(table)
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

fn db_error(signature: &str, err: ptool_engine::Error) -> mlua::Error {
    crate::lua_error::lua_error_from_engine(err, signature)
}

fn usize_to_i64(value: usize, signature: &str) -> mlua::Result<i64> {
    i64::try_from(value).map_err(|_| {
        crate::lua_error::invalid_argument(signature, "produced a value that is too large")
    })
}

fn u64_to_i64(value: u64, signature: &str) -> mlua::Result<i64> {
    i64::try_from(value).map_err(|_| {
        crate::lua_error::invalid_argument(signature, "produced a value that is too large")
    })
}
