use super::bindings;

pub use self::request::RequestHandler;
pub use self::response::ResponseHandler;

pub struct ParserSettings<T>(bindings::http_parser_settings);

macro_rules! cb(
  ($t:ident :: $f:ident ( )) => (
    extern "C" fn $f<T: $t>(parser: *mut bindings::http_parser) -> c_int {
      unsafe {
        get_handler::<T>(parser).$f(transmute(parser));
        0
      }
    }
  );

  ($t:ident :: $f:ident ([u8])) => (
    extern "C" fn $f<T: $t>(parser: *mut bindings::http_parser,
                            buf: *const c_char, len: size_t) -> c_int {
      unsafe {
        buf_as_slice(buf as *const u8, len as uint, |s| {
          get_handler::<T>(parser).$f(transmute(parser), s);
        })
      }
      0
    }
  );

  ($t:ident :: $f:ident (str)) => (
    extern "C" fn $f<T: $t>(parser: *mut bindings::http_parser,
                            buf: *const c_char, len: size_t) -> c_int {
      unsafe {
        buf_as_str(buf as *const u8, len as uint, |s| {
          get_handler::<T>(parser).$f(transmute(parser), s);
        })
      }
      0
    }
  );
)

macro_rules! cbs(
  ($t:ident $($method_name:ident $method_params:tt),* ) => {
    $(cb!($t :: $method_name $method_params))*
  };
)

mod request {
  use super::util::*;
  use super::super::RequestParser;

  pub trait RequestHandler {
    fn on_message_begin(&mut self, parser: &mut RequestParser);
    fn on_url(&mut self, parser: &mut RequestParser, url: &str);
    fn on_header_field(&mut self, parser: &mut RequestParser, field: &str);
    fn on_header_value(&mut self, parser: &mut RequestParser, value: &str);
    fn on_headers_complete(&mut self, parser: &mut RequestParser);
    fn on_body(&mut self, parser: &mut RequestParser, buf: &[u8]);
    fn on_message_complete(&mut self, parser: &mut RequestParser);

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

  cbs!(RequestHandler
    on_message_begin(),
    on_url(str),
    on_header_field(str),
    on_header_value(str),
    on_headers_complete(),
    on_body([u8]),
    on_message_complete()
  )
}

mod response {
  use super::util::*;
  use super::super::ResponseParser;

  pub trait ResponseHandler {
    fn on_message_begin(&mut self, parser: &mut ResponseParser);
    fn on_status(&mut self, parser: &mut ResponseParser, status: &str);
    fn on_header_field(&mut self, parser: &mut ResponseParser, field: &str);
    fn on_header_value(&mut self, parser: &mut ResponseParser, value: &str);
    fn on_headers_complete(&mut self, parser: &mut ResponseParser);
    fn on_body(&mut self, parser: &mut ResponseParser, buf: &[u8]);
    fn on_message_complete(&mut self, parser: &mut ResponseParser);

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

  cbs!(ResponseHandler
    on_message_begin(),
    on_status(str),
    on_header_field(str),
    on_header_value(str),
    on_headers_complete(),
    on_body([u8]),
    on_message_complete()
  )
}

pub fn to_raw_settings<T>(settings: &ParserSettings<T>) -> &bindings::http_parser_settings {
  &settings.0
}

mod util {
  pub use libc::{size_t, c_char, c_int};
  pub use super::super::bindings;
  pub use std::slice::raw::buf_as_slice;
  pub use super::ParserSettings;
  pub use std::mem::transmute;
  use std::str;

  #[inline(always)]
  pub unsafe fn get_handler<'a, T>(parser: *mut bindings::http_parser) -> &'a mut T {
    &mut *((*parser).data as *mut T)
  }

  pub unsafe fn buf_as_str<T>(ptr: *const u8, len: uint, f: |&str| -> T) -> T {
    buf_as_slice(ptr, len, |buf| {
      f(str::raw::from_utf8(buf))
    })
  }
}
