use amqp_macros::amqp_method;

#[derive(Debug)]
#[amqp_method]
struct AmqpMethod {
    #[byte]
    ver_maj: u8,
    #[byte]
    ver_min: u8
}

// impl TryFrom<Vec<u8>> for AmqpMethod {
//     type Error = ();
//
//     fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
//         todo!()
//     }
// }

fn main() {
    let d = AmqpMethod {
        ver_maj: 12,
        ver_min: 132
    };
    let v: Vec<u8> = d.try_into().unwrap();
    println!("data {:?}", v);

    let m2: AmqpMethod = v.try_into().unwrap();
    println!("m2 {:?}", m2);
}
