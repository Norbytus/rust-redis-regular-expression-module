Simple redis regular expression module writing in Rust lang

# Usage
1. Build module `cargo build --release`
2. Run redis server `redis-server --loadmodule ./target/release/librefid_regular_expression.so`
3. Enter in redis cli run command REG `pattern_for_search` see result
