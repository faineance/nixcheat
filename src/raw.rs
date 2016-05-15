extern crate libc;
// extern { 
//     // <sys/uio.h> process_vm_readv, process_vm_writev - transfer data between process address spaces 
//     pub fn process_vm_readv(pid: libc::pid_t, 
//         local_iov: iovec, 
//         liovcnt: u32, 
//         remote_iov: iovec,
//         riovcnt: u32, flags: u32  ) -> libc::ssize_t;

//     pub fn process_vm_writev(pid: libc::pid_t, 
//         local_iov: iovec, 
//         liovcnt: u32, 
//         remote_iov: iovec,
//         riovcnt: u32, flags: u32  ) -> libc::ssize_t;
// }

// #[repr(packed)]
// pub struct iovec {
//     pub iov_base: *mut libc::c_void,
//     pub iov_len: libc::size_t,
// }