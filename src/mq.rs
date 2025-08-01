use crate::event::LaunchEvent;

pub(crate) fn send_to_mq(event: &LaunchEvent) -> Result<(), Box<dyn std::error::Error>> {
    // Intended to send the message to RabbitMQ message broker. 
    // The prototype just prints the CA to the console instead.
    println!("{}", event.ca);
    Ok(())
}