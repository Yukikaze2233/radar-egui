use super::data_format::{
    ReceiveLaser, ReceiveLidarLocation, ReceiveSdr, ZmqMessageId,
    ZMQ_SUB_LASER, ZMQ_SUB_LIDAR_LOCATION, ZMQ_SUB_SDR,
};
/// Parse incoming ZMQ bytes into a known message type and update shared state.
pub fn zmq_parser(
    input: &Vec<u8>,
    receivelidarlocation: &mut ReceiveLidarLocation,
    receivesdr: &mut ReceiveSdr,
    receivelaser: &mut ReceiveLaser,
) -> Result<(), serde_json::Error> {
    let msg: ZmqMessageId = serde_json::from_slice(input)?;

    match msg.cmd_id {
        ZMQ_SUB_LIDAR_LOCATION => {
            *receivelidarlocation = serde_json::from_slice(input)?;
        }
        ZMQ_SUB_SDR => {
            *receivesdr = serde_json::from_slice(input)?;
        }
        ZMQ_SUB_LASER => {
            *receivelaser = serde_json::from_slice(input)?;
        }
        _ => {}
    }
    Ok(())
}
