use std::env::args;
use std::convert::TryFrom;
use std::net::IpAddr;
use std::io::ErrorKind;
use redis::Client;
use chrono::{NaiveDate, Duration};
use std::collections::HashMap;

fn main() {
    let args: Vec<String> = args().into_iter().skip(1).collect();
    let input_args = InputArgs::try_from(args).unwrap();

    let mut redis_client = Client::open(input_args.ip).unwrap();
    set_redis_date_uuid_values(&mut redis_client.get_connection().unwrap());
    delete_by_rg(input_args.regex, &mut redis_client.get_connection().unwrap());
}

struct InputArgs {
    ip: String,
    regex: String
}

impl TryFrom<Vec<String>> for InputArgs {
    type Error = ErrorKind;

    fn try_from(list: Vec<String>) -> Result<Self, Self::Error> {
        let ip: String = match list.get(0) {
            Some(value) => Ok(value.clone()),
            None => Err(ErrorKind::InvalidInput)
        }?;

        let regex: String = match list.get(1) {
            Some(value) => Ok(value.clone()),
            None => Err(ErrorKind::InvalidInput)
        }?;

        Ok(InputArgs  {
            ip: ip,
            regex: regex
        })
    }
}

fn set_redis_date_uuid_values(redis_connect: &mut redis::Connection) {
    let date_vec = generate_date_range();

    for date in date_vec {
        let redis_command_result = redis::cmd("SET")
            .arg(date.format("%Y:%m:%d").to_string())
            .arg(uuid::Uuid::new_v4().to_string())
            .query::<()>(redis_connect);

        match redis_command_result {
            Ok(_) => {},
            Err(error) => println!("{:?}", error),
        }
    }

}

fn delete_by_rg(regex: String, redis_connect: &mut redis::Connection) -> i64 {
    match redis::cmd("RGDELETE").arg(regex).query::<i64>(redis_connect) {
        Ok(value) => value,
        Err(_) => 0,
    }
}

fn generate_date_range() -> Vec<NaiveDate> {
    let mut result: Vec<NaiveDate> = Vec::new();
    let mut date = NaiveDate::from_ymd(2015, 1, 1);
    let duration = Duration::weeks(2);

    for _ in (0..100) {
        result.push(date);
        date += duration;
    }

    result
}
