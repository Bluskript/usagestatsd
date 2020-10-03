mod cnproc;

use procfs::process::Process;

fn main() {
    let mut monitor = match cnproc::lib::PidMonitor::new() {
        Ok(m) => m,
        Err(e) => panic!(e.kind()),
    };
    match monitor.listen() {
        Ok(r) => r,
        Err(e) => panic!(e.kind()),
    }
    loop {
        let events = match monitor.get_events() {
            Ok(r) => r,
            Err(e) => panic!(e.kind()),
        };
        for event in &events {
            match event {
                cnproc::lib::PidEvent::New(v) => on_new_process(*v),
                _ => (),
            }
        }
    }
}

fn on_new_process(pid: i32) {
    if let Ok(proc) = Process::new(pid) {
        if let Ok(path) = proc.exe() {
            println!("{}", path.to_str().unwrap())
        } else {
            println!("path not found")
        }
    } else {
        println!("err finding process")
    }
}
