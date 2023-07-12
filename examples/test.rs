extern crate launchpad;

use launchpad::{prelude::*, Server, State, Data, Error};

#[tokio::main]
async fn main() {
    Server::new(([127, 0, 0, 1], 3000))
        .router(routes![data])
        .serve()
        .await;
}

#[derive(Debug, Default)]
struct WorldState {
    pub count: u16,
}

#[get("/api/name/<firstname>/<lastname>/")]
fn data(
    state: &mut State<WorldState>,
    // data: Data<HomeData>,
    firstname: String,
    lastname: String,
) -> Result<String> {

    state.inner_mut().count += 1;
    // Ok(format!("Hello {} {} ({}): {:?}", firstname, lastname, state.inner().count, data.get_ref()));
    Error::code(500)
}

#[derive(Default, Debug)]
struct HomeData {
    name: String
}
