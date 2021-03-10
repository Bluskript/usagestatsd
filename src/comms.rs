use dbus::channel::MatchingReceiver;
use dbus::message::MatchRule;
use dbus::nonblock::SyncConnection;
use dbus_crossroads::Crossroads;
use dbus_tokio::connection;
use std::sync::{Arc, Mutex};
use tokio;

use crate::monitor::ProcessHandler;

pub struct IPC {
    connection: Arc<SyncConnection>,
    handler: Arc<Mutex<ProcessHandler>>,
}

impl IPC {
    pub async fn new(
        name: &'static str,
        handler: Arc<Mutex<ProcessHandler>>,
    ) -> Result<IPC, Box<dyn std::error::Error>> {
        let (resource, c) = connection::new_session_sync()?;
        let inst = IPC {
            connection: c.clone(),
            handler,
        };
        tokio::spawn(async {
            let err = resource.await;
            panic!("Lost connection to D-Bus: {}", err);
        });

        c.request_name(name, false, true, false).await?;

        let mut cr = Crossroads::new();

        cr.set_async_support(Some((
            c.clone(),
            Box::new(|x| {
                tokio::spawn(x);
            }),
        )));

        let iface_token = cr.register(name, |b| {
            b.method_with_cr_async(
                "GetUsageStats",
                ("test",),
                ("stats",),
                |mut ctx, _cr, _test: (String,)| async move {
                    let signal_msg = ctx.make_signal("UsageStatsResponse", ());
                    ctx.push_msg(signal_msg);
                    ctx.reply(Ok(("Test",)))
                },
            );
        });

        cr.insert("/usagestats", &[iface_token], 0);

        c.start_receive(
            MatchRule::new_method_call(),
            Box::new(move |msg, conn| return cr.handle_message(msg, conn).is_ok()),
        );

        return Ok(inst);
    }
}
