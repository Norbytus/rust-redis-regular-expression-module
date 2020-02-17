use redis_module::{Context, RedisError, RedisResult, redis_module, redis_command, RedisValue};
use regex::Regex;

fn execute_regular_expression(ctx: &Context, args: Vec<String>) -> RedisResult {
    let need_arg = args.into_iter().skip(1).map(|s| s).collect::<Vec<String>>();

    let first_arg: &str = match need_arg.get(0) {
        Some(data) => Ok(data),
        None => {
            ctx.log_debug("Not found regular expression arguments value");
            Err(RedisError::WrongArity)
        }
    }?;

    let reg = match Regex::new(first_arg) {
        Ok(reg) => {
            ctx.log_debug("Succes regular expression");
            Ok(reg)
        },
        Err(e) => {
            ctx.log_debug(
                &format!(
                    "{} error regular expression pattern",
                    e.to_string()
                )
            );
            Err(RedisError::String(format!("{}", e)))
        }
    }?;

    ctx.log_debug(&format!("{:?}", need_arg));

    let response_from_command = ctx.call("KEYS", &["*"])?;

    let result = match response_from_command {
        RedisValue::Array(data) => Ok(data),
        _ => {
            Err(RedisError::String(format!("{:?}", &response_from_command)))
        }
    }?;

    let mut result_redis_value: Vec<RedisValue> = Vec::new();
    for redis_value in result {
        let key = match redis_value {
            RedisValue::SimpleString(key) => {
                if reg.is_match(&key) {
                    Some(RedisValue::SimpleString(key))
                } else {
                    None
                }
            }
            _ => None
        };
        if key.is_some() {
            result_redis_value.push(key.unwrap());
        }
    }

    Ok(RedisValue::Array(result_redis_value))
}

redis_module!{
    name: "Regex",
    version: 0.1,
    data_types: [],
    commands: [
        ["rgkey", execute_regular_expression, "read"],
    ]
}
