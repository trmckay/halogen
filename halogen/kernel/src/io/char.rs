pub trait ByteProducer {
    fn read_byte(&mut self) -> Option<u8>;
}

pub trait ByteConsumer {
    fn write_byte(&mut self, byte: u8);
}
