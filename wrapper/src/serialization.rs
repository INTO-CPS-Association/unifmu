use crate::config::SerializationFormat;
use anyhow::Error;

// ----------------------------- Serialization -----------------------------------------------------

pub trait ObjectSender<T> {
    fn send_as_object(
        &self,
        value: T,
        serialization: SerializationFormat,
        timeout: Option<i32>,
    ) -> Result<(), zmq::Error>;
}

impl<T> ObjectSender<T> for zmq::Socket
where
    T: serde::ser::Serialize,
{
    fn send_as_object(
        &self,
        value: T,
        serialization: SerializationFormat,
        timeout: Option<i32>,
    ) -> Result<(), zmq::Error> {
        let data = match serialization {
            SerializationFormat::Pickle => {
                serde_pickle::to_vec(&value, true).expect("unable to pickle object")
            }
            SerializationFormat::Json => {
                serde_json::to_vec(&value).expect("unable to convert object to json")
            }
        };

        self.send(&data, 0)
    }
}

// ---------------------------- Binding ---------------------------

pub trait BindToRandom {
    /// Quality of life function inspired by
    /// https://pyzmq.readthedocs.io/en/latest/api/zmq.html?highlight=bind_random#zmq.Socket.bind_to_random_port
    fn bind_to_random_port(&self, addr: &str) -> Result<i32, Error>;
}

impl BindToRandom for zmq::Socket {
    fn bind_to_random_port(&self, addr: &str) -> Result<i32, Error> {
        let connection_str = format!("tcp://{}:*", addr);
        self.bind(&connection_str).unwrap();

        let endpoint = self.get_last_endpoint().unwrap().unwrap();
        let port: &str = endpoint.split(":").collect::<Vec<&str>>()[2];
        let port: i32 = port.parse().unwrap();
        return Ok(port);
    }
}

// --------------------- Pickling Traits --------------------------

pub trait PickleSender<T> {
    fn send_as_pickle(&self, value: T) -> Result<(), zmq::Error>;
}

pub trait PickleReceiver<T> {
    fn recv_from_pickle(&self) -> Result<T, zmq::Error>;
}

impl<T> PickleSender<T> for zmq::Socket
where
    T: serde::ser::Serialize,
{
    fn send_as_pickle(&self, value: T) -> Result<(), zmq::Error> {
        let pickle = serde_pickle::to_vec(&value, true).expect("unable to pickle object");
        self.send(&pickle, 0)
    }
}

impl<'a, T> PickleReceiver<T> for zmq::Socket
where
    T: serde::de::Deserialize<'a>,
{
    fn recv_from_pickle(&self) -> Result<T, zmq::Error> {
        let bytes = self.recv_bytes(0)?;

        let value: T = serde_pickle::from_slice(&bytes).expect("unable to un-pickle object");
        std::result::Result::Ok(value)
    }
}

// -------------------- JSON Traits ------------------------------------
pub trait JsonSender<T> {
    fn send_as_json(&self, value: T) -> Result<(), zmq::Error>;
}

pub trait JsonReceiver<T> {
    fn recv_from_json(&self) -> Result<T, zmq::Error>;
}

impl<T> JsonSender<T> for zmq::Socket
where
    T: serde::ser::Serialize,
{
    fn send_as_json(&self, value: T) -> Result<(), zmq::Error> {
        let pickle = serde_pickle::to_vec(&value, true).expect("unable to convert object to json");
        self.send(&pickle, 0)
    }
}

impl<'a, T> JsonReceiver<T> for zmq::Socket
where
    T: serde::de::DeserializeOwned,
{
    fn recv_from_json(&self) -> Result<T, zmq::Error> {
        let bytes = self
            .recv_string(0)?
            .expect("message appears does not appear to use valid UTF-8 encoding");

        let value: T = serde_json::from_str(&bytes).expect("unable to deserialize json object");
        std::result::Result::Ok(value)
    }
}
