use amqp_macros::amqp_method;
use amqp_protocol::types::AmqMethod;
#[derive(Debug)]
#[amqp_method(c_id=23, m_id=14)]
struct AmqpMethod {
    #[byte]
    ver_maj: u8,
    #[byte]
    ver_min: u8
}

fn main() {
    let d = AmqpMethod {
        ver_maj: 12,
        ver_min: 15
    };
    let v: Vec<u8> = d.try_into().unwrap();
    println!("data {:?}", v);

    let m2: AmqpMethod = v.try_into().unwrap();
    println!("m2 {:?}", m2);

    println!("Class id {}, Method id {}", m2.class_id(), m2.method_id());
}
