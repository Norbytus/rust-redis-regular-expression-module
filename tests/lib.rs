#[cfg(test)]
mod tests {
    use lazy_static::lazy_static;

    const REDIS_DEFAULT_ADDR_PORT: &'static str = "redis://127.0.0.1:6379";

    type RedisModuleList = Vec<Vec<String>>;

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
        let result: redis::RedisResult<RedisModuleList> = redis::cmd("MODULE")
            .arg("LIST")
            .query(&mut REDIS_CLIENT.get_connection().unwrap());

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
    fn find_key_by_regex() {
        set_values_for_find_by_keys();

        let get_value = redis::cmd("KEYS")
            .arg("hello:*:2012")
            .query::<Vec<String>>(&mut REDIS_CLIENT.get_connection().unwrap());

        assert_eq!(1, get_value.unwrap().len());

        let get_value = redis::cmd("RGKEYS")
            .arg("hello.*2012$")
            .query::<Vec<String>>(&mut REDIS_CLIENT.get_connection().unwrap());

        assert_eq!(2, get_value.unwrap().len());
    }

    #[test]
    fn find_values_by_regex() {
        set_values_for_find_by_value();

        assert_eq!(
            vec![
                "test1".to_string(),
                "test2".to_string(),
                "test3".to_string(),
            ],
            execute_rg_values("GET /user/.*")
        );

        assert_eq!(
            vec![
                "test3".to_string(),
                "test4".to_string(),
                "test6".to_string(),
            ],
            execute_rg_values("(.*) /user/.*123$")
        );

        assert_eq!(
            vec![
                "test5".to_string(),
            ],
            execute_rg_values("(.*) /news/.*12$")
        );
    }

    fn execute_rg_values(raw_regex: &str) -> Vec<String> {
        let mut result = redis::cmd("RGVALUES")
            .arg("*")
            .arg(raw_regex)
            .query::<Vec<String>>(&mut REDIS_CLIENT.get_connection().unwrap())
            .unwrap();
        result.sort();

        result
    }

    fn set_values_for_find_by_value() {
        let data: Vec<(&str, &str)> = vec![
            ("test1", "GET /user/12/134"),
            ("test6", "POST /user/12/01123"),
            ("test3", "GET /user/12/123123"),
            ("test4", "DELETE /user/12/01123"),
            ("test5", "GET /news/9912"),
            ("test2", "GET /user/4124/910"),
        ];

        let _: Vec<bool> = data.iter()
            .map(|(key, value)| set_value_in_redis(key, value))
            .filter(|is_write| *is_write)
            .collect();
    }

    fn set_values_for_find_by_keys() {
        let data: Vec<(&str, &str)> = vec![
            ("hello:world:2012", "1"),
            ("helloworld:2012", "1"),
            ("helloworld:2012:test", "1")
        ];

        let _: Vec<bool> = data.iter()
            .map(|(key, value)| set_value_in_redis(key, value))
            .filter(|is_write| *is_write)
            .collect();
    }

    fn set_value_in_redis(key: &str, value: &str) -> bool {
        redis::cmd("SET")
            .arg(key)
            .arg(value)
            .query::<()>(&mut REDIS_CLIENT.get_connection().unwrap()).is_ok()
    }
}
