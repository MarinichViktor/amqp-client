use amqp_client_macros::amqp_method;

#[amqp_method]
struct AmqpMethod {
    #[short]
    ver_maj: i32,
    mechanism: String,
}

fn main() {
    let method = AmqpMethod {
        ver_maj: 12,
        mechanism: "qweqwe".to_string()
    };
    method.greet();
}
