extern crate libc;
use libc::c_void;

use std::process::Command;
use libc::iovec;
use std::io::prelude::*;
use std::fs::File;
use std::io::BufReader;
use std::io;
use std::path::Path;
use std::mem;
use std::slice;
use std::ops::Range;
use raw::{process_vm_readv, process_vm_writev};
use std::io::ErrorKind;

fn get_pid(name: &'static str) -> Option <i32> {
    let pid = Command::new("pidof")
    .arg(name)
    .output()
    .unwrap_or_else(|e| { panic!("Failed to execute process: {}", e) });

    match String::from_utf8_lossy(&pid.stdout).as_ref() {
        "" => None,
        v => Some(v.trim_right().parse::<i32>().unwrap())
    }
}

pub fn is_root() -> bool {
    unsafe {libc::getuid() == 0}
}

fn get_module(module_name: &'static str, pid: i32) -> io::Result<Option<Range<*const u8>>> {
    let file = try!(File::open(format!("/proc/{}/maps", pid)));
    let maps = BufReader::new(file);

    for line in maps.lines() {
        let unwrapped = line.unwrap();
        if unwrapped.contains(module_name) {
            let start = usize::from_str_radix(&unwrapped[..12], 16).unwrap() as *const u8;
            let end = usize::from_str_radix(&unwrapped[14..25], 16).unwrap() as *const u8;
            return Ok(Some(start..end))
        }
    }
    Ok(None)
}

#[derive(Debug, Clone)]
pub struct Handle {
    pub pid: i32,
    pub module: Range<*const u8>,
}

impl Handle {
    pub fn new(process: &'static str, module_name: &'static str) -> Handle {
        if !is_root() {
            panic!("Please rerun as root.");
        }
        let pid = get_pid(process).unwrap_or_else(|| { 
            panic!("Could not find pid of process: {}. Is it running?", process) 
        });
        let module = get_module(module_name, pid).unwrap_or_else(|e| {
            panic!("Could not open /proc/{}/maps: {}", pid, e) 
        }).unwrap_or_else(|| {
            panic!("Could not find module: {} for process {}. Are you sure it's loaded?", module_name, process) 
        });
        Handle {
            pid: pid,
            module: module,
        }
    }
    unsafe fn write(&self, address: *mut u8, buffer: &[u8]) { 
        let local = iovec {
            iov_base: buffer.as_ptr() as *mut u8 as *mut c_void,
            iov_len: buffer.len()
        };

        let remote = iovec {
            iov_base: address as *mut c_void,
            iov_len: buffer.len()
        };
        
        assert_eq!(process_vm_writev(self.pid, local, 1, remote, 1, 0), buffer.len())
    }
    unsafe fn read(&self, into: &mut [u8], address: *const u8, size: usize) {
        let local = iovec {
            iov_base: into.as_mut_ptr() as *mut c_void,
            iov_len: into.len(),
        };

        let remote = iovec {
            iov_base: address as *mut u8 as *mut c_void,
            iov_len: size,
        };

        assert!(process_vm_readv(self.pid, local, 1, remote, 1, 0) == size);
    }
    pub unsafe fn read_type<T>(&self, address: *const T) -> T {
        let mut t = mem::uninitialized();
        let buffer = slice::from_raw_parts_mut(
            &mut t as *mut T as *mut u8,
            mem::size_of::<T>());
        self.read(buffer, address as *const u8, mem::size_of::<T>());
        t
    }
    pub unsafe fn write_type<T>(&self, address: *mut T, t: T) {
        let buffer = slice::from_raw_parts(&t as *const T as *const u8, mem::size_of::<T>());
        self.write(address as *mut u8, buffer);
    }
}