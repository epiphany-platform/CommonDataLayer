use log::trace;
use rpc::command_service::{command_service_server::CommandService, Empty, InsertMessage};

use tonic::{Request, Response, Status};
use utils::message_types::BorrowedInsertMessage;

use crate::{communication::MessageRouter, input::Error, output::OutputPlugin};

pub struct GRPCInput<P: OutputPlugin> {
    message_router: MessageRouter<P>,
}

impl<P: OutputPlugin> GRPCInput<P> {
    pub fn new(message_router: MessageRouter<P>) -> Self {
        Self { message_router }
    }

    async fn handle_message(router: MessageRouter<P>, message: InsertMessage) -> Result<(), Error> {
        let generic_message = Self::build_message(&message)?;
        trace!("Received message {:?}", generic_message);

        router
            .handle_message(generic_message)
            .await
            .map_err(Error::CommunicationError)?;

        Ok(())
    }

    fn build_message(message: &'_ InsertMessage) -> Result<BorrowedInsertMessage<'_>, Error> {
        let json = &message.data;
        let event: BorrowedInsertMessage =
            serde_json::from_slice(&json).map_err(Error::PayloadDeserializationFailed)?;

        Ok(event)
    }
}

#[tonic::async_trait]
impl<P: OutputPlugin> CommandService for GRPCInput<P> {
    async fn insert(&self, request: Request<InsertMessage>) -> Result<Response<Empty>, Status> {
        let message = request.into_inner();
        let router = self.message_router.clone();

        match Self::handle_message(router, message).await {
            Ok(_) => Ok(Response::new(Empty {})),
            Err(err) => Err(Status::internal(err.to_string())),
        }
    }
}
