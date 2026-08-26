pub trait ServerMetrics: Send + Sync {
    fn new_client(&self, session_id: &str);
    fn close_client(&self);

    fn new_tunnel(&self, name: &str, tunnel_type: &str, user: &str, session_id: &str);
    fn close_tunnel(&self, name: &str, tunnel_type: &str);

    fn open_connection(&self, name: &str, tunnel_type: &str);
    fn close_connection(&self, name: &str, tunnel_type: &str);

    fn add_traffic_in(&self, name: &str, tunnel_type: &str, bytes: u64);
    fn add_traffic_out(&self, name: &str, tunnel_type: &str, bytes: u64);
}
