#[cfg(test)]
mod tests {
    use lazy_static::lazy_static;

    const REDIS_DEFAULT_ADDR_PORT: &'static str = "redis://127.0.0.1:6379";

    lazy_static! {
        static ref REDIS_CLIENT: redis::Client = {
            redis::Client::open(REDIS_DEFAULT_ADDR_PORT)
                .expect("Can't create redis client")
        };
    }

    #[test]
    fn get_connection() {
        assert!(REDIS_CLIENT.get_connection().is_ok());
    }

    #[test]
    fn module_is_load() {
        let result = redis::cmd("MODULE")
            .arg("LIST")
            .query::<Vec<Vec<String>>>(&mut REDIS_CLIENT.get_connection().unwrap());

        let mut is_module_load = false;
        'outer: for module in result {
            for module_data in module {
                if module_data.contains(&"RustRegxCommand".to_string()) {
                    is_module_load = true;
                    break 'outer;
                }
            }
        }

        assert!(is_module_load);
    }

    #[test]
    fn find_value_by_regex() {
        let set_value = set_value_in_redis("hello:world:2012", "1");
        assert!(set_value.is_ok());

        let set_value = set_value_in_redis("helloworld:2012", "1");
        assert!(set_value.is_ok());

        let set_value = set_value_in_redis("helloworld:2012:test", "1");
        assert!(set_value.is_ok());

        let get_value = redis::cmd("KEYS")
            .arg("hello:*:2012")
            .query::<Vec<String>>(&mut REDIS_CLIENT.get_connection().unwrap());

        assert_eq!(1, get_value.unwrap().len());

        let get_value = redis::cmd("RGKEYS")
            .arg("hello.*2012$")
            .query::<Vec<String>>(&mut REDIS_CLIENT.get_connection().unwrap());

        assert_eq!(2, get_value.unwrap().len());
    }

    fn set_value_in_redis(key: &str, value: &str) -> redis::RedisResult<()> {
        redis::cmd("SET")
            .arg(key)
            .arg(value)
            .query::<()>(&mut REDIS_CLIENT.get_connection().unwrap())
    }
}
