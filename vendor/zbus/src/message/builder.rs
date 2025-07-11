use std::{
    borrow::Cow,
    io::{Cursor, Write},
    num::NonZeroU32,
    sync::Arc,
};
#[cfg(unix)]
use zvariant::OwnedFd;

use enumflags2::BitFlags;
use zbus_names::{BusName, ErrorName, InterfaceName, MemberName, UniqueName};
use zvariant::{serialized, Endian, Signature};

use crate::{
    message::{EndianSig, Fields, Flags, Header, Message, PrimaryHeader, Sequence, Type},
    utils::padding_for_8_bytes,
    zvariant::{serialized::Context, DynamicType, ObjectPath},
    Error, Result,
};

use crate::message::header::MAX_MESSAGE_SIZE;

#[cfg(unix)]
type BuildGenericResult = Vec<OwnedFd>;

#[cfg(not(unix))]
type BuildGenericResult = ();

macro_rules! dbus_context {
    ($self:ident, $n_bytes_before: expr) => {
        Context::new_dbus($self.header.primary().endian_sig().into(), $n_bytes_before)
    };
}

/// A builder for a [`Message`].
#[derive(Debug, Clone)]
pub struct Builder<'a> {
    header: Header<'a>,
}

impl<'a> Builder<'a> {
    pub(super) fn new(msg_type: Type) -> Self {
        let primary = PrimaryHeader::new(msg_type, 0);
        let fields = Fields::new();
        let header = Header::new(primary, fields);
        Self { header }
    }

    /// Add flags to the message.
    ///
    /// See [`Flags`] documentation for the meaning of the flags.
    ///
    /// The function will return an error if invalid flags are given for the message type.
    pub fn with_flags(mut self, flag: Flags) -> Result<Self> {
        if self.header.message_type() != Type::MethodCall
            && BitFlags::from_flag(flag).contains(Flags::NoReplyExpected)
        {
            return Err(Error::InvalidField);
        }
        let flags = self.header.primary().flags() | flag;
        self.header.primary_mut().set_flags(flags);
        Ok(self)
    }

    /// Set the unique name of the sending connection.
    pub fn sender<'s: 'a, S>(mut self, sender: S) -> Result<Self>
    where
        S: TryInto<UniqueName<'s>>,
        S::Error: Into<Error>,
    {
        self.header.fields_mut().sender = Some(sender.try_into().map_err(Into::into)?);
        Ok(self)
    }

    /// Set the object to send a call to, or the object a signal is emitted from.
    pub fn path<'p: 'a, P>(mut self, path: P) -> Result<Self>
    where
        P: TryInto<ObjectPath<'p>>,
        P::Error: Into<Error>,
    {
        self.header.fields_mut().path = Some(path.try_into().map_err(Into::into)?);
        Ok(self)
    }

    /// Set the interface to invoke a method call on, or that a signal is emitted from.
    pub fn interface<'i: 'a, I>(mut self, interface: I) -> Result<Self>
    where
        I: TryInto<InterfaceName<'i>>,
        I::Error: Into<Error>,
    {
        self.header.fields_mut().interface = Some(interface.try_into().map_err(Into::into)?);
        Ok(self)
    }

    /// Set the member, either the method name or signal name.
    pub fn member<'m: 'a, M>(mut self, member: M) -> Result<Self>
    where
        M: TryInto<MemberName<'m>>,
        M::Error: Into<Error>,
    {
        self.header.fields_mut().member = Some(member.try_into().map_err(Into::into)?);
        Ok(self)
    }

    pub(super) fn error_name<'e: 'a, E>(mut self, error: E) -> Result<Self>
    where
        E: TryInto<ErrorName<'e>>,
        E::Error: Into<Error>,
    {
        self.header.fields_mut().error_name = Some(error.try_into().map_err(Into::into)?);
        Ok(self)
    }

    /// Set the name of the connection this message is intended for.
    pub fn destination<'d: 'a, D>(mut self, destination: D) -> Result<Self>
    where
        D: TryInto<BusName<'d>>,
        D::Error: Into<Error>,
    {
        self.header.fields_mut().destination = Some(destination.try_into().map_err(Into::into)?);
        Ok(self)
    }

    /// Override the generated or inherited serial.  This is a low level modification,
    /// generally you should not need to use this.
    pub fn serial(mut self, serial: NonZeroU32) -> Self {
        self.header.primary_mut().set_serial_num(serial);
        self
    }

    /// Override the reply serial. This is a low level modification, generally you should use
    /// `Message::method_return` instead.
    pub fn reply_serial(mut self, serial: Option<NonZeroU32>) -> Self {
        self.header.fields_mut().reply_serial = serial;
        self
    }

    pub(super) fn reply_to(mut self, reply_to: &Header<'_>) -> Result<Self> {
        let serial = reply_to.primary().serial_num();
        self.header.fields_mut().reply_serial = Some(serial);
        self = self.endian(reply_to.primary().endian_sig().into());

        if let Some(sender) = reply_to.sender() {
            self.destination(sender.to_owned())
        } else {
            Ok(self)
        }
    }

    /// Set the endianness of the message.
    ///
    /// The default endianness is native.
    pub fn endian(mut self, endian: Endian) -> Self {
        let sig = EndianSig::from(endian);
        self.header.primary_mut().set_endian_sig(sig);

        self
    }

    /// Build the [`Message`] with the given body.
    ///
    /// You may pass `()` as the body if the message has no body.
    ///
    /// The caller is currently required to ensure that the resulting message contains the headers
    /// as compliant with the [specification]. Additional checks may be added to this builder over
    /// time as needed.
    ///
    /// [specification]:
    /// https://dbus.freedesktop.org/doc/dbus-specification.html#message-protocol-header-fields
    pub fn build<B>(self, body: &B) -> Result<Message>
    where
        B: serde::ser::Serialize + DynamicType,
    {
        let ctxt = dbus_context!(self, 0);

        // Note: this iterates the body twice, but we prefer efficient handling of large messages
        // to efficient handling of ones that are complex to serialize.
        let body_size = zvariant::serialized_size(ctxt, body)?;

        let signature = body.signature();

        self.build_generic(signature, body_size, move |cursor| {
            // SAFETY: build_generic puts FDs and the body in the same Message.
            unsafe { zvariant::to_writer(cursor, ctxt, body) }
                .map(|s| {
                    #[cfg(unix)]
                    {
                        s.into_fds()
                    }
                    #[cfg(not(unix))]
                    {
                        let _ = s;
                    }
                })
                .map_err(Into::into)
        })
    }

    /// Create a new message from a raw slice of bytes to populate the body with, rather than by
    /// serializing a value. The message body will be the exact bytes.
    ///
    /// # Safety
    ///
    /// This method is unsafe because it can be used to build an invalid message.
    pub unsafe fn build_raw_body<S>(
        self,
        body_bytes: &[u8],
        signature: S,
        #[cfg(unix)] fds: Vec<OwnedFd>,
    ) -> Result<Message>
    where
        S: TryInto<Signature>,
        S::Error: Into<Error>,
    {
        let signature = signature.try_into().map_err(Into::into)?;
        let body_size = serialized::Size::new(body_bytes.len(), dbus_context!(self, 0));
        #[cfg(unix)]
        let body_size = {
            let num_fds = fds.len().try_into().map_err(|_| Error::ExcessData)?;
            body_size.set_num_fds(num_fds)
        };

        self.build_generic(
            signature,
            body_size,
            move |cursor: &mut Cursor<&mut Vec<u8>>| {
                cursor.write_all(body_bytes)?;

                #[cfg(unix)]
                return Ok::<Vec<OwnedFd>, Error>(fds);

                #[cfg(not(unix))]
                return Ok::<(), Error>(());
            },
        )
    }

    fn build_generic<WriteFunc>(
        self,
        signature: Signature,
        body_size: serialized::Size,
        write_body: WriteFunc,
    ) -> Result<Message>
    where
        WriteFunc: FnOnce(&mut Cursor<&mut Vec<u8>>) -> Result<BuildGenericResult>,
    {
        let ctxt = dbus_context!(self, 0);
        let mut header = self.header;

        header.fields_mut().signature = Cow::Owned(signature);

        let body_len_u32 = body_size.size().try_into().map_err(|_| Error::ExcessData)?;
        header.primary_mut().set_body_len(body_len_u32);

        #[cfg(unix)]
        {
            let fds_len = body_size.num_fds();
            if fds_len != 0 {
                header.fields_mut().unix_fds = Some(fds_len);
            }
        }

        let hdr_len = *zvariant::serialized_size(ctxt, &header)?;
        // We need to align the body to 8-byte boundary.
        let body_padding = padding_for_8_bytes(hdr_len);
        let body_offset = hdr_len + body_padding;
        let total_len = body_offset + body_size.size();
        if total_len > MAX_MESSAGE_SIZE {
            return Err(Error::ExcessData);
        }
        let mut bytes: Vec<u8> = Vec::with_capacity(total_len);
        let mut cursor = Cursor::new(&mut bytes);

        // SAFETY: There are no FDs involved.
        unsafe { zvariant::to_writer(&mut cursor, ctxt, &header) }?;
        cursor.write_all(&[0u8; 8][..body_padding])?;
        #[cfg(unix)]
        let fds: Vec<_> = write_body(&mut cursor)?.into_iter().collect();
        #[cfg(not(unix))]
        write_body(&mut cursor)?;

        let primary_header = header.into_primary();
        #[cfg(unix)]
        let bytes = serialized::Data::new_fds(bytes, ctxt, fds);
        #[cfg(not(unix))]
        let bytes = serialized::Data::new(bytes, ctxt);

        Ok(Message {
            inner: Arc::new(super::Inner {
                primary_header,
                quick_fields: std::sync::OnceLock::new(),
                bytes,
                body_offset,
                recv_seq: Sequence::default(),
            }),
        })
    }
}

impl<'m> From<Header<'m>> for Builder<'m> {
    fn from(mut header: Header<'m>) -> Self {
        // Signature and Fds are added by body* methods.
        let fields = header.fields_mut();
        fields.signature = Cow::Owned(Signature::Unit);
        fields.unix_fds = None;

        Self { header }
    }
}

#[cfg(test)]
mod tests {
    use super::Message;
    use crate::Error;
    use test_log::test;

    #[test]
    fn test_raw() -> Result<(), Error> {
        let raw_body: &[u8] = &[16, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0, 0];
        let message_builder = Message::signal("/", "test.test", "test")?;
        let message = unsafe {
            message_builder.build_raw_body(
                raw_body,
                "ai",
                #[cfg(unix)]
                vec![],
            )?
        };

        let output: Vec<i32> = message.body().deserialize()?;
        assert_eq!(output, vec![1, 2, 3, 4]);

        Ok(())
    }
}
