#![feature(macro_rules)]
#![feature(tuple_indexing)]

extern crate native;
extern crate libc;
use std::mem::uninitialized;
use libc::{size_t, c_char};

pub use handler::{HttpParserSettings, HttpHandler};
use handler::to_raw_settings;

#[allow(non_camel_case_types, dead_code)]
mod bindings;
mod handler;

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

pub struct HttpParser(bindings::http_parser);

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
      bindings::http_parser_execute(&mut self.0, to_raw_settings(settings),
                                    data.as_ptr() as *const c_char, data.len() as size_t);
    }
  }

  pub fn should_keep_alive(&self) -> bool {
    unsafe {
      bindings::http_should_keep_alive(&self.0) != 0
    }
  }
}
