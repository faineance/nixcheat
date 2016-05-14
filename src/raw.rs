extern crate libc;
extern { 
    // <sys/uio.h> process_vm_readv, process_vm_writev - transfer data between process address spaces 
    pub fn process_vm_readv(pid: libc::pid_t, 
        local_iov: libc::iovec, 
        liovcnt: u32, 
        remote_iov: libc::iovec,
        riovcnt: u32, flags: u32  ) -> libc::size_t;

    pub fn process_vm_writev(pid: libc::pid_t, 
        local_iov: libc::iovec, 
        liovcnt: u32, 
        remote_iov: libc::iovec,
        riovcnt: u32, flags: u32  ) -> libc::size_t;
}