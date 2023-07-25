#[macro_export]
macro_rules! routes {
    ($endpoint: ident) => {
        [
            ::launchpad_router::Route::from_endpoint(
                std::sync::Arc::new($endpoint(std::sync::Mutex::new(::launchpad_router::request::State::default())))
            )
        ]
    };
    ($path: literal => $endpoint: expr) => {
        [
            ::launchpad_router::Route::new(
                $path,
                std::sync::Arc::new($endpoint(std::sync::Mutex::new(::launchpad_router::request::State::default())))
            )
        ]
    };
    ($endpoint: ident, $($rest: tt)*) => {
        ::launchpad_router::routes!(
            @nest
            [
                ::launchpad_router::Route::from_endpoint(
                    std::sync::Arc::new($endpoint(std::sync::Mutex::new(::launchpad_router::request::State::default())))
                )
            ],
            @rest $($rest)*
        )
    };
    ($path: literal => $endpoint: expr, $($rest: tt)*) => {
        ::launchpad_router::routes!(
            @nest
            [
                ::launchpad_router::Route::new(
                    $path.to_string(),
                    std::sync::Arc::new($endpoint(std::sync::Mutex::new(::launchpad_router::request::State::default())))
                )
            ],
            @rest $($rest)*
        )
    };
    (@nest [$($total: expr),*], @rest $endpoint: ident, $($rest: tt)*) => {
        ::launchpad_router::routes!(
            @nest
            [
                $($total,)*
                ::launchpad_router::Route::from_endpoint(
                    std::sync::Arc::new($endpoint(std::sync::Mutex::new(::launchpad_router::request::State::default())))
                )
            ],
            @rest $($rest)*
        )
    };
    (@nest [$($total: expr),*], @rest $path: literal => $endpoint: expr, $($rest: tt)*) => {
        ::launchpad_router::routes!(
            @nest
            [
                $($total,)*
                ::launchpad_router::Route::new(
                    $path,
                    std::sync::Arc::new($endpoint(std::sync::Mutex::new(::launchpad_router::request::State::default())))
                )
            ],
            @rest $($rest)*
        )
    };
    (@nest [$($total: expr),*], @rest $endpoint: ident $(,)?) => {
        [
            $($total,)*
            ::launchpad_router::Route::from_endpoint(
                std::sync::Arc::new($endpoint(std::sync::Mutex::new(::launchpad_router::request::State::default())))
            )
        ]
    };
    (@nest [$($total: expr),*], @rest $path: literal => $endpoint: expr $(,)?) => {
        [
            $($total,)*
            ::launchpad_router::Route::new(
                $path,
                std::sync::Arc::new($endpoint(std::sync::Mutex::new(::launchpad_router::request::State::default())))
            )
        ]
    };
    (@nest [$($total: expr),*], @rest) => {
        [
            $($total,)*
        ]
    };
}

#[macro_export]
macro_rules! errors {
    ($handler: ident) => {
        [
            ::launchpad_router::Catch::from_catch(
                std::sync::Arc::new($handler())
            )
        ]
    };
    ($code: literal => $handler: expr) => {
        [
            ::launchpad_router::Catch::new(
                $code,
                std::sync::Arc::new($handler())
            )
        ]
    };
    ($handler: ident, $($rest: tt)*) => {
        ::launchpad_router::errors!(
            @nest
            [
                ::launchpad_router::Catch::from_catch(
                    std::sync::Arc::new($handler())
                )
            ],
            @rest $($rest)*
        )
    };
    ($code: literal => $handler: expr, $($rest: tt)*) => {
        ::launchpad_router::errors!(
            @nest
            [
                ::launchpad_router::Catch::new(
                    $code,
                    std::sync::Arc::new($handler())
                )
            ],
            @rest $($rest)*
        )
    };
    (@nest [$($total: expr),*], @rest $handler: ident, $($rest: tt)*) => {
        ::launchpad_router::errors!(
            @nest
            [
                $($total,)*
                ::launchpad_router::Catch::from_catch(
                    std::sync::Arc::new($handler())
                )
            ],
            @rest $($rest)*
        )
    };
    (@nest [$($total: expr),*], @rest $code: literal => $handler: expr, $($rest: tt)*) => {
        ::launchpad_router::errors!(
            @nest
            [
                $($total,)*
                ::launchpad_router::Catch::new(
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
            ::launchpad_router::Catch::from_catch(
                std::sync::Arc::new($handler())
            )
        ],
    };
    (@nest [$($total: expr),*], @rest $code: literal => $handler: expr $(,)?) => {
        [
            $($total,)*
            ::launchpad_router::Catch::new(
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

/// Construct a router given a list of routes, and or a list of error handlers
///
/// # Example
///
/// Assume that the following method is in both examples
/// ```
/// #[get("/")]
/// fn home() -> Result<&'static str> {
///     Ok("Hello, world!")
/// }
///
/// #[catch(404)]
/// fn not_found(code: u16, message: String) -> String {
///     format!("<h1>{} {}</h1>", code, message)
/// }
/// ```
///
/// `rts!` can be used by writing `[]` for routes and `{}` for errors
/// ```
/// use launchpad_router::prelude::*;
///
/// let router = rts!{
///     [home],
///     {not_found}
/// }
/// ```
///
/// If you want to specify the `route/uri` for the endpoint in the macro you can
/// use it similar to a map macro.
/// ```
/// use launchpad_router::prelude::*;
///
/// let router = rts!{
///     ["/" => home],
///     {500 => not_found}
/// }
/// ```
///
/// There are optional labels/tags before each collection to help associate sections.
/// ```
/// use launchpad_router::prelude::*;
///
/// let router = rts! {
///     route [home],
///     catch {
///         500 => not_found
///     }
/// }
/// ```
#[macro_export]
macro_rules! rts {
    ($(routes )? [$($routes: tt)*], $(catch )? {$($errors: tt)*} $(,)?) => {
        ::launchpad_router::Router::from((::launchpad_router::routes!($($routes)*), ::launchpad_router::errors!($($errors)*)))
    };
    ($(catch )? {$($errors: tt)*}, $(route )? [$($routes: tt)*] $(,)?) => {
        ::launchpad_router::Router::from((::launchpad_router::routes!($($routes)*), ::launchpad_router::errors!($($errors)*)))
    };
    ($(routes )? [$($routes: tt)*] $(,)?) => {
        ::launchpad_router::Router::from(::launchpad_router::routes!($($routes)*))
    };
    ($(catch )? {$($errors: tt)*} $(,)?) => {
        ::launchpad_router::Router::from(::launchpad_router::errors!($($errors)*))
    };
}
