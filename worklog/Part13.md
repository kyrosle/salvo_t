# Main

unit test tool : 

`cargo install --locked cargo-nextest`

- [x] src\addr.rs
* async fn test_addr_ipv4()
* async fn test_addr_ipv6() 
- [x] src\http\request.rs
* async fn test_parse_queries() 
* async fn test_parse_json() 
* async fn test_query() 
- [x] src\http\response.rs:
* fn test_body_empty()
* async fn test_body_stream1()
* async fn test_body_stream2()
- [x] src\http\range.rs
* fn test_parse()
- [x] src\http\mod.rs
* fn test_guess_accept_mime() 
- [x] src\http\errors\parse_error.rs
* async fn test_writer_error()

- [ ] src\service.rs
- [ ] src\depot.rs
- [ ] src\catcher.rs 

- [x] src\serde\mod.rs
* async fn test_de_str_map() 
* async fn test_de_str_multi_map()
- [x] src\extract\metadata.rs
* fn test_parse_source_from()
* fn test_parse_source_format() 
* fn test_parse_rename_rule() 
* fn test_rename_rule()

- [ ] src\routing\router.rs
- [ ] src\routing\mod.rs
- [ ] src\routing\filter\mod.rs
- [ ] src\routing\filter\path.rs



- [ ] src\serde\request.rs
- [ ] src\writer\json.rs

- [ ] src\writer\mod.rs
- [ ] src\writer\text.rs

- [ ] src\error.rs
- [ ] src\server.rs
- [ ] src\listener\mod.rs
- [ ] src\listener\native_tls.rs
- [ ] src\listener\openssl.rs
- [ ] src\listener\rustls.rs
- [ ] src\listener\unix.rs
- [ ] src\lib.rs:
- [ ] src\fs\mod.rs:
