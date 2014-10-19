#![feature(macro_rules)]
#![feature(tuple_indexing)]

extern crate native;
extern crate libc;
use std::mem::uninitialized;
use libc::{size_t, c_char};
use handler::to_raw_settings;

pub use handler::{HttpParserSettings, HttpHandler};

pub use bindings::http_parser_type as ParserType;
pub use bindings::HTTP_REQUEST as Request;
pub use bindings::HTTP_RESPONSE as Response;
pub use bindings::HTTP_BOTH as Both;

mod bindings;
mod handler;

pub struct HttpParser(bindings::http_parser);

impl HttpParser {
  pub fn new(type_: ParserType) -> HttpParser {
    unsafe {
      let mut parser: HttpParser = uninitialized();
      bindings::http_parser_init(&mut parser.0, type_);
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
