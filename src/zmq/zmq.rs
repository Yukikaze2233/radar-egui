use zmq2;
pub fn zmq_init(
    thread_num: i32,
    pub_str: &str,
    sub_str: &[String],
) -> zmq2::Result<(zmq2::Socket, zmq2::Socket, &'static str)> {
    let context = zmq2::Context::new();
    context.set_io_threads(thread_num)?;
    let pub_socket = context.socket(zmq2::PUB)?;
    let sub_socket = context.socket(zmq2::SUB)?;
    pub_socket.bind(pub_str)?;
    for index in sub_str.iter() {
        sub_socket.connect(index)?;
    }
    Ok((
        pub_socket,
        sub_socket,
        "Has been initialized pub and sub socket successfully",
    ))
}
pub fn zmq_send(pub_socket: &zmq2::Socket, msg: &str) -> zmq2::Result<()> {
    pub_socket.send(msg, 0)?;
    Ok(())
}
pub fn zmq_recv(sub_socket: &zmq2::Socket) -> zmq2::Result<Vec<u8>> {
    sub_socket.recv_bytes(0)
}
