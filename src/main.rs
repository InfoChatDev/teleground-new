use rocket::{get, post, routes, State, Shutdown};
use rocket::fs::{relative, FileServer};
use rocket::form::Form;
use rocket::response::stream::{EventStream, Event};
use rocket::serde::{Serialize, Deserialize};
use rocket::tokio::sync::broadcast::{channel, Sender, error::RecvError};
use rocket::tokio::select;
use rocket::FromForm;
use rocket::launch;
use rocket_dyn_templates::{Template, context};

#[derive(Debug, Clone, FromForm, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
struct Message {
    pub room: String,
    pub username: String,
    pub message: String,
}

#[get("/events")]
async fn events(queue: &State<Sender<Message>>, mut end: Shutdown) -> EventStream![] {
    let mut rx = queue.subscribe();
    EventStream! {
        loop {
            let msg = select! {
                msg = rx.recv() => match msg {
                    Ok(msg) => msg,
                    Err(RecvError::Closed) => break,
                    Err(RecvError::Lagged(_)) => continue,
                },
                _ = &mut end => break,
            };
            yield Event::json(&msg);
        }
    }
}

#[post("/message", data = "<form>")]
fn post(form: Form<Message>, queue: &State<Sender<Message>>) {
    let msg = form.into_inner();
    if msg.room.len() > 30 || msg.username.len() > 20 {
        return;
    }
    let _ = queue.send(msg);
}

#[get("/chat")]
fn chat() -> Template {
    Template::render("chat", context! {})
}

#[get("/")]
fn index() ->Template{
    Template::render("index", context! {})
}

#[get("/login")]
fn user_login() -> Template {
    Template::render("login", context! {})
}

#[get("/register")]
fn user_register() -> Template {
    Template::render("r egister", context! {})
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .manage(channel::<Message>(1024).0)
        .mount("/", routes![post, events, chat, index])
        .mount("/user", routes![user_login, user_register])
        .mount("/static", FileServer::from(relative!("static")))
        .attach(Template::fairing())
}
