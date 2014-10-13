#![feature(macro_rules)]
#![feature(tuple_indexing)]

extern crate native;
extern crate libc;
use std::mem::uninitialized;
use std::slice::raw::buf_as_slice;
use std::str;

#[allow(dead_code, non_camel_case_types, non_uppercase_statics)]
mod c;

pub static HTTP_REQUEST: u32 = c::HTTP_REQUEST;
pub static HTTP_RESPONSE: u32 = c::HTTP_RESPONSE;
pub static HTTP_BOTH: u32 = c::HTTP_BOTH;

pub struct HttpParserSettings<T>(c::Struct_http_parser_settings);

pub trait HttpHandler {
  fn on_message_begin(&mut self);
  fn on_url(&mut self, url: &str);
  fn on_status(&mut self, status: &str);
  fn on_header_field(&mut self, field: &str);
  fn on_header_value(&mut self, value: &str);
  fn on_headers_complete(&mut self);
  fn on_body(&mut self, buf: &[u8]);
  fn on_message_complete(&mut self);
}

pub struct HttpParser {
  parser: c::Struct_http_parser,
  handler: *mut ()
}

impl<T: HttpHandler> HttpParserSettings<T> {
  pub fn new() -> HttpParserSettings<T> {
    // I tried making those macros but had problems passing self down to the 
    // macro and wanted to just get this done. TODO: investigate turning this
    // into macros once more.
    extern "C" fn on_message_begin<T: HttpHandler>(parser: *mut c::Struct_http_parser) -> i32 {
      unsafe {
        get_handler::<T>(parser).on_message_begin();
        0
      }
    }

    extern "C" fn on_url<T: HttpHandler>(parser: *mut c::Struct_http_parser,
                                         buf: *const libc::c_char, len: c::size_t) -> i32 {
      unsafe {
        buf_as_str(buf as *const u8, len as uint, |s| {
          get_handler::<T>(parser).on_url(s);
        })
      }
      0
    }

    extern "C" fn on_header_field<T: HttpHandler>(parser: *mut c::Struct_http_parser,
                                                  buf: *const libc::c_char, len: c::size_t) -> i32 {
      unsafe {
        buf_as_str(buf as *const u8, len as uint, |s| {
          get_handler::<T>(parser).on_header_field(s);
        })
      }
      0
    }

    extern "C" fn on_header_value<T: HttpHandler>(parser: *mut c::Struct_http_parser,
                                                  buf: *const libc::c_char, len: c::size_t) -> i32 {
      unsafe {
        buf_as_str(buf as *const u8, len as uint, |s| {
          get_handler::<T>(parser).on_header_value(s);
        })
      }
      0
    }

    extern "C" fn on_headers_complete<T: HttpHandler>(parser: *mut c::Struct_http_parser) -> i32 {
      unsafe {
        get_handler::<T>(parser).on_headers_complete();
        0
      }
    }

    extern "C" fn on_body<T: HttpHandler>(parser: *mut c::Struct_http_parser,
                                          buf: *const libc::c_char, len: c::size_t) -> i32 {
      unsafe {
        buf_as_slice(buf as *const u8, len as uint, |s| {
          get_handler::<T>(parser).on_body(s);
        })
      }
      0
    }

    extern "C" fn on_message_complete<T: HttpHandler>(parser: *mut c::Struct_http_parser) -> i32 {
      unsafe {
        get_handler::<T>(parser).on_message_complete();
        0
      }
    }

    #[inline(always)]
    unsafe fn get_handler<'a, T>(parser: *mut c::Struct_http_parser) -> &'a mut T {
      let parser = &mut *(parser as *mut HttpParser);
      &mut *(parser.handler as *mut T)
    }

    HttpParserSettings(c::Struct_http_parser_settings {
      on_message_begin: Some(on_message_begin::<T>),
      on_url: Some(on_url::<T>),
      on_status: None,
      on_header_field: Some(on_header_field::<T>),
      on_header_value: Some(on_header_value::<T>),
      on_headers_complete: Some(on_headers_complete::<T>),
      on_body: Some(on_body::<T>),
      on_message_complete: Some(on_message_complete::<T>)
    })
  }
}

impl HttpParser {
  pub fn new(type_: u32) -> HttpParser {
    unsafe {
      let mut parser: HttpParser = uninitialized();
      c::http_parser_init(&mut parser.parser, type_);
      parser
    }
  }

  pub fn execute<T>(&mut self, handler: &mut T, settings: &HttpParserSettings<T>, data: &[u8]) {
    unsafe {
      self.handler = (handler as *mut T) as *mut ();
      c::http_parser_execute(&mut self.parser, &settings.0,
                             data.as_ptr() as *const libc::c_char, data.len() as libc::size_t);
    }
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
