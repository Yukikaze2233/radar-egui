use serde_json;

/// Parse incoming ZMQ bytes into a `ZMQMessage`.
pub fn zmq_json_parser(_input_data: &[u8]) -> serde_json::Result<serde_json::Value> {
    serde_json::from_slice(_input_data)
}