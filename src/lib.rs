use redis_module::{Context, RedisError, RedisResult, redis_module, redis_command, RedisValue};
use regex::Regex;

const REDIS_COMMAND_KEYS: &'static str = "KEYS";

const REDIS_COMMAND_MGET: &'static str = "MGET";

fn get_regular_expression(raw_expression: &str) -> Result<Regex, RedisError> {
    match Regex::new(&raw_expression) {
        Ok(reg) => Ok(reg),
        Err(e) => Err(RedisError::String(format!("{}", e)))
    }
}

fn handle_redis_command_result(result: Vec<RedisValue>) -> Vec<String> {
    result
        .into_iter()
        .map(|value| match value {
            RedisValue::SimpleString(value) => Some(value),
            _ => None,
        })
        .filter(|value| value.is_some())
        .map(|value| value.unwrap())
        .collect::<Vec<String>>()
}


fn find_keys_by_rg(ctx: &Context, args: Vec<String>) -> RedisResult {
    let need_arg: Vec<String> = args.into_iter().skip(1).map(|s| s).collect();

    let first_arg: &str = match need_arg.get(0) {
        Some(data) => Ok(data),
        None => {
            ctx.log_debug("Not found regular expression arguments value");
            Err(RedisError::WrongArity)
        }
    }?;

    let reg = get_regular_expression(first_arg)?;

    let response_from_command = ctx.call(REDIS_COMMAND_KEYS, &["*"])?;

    let result = match response_from_command {
        RedisValue::Array(data) => Ok(data),
        _ => Err(RedisError::Str("Wrong return result from `KEYS` command, expected array.")),
    }?;

    let result_redis_value: Vec<RedisValue> = handle_redis_command_result(result)
        .into_iter()
        .filter(move |s| reg.is_match(&s))
        .map(|s| RedisValue::SimpleString(s))
        .collect();

    if result_redis_value.is_empty() {
        Ok(RedisValue::None)
    } else {
        Ok(RedisValue::Array(result_redis_value))
    }
}

fn find_values_by_rg(ctx: &Context, args: Vec<String>) -> RedisResult {
    let args: Vec<String> = args.into_iter().skip(1).collect();

    if args.len() < 2 {
        return Err(RedisError::WrongArity);
    }

    let key = match args.get(0) {
        Some(key) => Ok(key),
        None => Err(RedisError::Str("First argument must data key")),
    }?;

    let raw_reg = match args.get(1) {
        Some(reg) => Ok(reg),
        None => Err(RedisError::Str("Second argument must regular expression")),
    }?;

    let reg = get_regular_expression(raw_reg)?;

    let response_from_keys_command = ctx.call(REDIS_COMMAND_KEYS, &[key])?;

    let result_keys_command = match response_from_keys_command {
        RedisValue::Array(data) => Ok(data),
        _ => Err(RedisError::Str("Wrong return result from `MGET` command, expected array.")),
    }?;

    let result_redis_keys: Vec<String> = handle_redis_command_result(result_keys_command);

    let response_from_mget_command = ctx.call(
        REDIS_COMMAND_MGET,
        &result_redis_keys.iter().map(|k| k.as_ref()).collect::<Vec<&str>>()
    )?;

    let result_from_mget_command = match response_from_mget_command {
        RedisValue::Array(data) => Ok(data),
        _ => Err(RedisError::Str("Wrong return result from `KEYS` command, expected array.")),
    }?;

    let result_redis_value: Vec<RedisValue> = handle_redis_command_result(result_from_mget_command)
        .into_iter()
        .filter(move |s| reg.is_match(&s))
        .map(|s| RedisValue::SimpleString(s))
        .collect();

    if result_redis_value.is_empty() {
        Ok(RedisValue::None)
    } else {
        Ok(RedisValue::Array(result_redis_value))
    }
}

redis_module!{
    name: "Regex",
    version: 0.1,
    data_types: [],
    commands: [
        ["rgkey", find_keys_by_rg, "readonly"],
        ["rgvalue", find_values_by_rg, "readonly"],
    ]
}
