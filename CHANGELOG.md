### 0.1.0
  + Add server made with `hyper` version `1.0`
  + Add custom router
    * Matches request method and uri pattern
    * Pulls props/arguments from uri pattern
  + Add request macros
    * `get`, `post`, `put`, `delete`, etc...
    * request macro which can be multiple methods
  + Add `router` macro to help build a router from endpoint method handlers
  + Add state management, caching, for individual endpoints.
