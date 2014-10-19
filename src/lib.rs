#![feature(macro_rules)]
#![feature(tuple_indexing)]

extern crate native;
extern crate libc;
use std::mem::uninitialized;
use libc::{size_t, c_char};
use handler::to_raw_settings;

pub use handler::{ParserSettings, RequestHandler, ResponseHandler, Handler};

mod bindings;
mod handler;

pub trait Parser {
  fn should_keep_alive(&self) -> bool;
}

pub struct RequestParser(bindings::http_parser);
pub struct ResponseParser(bindings::http_parser);

impl RequestParser {
  pub fn new() -> RequestParser {
    unsafe {
      let mut parser: RequestParser = uninitialized();
      bindings::http_parser_init(&mut parser.0, bindings::HTTP_REQUEST);
      parser
    }
  }

  pub fn execute<T: RequestHandler>(&mut self, handler: &mut T,
                                    settings: &ParserSettings<T>,
                                    data: &[u8]) {
    unsafe {
      self.0.data = (handler as *mut T) as *mut ();
      bindings::http_parser_execute(&mut self.0, to_raw_settings(settings),
                                    data.as_ptr() as *const c_char,
                                    data.len() as size_t);
    }
  }
}

impl Parser for RequestParser {
  fn should_keep_alive(&self) -> bool {
    unsafe {
      bindings::http_should_keep_alive(&self.0) != 0
    }
  }
}

impl ResponseParser {
  pub fn new() -> ResponseParser {
    unsafe {
      let mut parser: ResponseParser = uninitialized();
      bindings::http_parser_init(&mut parser.0, bindings::HTTP_RESPONSE);
      parser
    }
  }

  pub fn execute<T: ResponseHandler>(&mut self, handler: &mut T,
                                     settings: &ParserSettings<T>,
                                     data: &[u8]) {
    unsafe {
      self.0.data = (handler as *mut T) as *mut ();
      bindings::http_parser_execute(&mut self.0, to_raw_settings(settings),
                                    data.as_ptr() as *const c_char,
                                    data.len() as size_t);
    }
  }
}

impl Parser for ResponseParser {
  fn should_keep_alive(&self) -> bool {
    unsafe {
      bindings::http_should_keep_alive(&self.0) != 0
    }
  }
}
