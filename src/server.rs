use crate::network::Network;
use log::info;
use serde::{Deserialize, Serialize, Serializer};
use std::collections::HashMap;
use std::{error, fmt};

/// Default server endpoint.
pub const DEFAULT_ENDPOINT: &str = "tcp://127.0.0.1:5555";

/// Protocol version.
pub const PROTOCOL_VERSION: u32 = 1;

/// Description of a node parameter.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeParamInfo {
    /// Type of the parameter.
    pub ty: String,
    /// Whether to show an input connector for this parameter.
    pub show_connector: bool,
}

/// Description of the output of a node.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeOutputInfo {
    /// Type of the output.
    pub ty: String,
}

/// Information needed to create a node
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeCreateInfo {
    /// Name of the node.
    pub name: String,
    /// Hash of parameter descriptions, indexed by name.
    pub params: HashMap<String, NodeParamInfo>,
    /// Hash of outputs, indexed by name. An output connector is always associated to each output.
    pub outputs: HashMap<String, NodeOutputInfo>,
}

/// Represents a connection between two connectors of a node
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Connection {
    /// Path to the source connector (source node path and output connector name).
    pub source_path: String,
    /// Path to the destination connector (destination node path and parameter name).
    pub destination_path: String,
}

/// Represents a request sent to the server.
///
/// This type is meant to be deserialized from a JSON object of the form
/// ```
/// {
///     "method": "<method name>",
///     "data": <object>
/// }
/// ```
///
/// When adding a method that has no parameters, use an empty struct variant (with `{}`)
/// instead of a unit variant. This is so that deserialization works correctly.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
#[serde(tag = "method", content = "data")]
pub enum Request {
    /// Return the protocol version.
    GetVersion {},
    /// Returns information about a node.
    GetNodeInfo {},
    Kill {},
}

/// Creates a successful JSON reply payload containing the specified data.
///
/// The generated JSON is of the form
/// ```
/// {
///     "status": 0,
///     "data": <data>
/// }
/// ```
///
pub fn make_reply<T: Serialize>(data: &T) -> Vec<u8> {
    use serde::ser::SerializeStruct;
    let mut out: Vec<u8> = Vec::new();
    let mut s = serde_json::Serializer::new(&mut out);
    let mut state = s.serialize_struct("Reply", 2).unwrap();
    state.serialize_field("status", &0).unwrap();
    state.serialize_field("data", &data).unwrap();
    state.end();
    out
}

/// Creates a JSON error reply with the given error message.
///
/// The generated JSON is of the form
/// ```
/// {
///     "status": 1,
///     "errorMessage", <error message string>,
/// }
/// ```
pub fn make_error_reply(message: &str) -> Vec<u8> {
    use serde::ser::SerializeStruct;
    let mut out: Vec<u8> = Vec::new();
    let mut s = serde_json::Serializer::new(&mut out);
    let mut state = s.serialize_struct("Reply", 2).unwrap();
    state.serialize_field("status", &1).unwrap();
    state.serialize_field("errorMessage", message).unwrap();
    state.end();
    out
}

/// The next action that should be done after receiving and replying to a request.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum NextAction {
    /// Continue processing messages (call run_server again).
    Continue,
    /// The server should exit (break the message loop).
    Break,
}

#[derive(Debug)]
pub enum ServerError {
    /// Error sending or receiving a message on the socket.
    SocketError(zmq::Error),
    /// Invalid JSON in request.
    JsonError(serde_json::Error),
    /// Received message was not UTF-8
    MessageNotUTF8,
}

impl fmt::Display for ServerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> std::result::Result<(), fmt::Error> {
        match self {
            ServerError::SocketError(err) => write!(f, "socket error: {}", err),
            ServerError::JsonError(err) => write!(f, "invalid JSON in request: {}", err),
            ServerError::MessageNotUTF8 => write!(f, "the received message was not UTF-8 text"),
        }
    }
}

impl From<zmq::Error> for ServerError {
    fn from(err: zmq::Error) -> Self {
        ServerError::SocketError(err)
    }
}

impl From<serde_json::Error> for ServerError {
    fn from(err: serde_json::Error) -> Self {
        ServerError::JsonError(err)
    }
}

impl error::Error for ServerError {}

/// The server receives and dispatches requests coming on a socket.
pub struct Server {
    ctx: zmq::Context,
    socket: zmq::Socket,
}

impl Server {
    /// Creates a new server instance, listening to requests on the specified endpoint.
    pub fn new(endpoint: &str) -> Result<Server, ServerError> {
        // Create a ZMQ socket in REP mode (we reply to requests from a client).
        let ctx = zmq::Context::new();
        let socket = ctx.socket(zmq::SocketType::REP)?;
        socket.bind(endpoint)?;
        info!("Listening on {}", endpoint);
        Ok(Server { ctx, socket })
    }

    /// Receives and dispatches incoming requests.
    ///
    /// The handler should return Ok(T) with T being the reply body, or Err(String) with a message
    /// if the request could not be processed successfully.
    pub fn run(
        &self,
        f: impl FnOnce(&str) -> Result<Vec<u8>, ServerError>,
    ) -> Result<NextAction, ServerError> {
        // receive the message
        let msg = self.socket.recv_msg(0)?;
        // check that it's a valid UTF-8 string
        let msg = msg.as_str().ok_or(ServerError::MessageNotUTF8)?;

        dbg!(msg);
        let rep = f(msg)?;

        // send reply
        self.socket.send(rep, 0);
        Ok(NextAction::Continue)
    }
}
