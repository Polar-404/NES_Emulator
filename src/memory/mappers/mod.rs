pub mod ines_mapper000;
pub mod ines_mapper001;
pub mod ines_mapper004;
pub mod ines_mapper163;

pub mod dummy_mapper;

pub use self::ines_mapper000::InesMapper000;
pub use self::ines_mapper001::InesMapper001;
pub use self::ines_mapper004::InesMapper004;
pub use self::ines_mapper163::InesMapper163;