use crate::error::Error;
use std::ffi::{c_void, CString};
use std::os::windows::ffi::OsStrExt;
use std::str::FromStr;
use windows::core::PCSTR;
use windows::Win32::Foundation::{HANDLE, HMODULE, MAX_PATH};
use windows::Win32::System::Diagnostics::Debug::WriteProcessMemory;
use windows::Win32::System::LibraryLoader::*;
use windows::Win32::System::Memory::*;
use windows::Win32::System::ProcessStatus::*;
use windows::Win32::System::Threading::*;

pub fn to_wstring(s: &str) -> Vec<u16> {
    std::ffi::OsStr::new(s)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

pub fn find_pid_by_name(find: &str) -> Result<Option<u32>, Error> {
    unsafe {
        let mut processes = [0u32; 1024];
        let mut needed = 0u32;

        let _ = EnumProcesses(
            &mut processes as *mut _,
            size_of_val(&processes) as _,
            &mut needed as *mut _,
        )?;

        let n = needed as usize / size_of::<u32>();

        for i in 0..n {
            let Ok(process) = OpenProcess(
                PROCESS_QUERY_INFORMATION | PROCESS_VM_READ,
                false,
                processes[i],
            ) else {
                continue;
            };

            let mut module = HMODULE::default();

            if EnumProcessModules(
                process,
                &mut module as *mut _,
                size_of::<HMODULE>() as _,
                &mut needed as *mut _,
            )
            .is_err()
            {
                continue;
            }

            let mut name = [0u8; MAX_PATH as usize];
            let len = GetModuleBaseNameA(process, Some(module), &mut name[..]);

            let Ok(name) = std::str::from_utf8(&name[..len as usize]) else {
                continue;
            };

            if name.starts_with(find) {
                return Ok(Some(processes[i]));
            }
        }
    }

    Ok(None)
}

pub fn allocate_memory(pid: u32, len: usize) -> Result<*mut c_void, Error> {
    unsafe {
        let handle = OpenProcess(PROCESS_ALL_ACCESS, false, pid)?;
        let address = VirtualAllocEx(handle, None, len, MEM_COMMIT | MEM_RESERVE, PAGE_READWRITE);

        if address.is_null() {
            return Err(Error::MemoryAllocationFailed);
        }

        Ok(address)
    }
}

pub fn get_proc_address() -> Result<*mut c_void, Error> {
    unsafe {
        let kernel = CString::new("kernel32")?;
        let loader = CString::new("LoadLibraryA")?;

        let module = GetModuleHandleA(PCSTR::from_raw(kernel.as_ptr() as _))?;

        Ok(
            GetProcAddress(module, PCSTR::from_raw(loader.as_ptr() as _))
                .ok_or_else(|| Error::LibraryLoaderNotFound)? as _,
        )
    }
}

pub fn write_memory(
    pid: u32,
    loader: *const c_void,
    address: *const c_void,
    path: &str,
) -> Result<HANDLE, Error> {
    unsafe {
        let handle = OpenProcess(PROCESS_ALL_ACCESS, false, pid)?;
        let cpath = CString::from_str(path)?;
        let len = cpath.as_bytes_with_nul().len();

        println!("--> {:?}", cpath);

        let _ = WriteProcessMemory(handle, address, cpath.as_ptr() as _, len, None)?;

        let start: LPTHREAD_START_ROUTINE = Some(std::mem::transmute(loader));

        let mut thread_id = 0;
        let handle = CreateRemoteThread(
            handle,
            None,
            0,
            start,
            Some(address),
            0,
            Some(&mut thread_id),
        )?;

        Ok(handle)
    }
}
