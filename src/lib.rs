#![feature(macro_rules)]
#![feature(tuple_indexing)]

extern crate native;
extern crate libc;
use std::mem::uninitialized;
use std::slice::raw::buf_as_slice;
use std::str;
use libc::{size_t, c_char, c_int};

#[allow(non_camel_case_types, dead_code)]
mod bindings;

#[repr(C)]
pub enum ParserType {
  Request,
  Response,
  Both
}

impl ParserType {
  fn to_c(&self) -> bindings::http_parser_type {
    match *self {
      Request  => bindings::HTTP_REQUEST,
      Response => bindings::HTTP_RESPONSE,
      Both     => bindings::HTTP_BOTH
    }
  }
}

pub struct HttpParserSettings<T>(bindings::http_parser_settings);

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

pub struct HttpParser(bindings::http_parser);

impl<T: HttpHandler> HttpParserSettings<T> {
  pub fn new() -> HttpParserSettings<T> {
    // I tried making those macros but had problems passing self down to the
    // macro and wanted to just get this done. TODO: investigate turning this
    // into macros once more.
    extern "C" fn on_message_begin<T: HttpHandler>(parser: *mut bindings::http_parser) -> c_int {
      unsafe {
        get_handler::<T>(parser).on_message_begin();
        0
      }
    }

    extern "C" fn on_url<T: HttpHandler>(parser: *mut bindings::http_parser,
                                         buf: *const c_char, len: size_t) -> c_int {
      unsafe {
        buf_as_str(buf as *const u8, len as uint, |s| {
          get_handler::<T>(parser).on_url(s);
        })
      }
      0
    }

    extern "C" fn on_header_field<T: HttpHandler>(parser: *mut bindings::http_parser,
                                                  buf: *const c_char, len: size_t) -> c_int {
      unsafe {
        buf_as_str(buf as *const u8, len as uint, |s| {
          get_handler::<T>(parser).on_header_field(s);
        })
      }
      0
    }

    extern "C" fn on_header_value<T: HttpHandler>(parser: *mut bindings::http_parser,
                                                  buf: *const c_char, len: size_t) -> c_int {
      unsafe {
        buf_as_str(buf as *const u8, len as uint, |s| {
          get_handler::<T>(parser).on_header_value(s);
        })
      }
      0
    }

    extern "C" fn on_headers_complete<T: HttpHandler>(parser: *mut bindings::http_parser) -> c_int {
      unsafe {
        get_handler::<T>(parser).on_headers_complete();
        0
      }
    }

    extern "C" fn on_body<T: HttpHandler>(parser: *mut bindings::http_parser,
                                          buf: *const c_char, len: size_t) -> c_int {
      unsafe {
        buf_as_slice(buf as *const u8, len as uint, |s| {
          get_handler::<T>(parser).on_body(s);
        })
      }
      0
    }

    extern "C" fn on_message_complete<T: HttpHandler>(parser: *mut bindings::http_parser) -> c_int {
      unsafe {
        get_handler::<T>(parser).on_message_complete();
        0
      }
    }

    #[inline(always)]
    unsafe fn get_handler<'a, T>(parser: *mut bindings::http_parser) -> &'a mut T {
      &mut *((*parser).data as *mut T)
    }

    HttpParserSettings(bindings::http_parser_settings {
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
  pub fn new(type_: ParserType) -> HttpParser {
    unsafe {
      let mut parser: HttpParser = uninitialized();
      bindings::http_parser_init(&mut parser.0, type_.to_c());
      parser
    }
  }

  pub fn execute<T>(&mut self, handler: &mut T, settings: &HttpParserSettings<T>, data: &[u8]) {
    unsafe {
      self.0.data = (handler as *mut T) as *mut ();
      bindings::http_parser_execute(&mut self.0, &settings.0,
                                    data.as_ptr() as *const c_char, data.len() as size_t);
    }
  }

  pub fn should_keep_alive(&self) -> bool {
    unsafe {
      bindings::http_should_keep_alive(&self.0) != 0
    }
  }
}

unsafe fn buf_as_str<T>(ptr: *const u8, len: uint, f: |&str| -> T) -> T {
  buf_as_slice(ptr, len, |buf| {
    f(str::raw::from_utf8(buf))
  })
}
