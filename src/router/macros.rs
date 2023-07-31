#[macro_export]
macro_rules! routes {
    ($endpoint: ident) => {
        [
            ::launchpad::router::Route::from_endpoint(
                std::sync::Arc::new($endpoint(std::sync::Mutex::new(::launchpad::router::request::State::default())))
            )
        ]
    };
    ($path: literal => $endpoint: expr) => {
        [
            ::launchpad::router::Route::new(
                $path,
                std::sync::Arc::new($endpoint(std::sync::Mutex::new(::launchpad::router::request::State::default())))
            )
        ]
    };
    ($endpoint: ident, $($rest: tt)*) => {
        $crate::routes!(
            @nest
            [
                ::launchpad::router::Route::from_endpoint(
                    std::sync::Arc::new($endpoint(std::sync::Mutex::new(::launchpad::router::request::State::default())))
                )
            ],
            @rest $($rest)*
        )
    };
    ($path: literal => $endpoint: expr, $($rest: tt)*) => {
        $crate::routes!(
            @nest
            [
                ::launchpad::router::Route::new(
                    $path.to_string(),
                    std::sync::Arc::new($endpoint(std::sync::Mutex::new(::launchpad::router::request::State::default())))
                )
            ],
            @rest $($rest)*
        )
    };
    (@nest [$($total: expr),*], @rest $endpoint: ident, $($rest: tt)*) => {
        $crate::routes!(
            @nest
            [
                $($total,)*
                ::launchpad::router::Route::from_endpoint(
                    std::sync::Arc::new($endpoint(std::sync::Mutex::new(::launchpad::router::request::State::default())))
                )
            ],
            @rest $($rest)*
        )
    };
    (@nest [$($total: expr),*], @rest $path: literal => $endpoint: expr, $($rest: tt)*) => {
        $crate::routes!(
            @nest
            [
                $($total,)*
                ::launchpad::router::Route::new(
                    $path.to_string(),
                    std::sync::Arc::new($endpoint(std::sync::Mutex::new(::launchpad::router::request::State::default())))
                )
            ],
            @rest $($rest)*
        )
    };
    (@nest [$($total: expr),*], @rest $endpoint: ident $(,)?) => {
        [
            $($total,)*
            ::launchpad::router::Route::from_endpoint(
                std::sync::Arc::new($endpoint(std::sync::Mutex::new(::launchpad::router::request::State::default())))
            )
        ]
    };
    (@nest [$($total: expr),*], @rest $path: literal => $endpoint: expr $(,)?) => {
        [
            $($total,)*
            ::launchpad::router::Route::new(
                $path,
                std::sync::Arc::new($endpoint(std::sync::Mutex::new(::launchpad::router::request::State::default())))
            )
        ]
    };
    (@nest [$($total: expr),*], @rest) => {
        [
            $($total,)*
        ]
    };
    () => {
        []
    };
}

#[macro_export]
macro_rules! errors {
    () => {
        []
    };
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
        $crate::errors!(
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
        $crate::errors!(
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
        $crate::errors!(
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
        $crate::errors!(
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
/// All of the endpoints and catches can be added via the individual syntax.
/// Anything prefixed with `~` is treated as a catch route while all other items are treated
/// as normal routes.
/// ```
/// use launchpad::prelude::*;
///
/// let router = rts! {
///     home,
///     "/" => home
///     ~not_found
///     ~500 => not_found,
/// }
/// ```
///
/// All endpoints and catches can also be added via the grouped syntax.
/// The idea is that all normal routes are grouped together and all catch routes are grouped
/// together. Routes are grouped with square brackets `[]` and catch handlers are grouped with
/// curly brackets `{}`.
/// ```
/// use launcpad::prelude::*;
///
/// let router = rts! {
///     [
///         home,
///         "/" => home
///     ],
///     {
///         not_found,
///         404 => not_found
///     }
/// }
/// ```
///
/// Tags are also optional. Any combination of tags can be used and the routes or catches group can
/// be omitted if desired.
/// ```
/// use launchpad::prelude::*;
///
/// let router = rts! {
///     routes [home],
///     catch { not_found }
/// }
/// ```
#[macro_export]
macro_rules! rts {
    // Grouped syntax
    ($(routes )? [$($routes: tt)*], $(catch )? {$($errors: tt)*} $(,)?) => {
        ::launchpad::router::Router::from(($crate::routes!($($routes)*), $crate::errors!($($errors)*)))
    };
    ($(catch )? {$($errors: tt)*}, $(route )? [$($routes: tt)*] $(,)?) => {
        ::launchpad::router::Router::from(($crate::routes!($($routes)*), $crate::errors!($($errors)*)))
    };
    ($(routes )? [$($routes: tt)*] $(,)?) => {
        ::launchpad::router::Router::from($crate::routes!($($routes)*))
    };
    ($(catch )? {$($errors: tt)*} $(,)?) => {
        ::launchpad::router::Router::from($crate::errors!($($errors)*))
    };

    // Individual syntax
    ($handler: ident $($rest: tt)*) => {
        rts!(@routes [$handler,], @catches [], $($rest)*)
    };
    ($path: literal => $handler: ident $($rest: tt)*) => {
        rts!(@routes [$path => $handler,], @catches [], $($rest)*)
    };
    (~ $handler: ident $($rest: tt)*) => {
        rts!(@routes [], @catches [$handler,], $($rest)*)
    };
    (~ $code: literal => $handler: ident $($rest: tt)*) => {
        rts!(@routes [], @catches [$code => $handler,], $($rest)*)
    };
    (
        @routes [$($routes: tt)*],
        @catches [$($catches: tt)*],
        $(,)?
        $handler: ident
        $($rest: tt)*
    ) => {
        rts!(@routes [$($routes)* $handler,], @catches [$($catches)*], $($rest)*)
    };
    (
        @routes [$($routes: tt)*],
        @catches [$($catches: tt)*],
        $(,)?
        $path: literal => $handler: ident
        $($rest: tt)*
    ) => {
        rts!(@routes [$($routes)* $path => $handler,], @catches [$($catches)*], $($rest)*)
    };
    (
        @routes [$($routes: tt)*],
        @catches [$($catches: tt)*],
        $(,)?
        ~ $handler: ident
        $($rest: tt)*
    ) => {
        rts!(@routes [$($routes)*], @catches [$($catches)* $handler,], $($rest)*)
    };
    (
        @routes [$($routes: tt)*],
        @catches [$($catches: tt)*],
        $(,)?
        ~ $code: literal => $handler: ident
        $($rest: tt)*
    ) => {
        rts!(@routes [$($routes)*], @catches [$($catches)* $code => $handler,], $($rest)*)
    };
    (@routes [$($routes: tt)*], @catches [$($catches: tt)*], $(,)?) => {
        ::launchpad::router::Router::from(
            (
                $crate::routes!($($routes)*),
                $crate::errors!($($catches)*)
            )
        )
    };
}
