pub use launchpad_macros::{connect, delete, get, head, options, patch, post, put, request, trace};
pub use launchpad_macros::catch;

pub use crate::Result;
pub use crate::Error;

macro_rules! routes {
    ($endpoint: ident) => {
        [
            ::launchpad::router::Route::from_endpoint(
                std::sync::Arc::new($endpoint(std::sync::Mutex::new(::launchpad::request::State::default())))
            )
        ]
    };
    ($path: literal => $endpoint: expr) => {
        [
            ::launchpad::router::Route::new(
                $path,
                std::sync::Arc::new($endpoint(std::sync::Mutex::new(::launchpad::request::State::default())))
            )
        ]
    };
    ($endpoint: ident, $($rest: tt)*) => {
        routes!(
            @nest 
            [
                ::launchpad::router::Route::from_endpoint(
                    std::sync::Arc::new($endpoint(std::sync::Mutex::new(::launchpad::request::State::default())))
                )
            ],
            @rest $($rest)*
        )
    };
    ($path: literal => $endpoint: expr, $($rest: tt)*) => {
        routes!(
            @nest
            [
                ::launchpad::router::Route::new(
                    $path.to_string(),
                    std::sync::Arc::new($endpoint(std::sync::Mutex::new(::launchpad::request::State::default())))
                )
            ],
            @rest $($rest)*
        )
    };
    (@nest [$($total: expr),*], @rest $endpoint: ident, $($rest: tt)*) => {
        routes!(
            @nest
            [
                $($total,)*
                ::launchpad::router::Route::from_endpoint(
                    std::sync::Arc::new($endpoint(std::sync::Mutex::new(::launchpad::request::State::default())))
                )
            ],
            @rest $($rest)*
        )
    };
    (@nest [$($total: expr),*], @rest $path: literal => $endpoint: expr, $($rest: tt)*) => {
        routes!(
            @nest
            [
                $($total,)*
                ::launchpad::router::Route::new(
                    $path,
                    std::sync::Arc::new($endpoint(std::sync::Mutex::new(::launchpad::request::State::default())))
                )
            ],
            @rest $($rest)*
        )
    };
    (@nest [$($total: expr),*], @rest $endpoint: ident $(,)?) => {
        [
            $($total,)*
            ::launchpad::router::Route::from_endpoint(
                std::sync::Arc::new($endpoint(std::sync::Mutex::new(::launchpad::request::State::default())))
            )
        ]
    };
    (@nest [$($total: expr),*], @rest $path: literal => $endpoint: expr $(,)?) => {
        [
            $($total,)*
            ::launchpad::router::Route::new(
                $path,
                std::sync::Arc::new($endpoint(std::sync::Mutex::new(::launchpad::request::State::default())))
            )
        ]
    };
    (@nest [$($total: expr),*], @rest) => {
        [
            $($total,)*
        ]
    };
}

macro_rules! errors {
    ($handler: ident) => {
        [
            ::launchpad::router::Catch::from_catch(
                std::sync::Arc::new($handler())
            )
        ]
    };
    ($code: literal => $handler: expr) => {
        [
            ::launchpad::router::Catch::new(
                $code,
                std::sync::Arc::new($handler())
            )
        ]
    };
    ($handler: ident, $($rest: tt)*) => {
        errors!(
            @nest 
            [
                ::launchpad::router::Catch::from_catch(
                    std::sync::Arc::new($handler())
                )
            ],
            @rest $($rest)*
        )
    };
    ($code: literal => $handler: expr, $($rest: tt)*) => {
        errors!(
            @nest
            [
                ::launchpad::router::Catch::new(
                    $code,
                    std::sync::Arc::new($handler())
                )
            ],
            @rest $($rest)*
        )
    };
    (@nest [$($total: expr),*], @rest $handler: ident, $($rest: tt)*) => {
        errors!(
            @nest
            [
                $($total,)*
                ::launchpad::router::Catch::from_catch(
                    std::sync::Arc::new($handler())
                )
            ],
            @rest $($rest)*
        )
    };
    (@nest [$($total: expr),*], @rest $code: literal => $handler: expr, $($rest: tt)*) => {
        errors!(
            @nest
            [
                $($total,)*
                ::launchpad::router::Catch::new(
                    $code,
                    std::sync::Arc::new($handler())
                )
            ],
            @rest $($rest)*
        )
    };
    (@nest [$($total: expr),*], @rest $handler: ident $(,)?) => {
        [
            $($total,)*
            ::launchpad::router::Catch::from_catch(
                std::sync::Arc::new($handler())
            )
        ],
    };
    (@nest [$($total: expr),*], @rest $code: literal => $handler: expr $(,)?) => {
        [
            $($total,)*
            ::launchpad::router::Catch::new(
                $code,
                std::sync::Arc::new($handler())
            )
        ]
    };
    (@nest [$($total: expr),*], @rest) => {
        [
            $($total,)*
        ]
    };
}

macro_rules! rts {
    ($routes: expr, $errors: expr $(,)?) => {
        ::launchpad::router::Router::from(($routes, $errors))
    };
    ($routes: expr $(,)?) => {
        ::launchpad::router::Router::from($routes)
    };
}

pub use routes;
pub use errors;
pub use rts;
