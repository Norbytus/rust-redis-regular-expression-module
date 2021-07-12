use redis_module::{RedisError, RedisString};
use regex::Regex;
use std::convert::TryFrom;

pub trait GetRegularExpression {
    fn get_regular_expression(&self) -> &Regex;
}

fn get_regular_expression(raw_expression: &RedisString) -> Result<Regex, RedisError> {
    match Regex::new(&raw_expression.to_string()) {
        Ok(reg) => Ok(reg),
        Err(e) => Err(RedisError::String(format!("{}", e))),
    }
}

pub struct FindByKey {
    regex: Regex,
}

impl GetRegularExpression for FindByKey {
    fn get_regular_expression(&self) -> &Regex {
        &self.regex
    }
}

impl TryFrom<Vec<RedisString>> for FindByKey {
    type Error = RedisError;

    fn try_from(args: Vec<RedisString>) -> Result<Self, Self::Error> {
        let args: Vec<RedisString> = args.into_iter().skip(1).collect();

        match args.get(0) {
            Some(arg) => {
                let regex = get_regular_expression(arg)?;
                Ok(Self { regex })
            }
            None => Err(RedisError::Str("Not found first argument")),
        }
    }
}

pub struct FindByValue {
    regex: Regex,
    redis_mask: String,
}

impl FindByValue {
    pub fn get_redis_mask(&self) -> &str {
        &self.redis_mask
    }
}

impl GetRegularExpression for FindByValue {
    fn get_regular_expression(&self) -> &Regex {
        &self.regex
    }
}

impl TryFrom<Vec<RedisString>> for FindByValue {
    type Error = RedisError;

    fn try_from(args: Vec<RedisString>) -> Result<Self, Self::Error> {
        let args: Vec<RedisString> = args.into_iter().skip(1).collect();

        let redis_mask = match args.get(0) {
            Some(arg) => Ok(arg.to_string()),
            None => Err(RedisError::Str("Not found first argument")),
        }?;

        let regex = match args.get(1) {
            Some(arg) => get_regular_expression(arg),
            None => Err(RedisError::Str("Not found second argument")),
        }?;

        Ok(Self { regex, redis_mask })
    }
}
