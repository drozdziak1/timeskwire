use chrono::Duration;

pub fn format_hms(d: &Duration) -> String {
    let mut tmp = d.clone();

    let h = tmp.num_hours();
    tmp = tmp - Duration::hours(h);
    let m = tmp.num_minutes();
    tmp = tmp - Duration::minutes(m);
    let s = tmp.num_seconds();
    format!("{}:{:02}:{:02}", h, m, s)
}
