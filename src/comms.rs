use dbus::channel::MatchingReceiver;
use dbus::message::MatchRule;
use dbus::nonblock::SyncConnection;
use dbus_crossroads::{Context, Crossroads, MethodErr};
use dbus_tokio::connection;
use std::sync::{Arc, Mutex};
use tokio;

use crate::store::Store;

pub struct IPC {
    connection: Arc<SyncConnection>,
}

impl IPC {
    pub async fn new(
        name: &'static str,
        store: Arc<Mutex<Store>>,
    ) -> Result<IPC, Box<dyn std::error::Error>> {
        let (resource, c) = connection::new_session_sync()?;
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
        let s = store.clone();
        let get_usage_stats = {
            let ss = s.clone();
            move |mut ctx: Context, _cr: &mut Crossroads, _args: ()| {
                let sss = ss.clone();
                async move {
                    match sss.lock().unwrap().get_least_used() {
                        Ok(least_used) => ctx.reply(Ok(("Test",))),
                        Err(e) => ctx.reply(Err(MethodErr::failed("unable to get usage stats"))),
                    }
                }
            }
        };

        let iface_token = cr.register(name, move |b| {
            let s = Arc::clone(&store);
            b.method_with_cr_async("GetUsageStats", (), ("stats",), get_usage_stats);
        });

        cr.insert("/usagestats", &[iface_token], 0);

        c.start_receive(
            MatchRule::new_method_call(),
            Box::new(move |msg, conn| return cr.handle_message(msg, conn).is_ok()),
        );

        return Ok(IPC {
            connection: c.clone(),
        });
    }
}
