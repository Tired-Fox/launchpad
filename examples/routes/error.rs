use wayfinder::{
    prelude::*,
    response::{File, Raw, HTML, JSON},
};

#[get("/error")]
pub fn server() -> Result<HTML<String>> {
    // Can return a result of Ok(T) or Err((code, message))
    // response! shortcuts and wraps what is inside making the syntax simpler
    response!(500, "Custom user error")
}

/**
 * Not Found error catch documentation
 * This will be put right below a doc about what the function handles
 */
#[catch(404)]
pub fn not_found(code: u16, message: String, reason: String) -> HTML<String> {
    // 404 Error handler. Must return valid response. Neither (code, data) or Result<data>
    // will work.
    html! {
        <h1>{code}" "{message}</h1>
        <p>{reason}</p>
    }
}
