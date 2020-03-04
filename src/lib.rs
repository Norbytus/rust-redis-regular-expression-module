use crate::args::GetRegularExpression;
use redis_module::{
    redis_command,
    redis_module,
    Context,
    RedisError,
    RedisResult,
    RedisValue
};
use std::convert::TryFrom;

pub mod args;

const REDIS_COMMAND_KEYS: &'static str = "KEYS";
const REDIS_PATTERN_KEY_ALL: &'static str = "*";

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

fn find_keys_by_rg(ctx: &Context, args: Vec<String>) -> RedisResult {
    let args = crate::args::FindByKey::try_from(args)?;

    let response_from_command = ctx.call(REDIS_COMMAND_KEYS, &[REDIS_PATTERN_KEY_ALL])?;

    let result = match response_from_command {
        RedisValue::Array(data) => Ok(data),
        _ => Err(RedisError::Str(
            "Wrong return result from `KEYS` command, expected array.",
        )),
    }?;

    let result_redis_value: Vec<RedisValue> = handle_redis_command_result(result)
        .filter(move |s| args.get_regular_expression().is_match(&s))
        .map(|s| RedisValue::SimpleString(s))
        .collect();

    if result_redis_value.is_empty() {
        Ok(RedisValue::None)
    } else {
        Ok(RedisValue::Array(result_redis_value))
    }
}

fn find_values_by_rg(ctx: &Context, args: Vec<String>) -> RedisResult {
    let args = crate::args::FindByValue::try_from(args)?;

    let response_from_keys_command = ctx.call(
        REDIS_COMMAND_KEYS,
        &[args.get_redis_mask()]
    )?;

    let result_keys_command = match response_from_keys_command {
        RedisValue::Array(data) => Ok(data),
        _ => Err(RedisError::Str(
            "Wrong return result from `MGET` command, expected array.",
        )),
    }?;

    let result_redis_value: Vec<RedisValue> = handle_redis_command_result(result_keys_command)
        .map(move |key| {
            let redis_key = ctx.open_key(&key);
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

    if result_redis_value.is_empty() {
        Ok(RedisValue::None)
    } else {
        Ok(RedisValue::Array(result_redis_value))
    }
}

redis_module! {
    name: "Regex",
    version: 0.1,
    data_types: [],
    commands: [
        ["rgkeys", find_keys_by_rg, "readonly"],
        ["rgvalues", find_values_by_rg, "readonly"],
    ]
}
