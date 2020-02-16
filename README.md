Simple redis regular expression module writing in Rust lang

# Usage
Build module `cargo build --release`
Run redis server `redis-server --loadmodule ./target/release/librefid_regular_expression.so`
Enter in redis cli run command REG `pattern_for_search` see result
