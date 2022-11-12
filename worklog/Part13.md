# Main

unit test tool : 

`cargo install --locked cargo-nextest`

- [x] src\addr.rs
* test_addr_ipv4()
* test_addr_ipv6() 
- [x] src\http\request.rs
* test_parse_queries() 
* test_parse_json() 
* test_query() 
- [x] src\http\response.rs:
* test_body_empty()
* test_body_stream1()
* test_body_stream2()
- [x] src\http\range.rs
* test_parse()
- [x] src\http\mod.rs
* test_guess_accept_mime() 
- [x] src\http\errors\parse_error.rs
* test_writer_error()
- [x] src\depot.rs
* test_depot
* test_transfer
* test_middleware_use_depot
- [x] src\catcher.rs 
* test_handle_error
* handler_custom
* test_custom_catcher
- [x] src\serde\mod.rs
* test_de_str_map() 
* test_de_str_multi_map()
- [x] src\extract\metadata.rs
* test_parse_source_from()
* test_parse_source_format() 
* test_parse_rename_rule() 
* test_rename_rule()
- [x] src\routing\filter\path.rs
* test_parse_empty
* test_parse_root
* test_parse_rest_without_name
* test_parse_single_const
* test_parse_multi_const
* test_parse_single_regex
* test_parse_wildcard_regex
* test_parse_single_regex_with_prefix
* test_parse_single_regex_with_suffix
* test_parse_single_regex_with_prefix_and_suffix
* test_parse_multi_regex
* test_parse_multi_regex_with_prefix
* test_parse_multi_regex_with_suffix
* test_parse_multi_regex_with_prefix_and_suffix
* test_parse_rest
* test_parse_num0
* test_parse_num1
* test_parse_num2
* test_parse_num3
* test_parse_num4
* test_parse_num5
* test_parse_named_failed1
* test_parse_rest_failed1
* test_parse_rest_failed2
* test_parse_many_slashes
* test_detect_consts
* test_detect_consts0
* test_detect_consts1
* test_detect_consts2
* test_detect_const_and_named
* test_detect_many
* test_detect_many_slashes
* test_detect_named_regex
* test_detect_wildcard
- [x] src\routing\filter\mod.rs
* test_methods
* test_opts
- [x] src\routing\mod.rs
* test_custom_filter() {
- [x] src\routing\router.rs
* fake_handler
* fn(_res: &mut Response) test_router_debug
* test_router_detect1
* test_router_detect2
* test_router_detect3
* test_router_detect4
* test_router_detect5
* test_router_detect6
* test_router_detect_utf8
* test_router_detect9
* test_router_detect10
* test_router_detect11
* test_router_detect12
* test_router_detect13
* test_router_detect_path_encoded
- [x] src\writer\json.rs
* test_write_json_content
- [x] src\writer\mod.rs
* test_write_str
* test_write_string
- [x] src\writer\text.rs
* test_write_str
* test_write_string
* test_write_plain_text
* test_write_html_text
- [x] src\error.rs
* test_anyhow()
* test_error()
- [x] src\listener\mod.rs
* test_tcp_listener
* test_joined_listener
- [x] src\serde\request.rs
* test_de_request_from_query
* test_de_request_with_lifetime
* test_de_request_with_rename
* test_de_request_with_rename_all
* test_de_request_with_multi_sources
* test_de_request_with_json_vec
* test_de_request_with_json_bool
* test_de_request_with_json_str
- [x] src\service.rs
* test_service()
__add module__ : `reqwest`
- [x] src\server.rs
* test_server()


// none
- [ ] src\fs\mod.rs:
- [ ] src\listener\native_tls.rs
- [ ] src\listener\openssl.rs
- [ ] src\listener\rustls.rs
- [ ] src\listener\unix.rs
