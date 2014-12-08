#![feature(macro_rules, globs)]

extern crate libc;
use std::mem::uninitialized;
use libc::{size_t, c_char};
use handler::{to_raw_settings, HandlerContext};

pub use handler::{ParserSettings, RequestHandler, ResponseHandler};

mod bindings;
mod handler;

#[repr(C)]
pub struct RequestParser(bindings::http_parser);
#[repr(C)]
pub struct ResponseParser(bindings::http_parser);

impl RequestParser {
  pub fn new() -> RequestParser {
    unsafe {
      let mut parser: RequestParser = uninitialized();
      bindings::http_parser_init(&mut parser.0, bindings::HTTP_REQUEST);
      parser
    }
  }

  pub fn execute<H, R, E>(&mut self, handler: &mut H,
                          settings: &ParserSettings<H>,
                          data: &[u8], ret: &mut Option<Result<R, E>>) -> uint
                          where H: RequestHandler<R, E> {
    unsafe {
      *ret = None;
      let ctx = HandlerContext {
        handler: handler,
        ret: ret
      };
      self.0.data = &ctx as *const _ as *mut ();
      let bytes_read = {
        bindings::http_parser_execute(&mut self.0,
                                      to_raw_settings(settings),
                                      data.as_ptr() as *const c_char,
                                      data.len() as size_t)
      };
      match self.0.errno() {
        bindings::HPE_PAUSED | bindings::HPE_OK => {},
        errno => panic!("unexpected error: {}", errno)
      }
      bytes_read as uint
    }
  }
  pub fn http_version(&self) -> (u16, u16) {
    (self.0.http_major as u16,
     self.0.http_minor as u16)
  }

  pub fn should_keep_alive(&self) -> bool {
    unsafe {
      bindings::http_should_keep_alive(&self.0) != 0
    }
  }

  pub fn is_upgrade(&self) -> bool {
    self.0.is_upgrade()
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

  pub fn execute<H, R, E>(&mut self, handler: &mut H,
                          settings: &ParserSettings<H>,
                          data: &[u8], ret: &mut Option<Result<R, E>>) -> uint
                          where H: ResponseHandler<R, E> {
    unsafe {
      *ret = None;
      let ctx = HandlerContext {
        handler: handler,
        ret: ret
      };
      self.0.data = &ctx as *const _ as *mut ();
      let bytes_read = {
        bindings::http_parser_execute(&mut self.0,
        to_raw_settings(settings),
        data.as_ptr() as *const c_char,
        data.len() as size_t)
      };
      assert_eq!(self.0.errno(), bindings::HPE_OK);
      bytes_read as uint
    }
  }

  pub fn status_code(&self) -> u16 {
    self.0.status_code
  }
}
