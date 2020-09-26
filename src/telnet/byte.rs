pub const BYTE_IAC: u8 = 255;   // FF: interpret as command:
pub const BYTE_DONT: u8 = 254;  // FE: you are not to use option
pub const BYTE_DO: u8 = 253;    // FD: please, you use option
pub const BYTE_WONT: u8 = 252;  // FC: I won't use option
pub const BYTE_WILL: u8 = 251;  // FB: I will use option
pub const BYTE_SB: u8 = 250;    // FA: interpret as subnegotiation
pub const BYTE_SE: u8 = 240;    // F0: end sub negotiation
