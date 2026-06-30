use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

use crate::serial::data_format::{MINIMAP_RECEIVE_RADAR_CMD_ID, SDR_ENEMY_ROBOT_POSITION_CMD_ID};

use super::data_format::{ReceiveLidarLocation, ReceiveSdr, ZMQMessage};
/// Parse incoming ZMQ bytes into a known message type and update shared state.
pub fn zmq_parser(
    input: &Vec<u8>,
    receivelidarlocation: &mut ReceiveLidarLocation,
    receivesdr: &mut ReceiveSdr,
) -> Result<(), serde_json::Error> {
    let msg: ZMQMessage = serde_json::from_slice(input)?;

    match msg.cmd_id {
        MINIMAP_RECEIVE_RADAR_CMD_ID => {
            *receivelidarlocation = serde_json::from_slice(input)?;
        }
        SDR_ENEMY_ROBOT_POSITION_CMD_ID => {
            *receivesdr = serde_json::from_slice(input)?;
        }
        _ => {}
    }
    Ok(())
}
