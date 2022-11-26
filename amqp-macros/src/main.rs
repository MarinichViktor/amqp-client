use amqp_macros::amqp_method;
use amqp_protocol::response;

#[amqp_method]
struct AmqpMethod {
    #[byte]
    ver_maj: u8,
    #[byte]
    ver_min: u8
}

// impl TryInto<Vec<u8>> for AmqpMethod {
//     type Error = ();
//
//     fn try_into(self) -> response::Result<Vec<u8>, Self::Error> {
//         let data = vec![];
//         // Encode::write_int(&data, self.ver_maj);
//
//         Ok(data)
//     }
// }

fn main() {

    let d = AmqpMethod {
        ver_maj: 12,
        ver_min: 132
    };
    let v: Vec<u8> = d.try_into().unwrap();
    println!("data {:?}", v);
    // method.greet();
}
