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

macro_rules! debug_assert_arg {
    ($arg:ident, $expr:expr) => {
        $crate::common::macros::debug_asserts!({
            if !($expr) {
                $crate::common::macros::invalid_arg!($arg);
            }
        })
    };
    ($arg:ident, $expr:expr, $message:expr) => {
        $crate::common::macros::debug_asserts!({
            if !($expr) {
                $crate::common::macros::invalid_arg!($arg, $message);
            }
        })
    };
}
pub(crate) use debug_assert_arg;

macro_rules! token_type {
    ($type:ident) => {
        #[derive(Clone, Copy)]
        pub struct $type;

        impl $type {
            pub const unsafe fn new() -> Self {
                $type
            }
        }
    };
}
pub(crate) use token_type;

macro_rules! token_from_unsafe {
    ($from:ty, $to:ty) => {
        impl ::core::convert::From<$from> for $to {
            fn from(token: $from) -> Self {
                let _ = token;
                unsafe { <$to>::new() }
            }
        }
    };
}
pub(crate) use token_from_unsafe;
