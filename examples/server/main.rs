use actix::{Actor, StreamHandler};
use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;
use sapio::frontend::session;
mod contracts;

/// Define HTTP actor
struct MyWs {
    sesh: session::Session,
}

impl Actor for MyWs {
    type Context = ws::WebsocketContext<Self>;
    fn started(&mut self, ctx: &mut Self::Context) {
        let r = self.sesh.open();
        ctx.text(r);
    }
}

/// Handler for ws::Message message
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for MyWs {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        let m = match msg {
            Ok(ws::Message::Text(text)) => Ok(session::Msg::Text(text)),
            Ok(ws::Message::Binary(bin)) => Ok(session::Msg::Bytes(bin)),
            _ => Err(()),
        };
        if let Ok(m) = m {
            if let Ok(Some(Ok(s))) = self
                .sesh
                .handle(m)
                .map(|v| v.map(|v2| serde_json::to_string(&v2)))
            {
                ctx.text(s);
                return;
            }
        }

        ctx.close(None);
    }
}

async fn index(
    m: &'static session::Menu,
    req: HttpRequest,
    stream: web::Payload,
) -> Result<HttpResponse, Error> {
    let resp = ws::start(
        MyWs {
            sesh: session::Session::new(m),
        },
        &req,
        stream,
    );
    println!("{:?}", resp);
    resp
}

lazy_static::lazy_static! {
    static ref menu : session::Menu = {
        let mut m = session::MenuBuilder::new();
        m.register_as::<contracts::ExampleA>("ExampleA".to_string().into());
        m.register_as::<contracts::ExampleB<contracts::Start>>("ExampleB".to_string().into());
        m.into()
    };
}
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().route("/", web::get().to(|r, s| index(&menu, r, s))))
        .bind("127.0.0.1:8888")?
        .run()
        .await
}
