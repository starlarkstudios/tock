#![feature(core,no_std)]
#![no_main]
#![no_std]

extern crate core;
extern crate common;
extern crate support;
extern crate hil;
extern crate platform;

mod apps;

pub mod process;
pub mod syscall;

#[no_mangle]
pub extern fn main() {
    use core::prelude::*;
    use process::Process;

    let mut platform = unsafe {
        platform::init()
    };

    let app1 = unsafe { Process::create(apps::app1_init).unwrap() };

    let mut process = app1;

    unsafe {
        match process.state {
            process::State::Running => {
                process.switch_to();
            }
            process::State::Waiting => {
                match process.callbacks.dequeue() {
                    None => { },
                    Some(cb) => {
                        process.state = process::State::Running;
                        process.switch_to_callback(cb);
                    }
                }
            }
        }
        match process.svc_number() {
            Some(syscall::WAIT) => {
                platform.with_driver(1, |console| {
                    console.map(|c| c. command(0, 'w' as usize));
                });
                process.state = process::State::Waiting;
                process.pop_syscall_stack();
                // TODO(alevy): iterate `process` to next available app
            },
            Some(syscall::SUBSCRIBE) => {
                platform.with_driver(1, |console| {
                    console.map(|c| c. command(0, 's' as usize));
                });
                let res = platform.with_driver(process.r0(), |driver| {
                    match driver {
                        Some(d) => d.subscribe(process.r1(),
                                                    process.r2()),
                        None => -1
                    }
                });
                process.set_r0(res);
            },
            Some(syscall::COMMAND) => {
                platform.with_driver(1, |console| {
                    console.map(|c| c. command(0, 'c' as usize));
                });
                let res = platform.with_driver(process.r0(), |driver| {
                    match driver {
                        Some(d) => d.command(process.r1(),
                                             process.r2()),
                        None => -1
                    }
                });
                process.set_r0(res);
            },
            _ => {}
        }
    }

    loop {
        unsafe {
            platform.service_pending_interrupts();

            support::atomic(|| {
                if !platform.has_pending_interrupts() {
                    support::wfi();
                }
            })
        };
    }
}

