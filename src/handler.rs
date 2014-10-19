use super::bindings;
use libc::{size_t, c_char, c_int};
use std::slice::raw::buf_as_slice;
use std::str;

pub struct ParserSettings<T>(bindings::http_parser_settings);

pub trait ResponseHandler: Handler {
  fn on_status(&mut self, status: &str);
  fn to_settings() -> ParserSettings<Self> {
    ParserSettings(bindings::http_parser_settings {
      on_message_begin: Some(on_message_begin::<Self>),
      on_url: None,
      on_status: Some(on_status::<Self>),
      on_header_field: Some(on_header_field::<Self>),
      on_header_value: Some(on_header_value::<Self>),
      on_headers_complete: Some(on_headers_complete::<Self>),
      on_body: Some(on_body::<Self>),
      on_message_complete: Some(on_message_complete::<Self>)
    })
  }
}

pub trait RequestHandler: Handler {
  fn on_url(&mut self, url: &str);
  fn to_settings() -> ParserSettings<Self> {
    ParserSettings(bindings::http_parser_settings {
      on_message_begin: Some(on_message_begin::<Self>),
      on_url: Some(on_url::<Self>),
      on_status: None,
      on_header_field: Some(on_header_field::<Self>),
      on_header_value: Some(on_header_value::<Self>),
      on_headers_complete: Some(on_headers_complete::<Self>),
      on_body: Some(on_body::<Self>),
      on_message_complete: Some(on_message_complete::<Self>)
    })
  }
}

pub trait Handler {
  fn on_message_begin(&mut self);
  fn on_header_field(&mut self, field: &str);
  fn on_header_value(&mut self, value: &str);
  fn on_headers_complete(&mut self);
  fn on_body(&mut self, buf: &[u8]);
  fn on_message_complete(&mut self);
}

pub fn to_raw_settings<T>(settings: &ParserSettings<T>) -> &bindings::http_parser_settings {
  &settings.0
}

#[inline(always)]
unsafe fn get_handler<'a, T>(parser: *mut bindings::http_parser) -> &'a mut T {
  &mut *((*parser).data as *mut T)
}

extern "C" fn on_message_begin<T: Handler>(parser: *mut bindings::http_parser) -> c_int {
  unsafe {
    get_handler::<T>(parser).on_message_begin();
    0
  }
}

extern "C" fn on_url<T: RequestHandler>(parser: *mut bindings::http_parser,
                                        buf: *const c_char, len: size_t) -> c_int {
  unsafe {
    buf_as_str(buf as *const u8, len as uint, |s| {
      get_handler::<T>(parser).on_url(s);
    })
  }
  0
}

extern "C" fn on_status<T: ResponseHandler>(parser: *mut bindings::http_parser,
                                            buf: *const c_char, len: size_t) -> c_int {
  unsafe {
    buf_as_str(buf as *const u8, len as uint, |s| {
      get_handler::<T>(parser).on_status(s);
    })
  }
  0
}


extern "C" fn on_header_field<T: Handler>(parser: *mut bindings::http_parser,
                                          buf: *const c_char, len: size_t) -> c_int {
  unsafe {
    buf_as_str(buf as *const u8, len as uint, |s| {
      get_handler::<T>(parser).on_header_field(s);
    })
  }
  0
}

extern "C" fn on_header_value<T: Handler>(parser: *mut bindings::http_parser,
                                          buf: *const c_char, len: size_t) -> c_int {
  unsafe {
    buf_as_str(buf as *const u8, len as uint, |s| {
      get_handler::<T>(parser).on_header_value(s);
    })
  }
  0
}

extern "C" fn on_headers_complete<T: Handler>(parser: *mut bindings::http_parser) -> c_int {
  unsafe {
    get_handler::<T>(parser).on_headers_complete();
    0
  }
}

extern "C" fn on_body<T: Handler>(parser: *mut bindings::http_parser,
                                  buf: *const c_char, len: size_t) -> c_int {
  unsafe {
    buf_as_slice(buf as *const u8, len as uint, |s| {
      get_handler::<T>(parser).on_body(s);
    })
  }
  0
}

extern "C" fn on_message_complete<T: Handler>(parser: *mut bindings::http_parser) -> c_int {
  unsafe {
    get_handler::<T>(parser).on_message_complete();
    0
  }
}

unsafe fn buf_as_str<T>(ptr: *const u8, len: uint, f: |&str| -> T) -> T {
  buf_as_slice(ptr, len, |buf| {
    f(str::raw::from_utf8(buf))
  })
}
