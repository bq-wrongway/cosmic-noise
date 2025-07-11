use std::{
    num::NonZeroU32,
    sync::atomic::{AtomicU32, Ordering::Relaxed},
};

use enumflags2::{bitflags, BitFlags};
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

use zbus_names::{BusName, ErrorName, InterfaceName, MemberName, UniqueName};
use zvariant::{
    serialized::{self, Context},
    Endian, ObjectPath, Signature, Type as VariantType,
};

use crate::{message::Fields, Error};

pub(crate) const PRIMARY_HEADER_SIZE: usize = 12;
pub(crate) const MIN_MESSAGE_SIZE: usize = PRIMARY_HEADER_SIZE + 4;
pub(crate) const MAX_MESSAGE_SIZE: usize = 128 * 1024 * 1024; // 128 MiB

/// D-Bus code for endianness.
#[repr(u8)]
#[derive(Debug, Copy, Clone, Deserialize_repr, PartialEq, Eq, Serialize_repr, VariantType)]
pub enum EndianSig {
    /// The D-Bus message is in big-endian (network) byte order.
    Big = b'B',

    /// The D-Bus message is in little-endian byte order.
    Little = b'l',
}

// Such a shame I've to do this manually
impl TryFrom<u8> for EndianSig {
    type Error = Error;

    fn try_from(val: u8) -> Result<EndianSig, Error> {
        match val {
            b'B' => Ok(EndianSig::Big),
            b'l' => Ok(EndianSig::Little),
            _ => Err(Error::IncorrectEndian),
        }
    }
}

#[cfg(target_endian = "big")]
/// Signature of the target's native endian.
pub const NATIVE_ENDIAN_SIG: EndianSig = EndianSig::Big;
#[cfg(target_endian = "little")]
/// Signature of the target's native endian.
pub const NATIVE_ENDIAN_SIG: EndianSig = EndianSig::Little;

impl From<Endian> for EndianSig {
    fn from(endian: Endian) -> Self {
        match endian {
            Endian::Little => EndianSig::Little,
            Endian::Big => EndianSig::Big,
        }
    }
}

impl From<EndianSig> for Endian {
    fn from(endian_sig: EndianSig) -> Self {
        match endian_sig {
            EndianSig::Little => Endian::Little,
            EndianSig::Big => Endian::Big,
        }
    }
}

/// Message header representing the D-Bus type of the message.
#[repr(u8)]
#[derive(
    Debug, Copy, Clone, Deserialize_repr, PartialEq, Eq, Hash, Serialize_repr, VariantType,
)]
pub enum Type {
    /// Method call. This message type may prompt a reply (and typically does).
    MethodCall = 1,
    /// A reply to a method call.
    MethodReturn = 2,
    /// An error in response to a method call.
    Error = 3,
    /// Signal emission.
    Signal = 4,
}

/// Pre-defined flags that can be passed in message headers.
#[bitflags]
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, VariantType)]
pub enum Flags {
    /// This message does not expect method return replies or error replies, even if it is of a
    /// type that can have a reply; the reply should be omitted.
    ///
    /// Note that `Type::MethodCall` is the only message type currently defined in the
    /// specification that can expect a reply, so the presence or absence of this flag in the other
    /// three message types that are currently documented is meaningless: replies to those message
    /// types should not be sent, whether this flag is present or not.
    NoReplyExpected = 0x1,
    /// The bus must not launch an owner for the destination name in response to this message.
    NoAutoStart = 0x2,
    /// This flag may be set on a method call message to inform the receiving side that the caller
    /// is prepared to wait for interactive authorization, which might take a considerable time to
    /// complete. For instance, if this flag is set, it would be appropriate to query the user for
    /// passwords or confirmation via Polkit or a similar framework.
    AllowInteractiveAuth = 0x4,
}

/// The primary message header, which is present in all D-Bus messages.
///
/// This header contains all the essential information about a message, regardless of its type.
#[derive(Clone, Debug, Serialize, Deserialize, VariantType)]
pub struct PrimaryHeader {
    endian_sig: EndianSig,
    msg_type: Type,
    flags: BitFlags<Flags>,
    protocol_version: u8,
    body_len: u32,
    serial_num: NonZeroU32,
}

impl PrimaryHeader {
    /// Create a new `PrimaryHeader` instance.
    pub fn new(msg_type: Type, body_len: u32) -> Self {
        let mut serial_num = SERIAL_NUM.fetch_add(1, Relaxed);
        if serial_num == 0 {
            serial_num = SERIAL_NUM.fetch_add(1, Relaxed);
        }

        Self {
            endian_sig: NATIVE_ENDIAN_SIG,
            msg_type,
            flags: BitFlags::empty(),
            protocol_version: 1,
            body_len,
            serial_num: serial_num.try_into().unwrap(),
        }
    }

    pub(crate) fn read(buf: &[u8]) -> Result<(PrimaryHeader, u32), Error> {
        let endian = Endian::from(EndianSig::try_from(buf[0])?);
        let ctx = Context::new_dbus(endian, 0);
        let data = serialized::Data::new(buf, ctx);

        Self::read_from_data(&data)
    }

    pub(crate) fn read_from_data(
        data: &serialized::Data<'_, '_>,
    ) -> Result<(PrimaryHeader, u32), Error> {
        let (primary_header, size) = data.deserialize()?;
        assert_eq!(size, PRIMARY_HEADER_SIZE);
        let (fields_len, _) = data.slice(PRIMARY_HEADER_SIZE..).deserialize()?;
        Ok((primary_header, fields_len))
    }

    /// D-Bus code for endian encoding of the message.
    pub fn endian_sig(&self) -> EndianSig {
        self.endian_sig
    }

    /// Set the D-Bus code for endian encoding of the message.
    pub fn set_endian_sig(&mut self, sig: EndianSig) {
        self.endian_sig = sig;
    }

    /// The message type.
    pub fn msg_type(&self) -> Type {
        self.msg_type
    }

    /// Set the message type.
    pub fn set_msg_type(&mut self, msg_type: Type) {
        self.msg_type = msg_type;
    }

    /// The message flags.
    pub fn flags(&self) -> BitFlags<Flags> {
        self.flags
    }

    /// Set the message flags.
    pub fn set_flags(&mut self, flags: BitFlags<Flags>) {
        self.flags = flags;
    }

    /// The major version of the protocol the message is compliant to.
    ///
    /// Currently only `1` is valid.
    pub fn protocol_version(&self) -> u8 {
        self.protocol_version
    }

    /// Set the major version of the protocol the message is compliant to.
    ///
    /// Currently only `1` is valid.
    pub fn set_protocol_version(&mut self, version: u8) {
        self.protocol_version = version;
    }

    /// The byte length of the message body.
    pub fn body_len(&self) -> u32 {
        self.body_len
    }

    /// Set the byte length of the message body.
    pub fn set_body_len(&mut self, len: u32) {
        self.body_len = len;
    }

    /// The serial number of the message.
    ///
    /// This is used to match a reply to a method call.
    pub fn serial_num(&self) -> NonZeroU32 {
        self.serial_num
    }

    /// Set the serial number of the message.
    ///
    /// This is used to match a reply to a method call.
    pub fn set_serial_num(&mut self, serial_num: NonZeroU32) {
        self.serial_num = serial_num;
    }
}

/// The message header, containing all the metadata about the message.
///
/// This includes both the [`PrimaryHeader`] and the dynamic fields.
///
/// [`PrimaryHeader`]: struct.PrimaryHeader.html
#[derive(Debug, Clone, Serialize, Deserialize, VariantType)]
pub struct Header<'m> {
    primary: PrimaryHeader,
    #[serde(borrow)]
    fields: Fields<'m>,
}

impl<'m> Header<'m> {
    /// Create a new `Header` instance.
    pub(super) fn new(primary: PrimaryHeader, fields: Fields<'m>) -> Self {
        Self { primary, fields }
    }

    /// Get a reference to the primary header.
    pub fn primary(&self) -> &PrimaryHeader {
        &self.primary
    }

    /// Get a mutable reference to the primary header.
    pub fn primary_mut(&mut self) -> &mut PrimaryHeader {
        &mut self.primary
    }

    /// Get the primary header, consuming `self`.
    pub fn into_primary(self) -> PrimaryHeader {
        self.primary
    }

    /// Get a mutable reference to the message fields.
    pub(super) fn fields_mut(&mut self) -> &mut Fields<'m> {
        &mut self.fields
    }

    /// The message type.
    pub fn message_type(&self) -> Type {
        self.primary().msg_type()
    }

    /// The object to send a call to, or the object a signal is emitted from.
    pub fn path(&self) -> Option<&ObjectPath<'m>> {
        self.fields.path.as_ref()
    }

    /// The interface to invoke a method call on, or that a signal is emitted from.
    pub fn interface(&self) -> Option<&InterfaceName<'m>> {
        self.fields.interface.as_ref()
    }

    /// The member, either the method name or signal name.
    pub fn member(&self) -> Option<&MemberName<'m>> {
        self.fields.member.as_ref()
    }

    /// The name of the error that occurred, for errors.
    pub fn error_name(&self) -> Option<&ErrorName<'m>> {
        self.fields.error_name.as_ref()
    }

    /// The serial number of the message this message is a reply to.
    pub fn reply_serial(&self) -> Option<NonZeroU32> {
        self.fields.reply_serial
    }

    /// The name of the connection this message is intended for.
    pub fn destination(&self) -> Option<&BusName<'m>> {
        self.fields.destination.as_ref()
    }

    /// Unique name of the sending connection.
    pub fn sender(&self) -> Option<&UniqueName<'m>> {
        self.fields.sender.as_ref()
    }

    /// The signature of the message body.
    pub fn signature(&self) -> &Signature {
        &self.fields.signature
    }

    /// The number of Unix file descriptors that accompany the message.
    pub fn unix_fds(&self) -> Option<u32> {
        self.fields.unix_fds
    }
}

static SERIAL_NUM: AtomicU32 = AtomicU32::new(0);

#[cfg(test)]
mod tests {
    use crate::message::{Fields, Header, PrimaryHeader, Type};

    use std::{borrow::Cow, error::Error};
    use test_log::test;
    use zbus_names::{InterfaceName, MemberName};
    use zvariant::{ObjectPath, Signature};

    #[test]
    fn header() -> Result<(), Box<dyn Error>> {
        let path = ObjectPath::try_from("/some/path")?;
        let iface = InterfaceName::try_from("some.interface")?;
        let member = MemberName::try_from("Member")?;
        let mut f = Fields::new();
        f.path = Some(path.clone());
        f.interface = Some(iface.clone());
        f.member = Some(member.clone());
        f.sender = Some(":1.84".try_into()?);
        let h = Header::new(PrimaryHeader::new(Type::Signal, 77), f);

        assert_eq!(h.message_type(), Type::Signal);
        assert_eq!(h.path(), Some(&path));
        assert_eq!(h.interface(), Some(&iface));
        assert_eq!(h.member(), Some(&member));
        assert_eq!(h.error_name(), None);
        assert_eq!(h.destination(), None);
        assert_eq!(h.reply_serial(), None);
        assert_eq!(h.sender().unwrap(), ":1.84");
        assert_eq!(h.signature(), &Signature::Unit);
        assert_eq!(h.unix_fds(), None);

        let mut f = Fields::new();
        f.error_name = Some("org.zbus.Error".try_into()?);
        f.destination = Some(":1.11".try_into()?);
        f.reply_serial = Some(88.try_into()?);
        f.signature = Cow::Owned("say".try_into().unwrap());
        f.unix_fds = Some(12);
        let h = Header::new(PrimaryHeader::new(Type::MethodReturn, 77), f);

        assert_eq!(h.message_type(), Type::MethodReturn);
        assert_eq!(h.path(), None);
        assert_eq!(h.interface(), None);
        assert_eq!(h.member(), None);
        assert_eq!(h.error_name().unwrap(), "org.zbus.Error");
        assert_eq!(h.destination().unwrap(), ":1.11");
        assert_eq!(h.reply_serial().map(Into::into), Some(88));
        assert_eq!(h.sender(), None);
        assert_eq!(h.signature(), &Signature::try_from("say").unwrap());
        assert_eq!(h.unix_fds(), Some(12));

        Ok(())
    }
}
