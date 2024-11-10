
use amiquip::{Connection, ConsumerMessage, ConsumerOptions, QueueDeclareOptions, Result};
use amiquip::{Exchange, Publish};

pub fn send(msg: String, queue_addr: &str) -> Result<()>{
// Open connection.
let mut connection = Connection::insecure_open("amqp://guest:guest@localhost:5672")?;

// Open a channel - None says let the library choose the channel ID.
let channel = connection.open_channel(None)?;

// Get a handle to the direct exchange on our channel.
let exchange = Exchange::direct(&channel);

// Publish a message to the "hello" queue.
exchange.publish(Publish::new(msg.as_bytes(), queue_addr))?;

connection.close()

}

//pub fn consume(queue_name: &str)-> Result<()> {
pub fn consume(queue_name: &str)-> String {
    let mut msg = "".to_string();
    
    // Open connection.
    let mut connection: Connection = Connection::insecure_open("amqp://guest:guest@localhost:5672").unwrap();

    // Open a channel - None says let the library choose the channel ID.
    let channel: amiquip::Channel = connection.open_channel(None).unwrap();

    // Declare the "hello" queue.
    let queue = channel.queue_declare(queue_name, QueueDeclareOptions::default()).unwrap();

    // Start a consumer.
    let consumer = queue.consume(ConsumerOptions::default()).unwrap();
   

    for (_i, message) in consumer.receiver().iter().enumerate() {
        match message {
            ConsumerMessage::Delivery(delivery) => {
                let body = String::from_utf8_lossy(&delivery.body);

                msg = body.to_string();
                consumer.ack(delivery).unwrap();
                break;
            }
            other => {
                println!("Consumer ended: {:?}", other);
                break;
            }
        }
    }

    let _ = connection.close();
       
    msg

}
