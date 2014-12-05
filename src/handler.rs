use super::bindings;

pub use self::request::RequestHandler;
pub use self::response::ResponseHandler;

pub struct ParserSettings<T>(bindings::http_parser_settings);

type OptionalResult<T, E> = Option<Result<T, E>>;
pub struct HandlerContext<T: Sized, R, E> {
  pub handler: *mut T,
  pub ret: *mut OptionalResult<R, E>
}

impl<T, R, E> HandlerContext<T, R, E> {
  #[inline(always)]
  unsafe fn get<'a>(parser: *mut bindings::http_parser) -> &'a HandlerContext<T, R, E> {
    &*((*parser).data as *const HandlerContext<T, R, E>)
  }

  #[inline(always)]
  unsafe fn get_handler(&self) -> &mut T {
    &mut *self.handler
  }

  #[inline(always)]
  unsafe fn ret_pause(&self, parser: *mut bindings::http_parser) -> util::c_int {
    if (*self.ret).is_some() {
      bindings::http_parser_pause(parser, 1)
    };
    0
  }
}

macro_rules! cb(
  ($t:ident :: $f:ident ( )) => (
    extern "C" fn $f<T: $t<R, E>, R, E>(parser: *mut bindings::http_parser) -> c_int {
      unsafe {
        let ctx: &HandlerContext<T, R, E> = HandlerContext::get(parser);
        *ctx.ret = ctx.get_handler().$f(transmute(parser));
        ctx.ret_pause(parser)
      }
    }
  );

  ($t:ident :: $f:ident ([u8])) => (
    extern "C" fn $f<T: $t<R, E>, R, E>(parser: *mut bindings::http_parser,
                                        buf: *const c_char, len: size_t) -> c_int {
      let buf = buf as *const u8;
      unsafe {
        let s = slice::from_raw_buf(&buf, len as uint);
        let ctx: &HandlerContext<T, R, E> = HandlerContext::get(parser);
        *ctx.ret = ctx.get_handler().$f(transmute(parser), s);
        ctx.ret_pause(parser)
      }
    }
  );

  ($t:ident :: $f:ident (str)) => (
    extern "C" fn $f<T: $t<R, E>, R, E>(parser: *mut bindings::http_parser,
                                        buf: *const c_char, len: size_t) -> c_int {
      let buf = buf as *const u8;
      unsafe {
        let s = str::from_utf8_unchecked(slice::from_raw_buf(&buf, len as uint));
        let ctx: &HandlerContext<T, R, E> = HandlerContext::get(parser);
        *ctx.ret = ctx.get_handler().$f(transmute(parser), s);
        ctx.ret_pause(parser)
      }
    }
  );
)

macro_rules! cbs(
  ($t:ident $($method_name:ident $method_params:tt),* ) => {
    $(cb!($t :: $method_name $method_params))*
  };
)

mod request {
  use super::{OptionalResult, HandlerContext};
  use super::util::*;
  use super::super::RequestParser;

  pub trait RequestHandler<R, E> {
    fn on_message_begin(&mut self, parser: &mut RequestParser) -> OptionalResult<R, E>;
    fn on_url(&mut self, parser: &mut RequestParser, url: &str) -> OptionalResult<R, E>;
    fn on_header_field(&mut self, parser: &mut RequestParser, field: &str) -> OptionalResult<R, E>;
    fn on_header_value(&mut self, parser: &mut RequestParser, value: &str) -> OptionalResult<R, E>;
    fn on_headers_complete(&mut self, parser: &mut RequestParser) -> OptionalResult<R, E>;
    fn on_body(&mut self, parser: &mut RequestParser, buf: &[u8]) -> OptionalResult<R, E>;
    fn on_message_complete(&mut self, parser: &mut RequestParser) -> OptionalResult<R, E>;

    fn to_settings() -> ParserSettings<Self> {
      ParserSettings(bindings::http_parser_settings {
        on_message_begin: Some(on_message_begin::<Self, R, E>),
        on_url: Some(on_url::<Self, R, E>),
        on_status: None,
        on_header_field: Some(on_header_field::<Self, R, E>),
        on_header_value: Some(on_header_value::<Self, R, E>),
        on_headers_complete: Some(on_headers_complete::<Self, R, E>),
        on_body: Some(on_body::<Self, R, E>),
        on_message_complete: Some(on_message_complete::<Self, R, E>)
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
  use super::{OptionalResult, HandlerContext};
  use super::util::*;
  use super::super::ResponseParser;

  pub trait ResponseHandler<R, E> {
    fn on_message_begin(&mut self, parser: &mut ResponseParser) -> OptionalResult<R, E>;
    fn on_status(&mut self, parser: &mut ResponseParser, status: &str) -> OptionalResult<R, E>;
    fn on_header_field(&mut self, parser: &mut ResponseParser, field: &str) -> OptionalResult<R, E>;
    fn on_header_value(&mut self, parser: &mut ResponseParser, value: &str) -> OptionalResult<R, E>;
    fn on_headers_complete(&mut self, parser: &mut ResponseParser) -> OptionalResult<R, E>;
    fn on_body(&mut self, parser: &mut ResponseParser, buf: &[u8]) -> OptionalResult<R, E>;
    fn on_message_complete(&mut self, parser: &mut ResponseParser) -> OptionalResult<R, E>;

    fn to_settings() -> ParserSettings<Self> {
      ParserSettings(bindings::http_parser_settings {
        on_message_begin: Some(on_message_begin::<Self, R, E>),
        on_url: None,
        on_status: Some(on_status::<Self, R, E>),
        on_header_field: Some(on_header_field::<Self, R, E>),
        on_header_value: Some(on_header_value::<Self, R, E>),
        on_headers_complete: Some(on_headers_complete::<Self, R, E>),
        on_body: Some(on_body::<Self, R, E>),
        on_message_complete: Some(on_message_complete::<Self, R, E>)
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
  pub use std::{str, slice};
  pub use super::ParserSettings;
  pub use std::mem::transmute;
}
