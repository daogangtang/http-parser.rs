#![feature(macro_rules)]

extern crate native;
extern crate libc;
use std::mem::uninitialized;
use std::slice::raw::buf_as_slice;
use std::str;

#[allow(dead_code, uppercase_variables, non_camel_case_types, non_uppercase_statics)]
mod c;

pub static HTTP_REQUEST: u32 = c::HTTP_REQUEST;
pub static HTTP_RESPONSE: u32 = c::HTTP_RESPONSE;
pub static HTTP_BOTH: u32 = c::HTTP_BOTH;

pub type HttpCb = fn (parser: &HttpParser) -> Result<(), ()>;
pub type HttpDataCb = fn (parser: &HttpParser, data: &str) -> Result<(), ()>;

pub struct HttpParserSettings {
  pub on_message_begin: Option<HttpCb>,
  pub on_url: Option<HttpDataCb>,
  pub on_status: Option<HttpDataCb>,
  pub on_header_field: Option<HttpDataCb>,
  pub on_header_value: Option<HttpDataCb>,
  pub on_headers_complete: Option<HttpCb>,
  pub on_body: Option<HttpDataCb>,
  pub on_message_complete: Option<HttpCb>,
}

pub struct HttpParser {
  parser: c::Struct_http_parser,
}

impl HttpParserSettings {
  pub fn to_native(&self) -> c::Struct_http_parser_settings {
    // I tried making those macros but had problems passing self down to the 
    // macro and wanted to just get this done. TODO: investigate turning this
    // into macros once more.
    extern "C" fn on_message_begin_wrap(parser: *mut c::Struct_http_parser) -> i32 {
      println!("on_message_begin");
      0
    }

    extern "C" fn on_url_wrap(parser: *mut c::Struct_http_parser,
                              data: *const libc::c_char, data_len: c::size_t) -> i32 {
      unsafe {
        buf_as_str(data as *const u8, data_len as uint, |url| {
          println!("on_url: {}", url);
        })
      }
      0
    }

    c::Struct_http_parser_settings {
      on_message_begin: self.on_message_begin.map(|_| on_message_begin_wrap),
      on_url: self.on_message_begin.map(|_| on_url_wrap),
      on_status: None,
      on_header_field: None,
      on_header_value: None,
      on_headers_complete: None,
      on_body: None,
      on_message_complete: None
    }
  }
}

impl HttpParser {
  pub fn new(type_: u32) -> HttpParser {
    let mut parser: c::Struct_http_parser = unsafe { uninitialized() };
    unsafe { c::http_parser_init(&mut parser, type_) };
    HttpParser {
      parser: parser
    }
  }

  pub fn execute(&mut self, settings: HttpParserSettings, data: &[u8]) {
    unsafe { c::http_parser_execute(&mut self.parser, &settings.to_native(),
                                    data.as_ptr() as *const libc::c_char, data.len() as libc::size_t) };
  }

  pub fn should_keep_alive(&self) -> bool {
    unsafe {
      c::http_should_keep_alive(&self.parser) != 0
    }
  }
}

unsafe fn buf_as_str<T>(ptr: *const u8, len: uint, f: |&str| -> T) -> T {
  buf_as_slice(ptr, len, |buf| {
    f(str::raw::from_utf8(buf))
  })
}
