extern crate byteorder;

use byteorder::{LE, WriteBytesExt, ReadBytesExt};
use std::io::{Read, Write};
use std::net::TcpStream;

fn write_str(io: &mut Write, s: &str) -> std::io::Result<()> {
    io.write_u64::<LE>(s.len() as u64)?;
    write!(io, "{}", s)?;
    Ok(())
}

#[derive(Debug)]
struct OutgoingIPCMessage {
    pub ty: u64,
    pub data: Vec<u64>,
    pub pid: i64,
    pub copiedHandles: Vec<u64>,
    pub movedHandles: Vec<u64>,
    pub aBufs: Vec<Vec<u8>>,
    pub bBufs: Vec<Vec<u8>>,
    pub cBufs: Vec<Vec<u8>>,
    pub xBufs: Vec<Vec<u8>>,
}

impl OutgoingIPCMessage {
    fn new(ty: u8, cmd: u64) -> OutgoingIPCMessage {
        OutgoingIPCMessage {
            ty: ty as u64,
            data: vec![cmd],
            pid: -1,
            copiedHandles: Vec::new(),
            movedHandles: Vec::new(),
            aBufs: vec![],
            bBufs: vec![],
            cBufs: vec![],
            xBufs: vec![]
        }
    }

    fn write(&self, io: &mut Write) -> std::io::Result<()> {
        io.write_u64::<LE>(self.ty)?;

        io.write_u64::<LE>(self.data.len() as u64)?;
        for elem in &self.data {
            io.write_u64::<LE>(*elem)?;
        }

        io.write_i64::<LE>(self.pid)?;

        io.write_u64::<LE>(self.copiedHandles.len() as u64)?;
        for hnd in self.copiedHandles.iter() {
            io.write_u64::<LE>(*hnd)?;
        }

        io.write_u64::<LE>(self.movedHandles.len() as u64)?;
        for hnd in self.movedHandles.iter() {
            io.write_u64::<LE>(*hnd)?;
        }

        // At another time.
        io.write_u64::<LE>(0)?;
        io.write_u64::<LE>(0)?;
        io.write_u64::<LE>(0)?;
        io.write_u64::<LE>(0)?;
        Ok(())
    }
}

#[derive(Debug)]
struct IncomingBridgeMessage {
    pub res: u64,
    pub ty: u64,
    pub data: Vec<u64>,
    pub copiedHandles: Vec<u64>,
    pub movedHandles: Vec<u64>,
    pub aBufs: Vec<(Vec<u8>, u64)>,
    pub bBufs: Vec<(Vec<u8>, u64)>,
    pub cBufs: Vec<(Vec<u8>, u64)>,
    pub xBufs: Vec<(Vec<u8>, u64)>,
}

impl IncomingBridgeMessage {
    fn read(io: &mut Read) -> std::io::Result<IncomingBridgeMessage> {
        let res = io.read_u64::<LE>()?;
        if res == 0 {
            let size = io.read_u64::<LE>()? as usize;
            let mut data = vec![0; size];
            for i in 0..size {
                data[i] = io.read_u64::<LE>()?;
            }

            let size = io.read_u64::<LE>()? as usize;
            let mut copiedHandles = vec![0; size];
            for i in 0..size {
                copiedHandles[i] = io.read_u64::<LE>()?;
            }

            let size = io.read_u64::<LE>()? as usize;
            let mut movedHandles = vec![0; size];
            for i in 0..size {
                movedHandles[i] = io.read_u64::<LE>()?;
            }

            let size = io.read_u64::<LE>()? as usize;
            let mut aBufs = vec![(vec![], 0); size];
            for i in 0..size {
                let data_size = io.read_u64::<LE>()? as usize;
                let mut buf = vec![0; data_size];
                io.read_exact(&mut buf)?;
                let flags = io.read_u64::<LE>()?;
                aBufs[i] = (buf, flags);
            }

            let size = io.read_u64::<LE>()? as usize;
            let mut bBufs = vec![(vec![], 0); size];
            for i in 0..size {
                let data_size = io.read_u64::<LE>()? as usize;
                let mut buf = vec![0; data_size];
                io.read_exact(&mut buf)?;
                let flags = io.read_u64::<LE>()?;
                bBufs[i] = (buf, flags);
            }

            let size = io.read_u64::<LE>()? as usize;
            let mut cBufs = vec![(vec![], 0); size];
            for i in 0..size {
                let data_size = io.read_u64::<LE>()? as usize;
                let mut buf = vec![0; data_size];
                io.read_exact(&mut buf)?;
                let flags = io.read_u64::<LE>()?;
                cBufs[i] = (buf, flags);
            }

            let size = io.read_u64::<LE>()? as usize;
            let mut xBufs = vec![(vec![], 0); size];
            for i in 0..size {
                let data_size = io.read_u64::<LE>()? as usize;
                let mut buf = vec![0; data_size];
                io.read_exact(&mut buf)?;
                let flags = io.read_u64::<LE>()?;
                xBufs[i] = (buf, flags);
            }

            let ty = io.read_u64::<LE>()?;

            Ok(IncomingBridgeMessage {
                res: 0,
                ty: ty,
                data: data,
                copiedHandles: copiedHandles,
                movedHandles: movedHandles,
                aBufs: aBufs,
                bBufs: bBufs,
                cBufs: cBufs,
                xBufs: xBufs
            })
        } else {
            Ok(IncomingBridgeMessage {
                res: res,
                ty: 0,
                data: vec![],
                copiedHandles: vec![],
                movedHandles: vec![],
                aBufs: vec![],
                bBufs: vec![],
                cBufs: vec![],
                xBufs: vec![]
            })
        }
    }
}

fn main() {
    let mut stream = TcpStream::connect("127.0.0.1:31337").expect("Run an IPC bridge first !");

    println!("Connected !");
    stream.write_u64::<LE>(0).unwrap();
    write_str(&mut stream, "ldr:ro").unwrap();
    let hnd = stream.read_u64::<LE>().unwrap();

    println!("Opened handle to ldr:ro: {}", hnd);
    println!("Sending initialize");
    let mut msg = OutgoingIPCMessage::new(4, 4);
    msg.data.push(0);
    msg.copiedHandles.push(0xFFFF8001);
    msg.pid = 0xA;

    stream.write_u64::<LE>(2).unwrap();
    msg.write(&mut stream).unwrap();
    stream.write_u64::<LE>(hnd).unwrap();

    let msg = IncomingBridgeMessage::read(&mut stream).unwrap();
    println!("{:?}", msg);

    if msg.res != 0 {
        println!("IPC ERROR ! res = {}", msg.res);
        return;
    }
    if msg.data.len() == 0 {
        println!("IPC ERROR ! No result");
        return;
    }
    if msg.data[0] != 0 {
        println!("MSG ERROR ! res = {}", msg.data[0]);
        return;
    }

    println!("We're inited. Asking to allocate some memory");
    stream.write_u64::<LE>(3).unwrap();
    let data = vec![0u8; 0x1000];
    stream.write_u64::<LE>(data.len() as u64 / 8).unwrap();
    stream.write_all(&data).unwrap();

    let addr = stream.read_u64::<LE>().unwrap();

    println!("Got an address {}", addr);

    println!("Sending LoadNrr");
    let mut msg = OutgoingIPCMessage::new(4, 2);
    msg.pid = 0xA;
    msg.data.push(0);
    msg.data.push(addr);
    msg.data.push(0x1000);

    stream.write_u64::<LE>(2).unwrap();
    msg.write(&mut stream).unwrap();
    stream.write_u64::<LE>(hnd).unwrap();

    let msg = IncomingBridgeMessage::read(&mut stream).unwrap();
    println!("{:?}", msg);

    if msg.res != 0 {
        println!("IPC ERROR ! res = {}", msg.res);
        return;
    }
    if msg.data.len() == 0 {
        println!("IPC ERROR ! No result");
        return;
    }
    if msg.data[0] != 0 {
        println!("MSG ERROR ! res = {}", msg.data[0]);
        return;
    }
}
