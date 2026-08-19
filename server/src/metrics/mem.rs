use super::counter::Counter;
use super::date_counter::DateCounter;
use super::hour_counter::HourCounter;
use super::traits::ServerMetrics;
use super::{RESERVE_DAYS, RESERVE_HOURS};
use chrono::{Duration, Local, NaiveDate, Timelike};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrafficWindow {
    Days7,
    Hours24,
}

impl TrafficWindow {
    pub fn parse(raw: &str) -> Self {
        match raw.trim().to_ascii_lowercase().as_str() {
            "24h" | "hour" | "hours" | "1d" => Self::Hours24,
            _ => Self::Days7,
        }
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ServerSnapshot {
    pub total_traffic_in: u64,
    pub total_traffic_out: u64,
    pub cur_conns: usize,
    pub client_counts: usize,
    pub total_client_counts: usize,
    pub proxy_type_counts: HashMap<String, usize>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ProxySnapshot {
    pub name: String,
    pub proxy_type: String,
    pub user: String,
    pub client_id: String,
    pub today_traffic_in: u64,
    pub today_traffic_out: u64,
    pub cur_conns: usize,
    pub last_start_at: Option<i64>,
    pub last_close_at: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct TrafficPoint {
    pub date: String,
    pub traffic_in: u64,
    pub traffic_out: u64,
}

#[derive(Debug, Clone)]
pub struct ProxyTrafficHistory {
    pub name: String,
    pub unit: &'static str,
    pub granularity: &'static str,
    pub history: Vec<TrafficPoint>,
}

#[derive(Debug, Clone)]
pub struct TokenConnSnapshot {
    pub token: String,
    pub active_conns: usize,
}

struct ProxyStats {
    proxy_type: String,
    user: String,
    client_id: String,
    traffic_in: DateCounter,
    traffic_out: DateCounter,
    traffic_in_hourly: HourCounter,
    traffic_out_hourly: HourCounter,
    cur_conns: Counter,
    last_start_unix: Option<i64>,
    last_close_unix: Option<i64>,
}

impl ProxyStats {
    fn new(proxy_type: &str, user: &str, client_id: &str) -> Self {
        Self {
            proxy_type: proxy_type.to_string(),
            user: user.to_string(),
            client_id: client_id.to_string(),
            traffic_in: DateCounter::new(RESERVE_DAYS),
            traffic_out: DateCounter::new(RESERVE_DAYS),
            traffic_in_hourly: HourCounter::new(RESERVE_HOURS),
            traffic_out_hourly: HourCounter::new(RESERVE_HOURS),
            cur_conns: Counter::new(),
            last_start_unix: None,
            last_close_unix: None,
        }
    }
}

struct State {
    total_traffic_in: DateCounter,
    total_traffic_out: DateCounter,
    total_traffic_in_hourly: HourCounter,
    total_traffic_out_hourly: HourCounter,
    cur_conns: Counter,
    client_counts: Counter,
    token_conns: HashMap<String, Counter>,
    seen_clients: HashSet<String>,
    proxy_type_counts: HashMap<String, Counter>,
    proxies: HashMap<String, ProxyStats>,
}

pub struct MemMetrics {
    state: Mutex<State>,
}

impl MemMetrics {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            state: Mutex::new(State {
                total_traffic_in: DateCounter::new(RESERVE_DAYS),
                total_traffic_out: DateCounter::new(RESERVE_DAYS),
                total_traffic_in_hourly: HourCounter::new(RESERVE_HOURS),
                total_traffic_out_hourly: HourCounter::new(RESERVE_HOURS),
                cur_conns: Counter::new(),
                client_counts: Counter::new(),
                token_conns: HashMap::new(),
                seen_clients: HashSet::new(),
                proxy_type_counts: HashMap::new(),
                proxies: HashMap::new(),
            }),
        })
    }

    pub fn server_snapshot(&self) -> ServerSnapshot {
        let g = self.state.lock().expect("metrics lock");
        let mut proxy_type_counts = HashMap::new();
        for (k, v) in &g.proxy_type_counts {
            let n = v.count().max(0) as usize;
            if n > 0 {
                proxy_type_counts.insert(k.clone(), n);
            }
        }
        ServerSnapshot {
            total_traffic_in: g.total_traffic_in.today_count().max(0) as u64,
            total_traffic_out: g.total_traffic_out.today_count().max(0) as u64,
            cur_conns: g.cur_conns.count().max(0) as usize,
            client_counts: g.client_counts.count().max(0) as usize,
            total_client_counts: g.seen_clients.len(),
            proxy_type_counts,
        }
    }

    pub fn proxy_snapshot(&self, name: &str) -> Option<ProxySnapshot> {
        let g = self.state.lock().expect("metrics lock");
        g.proxies.get(name).map(|p| to_proxy_snapshot(name, p))
    }

    pub fn proxy_traffic(&self, name: &str, window: TrafficWindow) -> Option<ProxyTrafficHistory> {
        let g = self.state.lock().expect("metrics lock");
        let p = g.proxies.get(name)?;
        Some(match window {
            TrafficWindow::Days7 => {
                let inbound = p.traffic_in.last_days(RESERVE_DAYS);
                let outbound = p.traffic_out.last_days(RESERVE_DAYS);
                build_daily_history(name, &inbound, &outbound)
            }
            TrafficWindow::Hours24 => {
                let inbound = p.traffic_in_hourly.last_hours(RESERVE_HOURS);
                let outbound = p.traffic_out_hourly.last_hours(RESERVE_HOURS);
                build_hourly_history(name, &inbound, &outbound)
            }
        })
    }

    pub fn server_traffic(&self, window: TrafficWindow) -> ProxyTrafficHistory {
        let g = self.state.lock().expect("metrics lock");
        match window {
            TrafficWindow::Days7 => {
                let inbound = g.total_traffic_in.last_days(RESERVE_DAYS);
                let outbound = g.total_traffic_out.last_days(RESERVE_DAYS);
                build_daily_history("server", &inbound, &outbound)
            }
            TrafficWindow::Hours24 => {
                let inbound = g.total_traffic_in_hourly.last_hours(RESERVE_HOURS);
                let outbound = g.total_traffic_out_hourly.last_hours(RESERVE_HOURS);
                build_hourly_history("server", &inbound, &outbound)
            }
        }
    }

    pub fn inc_token_conn(&self, token: &str) {
        let token = token.trim();
        if token.is_empty() {
            return;
        }
        let mut g = self.state.lock().expect("metrics lock");
        g.token_conns
            .entry(token.to_string())
            .or_insert_with(Counter::new)
            .inc(1);
    }

    pub fn dec_token_conn(&self, token: &str) {
        let token = token.trim();
        if token.is_empty() {
            return;
        }
        let mut g = self.state.lock().expect("metrics lock");
        if let Some(counter) = g.token_conns.get(token) {
            counter.dec(1);
            if counter.count() <= 0 {
                g.token_conns.remove(token);
            }
        }
    }

    pub fn token_conn_snapshot(&self) -> Vec<TokenConnSnapshot> {
        let g = self.state.lock().expect("metrics lock");
        let mut items: Vec<TokenConnSnapshot> = g
            .token_conns
            .iter()
            .filter_map(|(token, counter)| {
                let n = counter.count().max(0) as usize;
                (n > 0).then(|| TokenConnSnapshot {
                    token: token.clone(),
                    active_conns: n,
                })
            })
            .collect();
        items.sort_by(|a, b| {
            b.active_conns
                .cmp(&a.active_conns)
                .then_with(|| a.token.cmp(&b.token))
        });
        items
    }

    pub fn track_connection(
        self: &Arc<Self>,
        name: impl Into<String>,
        proxy_type: impl Into<String>,
    ) -> ConnGuard {
        let name = name.into();
        let proxy_type = proxy_type.into();
        self.open_connection(&name, &proxy_type);
        ConnGuard {
            metrics: Arc::clone(self),
            name,
            proxy_type,
        }
    }
}

pub struct ConnGuard {
    metrics: Arc<MemMetrics>,
    name: String,
    proxy_type: String,
}

impl Drop for ConnGuard {
    fn drop(&mut self) {
        self.metrics.close_connection(&self.name, &self.proxy_type);
    }
}

impl ServerMetrics for MemMetrics {
    fn new_client(&self, run_id: &str) {
        let mut g = self.state.lock().expect("metrics lock");
        if !run_id.is_empty() {
            g.seen_clients.insert(run_id.to_string());
            const MAX_SEEN: usize = 256;
            if g.seen_clients.len() > MAX_SEEN {
                let overflow = g.seen_clients.len() - MAX_SEEN;
                let drop_keys: Vec<String> =
                    g.seen_clients.iter().take(overflow).cloned().collect();
                for k in drop_keys {
                    g.seen_clients.remove(&k);
                }
            }
        }
        g.client_counts.inc(1);
    }

    fn close_client(&self) {
        self.state
            .lock()
            .expect("metrics lock")
            .client_counts
            .dec(1);
    }

    fn new_proxy(&self, name: &str, proxy_type: &str, user: &str, client_id: &str) {
        let mut g = self.state.lock().expect("metrics lock");
        g.proxy_type_counts
            .entry(proxy_type.to_string())
            .or_insert_with(Counter::new)
            .inc(1);

        let now_unix = Local::now().timestamp();
        let entry = g
            .proxies
            .entry(name.to_string())
            .or_insert_with(|| ProxyStats::new(proxy_type, user, client_id));

        if entry.proxy_type != proxy_type {
            *entry = ProxyStats::new(proxy_type, user, client_id);
        } else {
            entry.user = user.to_string();
            entry.client_id = client_id.to_string();
        }
        entry.last_start_unix = Some(now_unix);
    }

    fn close_proxy(&self, name: &str, proxy_type: &str) {
        let mut g = self.state.lock().expect("metrics lock");
        if let Some(counter) = g.proxy_type_counts.get(proxy_type) {
            counter.dec(1);
        }
        if let Some(entry) = g.proxies.get_mut(name) {
            entry.last_close_unix = Some(Local::now().timestamp());
        }
    }

    fn open_connection(&self, name: &str, _proxy_type: &str) {
        let mut g = self.state.lock().expect("metrics lock");
        g.cur_conns.inc(1);
        if let Some(p) = g.proxies.get_mut(name) {
            p.cur_conns.inc(1);
        }
    }

    fn close_connection(&self, name: &str, _proxy_type: &str) {
        let mut g = self.state.lock().expect("metrics lock");
        g.cur_conns.dec(1);
        if let Some(p) = g.proxies.get_mut(name) {
            p.cur_conns.dec(1);
        }
    }

    fn add_traffic_in(&self, name: &str, _proxy_type: &str, bytes: u64) {
        if bytes == 0 {
            return;
        }
        let delta = bytes as i64;
        let mut g = self.state.lock().expect("metrics lock");
        g.total_traffic_in.inc(delta);
        g.total_traffic_in_hourly.inc(delta);
        if let Some(p) = g.proxies.get_mut(name) {
            p.traffic_in.inc(delta);
            p.traffic_in_hourly.inc(delta);
        }
    }

    fn add_traffic_out(&self, name: &str, _proxy_type: &str, bytes: u64) {
        if bytes == 0 {
            return;
        }
        let delta = bytes as i64;
        let mut g = self.state.lock().expect("metrics lock");
        g.total_traffic_out.inc(delta);
        g.total_traffic_out_hourly.inc(delta);
        if let Some(p) = g.proxies.get_mut(name) {
            p.traffic_out.inc(delta);
            p.traffic_out_hourly.inc(delta);
        }
    }
}

fn to_proxy_snapshot(name: &str, p: &ProxyStats) -> ProxySnapshot {
    ProxySnapshot {
        name: name.to_string(),
        proxy_type: p.proxy_type.clone(),
        user: p.user.clone(),
        client_id: p.client_id.clone(),
        today_traffic_in: p.traffic_in.today_count().max(0) as u64,
        today_traffic_out: p.traffic_out.today_count().max(0) as u64,
        cur_conns: p.cur_conns.count().max(0) as usize,
        last_start_at: p.last_start_unix,
        last_close_at: p.last_close_unix,
    }
}

fn build_daily_history(name: &str, inbound: &[i64], outbound: &[i64]) -> ProxyTrafficHistory {
    let today = Local::now().date_naive();
    let n = RESERVE_DAYS.min(inbound.len()).min(outbound.len());
    let mut history = Vec::with_capacity(n);
    for age in (0..n).rev() {
        let date = today
            .checked_sub_signed(Duration::days(age as i64))
            .unwrap_or(NaiveDate::from_ymd_opt(1970, 1, 1).unwrap());
        history.push(TrafficPoint {
            date: date.format("%Y-%m-%d").to_string(),
            traffic_in: inbound[age].max(0) as u64,
            traffic_out: outbound[age].max(0) as u64,
        });
    }
    ProxyTrafficHistory {
        name: name.to_string(),
        unit: "bytes",
        granularity: "day",
        history,
    }
}

fn build_hourly_history(name: &str, inbound: &[i64], outbound: &[i64]) -> ProxyTrafficHistory {
    let now_hour = Local::now()
        .with_minute(0)
        .and_then(|t| t.with_second(0))
        .and_then(|t| t.with_nanosecond(0))
        .unwrap_or_else(Local::now);
    let n = RESERVE_HOURS.min(inbound.len()).min(outbound.len());
    let mut history = Vec::with_capacity(n);
    for age in (0..n).rev() {
        let ts = now_hour - Duration::hours(age as i64);
        history.push(TrafficPoint {
            date: ts.format("%Y-%m-%dT%H:00").to_string(),
            traffic_in: inbound[age].max(0) as u64,
            traffic_out: outbound[age].max(0) as u64,
        });
    }
    ProxyTrafficHistory {
        name: name.to_string(),
        unit: "bytes",
        granularity: "hour",
        history,
    }
}
