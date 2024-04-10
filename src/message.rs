// SPDX-License-Identifier: MIT

use alloc::vec;
use axerrno::AxError;
use netlink_packet_utils::{
    DecodeError, Emitable, Parseable, ParseableParametrized,
};

use netlink_packet_core::{
    NetlinkDeserializable, NetlinkHeader, NetlinkPayload, NetlinkSerializable,
};

use crate::{
    address::{AddressHeader, AddressMessage, AddressMessageBuffer},
    link::{LinkMessage, LinkMessageBuffer},
};

const RTM_NEWLINK: u16 = 16;
const RTM_DELLINK: u16 = 17;
const RTM_GETLINK: u16 = 18;
const RTM_SETLINK: u16 = 19;
const RTM_NEWADDR: u16 = 20;
const RTM_DELADDR: u16 = 21;
const RTM_GETADDR: u16 = 22;
const RTM_NEWROUTE: u16 = 24;
const RTM_DELROUTE: u16 = 25;
const RTM_GETROUTE: u16 = 26;
const RTM_NEWNEIGH: u16 = 28;
const RTM_DELNEIGH: u16 = 29;
const RTM_GETNEIGH: u16 = 30;
const RTM_NEWRULE: u16 = 32;
const RTM_DELRULE: u16 = 33;
const RTM_GETRULE: u16 = 34;
const RTM_NEWQDISC: u16 = 36;
const RTM_DELQDISC: u16 = 37;
const RTM_GETQDISC: u16 = 38;
const RTM_NEWTCLASS: u16 = 40;
const RTM_DELTCLASS: u16 = 41;
const RTM_GETTCLASS: u16 = 42;
const RTM_NEWTFILTER: u16 = 44;
const RTM_DELTFILTER: u16 = 45;
const RTM_GETTFILTER: u16 = 46;
// const RTM_NEWACTION: u16 = 48;
// const RTM_DELACTION: u16 = 49;
// const RTM_GETACTION: u16 = 50;
const RTM_NEWPREFIX: u16 = 52;
// const RTM_GETMULTICAST: u16 = 58;
// const RTM_GETANYCAST: u16 = 62;
const RTM_NEWNEIGHTBL: u16 = 64;
const RTM_GETNEIGHTBL: u16 = 66;
const RTM_SETNEIGHTBL: u16 = 67;
// const RTM_NEWNDUSEROPT: u16 = 68;
// const RTM_NEWADDRLABEL: u16 = 72;
// const RTM_DELADDRLABEL: u16 = 73;
// const RTM_GETADDRLABEL: u16 = 74;
// const RTM_GETDCB: u16 = 78;
// const RTM_SETDCB: u16 = 79;
// const RTM_NEWNETCONF: u16 = 80;
// const RTM_DELNETCONF: u16 = 81;
// const RTM_GETNETCONF: u16 = 82;
// const RTM_NEWMDB: u16 = 84;
// const RTM_DELMDB: u16 = 85;
// const RTM_GETMDB: u16 = 86;
const RTM_NEWNSID: u16 = 88;
const RTM_DELNSID: u16 = 89;
const RTM_GETNSID: u16 = 90;
// const RTM_NEWSTATS: u16 = 92;
// const RTM_GETSTATS: u16 = 94;
// const RTM_NEWCACHEREPORT: u16 = 96;
const RTM_NEWCHAIN: u16 = 100;
const RTM_DELCHAIN: u16 = 101;
const RTM_GETCHAIN: u16 = 102;
const RTM_NEWLINKPROP: u16 = 108;
const RTM_DELLINKPROP: u16 = 109;

buffer!(RouteNetlinkMessageBuffer);

impl<'a, T: AsRef<[u8]> + ?Sized>
    ParseableParametrized<RouteNetlinkMessageBuffer<&'a T>, u16>
    for RouteNetlinkMessage
{
    fn parse_with_param(
        buf: &RouteNetlinkMessageBuffer<&'a T>,
        message_type: u16,
    ) -> Result<Self, DecodeError> {
        let message = match message_type {
            // Link messages
            RTM_NEWLINK | RTM_GETLINK | RTM_DELLINK | RTM_SETLINK => {
                let msg = match LinkMessageBuffer::new_checked(&buf.inner()) {
                    Ok(buf) => LinkMessage::parse(&buf)
                        ?,
                    // HACK: iproute2 sends invalid RTM_GETLINK message, where
                    // the header is limited to the
                    // interface family (1 byte) and 3 bytes of padding.
                    Err(e) => {
                        if buf.inner().len() == 4 && message_type == RTM_GETLINK
                        {
                            let mut msg = LinkMessage::default();
                            msg.header.interface_family = buf.inner()[0].into();
                            msg
                        } else {
                            return Err(e);
                        }
                    }
                };
                match message_type {
                    RTM_NEWLINK => RouteNetlinkMessage::NewLink(msg),
                    RTM_GETLINK => RouteNetlinkMessage::GetLink(msg),
                    RTM_DELLINK => RouteNetlinkMessage::DelLink(msg),
                    RTM_SETLINK => RouteNetlinkMessage::SetLink(msg),
                    _ => unreachable!(),
                }
            }

            // Address messages
            RTM_NEWADDR | RTM_GETADDR | RTM_DELADDR => {
                let msg = match AddressMessageBuffer::new_checked(&buf.inner())
                {
                    Ok(buf) => AddressMessage::parse(&buf)
                        ?,
                    // HACK: iproute2 sends invalid RTM_GETADDR message, where
                    // the header is limited to the
                    // interface family (1 byte) and 3 bytes of padding.
                    Err(e) => {
                        if buf.inner().len() == 4 && message_type == RTM_GETADDR
                        {
                            let mut msg = AddressMessage {
                                header: AddressHeader::default(),
                                attributes: vec![],
                            };
                            msg.header.family = buf.inner()[0].into();
                            msg
                        } else {
                            return Err(e);
                        }
                    }
                };
                match message_type {
                    RTM_NEWADDR => RouteNetlinkMessage::NewAddress(msg),
                    RTM_GETADDR => RouteNetlinkMessage::GetAddress(msg),
                    RTM_DELADDR => RouteNetlinkMessage::DelAddress(msg),
                    _ => unreachable!(),
                }
            }
            _ => {
                return Err(
                    AxError::InvalidInput
                )
            }
        };
        Ok(message)
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
#[non_exhaustive]
pub enum RouteNetlinkMessage {
    NewLink(LinkMessage),
    DelLink(LinkMessage),
    GetLink(LinkMessage),
    SetLink(LinkMessage),
    NewLinkProp(LinkMessage),
    DelLinkProp(LinkMessage),
    NewAddress(AddressMessage),
    DelAddress(AddressMessage),
    GetAddress(AddressMessage),
}

impl RouteNetlinkMessage {
    pub fn is_new_link(&self) -> bool {
        matches!(self, RouteNetlinkMessage::NewLink(_))
    }

    pub fn is_del_link(&self) -> bool {
        matches!(self, RouteNetlinkMessage::DelLink(_))
    }

    pub fn is_get_link(&self) -> bool {
        matches!(self, RouteNetlinkMessage::GetLink(_))
    }

    pub fn is_set_link(&self) -> bool {
        matches!(self, RouteNetlinkMessage::SetLink(_))
    }

    pub fn is_new_address(&self) -> bool {
        matches!(self, RouteNetlinkMessage::NewAddress(_))
    }

    pub fn is_del_address(&self) -> bool {
        matches!(self, RouteNetlinkMessage::DelAddress(_))
    }

    pub fn is_get_address(&self) -> bool {
        matches!(self, RouteNetlinkMessage::GetAddress(_))
    }

    pub fn message_type(&self) -> u16 {
        use self::RouteNetlinkMessage::*;

        match self {
            NewLink(_) => RTM_NEWLINK,
            DelLink(_) => RTM_DELLINK,
            GetLink(_) => RTM_GETLINK,
            SetLink(_) => RTM_SETLINK,
            NewLinkProp(_) => RTM_NEWLINKPROP,
            DelLinkProp(_) => RTM_DELLINKPROP,
            NewAddress(_) => RTM_NEWADDR,
            DelAddress(_) => RTM_DELADDR,
            GetAddress(_) => RTM_GETADDR,
        }
    }
}

impl Emitable for RouteNetlinkMessage {
    #[rustfmt::skip]
    fn buffer_len(&self) -> usize {
        use self::RouteNetlinkMessage::*;
        match self {
            | NewLink(ref msg)
            | DelLink(ref msg)
            | GetLink(ref msg)
            | SetLink(ref msg)
            | NewLinkProp(ref msg)
            | DelLinkProp(ref msg)
            =>  msg.buffer_len(),

            | NewAddress(ref msg)
            | DelAddress(ref msg)
            | GetAddress(ref msg)
            => msg.buffer_len(),
        }
    }

    #[rustfmt::skip]
    fn emit(&self, buffer: &mut [u8]) {
        use self::RouteNetlinkMessage::*;
        match self {
            | NewLink(ref msg)
            | DelLink(ref msg)
            | GetLink(ref msg)
            | SetLink(ref msg)
            | NewLinkProp(ref msg)
            | DelLinkProp(ref msg)
            => msg.emit(buffer),

            | NewAddress(ref msg)
            | DelAddress(ref msg)
            | GetAddress(ref msg)
            => msg.emit(buffer),
        }
    }
}

impl NetlinkSerializable for RouteNetlinkMessage {
    fn message_type(&self) -> u16 {
        self.message_type()
    }

    fn buffer_len(&self) -> usize {
        <Self as Emitable>::buffer_len(self)
    }

    fn serialize(&self, buffer: &mut [u8]) {
        self.emit(buffer)
    }
}

impl NetlinkDeserializable for RouteNetlinkMessage {
    fn deserialize(
        header: &NetlinkHeader,
        payload: &[u8],
    ) -> Result<Self, AxError> {
        let buf = RouteNetlinkMessageBuffer::new(payload);
        match RouteNetlinkMessage::parse_with_param(&buf, header.message_type) {
            Err(e) => Err(e),
            Ok(message) => Ok(message),
        }
    }
}

impl From<RouteNetlinkMessage> for NetlinkPayload<RouteNetlinkMessage> {
    fn from(message: RouteNetlinkMessage) -> Self {
        NetlinkPayload::InnerMessage(message)
    }
}
