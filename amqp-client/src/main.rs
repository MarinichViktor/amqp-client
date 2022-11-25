use amqp;
use log::{info};
use amqp::response;
use amqp_client_macros::amqp_method;
// fn main() -> response::Result<()> {
//     env_logger::init();
//
//     let mut connection = Connection::new("localhost".to_string(), 5672, "user".to_string(), "password".to_string());
//     info!("Connection connect ...");
//     match connection.connect() {
//         Err(e) => {
//             for x in e.chain() {
//                 info!("{}", x)
//             }
//         },
//         _ => {}
//     }
//     info!("Connection connect finished");
//     Ok(())
// }

#[amqp_method]
struct AmqpMethod {
    ver_maj: i32,
    mechanism: String,
}


fn main() -> response::Result<()> {
    env_logger::init();

    info!("testing macro");
    let method = AmqpMethod {
        ver_maj: 12,
        mechanism: "qweqwe".to_string()
    };
    method.greet();

    Ok(())
}
