mod cnproc;
mod process_handler;

fn main() {
    let mut handler = match process_handler::Handler::new() {
        Ok(h) => h,
        Err(e) => panic!("{}", e),
    };

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
        for event in events {
            match event {
                cnproc::lib::PidEvent::Exec(v) => match handler.on_process_start(v) {
                    Err(e) => println!("{}", e),
                    _ => (),
                },
                _ => (),
            }
        }
    }
}
