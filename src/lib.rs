use crate::args::GetRegularExpression;
use redis_module::{Context, RedisError, RedisResult, RedisString, RedisValue, Status, redis_command, redis_module};
use std::convert::TryFrom;

pub mod args;

const REDIS_COMMAND_KEYS: &'static str = "KEYS";
const REDIS_PATTERN_KEY_ALL: &'static str = "*";

const RG_KEYS: &'static str = "rgkeys";
const RG_VALUES: &'static str = "rgvalues";
const RG_DELETE: &'static str = "rgdelete";

const READ_ONLY: &'static str = "readonly";
const WRITE: &'static str = "write";

fn handle_redis_command_result(result: Vec<RedisValue>) -> impl Iterator<Item = String> {
    result
        .into_iter()
        .map(|value| match value {
            RedisValue::SimpleString(value) => Some(value),
            _ => None,
        })
        .filter(|value| value.is_some())
        .map(|value| value.unwrap())
}

fn find_keys_by_rg(ctx: &Context, args: Vec<RedisString>) -> RedisResult {
    ctx.log_debug(&format!("Run {}", RG_KEYS));

    let args = crate::args::FindByKey::try_from(args)?;

    let command_result = ctx.call(REDIS_COMMAND_KEYS, &[REDIS_PATTERN_KEY_ALL])?;

    let command_result = match command_result {
        RedisValue::Array(data) => Ok(data),
        _ => Err(RedisError::Str(
            "Wrong return result from `KEYS` command, expected array.",
        )),
    }?;

    ctx.log_debug(&format!("{:?}", command_result));

    let result: Vec<RedisValue> = handle_redis_command_result(command_result)
        .filter(move |s| args.get_regular_expression().is_match(&s.to_string()))
        .map(move |s| RedisValue::SimpleString(s))
        .collect();

    Ok(RedisValue::Array(result))
}

fn find_values_by_rg(ctx: &Context, args: Vec<RedisString>) -> RedisResult {
    ctx.log_debug(&format!("Run {}", RG_VALUES));

    let args = crate::args::FindByValue::try_from(args)?;

    let command_keys_result = ctx.call(
        REDIS_COMMAND_KEYS,
        &[args.get_redis_mask()]
    )?;

    let result_keys_command = match command_keys_result {
        RedisValue::Array(data) => Ok(data),
        _ => Err(RedisError::Str(
            "Wrong return result from `MGET` command, expected array.",
        )),
    }?;

    let result_redis_value: Vec<RedisValue> = handle_redis_command_result(result_keys_command)
        .map(move |key| {
            let redis_key = ctx.open_key(&ctx.create_string(&key));
            match redis_key.read() {
                Ok(value) => (Some(key), value),
                Err(e) => {
                    ctx.log_debug(&format!("{}", e));
                    (None, None)
                }
            }
        })
        .filter(|(key, value)| value.is_some() && key.is_some())
        .map(|(key, value)| (key.unwrap(), value.unwrap()))
        .filter(move |(_, value)| args.get_regular_expression().is_match(&value))
        .map(|(key, _)| RedisValue::SimpleString(key))
        .collect();

    Ok(RedisValue::Array(result_redis_value))
}

fn delete_keys_by_rg(ctx: &Context, args: Vec<RedisString>) -> RedisResult {
    ctx.log_debug(&format!("Run {}", RG_DELETE));

    let args = crate::args::FindByKey::try_from(args)?;
    let raw_regular_expression = format!("{}", args.get_regular_expression());

    let command_result = ctx.call(RG_KEYS, &[&raw_regular_expression])?;

    let result = match command_result {
        RedisValue::Array(data) => Ok(data),
        RedisValue::NoReply => {
            return Ok(RedisValue::NoReply);
        }
        _ => Err(RedisError::Str(
            "Wrong return result from `KEYS` command, expected array.",
        )),
    }?;

    let count_delete_redis_key: u64 = handle_redis_command_result(result)
        .map(move |key| {
            let redis_key = ctx.open_key_writable(&ctx.create_string(&key));
            redis_key.delete()
        })
        .fold(0, |acc, data| if data.is_ok() {acc + 1} else {acc});

    Ok(RedisValue::Integer(count_delete_redis_key as i64))
}

redis_module! {
    name: "RustRegxCommand",
    version: 0.5,
    data_types: [],
    init: init,
    deinit: deinit,
    commands: [
        [RG_KEYS, find_keys_by_rg, READ_ONLY, 1, 1, 1],
        [RG_VALUES, find_values_by_rg, READ_ONLY, 1, 1, 1],
        [RG_DELETE, delete_keys_by_rg, WRITE, 1, 1, 1],
    ]
}

fn init(ctx: &Context, _args: &Vec<RedisString>) -> Status {
    ctx.log_debug("Start");

    Status::Ok
}

fn deinit(ctx: &Context) -> Status {
    ctx.log_debug("End");

    Status::Ok
}
