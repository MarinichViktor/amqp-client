use amqp_macros::amqp_method;

#[amqp_method]
struct AmqpMethod {
    #[byte]
    ver_maj: u8,
    #[byte]
    ver_min: u8
}

fn main() {

    let d = AmqpMethod {
        ver_maj: 12,
        ver_min: 132
    };
    let v: Vec<u8> = d.try_into().unwrap();
    println!("data {:?}", v);
}
