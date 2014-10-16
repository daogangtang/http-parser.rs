#![allow(non_camel_case_types, non_snake_case)]
extern crate libc;
use libc::{size_t, c_ulong, c_ushort, c_int, c_char};

pub type http_cb = Option<extern "C" fn(parser: *mut http_parser) -> c_int>;
pub type http_data_cb = Option<extern "C" fn(parser: *mut http_parser,
                                             buf: *const c_char,
                                             buf_len: size_t) -> c_int>;

#[repr(C)]
pub struct http_parser_settings {
    pub on_message_begin: http_cb,
    pub on_url: http_data_cb,
    pub on_status: http_data_cb,
    pub on_header_field: http_data_cb,
    pub on_header_value: http_data_cb,
    pub on_headers_complete: http_cb,
    pub on_body: http_data_cb,
    pub on_message_complete: http_cb,
}

#[repr(C)]
pub enum http_errno {
   HPE_OK,
   HPE_CB_message_begin,
   HPE_CB_url,
   HPE_CB_header_field,
   HPE_CB_header_value,
   HPE_CB_headers_complete,
   HPE_CB_body,
   HPE_CB_message_complete,
   HPE_CB_status,
   HPE_INVALID_EOF_STATE,
   HPE_HEADER_OVERFLOW,
   HPE_CLOSED_CONNECTION,
   HPE_INVALID_VERSION,
   HPE_INVALID_STATUSk,
   HPE_INVALID_METHOD,
   HPE_INVALID_URL,
   HPE_INVALID_HOST,
   HPE_INVALID_PORT,
   HPE_INVALID_PATH,
   HPE_INVALID_QUERY_STRING,
   HPE_INVALID_FRAGMENT,
   HPE_LF_EXPECTED,
   HPE_INVALID_HEADER_TOKEN,
   HPE_INVALID_CONTENT_LENGTH,
   HPE_INVALID_CHUNK_SIZE,
   HPE_INVALID_CONSTANT,
   HPE_INVALID_INTERNAL_STATE,
   HPE_STRICT,
   HPE_PAUSED,
   HPE_UNKNOWN,
}

#[repr(C)]
pub enum http_method {
  HTTP_DELETE,
  HTTP_GET,
  HTTP_HEAD,
  HTTP_POST,
  HTTP_PUT,
  HTTP_CONNECT,
  HTTP_OPTIONS,
  HTTP_TRACE,
  HTTP_COPY,
  HTTP_LOCK,
  HTTP_MKCOL,
  HTTP_MOVE,
  HTTP_PROPFIND,
  HTTP_PROPPATCH,
  HTTP_SEARCH,
  HTTP_UNLOCK,
  HTTP_REPORT,
  HTTP_MKACTIVITY,
  HTTP_CHECKOUT,
  HTTP_MERGE,
  HTTP_MSEARCH,
  HTTP_NOTIFY,
  HTTP_SUBSCRIBE,
  HTTP_UNSUBSCRIBE,
  HTTP_PATCH,
  HTTP_PURGE,
  HTTP_MKCALENDAR
}

#[repr(C)]
pub enum http_parser_type {
  HTTP_REQUEST,
  HTTP_RESPONSE,
  HTTP_BOTH
}

#[repr(C)]
pub struct http_parser {
  /// type: 2, flags: 6
  _type__flags: u8,
  _state: u8,
  _header_state: u8,
  _index: u8,
  _nread: u32,
  _content_length: u64,
  pub http_major: c_ushort,
  pub http_minor: c_ushort,
  pub status_code: u16,
  pub method: u8,
  /// http_errno: 7, upgrade: 1
  pub http_errno__upgrade: u8,
  pub data: *mut ()
}

#[link(name = "http_parser", kind = "static")]
extern "C" {
  pub fn http_parser_version() -> c_ulong;
  pub fn http_parser_init(parser: *mut http_parser,
                          type_: http_parser_type);
  pub fn http_parser_execute(parser: *mut http_parser,
                             settings: *const http_parser_settings,
                             data: *const c_char, len: size_t) -> size_t;
  pub fn http_should_keep_alive(parser: *const http_parser) -> c_int;
  pub fn http_method_str(m: http_method) -> *const c_char;
  pub fn http_errno_name(err: http_errno) -> *const c_char;
  pub fn http_errno_description(err: http_errno) -> *const c_char;
/*pub fn http_parser_parse_url(buf: *const c_char, buflen: size_t,
                               is_connect: c_int,
                               u: *mut Struct_http_parser_url) -> c_int; */
  pub fn http_parser_pause(parser: *mut http_parser, paused: c_int);
  pub fn http_body_is_final(parser: *const http_parser) -> c_int;
}
