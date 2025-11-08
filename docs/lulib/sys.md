# Sys

Information about the current system.

## Methods

- **`sys.battery()`**: Returns battery information.
    - **`state`**: (`Charing`, `Discharging`, `Unknown`) The battery state.
    - **`percentage`**: (`f64`) The battery percentage
    - **`energy`**: (`f64`) Amount of energy currently available in the battery
    - **`energy_full`**: (`f64`) Amount of energy in the battery when it's considered full.
    - **`energy_full_design`**: (`f64`) Amount of energy the battery is designed to hold when it's considered full.
    - **`energy_rate`**: (`f64`) Amount of energy being drained from the battery.
    - **`voltage`**: (`f64`) Battery voltage.
    - **`temperature`**: (`f64`) Battery temperature.
    - **`cycle_count`**: (`f64`) Number of charge/discharge cycles.
    - **`time_to_full`**: (`f64`) Remaining time till full battery.
    - **`time_to_empty`**: (`f64`) Remaining time till empty battery.

- **`sys.processes()`**: Returns a table of processes.
    - **`memory`**: (`u64`) Memory usage of a process.
    - **`cpu_usage`**: (`f32`) CPU usage of a process.
    - **`cmd`**: (`string`) Command of process.
    - **`namd`**: (`string`) Name of process.
    - **`exe`**: (`string`) Executable path of process.
    - **`pid`**: (`u64`) Process ID.
    - **`kill()`**: Kills the process.
    - **`exists()`**: Checks if the process exists.
    - **`kill(signal)`**: Kills the process with signal.
      - `Hangup`, `Interrupt`, `Quit`, `Illegal`, `Trap`, `Abort`, `IOT`, `Bus`, `FloatingPointException`, `Kill`, `User1`, `Segv`, `User2`, `Pipe`, `Alarm`, `Term`, `Child`, `Continue`, `Stop`, `TSTP`, `TTIN`, `TTOU`, `Urgent`, `XCPU`, `XFSZ`, `VirtualAlarm`, `Profiling`, `Winch`, `IO`, `Poll`, `Power`, `Sys`

- **`sys.process(pid)`**: Returns a process by pid.

- **`sys.cpus()`**: Returns a table of CPU cores.
    - **`name`**: (`string`) CPU name.
    - **`usage`**: (`f32`) CPU usage.
    - **`vendor_id`**: (`string`) CPU vendor.
    - **`brand`**: (`string`) CPU brand.
    - **`frequency`**: (`u64`) CPU frequency.

- **`sys.global_cpu_usage()`**: (`f32`) Global cpu usage.
- **`sys.total_memory()`**: (`usize`) Total memory size in bytes.
- **`sys.free_memory()`**: (`usize`) Free momory size in bytes.
- **`sys.available_memory()`**: (`usize`) Available memory size in bytes.
- **`sys.used_memory()`**: (`usize`) Used memory size in bytes.
- **`sys.total_swap()`**: (`usize`) Total swap size in bytes.
- **`sys.free_swap()`**: (`usize`) Free swap size in bytes.
- **`sys.used_swap()`**: (`usize`) Used swap size in bytes.
- **`sys.uptime()`**: (`u64`) System uptime in seconds.
- **`sys.boot_time()`**: (`u64`) System boot time.
- **`sys.load_average()`**: (`f64`, `f64`, `f64`) System load.
- **`sys.name()`**: (`string`) System name.
- **`sys.kernel_version()`**: (`string`) System kernel version.
- **`sys.os_version()`**: (`string`) System os version.
- **`sys.long_os_version()`**: (`string`) System os version long.
- **`sys.distribution_id()`**: (`string`) System distro id.
- **`sys.distribution_id_like()`**: (`string`) System distro id.
- **`sys.kernel_long_version()`**: (`string`) System kernel version long.
- **`sys.host_name()`**: (`string`) System hostname.
- **`sys.cpu_arch()`**: (`string`) CPU architecture.
- **`sys.physical_core_count()`**: (`usize`) Physical cores.

```lua
using { lulib.sys }

print(sys.total_memory() / 1024 / 1024)
```