# TODO

- [x] Result return for error catching and error responses
- [x] Fallback for endpoints and methods
- [x] Check if `FnOnce` is more appropriate
- [x] Extractor magic for handler params. Includes auto parsing of request body
- [x] Merge duplicate route definitions
- [x] Serve specific static file directories
- [x] Cookie utils. Allows for retrieving, setting, and deleting cookies
- [x] Dynamic routes and path captures
  ```
  "/blog/{...slug}/updates/"
  
  async fn handler(captures: Captures) -> impl IntoResponse {
    let slug: String = captures.get("slug");
  }
  ```
- [ ] State that is set up by the user and passed down to handlers
  - This allows for things like template engines to be set up and then
  implemented in the users state
- [ ] Path groupings for routes so they may be defined elsewhere.
- [ ] Add service/Tower layer support per handler and per route
- [ ] Sensitive data redaction for logs
- [ ] Refine features and make them rich, fast, and easy to use

- HTML macro
  - [x] Basic macro with components and loops
  - [x] Props
  - [x] Capture sanatization
  - [x] IntoResponse for new Element
