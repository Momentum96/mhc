//! Read-only cgroup reporting from senpi `host-unit-placement.ts`.
use serde::Serialize;

#[derive(Serialize)]
pub(crate) struct HostUnit {
    pub cgroup: String,
    pub name: Option<String>,
    pub own: bool,
}

pub(crate) fn read_host_unit(pid: u32, instance_id: Option<&str>) -> Option<HostUnit> {
    if !cfg!(target_os = "linux") {
        return None;
    }
    let table = std::fs::read_to_string(format!("/proc/{pid}/cgroup")).ok()?;
    host_unit_from_cgroup_table(&table, instance_id)
}

fn host_unit_from_cgroup_table(table: &str, instance_id: Option<&str>) -> Option<HostUnit> {
    let entry = table
        .lines()
        .find(|line| line.starts_with("0::"))
        .or_else(|| table.lines().find(|line| line.contains(":name=systemd:")))?;
    let cgroup = entry.splitn(3, ':').nth(2)?;
    let leaf = cgroup.rsplit('/').next()?;
    let name = (leaf.ends_with(".service") || leaf.ends_with(".scope")).then_some(leaf);
    let own = name.zip(instance_id).is_some_and(|(name, instance_id)| {
        name.strip_prefix("senpi-host-")
            .and_then(|name| name.strip_suffix(".scope"))
            == Some(instance_id)
    });
    Some(HostUnit {
        cgroup: cgroup.into(),
        name: name.map(str::to_owned),
        own,
    })
}
