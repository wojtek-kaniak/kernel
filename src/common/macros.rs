macro_rules! include_data_bytes {
    ($path:literal) => {
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/data/", $path))
    };
}
pub(crate) use include_data_bytes;

macro_rules! function_name {
    () => {{
        #[allow(dead_code)]
        fn f() {}
        #[allow(dead_code)]
        fn type_name_of<T>(_: T) -> &'static str {
            core::any::type_name::<T>()
        }
        let name = type_name_of(f);
        &name[..name.len() - 3]
    }};
}
pub(crate) use function_name;

macro_rules! invalid_arg {
    ($arg:ident) => {
        panic!(
            "Invalid argument value ('{}' at {})",
            stringify!($arg),
            $crate::common::macros::function_name!()
        )
    };
    ($arg:ident, $message:expr) => {
        panic!(
            "{} ('{}' at {})",
            $message,
            stringify!($arg),
            $crate::common::macros::function_name!()
        )
    };
}
pub(crate) use invalid_arg;

macro_rules! debug_asserts {
    ($block:block) => {
        cfg_if::cfg_if! {
            if #[cfg(debug_assertions)] {
                $block
            }
        }
    };
}
pub(crate) use debug_asserts;

macro_rules! assert_arg {
    ($arg:ident, $expr:expr) => {
        if !($expr) {
            $crate::common::macros::invalid_arg!($arg);
        }
    };
    ($arg:ident, $expr:expr, $message:expr) => {
        if !($expr) {
            $crate::common::macros::invalid_arg!($arg, $message);
        }
    };
}
pub(crate) use assert_arg;

macro_rules! debug_assert_arg {
    ($arg:ident, $expr:expr) => {
        $crate::common::macros::debug_asserts!({
            $crate::common::macros::assert_arg!($arg, $expr);
        })
    };
    ($arg:ident, $expr:expr, $message:expr) => {
        $crate::common::macros::debug_asserts!({
            $crate::common::macros::assert_arg!($arg, $expr, $message);
        })
    };
}
pub(crate) use debug_assert_arg;

/// Prevents creating tokens safely
#[derive(Clone, Copy)]
pub struct InnerToken {
    _private: ()
}

impl InnerToken {
    pub const unsafe fn new() -> Self {
        Self { _private: () }
    }
}

macro_rules! token_type {
    ($type:ident) => {
        #[derive(Clone, Copy)]
        pub struct $type {
            _inner: crate::common::macros::InnerToken
        }

        impl $type {
            pub const unsafe fn new() -> Self {
                unsafe {
                    $type { _inner: crate::common::macros::InnerToken::new() }
                }
            }
        }
    };
}
pub(crate) use token_type;

macro_rules! token_from {
    ($from:ty, $to:ty) => {
        impl ::core::convert::From<$from> for $to {
            fn from(token: $from) -> Self {
                let _ = token;
                unsafe { <$to>::new() }
            }
        }
    };
}
pub(crate) use token_from;
