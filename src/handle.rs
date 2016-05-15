extern crate libc;


use std::process::Command;

use std::io::prelude::*;
use std::fs::File;
use std::io::BufReader;
use std::io;
use std::path::Path;
use std::mem;
use std::slice;
use std::ops::Range;
use libc::*;
use std::io::Error;
use std::ptr;
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

fn get_module(module_name: &'static str, pid: i32) -> io::Result<Option<Module>> {
    let file = try!(File::open(format!("/proc/{}/maps", pid)));
    let maps = BufReader::new(file);

    for line in maps.lines() {
        let unwrapped = line.unwrap();
        if unwrapped.contains(module_name) {
            let start = u32::from_str_radix(&unwrapped[..8], 16).unwrap();
            let end = u32::from_str_radix(&unwrapped[10..17], 16).unwrap();
            return Ok(Some(Module(start..end)))
        }
    }
    Ok(None)
}

#[derive(Debug, Clone)]
pub struct Module(pub Range<u32>);
unsafe impl Send for Module {}
unsafe impl Sync for Module {}

#[derive(Debug, Clone)]
pub struct Handle {
    pub pid: i32,
    pub module: Module,
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
    // pub unsafe fn write(&self, address: *mut u8, buffer: &[u8]) { 
    //     let local = iovec {
    //         iov_base: buffer.as_ptr() as *mut u8 as *mut c_void,
    //         iov_len: buffer.len()
    //     };

    //     let remote = iovec {
    //         iov_base: address as *mut c_void,
    //         iov_len: buffer.len()
    //     };
        
    //     assert_eq!(process_vm_writev(self.pid, local, 1, remote, 1, 0), buffer.len() as libc::ssize_t)
    // }
    pub unsafe fn read(&self, address: *const c_void, into: *mut c_void, size: usize) {
        let local = iovec {
            iov_base: into,
            iov_len: size,
        };

        let remote = iovec {
            iov_base: address as *mut c_void,
            iov_len: size,
        };
        assert_eq!(process_vm_readv(self.pid as libc::pid_t, &local, 1, &remote, 1, 0), size as libc::ssize_t);
    }
    pub unsafe fn read_type<T>(&self, address: *const c_void) -> T {
        let mut result: T = mem::uninitialized();

        let length = mem::size_of::<T>();

        let mut bytes: Vec<u8> = unsafe {
                let mut vec = Vec::with_capacity(length);
                vec.set_len(length);
                vec
        };
        self.read(address, bytes.as_mut_ptr() as *mut libc::c_void, mem::size_of::<u32>());
        // println!("{:?}", );
        ptr::copy_nonoverlapping(bytes.as_ptr(), &mut result as *mut _ as *mut u8, length);
        println!("{:?}", bytes );
        result
    }
    // pub unsafe fn write_type<T>(&self, address: *mut T, t: T) {
    //     let buffer = slice::from_raw_parts(&t as *const T as *const u8, mem::size_of::<T>());
    //     self.write(address as *mut u8, buffer);
    // }
    pub fn is_running(&self) -> bool {
        Path::is_dir(format!("/proc/{}", self.pid).as_ref())
    }
}