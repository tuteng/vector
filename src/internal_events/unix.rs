use std::{io::Error, path::Path};

use metrics::counter;
use vector_common::internal_event::{error_stage, error_type};
use vector_core::internal_event::InternalEvent;

use crate::{emit, internal_events::SocketOutgoingConnectionError};

#[derive(Debug)]
pub struct UnixSocketConnectionEstablished<'a> {
    pub path: &'a std::path::Path,
}

impl InternalEvent for UnixSocketConnectionEstablished<'_> {
    fn emit(self) {
        debug!(message = "Connected.", path = ?self.path);
        counter!("connection_established_total", 1, "mode" => "unix");
    }
}

#[derive(Debug)]
pub struct UnixSocketOutgoingConnectionError<E> {
    pub error: E,
}

impl<E: std::error::Error> InternalEvent for UnixSocketOutgoingConnectionError<E> {
    fn emit(self) {
        // ## skip check-duplicate-events ##
        // ## skip check-validity-events ##
        emit!(SocketOutgoingConnectionError { error: self.error });
        // deprecated
        counter!("connection_failed_total", 1, "mode" => "unix");
    }
}

#[derive(Debug)]
pub struct UnixSocketError<'a, E> {
    pub(crate) error: &'a E,
    pub path: &'a std::path::Path,
}

impl<E: std::fmt::Display> InternalEvent for UnixSocketError<'_, E> {
    fn emit(self) {
        error!(
            message = "Unix socket error.",
            error = %self.error,
            path = ?self.path,
            error_type = error_type::CONNECTION_FAILED,
            stage = error_stage::PROCESSING,
        );
        counter!(
            "component_errors_total", 1,
            "error_type" => error_type::CONNECTION_FAILED,
            "stage" => error_stage::PROCESSING,
        );
        // deprecated
        counter!("connection_errors_total", 1, "mode" => "unix");
    }
}

#[derive(Debug)]
pub struct UnixSocketFileDeleteError<'a> {
    pub path: &'a Path,
    pub error: Error,
}

impl<'a> InternalEvent for UnixSocketFileDeleteError<'a> {
    fn emit(self) {
        error!(
            message = "Failed in deleting unix socket file.",
            path = %self.path.display(),
            error = %self.error,
            error_code = "delete_socket_file",
            error_type = error_type::WRITER_FAILED,
            stage = error_stage::PROCESSING,
        );
        counter!(
            "component_errors_total", 1,
            "error_code" => "delete_socket_file",
            "error_type" => error_type::WRITER_FAILED,
            "stage" => error_stage::PROCESSING,
        );
    }
}
