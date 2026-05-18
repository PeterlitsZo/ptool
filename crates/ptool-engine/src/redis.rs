use crate::{Error, ErrorKind, Result};
use redis::{Client, Cmd, RedisError};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use tokio::runtime::Runtime;

#[derive(Clone)]
pub struct RedisConnection {
    runtime: Arc<Runtime>,
    state: Rc<RefCell<ConnectionState>>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum RedisArg {
    Boolean(bool),
    Integer(i64),
    Number(f64),
    Bytes(Vec<u8>),
}

#[derive(Clone, Debug, PartialEq)]
pub enum RedisReply {
    Nil,
    Integer(i64),
    Bytes(Vec<u8>),
    String(String),
    Array(Vec<RedisReply>),
    Map(Vec<(RedisReply, RedisReply)>),
    Set(Vec<RedisReply>),
    Number(f64),
    Boolean(bool),
    BigNumber(String),
    Push { kind: String, data: Vec<RedisReply> },
}

struct ConnectionState {
    conn: Option<redis::aio::MultiplexedConnection>,
}

pub(crate) fn connect(runtime: Arc<Runtime>, url: &str) -> Result<RedisConnection> {
    let client = Client::open(url).map_err(|err| redis_error("failed to parse Redis URL", err))?;
    let conn = runtime
        .block_on(client.get_multiplexed_async_connection())
        .map_err(|err| redis_error("failed to connect to Redis", err))?;

    Ok(RedisConnection {
        runtime,
        state: Rc::new(RefCell::new(ConnectionState { conn: Some(conn) })),
    })
}

impl RedisConnection {
    pub fn call(&self, command: &str, args: &[RedisArg]) -> Result<RedisReply> {
        let mut conn = self.with_connection("call", |conn| Ok(conn.clone()))?;
        let mut cmd = Cmd::new();
        cmd.arg(command);
        for arg in args {
            push_arg(&mut cmd, arg);
        }

        let value = self
            .runtime
            .block_on(async move { cmd.query_async::<redis::Value>(&mut conn).await })
            .map_err(|err| redis_error("failed to execute Redis command", err))?;
        redis_value_to_reply(value)
    }

    pub fn close(&self) -> Result<()> {
        self.with_connection_mut("close", |state| {
            state.conn.take();
            Ok(())
        })
    }

    fn with_connection<T>(
        &self,
        op: &str,
        f: impl FnOnce(&redis::aio::MultiplexedConnection) -> Result<T>,
    ) -> Result<T> {
        let state = self.state.borrow();
        let Some(conn) = state.conn.as_ref() else {
            return Err(closed_error(op));
        };
        f(conn)
    }

    fn with_connection_mut<T>(
        &self,
        op: &str,
        f: impl FnOnce(&mut ConnectionState) -> Result<T>,
    ) -> Result<T> {
        let mut state = self.state.borrow_mut();
        if state.conn.is_none() {
            return Err(closed_error(op));
        }
        f(&mut state)
    }
}

fn push_arg(cmd: &mut Cmd, arg: &RedisArg) {
    match arg {
        RedisArg::Boolean(value) => {
            cmd.arg(*value);
        }
        RedisArg::Integer(value) => {
            cmd.arg(*value);
        }
        RedisArg::Number(value) => {
            cmd.arg(*value);
        }
        RedisArg::Bytes(value) => {
            cmd.arg(value);
        }
    }
}

fn redis_value_to_reply(value: redis::Value) -> Result<RedisReply> {
    match value {
        redis::Value::Nil => Ok(RedisReply::Nil),
        redis::Value::Int(value) => Ok(RedisReply::Integer(value)),
        redis::Value::BulkString(value) => Ok(RedisReply::Bytes(value)),
        redis::Value::Array(values) => values
            .into_iter()
            .map(redis_value_to_reply)
            .collect::<Result<Vec<_>>>()
            .map(RedisReply::Array),
        redis::Value::SimpleString(value) => Ok(RedisReply::String(value)),
        redis::Value::Okay => Ok(RedisReply::String("OK".to_string())),
        redis::Value::Map(values) => values
            .into_iter()
            .map(|(key, value)| Ok((redis_value_to_reply(key)?, redis_value_to_reply(value)?)))
            .collect::<Result<Vec<_>>>()
            .map(RedisReply::Map),
        redis::Value::Attribute { data, .. } => redis_value_to_reply(*data),
        redis::Value::Set(values) => values
            .into_iter()
            .map(redis_value_to_reply)
            .collect::<Result<Vec<_>>>()
            .map(RedisReply::Set),
        redis::Value::Double(value) => Ok(RedisReply::Number(value)),
        redis::Value::Boolean(value) => Ok(RedisReply::Boolean(value)),
        redis::Value::VerbatimString { text, .. } => Ok(RedisReply::String(text)),
        redis::Value::BigNumber(value) => Ok(RedisReply::BigNumber(value.to_string())),
        redis::Value::Push { kind, data } => Ok(RedisReply::Push {
            kind: kind.to_string(),
            data: data
                .into_iter()
                .map(redis_value_to_reply)
                .collect::<Result<Vec<_>>>()?,
        }),
        redis::Value::ServerError(err) => {
            Err(redis_error("Redis server returned an error", err.into()))
        }
        _ => Err(Error::new(
            ErrorKind::Redis,
            "Redis returned a reply variant that ptool does not support yet",
        )
        .with_op("ptool.redis")),
    }
}

fn redis_error(message: impl Into<String>, err: RedisError) -> Error {
    Error::new(ErrorKind::Redis, format!("{}: {err}", message.into()))
        .with_op("ptool.redis")
        .with_detail(err.to_string())
}

fn closed_error(op: &str) -> Error {
    Error::new(ErrorKind::Redis, "Redis connection is closed").with_op(op)
}
