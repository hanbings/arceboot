use core::ptr::null_mut;

use axsync::Mutex;
use lazyinit::LazyInit;
use uefi_raw::table::{Header, configuration::ConfigurationTable, system::SystemTable};

use crate::runtime::protocol::simple_text_output::{
    get_simple_text_output, init_simple_text_output,
};

use alloc::boxed::Box;

#[derive(Debug)]
pub struct Table {
    system_table: &'static mut SystemTable,
    system_table_raw: *mut SystemTable,
}

unsafe impl Send for Table {}
unsafe impl Sync for Table {}

impl Drop for Table {
    fn drop(&mut self) {
        unsafe {
            let config_raw = self.system_table.configuration_table;
            drop(Box::from_raw(config_raw));
            drop(Box::from_raw(self.system_table_raw));
        }
    }
}

static SYSTEM_TABLE: LazyInit<Mutex<Table>> = LazyInit::new();
// The vendor string is encoded in UTF-16 for UEFI compliance.
// "Rust SBI, Arceboot" in UTF-16.
static VENDOR: &[u16] = &[
    0x0052, 0x0075, 0x0073, 0x0074, 0x0020, 0x0053, 0x0042, 0x0049, 0x002C, 0x0020, 0x0041, 0x0072,
    0x0063, 0x0065, 0x0062, 0x006F, 0x006F, 0x0074, 0x0000,
];
static REVERSION: u32 = 0x0001_0000;

pub fn init_system_table() {
    let simple_text_output = {
        init_simple_text_output();
        get_simple_text_output().lock().get_protocol()
    };

    let configuration_table = Box::new(ConfigurationTable {
        vendor_guid: uefi_raw::Guid::default(),
        vendor_table: null_mut(),
    });
    let configuration_table_raw = Box::into_raw(configuration_table);

    let system_table = Box::new(SystemTable {
        // Build the UEFI Table Header. 
        // For the System Table, its signature is 'IBI SYST' (little-endian).
        // The Header size is the size of the entire Header structure,
        // and the CRC32 calculation will first fill the CRC32 field with 0 before calculation.
        header: Header::default(),

        firmware_vendor: VENDOR.as_ptr(),
        firmware_revision: REVERSION,

        stdin_handle: null_mut(),
        stdin: null_mut(),

        stdout_handle: null_mut(),
        stdout: simple_text_output,

        stderr_handle: null_mut(),
        stderr: simple_text_output,

        runtime_services: null_mut(),
        boot_services: null_mut(),

        number_of_configuration_table_entries: 0,
        configuration_table: configuration_table_raw,
    });
    let system_table_raw = Box::into_raw(system_table);
    let system_table = unsafe { &mut *system_table_raw };

    SYSTEM_TABLE.init_once(Mutex::new(Table {
        system_table: system_table,
        system_table_raw,
    }));
}

pub fn get_system_table_raw() -> *mut SystemTable {
    SYSTEM_TABLE
        .get()
        .expect("SystemTable not initialized")
        .lock()
        .system_table_raw
}
