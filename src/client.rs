use std::io::Read;
use std::sync::Arc;
use std::{io, thread};

use futures::sync::oneshot;
use futures::Future;
use grpcio::{Environment, RpcContext, ServerBuilder, UnarySink, Server};

mod protos;
mod op;
mod param;

use crate::protos::rendergraph;
use crate::protos::rendergraph_grpc;
use crate::protos::rendergraph::{ImportRequest, ItemRequest, ImportReply, InstantiateRequest, ItemPortRequest, InstantiateReply, ResourceInfo, Ports};

#[derive(Clone)]
struct BlackboardService;

impl rendergraph_grpc::Blackboard for BlackboardService {
    fn import(&mut self, ctx: RpcContext, req: ImportRequest, sink: UnarySink<ImportReply>) {
        unimplemented!()
    }

    fn get_inputs(&mut self, ctx: RpcContext, req: ItemRequest, sink: UnarySink<Ports>) {
        unimplemented!()
    }

    fn get_outputs(&mut self, ctx: RpcContext, req: ItemRequest, sink: UnarySink<Ports>) {
        unimplemented!()
    }

    fn instantiate(&mut self, ctx: RpcContext, req: InstantiateRequest, sink: UnarySink<InstantiateReply>) {
        unimplemented!()
    }

    fn instantiate_copy(&mut self, ctx: RpcContext, req: InstantiateRequest, sink: UnarySink<InstantiateReply>) {
        unimplemented!()
    }

    fn get_resource_info(&mut self, ctx: RpcContext, req: ItemPortRequest, sink: UnarySink<ResourceInfo>) {
        unimplemented!()
    }
}

fn main() {
    let env = Arc::new(Environment::new(1));

    let service = rendergraph_grpc::create_blackboard(BlackboardService);
    let mut server = ServerBuilder::new(env).register_service(service).bind("127.0.0.1",0).build().unwrap();
    server.start();
    for &(ref host, port) in server.bind_addrs() {
        println!("listening on {}:{}", host, port);
    }

    let (tx, rx) = oneshot::channel();
    thread::spawn(move || {
        println!("Press ENTER to exit...");
        let _ = io::stdin().read(&mut [0]).unwrap();
        tx.send(())
    });
    let _ = rx.wait();
    let _ = server.shutdown().wait();
}
