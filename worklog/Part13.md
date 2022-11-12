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

PASS [   0.431s] salvo_t addr::tests::test_addr_ipv4
PASS [   0.375s] salvo_t addr::tests::test_addr_ipv6
PASS [   0.322s] salvo_t catcher::tests::test_custom_catcher
PASS [   0.269s] salvo_t catcher::tests::test_handle_error
PASS [   0.217s] salvo_t depot::test::test_depot
PASS [   0.163s] salvo_t depot::test::test_middleware_use_depot
PASS [   0.110s] salvo_t depot::test::test_transfer
PASS [   0.267s] salvo_t extract::metadata::tests::test_parse_source_format
PASS [   0.453s] salvo_t error::tests::test_anyhow
PASS [   0.396s] salvo_t error::tests::test_error
PASS [   0.332s] salvo_t extract::metadata::tests::test_parse_rename_rule
PASS [   0.270s] salvo_t extract::metadata::tests::test_parse_source_from
PASS [   0.218s] salvo_t extract::metadata::tests::test_rename_rule
PASS [   0.165s] salvo_t http::errors::parse_error::test::test_writer_error
LEAK [   0.424s] salvo_t http::range::tests::test_parse
PASS [   0.159s] salvo_t http::response::tests::test_body_stream1
PASS [   0.424s] salvo_t http::request::tests::test_form
PASS [   0.368s] salvo_t http::request::tests::test_parse_json
PASS [   0.316s] salvo_t http::request::tests::test_parse_queries
PASS [   0.263s] salvo_t http::request::tests::test_query
PASS [   0.211s] salvo_t http::response::tests::test_body_empty
LEAK [   0.318s] salvo_t http::response::tests::test_body_stream2
PASS [   0.263s] salvo_t routing::filter::path::tests::test_detect_const_and_named
PASS [   0.158s] salvo_t routing::filter::path::tests::test_detect_consts0
PASS [   0.424s] salvo_t http::tests::test_guess_accept_mime
PASS [   0.370s] salvo_t listener::tests::test_joined_listener
PASS [   0.317s] salvo_t listener::tests::test_tcp_listener
PASS [   0.262s] salvo_t routing::filter::path::tests::test_detect_consts
LEAK [   0.368s] salvo_t routing::filter::path::tests::test_detect_consts1
PASS [   0.265s] salvo_t routing::filter::path::tests::test_detect_named_regex
LEAK [   0.424s] salvo_t routing::filter::path::tests::test_detect_consts2
PASS [   0.369s] salvo_t routing::filter::path::tests::test_detect_many
PASS [   0.317s] salvo_t routing::filter::path::tests::test_detect_many_slashes
PASS [   0.266s] salvo_t routing::filter::path::tests::test_detect_wildcard
PASS [   0.266s] salvo_t routing::filter::path::tests::test_parse_empty
LEAK [   0.372s] salvo_t routing::filter::path::tests::test_parse_many_slashes
LEAK [   0.431s] salvo_t routing::filter::path::tests::test_parse_multi_const
PASS [   0.217s] salvo_t routing::filter::path::tests::test_parse_multi_regex_with_suffix
LEAK [   0.375s] salvo_t routing::filter::path::tests::test_parse_multi_regex
PASS [   0.321s] salvo_t routing::filter::path::tests::test_parse_multi_regex_with_prefix
PASS [   0.269s] salvo_t routing::filter::path::tests::test_parse_multi_regex_with_prefix_and_suffix
LEAK [   0.273s] salvo_t routing::filter::path::tests::test_parse_num0
PASS [   0.325s] salvo_t routing::filter::path::tests::test_parse_named_failed1
PASS [   0.212s] salvo_t routing::filter::path::tests::test_parse_num5
PASS [   0.433s] salvo_t routing::filter::path::tests::test_parse_num1
PASS [   0.375s] salvo_t routing::filter::path::tests::test_parse_num2
PASS [   0.320s] salvo_t routing::filter::path::tests::test_parse_num3
PASS [   0.105s] salvo_t routing::filter::path::tests::test_parse_rest_failed1
PASS [   0.267s] salvo_t routing::filter::path::tests::test_parse_num4
LEAK [   0.371s] salvo_t routing::filter::path::tests::test_parse_rest
PASS [   0.434s] salvo_t routing::filter::path::tests::test_parse_rest_failed2
PASS [   0.381s] salvo_t routing::filter::path::tests::test_parse_rest_without_case
PASS [   0.324s] salvo_t routing::filter::path::tests::test_parse_root
PASS [   0.272s] salvo_t routing::filter::path::tests::test_parse_single_const
PASS [   0.220s] salvo_t routing::filter::path::tests::test_parse_single_regex
PASS [   0.168s] salvo_t routing::filter::path::tests::test_parse_single_regex_with_prefix
LEAK [   0.329s] salvo_t routing::filter::path::tests::test_parse_single_regex_with_prefix_and_suffix
PASS [   0.431s] salvo_t routing::filter::path::tests::test_parse_single_regex_with_suffix
PASS [   0.372s] salvo_t routing::filter::path::tests::test_parse_wildcard_regex
PASS [   0.316s] salvo_t routing::filter::tests::test_methods
PASS [   0.263s] salvo_t routing::filter::tests::test_opts
PASS [   0.107s] salvo_t routing::router::tests::test_router_detect10
PASS [   0.211s] salvo_t routing::router::tests::test_router_debug
PASS [   0.158s] salvo_t routing::router::tests::test_router_detect1
PASS [   0.265s] salvo_t routing::router::tests::test_router_detect2
PASS [   0.213s] salvo_t routing::router::tests::test_router_detect3
PASS [   0.428s] salvo_t routing::router::tests::test_router_detect11
PASS [   0.374s] salvo_t routing::router::tests::test_router_detect12
PASS [   0.318s] salvo_t routing::router::tests::test_router_detect13
PASS [   0.269s] salvo_t routing::router::tests::test_router_detect4
PASS [   0.217s] salvo_t routing::router::tests::test_router_detect5
LEAK [   0.423s] salvo_t routing::router::tests::test_router_detect6
LEAK [   0.368s] salvo_t routing::router::tests::test_router_detect9
PASS [   0.209s] salvo_t routing::tests::test_custom_filter
LEAK [   0.265s] salvo_t serde::request::tests::test_de_request_from_query
LEAK [   0.213s] salvo_t serde::request::tests::test_de_request_with_json_bool
LEAK [   0.423s] salvo_t routing::router::tests::test_router_detect_path_encoded
PASS [   0.370s] salvo_t routing::router::tests::test_router_detect_utf8
LEAK [   0.422s] salvo_t serde::request::tests::test_de_request_with_json_str
LEAK [   0.370s] salvo_t serde::request::tests::test_de_request_with_json_vec
PASS [   0.314s] salvo_t serde::request::tests::test_de_request_with_lifetime
LEAK [   0.426s] salvo_t serde::request::tests::test_de_request_with_multi_sources
LEAK [   0.374s] salvo_t serde::request::tests::test_de_request_with_rename
LEAK [   0.322s] salvo_t serde::request::tests::test_de_request_with_rename_all
LEAK [   0.271s] salvo_t serde::tests::test_de_str_map
LEAK [   0.430s] salvo_t serde::tests::test_de_str_multi_map
LEAK [   0.320s] salvo_t writer::json::tests::test_write_json_content
PASS [   0.374s] salvo_t writer::tests::test_write_str
PASS [   0.321s] salvo_t writer::tests::test_write_string
PASS [   0.269s] salvo_t writer::text::tests::test_write_html_text
LEAK [   0.215s] salvo_t writer::text::tests::test_write_json_text
PASS [   0.162s] salvo_t writer::text::tests::test_write_plain_text
PASS [   0.110s] salvo_t writer::text::tests::test_write_str
PASS [   0.063s] salvo_t writer::text::tests::test_write_string
PASS [   5.105s] salvo_t server::tests::test_server