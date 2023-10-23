# TODO

- [x] Result return for error catching and error responses
- [x] Fallback for endpoints and methods
- [x] Check if `FnOnce` is more appropriate
- [x] Extractor magic for handler params. Includes auto parsing of request body
- [x] Merge duplicate route definitions
- [x] Serve specific static file directories
- [ ] Path groupings for routes so they may be defined elsewhere.
- [x] Dynamic routes and path captures
  ```
  "/blog/{...slug}/updates/"
  
  async fn handler(captures: Captures) -> impl IntoResponse {
    let slug: String = captures.get("slug");
  }
  ```
- [ ] Add service/Tower layer support per handler and per route
- [ ] Traits and API for adding markup languages like Handlebars and Tera
  ```
  Server.templates("/templates/").server(...);
  
  fn home(templates: Templates) -> impl IntoResponse {
    templates.render("home", &Context::default()) 
  }
  ```
- [ ] Sensitive data redaction for logs
- [x] Cookie utils. Allows for retrieving, setting, and deleting cookies
- [ ] Refine features and make them rich, fast, and easy to use

- HTML macro
  - [x] Basic macro with components and loops
  - [x] Props
  - [x] Capture sanatization
  - [x] IntoResponse for new Element
