//! Connection handling: connecting to a server and processing its events.

use std::path::PathBuf;
use std::time::Instant;

use elora_client::online::{OnlineClient, Status};
use elora_net::{ClientEvent, DisconnectReason};

use crate::app_state::known_servers_path;
use crate::connection::Connection;
use crate::debug_ui::KeyWarning;
use crate::{App, Online, hosting, settings};

impl App {
    pub(crate) fn connect(&mut self) {
        let address = self.net.address.trim().to_owned();
        let expected = address
            .parse()
            .ok()
            .and_then(|a| self.known.get(a))
            .or_else(|| {
                std::net::ToSocketAddrs::to_socket_addrs(&address)
                    .ok()?
                    .next()
                    .and_then(|a| self.known.get(a))
            });
        match Connection::open(&address, expected, self.net.conditions()) {
            Ok(conn) => {
                self.status = self.lang.f("menu.connecting", &[("server", &conn.server)]);
                let client = OnlineClient::new(&self.net.name, self.net.skin, Instant::now())
                    .with_store(Box::new(elora_client::map_store::DiskStore {
                        maps_dirs: hosting::map_dirs(),
                        download_dir: settings::data_dir()
                            .map_or_else(|| PathBuf::from("downloads"), |d| d.join("downloads")),
                    }));
                self.online = Some(Online { client, conn });
                self.sandbox.stop_recording("Online");
            }
            Err(e) => {
                self.status = self
                    .lang
                    .f("menu.connect_failed", &[("e", &format!("{e:#}"))]);
            }
        }
    }

    /// Process network events and advance the online client.
    pub(crate) fn update_online(&mut self, now: Instant) {
        let Some(o) = &mut self.online else { return };
        let mut ended = None;
        for (event, at) in o.conn.events() {
            match event {
                ClientEvent::Connected { server_key } => {
                    if self.known.get(o.conn.server).is_none()
                        && let Err(e) =
                            self.known
                                .trust(&known_servers_path(), o.conn.server, &server_key)
                    {
                        tracing::warn!("server key not saved: {e:#}");
                    }
                    o.client.on_connected();
                    self.status = self.lang.f("menu.connected", &[("server", &o.conn.server)]);
                }
                ClientEvent::Message { data, .. } => o.client.on_message(&data, at),
                ClientEvent::Disconnected(DisconnectReason::KeyMismatch { expected, got }) => {
                    self.net.key_warning = Some(KeyWarning {
                        server: o.conn.server,
                        expected: elora_net::hex(&expected),
                        got: elora_net::hex(&got),
                    });
                    ended = Some(self.lang.t("reason.key_changed").to_owned());
                }
                ClientEvent::Disconnected(reason) => {
                    ended = Some(match reason {
                        DisconnectReason::Timeout => self.lang.t("reason.timeout").to_owned(),
                        DisconnectReason::Remote(r) | DisconnectReason::Rejected(r) => {
                            self.lang.reason(&r)
                        }
                        other => format!("{other:?}"),
                    });
                }
            }
        }
        if let Status::Disconnected(reason) = &o.client.status {
            ended = ended.or_else(|| Some(self.lang.reason(reason)));
        }
        if let Some(reason) = ended {
            self.status = self.lang.f("menu.disconnected", &[("reason", &reason)]);
            self.online = None;
            return;
        }
        let controls = &mut self.controls;
        o.client.update(now, || controls.player_input());
        for (data, reliable) in o.client.take_outgoing() {
            o.conn.send(data, reliable);
        }
    }
}
