use async_trait::async_trait;
use tracing::{instrument, warn};
use windows_sys::Win32::Foundation::*;
use windows_sys::Win32::Graphics::Printing::*;
use windows_sys::Win32::UI::Controls::Dialogs::*;

use crate::errors::infrastructure_error::InfrastructureError;
use crate::infrastructure::printer::printer_transport::PrinterTransport;

extern "system" {
    fn GlobalLock(hMem: HGLOBAL) -> *mut core::ffi::c_void;
    fn GlobalUnlock(hMem: HGLOBAL) -> i32;
}

#[derive(Debug, Clone)]
pub struct WindowsPrinterInfo {
    pub name: String,
    pub driver_name: String,
}

pub fn list_installed_printers() -> Result<Vec<WindowsPrinterInfo>, InfrastructureError> {
    let flags = PRINTER_ENUM_LOCAL | PRINTER_ENUM_CONNECTIONS;
    let level = 4u32;

    let mut needed: u32 = 0;
    let mut returned: u32 = 0;

    unsafe {
        EnumPrintersW(
            flags,
            std::ptr::null(),
            level,
            std::ptr::null_mut(),
            0,
            &mut needed,
            &mut returned,
        );
    }

    if needed == 0 {
        return Ok(Vec::new());
    }

    let mut buffer: Vec<u8> = vec![0u8; needed as usize];

    unsafe {
        EnumPrintersW(
            flags,
            std::ptr::null(),
            level,
            buffer.as_mut_ptr(),
            needed,
            &mut needed,
            &mut returned,
        );
    }

    let info_size = std::mem::size_of::<PRINTER_INFO_4W>();
    let count = (returned as usize).min(buffer.len() / info_size);

    let mut printers = Vec::with_capacity(count);
    for i in 0..count {
        let info = unsafe { &*((buffer.as_ptr() as *const PRINTER_INFO_4W).add(i)) };
        let name = unsafe { wide_ptr_to_string(info.pPrinterName) };
        if let Some(name) = name {
            printers.push(WindowsPrinterInfo {
                name,
                driver_name: String::new(),
            });
        }
    }

    Ok(printers)
}

pub fn show_print_dialog(hwnd_owner: Option<isize>) -> Option<String> {
    let mut pd: PRINTDLGW = unsafe { std::mem::zeroed() };
    pd.lStructSize = std::mem::size_of::<PRINTDLGW>() as u32;
    pd.hwndOwner = hwnd_owner.unwrap_or(0) as *mut core::ffi::c_void;
    pd.Flags = 0;

    tracing::info!("Calling PrintDlgW, struct size={}", pd.lStructSize);

    let shown = unsafe { PrintDlgW(&mut pd) };

    if shown == 0 {
        tracing::info!("PrintDlgW returned 0 (cancelled by user)");
        return None;
    }

    if pd.hDevNames.is_null() {
        warn!("hDevNames is null after PrintDlgW success");
        return None;
    }

    unsafe {
        let devnames_ptr = GlobalLock(pd.hDevNames) as *const DEVNAMES;
        if devnames_ptr.is_null() {
            warn!("GlobalLock returned null for hDevNames");
            return None;
        }

        let devnames = &*devnames_ptr;
        tracing::info!(
            "DEVNAMES: wDriverOffset={}, wDeviceOffset={}, wOutputOffset={}",
            devnames.wDriverOffset,
            devnames.wDeviceOffset,
            devnames.wOutputOffset,
        );

        let device_ptr = (devnames_ptr as *const u8).add(devnames.wDeviceOffset as usize);
        let name = wide_ptr_to_string(device_ptr as *const u16);

        tracing::info!("Extracted printer name: {:?}", name);

        GlobalUnlock(pd.hDevNames);

        name
    }
}

pub fn print_to_printer(printer_name: &str, data: &[u8]) -> Result<(), InfrastructureError> {
    let printer_name_wide = wide_string(printer_name);

    unsafe {
        let mut handle = PRINTER_HANDLE { Value: std::ptr::null_mut() };
        if OpenPrinterW(
            printer_name_wide.as_ptr(),
            &mut handle,
            std::ptr::null(),
        ) == 0
        {
            let err = std::io::Error::last_os_error();
            return Err(InfrastructureError::PrinterConnection(format!(
                "OpenPrinterW failed for '{}': {}",
                printer_name, err
            )));
        }

        let mut doc_name = wide_string("Zebra Label");
        let mut datatype = wide_string("RAW");

        let doc_info = DOC_INFO_1W {
            pDocName: doc_name.as_mut_ptr(),
            pOutputFile: std::ptr::null_mut(),
            pDatatype: datatype.as_mut_ptr(),
        };

        if StartDocPrinterW(handle, 1, &doc_info as *const DOC_INFO_1W) == 0 {
            let err = std::io::Error::last_os_error();
            ClosePrinter(handle);
            return Err(InfrastructureError::PrinterConnection(format!(
                "StartDocPrinterW failed: {}",
                err
            )));
        }

        let mut bytes_written: u32 = 0;
        if WritePrinter(
            handle,
            data.as_ptr() as *const core::ffi::c_void,
            data.len() as u32,
            &mut bytes_written,
        ) == 0
        {
            let err = std::io::Error::last_os_error();
            EndDocPrinter(handle);
            ClosePrinter(handle);
            return Err(InfrastructureError::PrinterConnection(format!(
                "WritePrinter failed: {}",
                err
            )));
        }

        EndDocPrinter(handle);
        ClosePrinter(handle);
    }

    Ok(())
}

fn wide_string(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

unsafe fn wide_ptr_to_string(ptr: *const u16) -> Option<String> {
    if ptr.is_null() {
        return None;
    }
    let base = ptr as usize;
    let mut len = 0usize;
    loop {
        let ch = core::ptr::read_unaligned((base + len * 2) as *const u16);
        if ch == 0 {
            break;
        }
        len += 1;
    }
    let mut buf = Vec::with_capacity(len);
    for i in 0..len {
        buf.push(core::ptr::read_unaligned((base + i * 2) as *const u16));
    }
    String::from_utf16(&buf).ok()
}

pub struct WindowsPrintTransport {
    printer_name: String,
}

impl WindowsPrintTransport {
    pub fn new(printer_name: &str) -> Self {
        Self {
            printer_name: printer_name.to_string(),
        }
    }
}

#[async_trait]
impl PrinterTransport for WindowsPrintTransport {
    #[instrument(skip(self, data), fields(printer = %self.printer_name))]
    async fn send(&self, data: &[u8]) -> Result<(), InfrastructureError> {
        let printer_name = self.printer_name.clone();
        let data = data.to_vec();

        tokio::task::spawn_blocking(move || print_to_printer(&printer_name, &data))
            .await
            .map_err(|e| {
                warn!(error = %e, "spawn_blocking failed for print_to_printer");
                InfrastructureError::PrinterConnection(format!("Task join error: {}", e))
            })?
    }

    #[instrument(skip(self))]
    async fn test_connection(&self) -> Result<(), InfrastructureError> {
        Ok(())
    }
}
