use crate::io::DeviceError;

pub trait ByteProducer {
    fn read_byte(&mut self) -> Result<Option<u8>, DeviceError>;
}

pub trait ByteConsumer {
    fn write_byte(&mut self, byte: u8) -> Result<(), DeviceError>;
}
