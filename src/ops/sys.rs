use crate::ops::std::create_std_module;


pub fn into_module(){

  create_std_module("sys")
    .add_function("battery", |lua, _: ()| {
      use battery::Manager;

      let manager = Manager::new().map_err(mlua::Error::external)?;
      let mut batteries = manager.batteries().map_err(mlua::Error::external)?;
      if let Some(Ok(bat)) = batteries.next() {
        let tbl = lua.create_table()?;

        tbl.set("state", format!("{:?}", bat.state()))?;
        tbl.set("percentage", bat.state_of_charge().value * 100.0)?;
        tbl.set("energy", bat.energy().value)?;
        tbl.set("energy_full", bat.energy_full().value)?;
        tbl.set("energy_full_design", bat.energy_full_design().value)?;
        tbl.set("energy_rate", bat.energy_rate().value)?;
        tbl.set("voltage", bat.voltage().value)?;
        tbl.set("temperature", bat.temperature().map(|t| t.value))?;
        tbl.set("cycle_count", bat.cycle_count())?;
        tbl.set("time_to_full", bat.time_to_full().map(|t| t.value))?;
        tbl.set("time_to_empty", bat.time_to_empty().map(|t| t.value))?;

        Ok(Some(tbl))
      } else {
        Ok(None)
      }
    })
    .on_register(|lua, sys| {
      let get_proc = |lua: &mlua::Lua, (pid, proc): (&sysinfo::Pid, &sysinfo::Process)| {
        let pid_val = pid.as_u32();
        let name = proc.name();
        let exe = proc.exe();
        let cmd = proc.cmd().to_vec();
        let memory = proc.memory();
        let cpu_usage = proc.cpu_usage();

        let process = lua.create_table().unwrap();

        process.set(
          "kill",
          lua.create_function(move |_, ()| {
            let sys = sysinfo::System::new_all();
            if let Some(proc) = sys.process(sysinfo::Pid::from_u32(pid_val)) {
              proc.kill();
            }
            Ok(())
          })?,
        )?;

        process.set(
          "kill_with",
          lua.create_function(move |_, signal: String| {
            let sys = sysinfo::System::new_all();
            if let Some(proc) = sys.process(sysinfo::Pid::from_u32(pid_val)) {
              use sysinfo::Signal::*;
              let sig = match signal.as_str() {
                "Hangup" => Hangup,
                "Interrupt" => Interrupt,
                "Quit" => Quit,
                "Illegal" => Illegal,
                "Trap" => Trap,
                "Abort" => Abort,
                "IOT" => IOT,
                "Bus" => Bus,
                "FloatingPointException" => FloatingPointException,
                "Kill" => Kill,
                "User1" => User1,
                "Segv" => Segv,
                "User2" => User2,
                "Pipe" => Pipe,
                "Alarm" => Alarm,
                "Term" => Term,
                "Child" => Child,
                "Continue" => Continue,
                "Stop" => Stop,
                "TSTP" => TSTP,
                "TTIN" => TTIN,
                "TTOU" => TTOU,
                "Urgent" => Urgent,
                "XCPU" => XCPU,
                "XFSZ" => XFSZ,
                "VirtualAlarm" => VirtualAlarm,
                "Profiling" => Profiling,
                "Winch" => Winch,
                "IO" => IO,
                "Poll" => Poll,
                "Power" => Power,
                "Sys" => Sys,
                _ => Kill,
              };
              proc.kill_with(sig);
            }
            Ok(())
          })?,
        )?;

        process.set(
          "exists",
          lua.create_function(move |_, ()| {
            let sys = sysinfo::System::new_all();
            Ok(sys.process(sysinfo::Pid::from_u32(pid_val)).is_some())
          })?,
        )?;

        process.set("pid", pid_val)?;
        process.set("name", name)?;
        process.set("exe", exe)?;
        process.set("cmd", cmd)?;
        process.set("memory", memory)?;
        process.set("cpu_usage", cpu_usage)?;

        Ok::<mlua::Table, mlua::Error>(process)
      };

      sys.set(
        "cpus",
        lua.create_function(move |lua, ()| {
          Ok(
            sysinfo::System::new_all()
              .cpus()
              .into_iter()
              .map(|c| {
                let cpu = lua.create_table().unwrap();

                cpu.set("name", c.name()).unwrap();
                cpu.set("usage", c.cpu_usage()).unwrap();
                cpu.set("vendor_id", c.vendor_id()).unwrap();
                cpu.set("brand", c.brand()).unwrap();
                cpu.set("frequency", c.frequency()).unwrap();

                cpu
              })
              .collect::<Vec<_>>(),
          )
        })?,
      )?;

      sys.set(
        "processes",
        lua.create_function(move |lua, ()| {
          let procs = lua.create_table().unwrap();
          sysinfo::System::new_all()
            .processes()
            .into_iter()
            .for_each(|(pid, proc)| {
              let process = get_proc(lua, (pid, proc)).unwrap();
              procs.set(pid.as_u32(), process).unwrap();
            });
          Ok(procs)
        })?,
      )?;

      sys.set(
        "process",
        lua.create_function(move |lua, pid: usize| {
          let sys = sysinfo::System::new_all();
          let pid = sysinfo::Pid::from(pid);
          let proc = sys.process(pid).unwrap();
          Ok(get_proc(lua, (&pid, proc))?)
        })?,
      )?;

      sys.set(
        "global_cpu_usage",
        lua.create_function(move |_, ()| Ok(sysinfo::System::new_all().global_cpu_usage()))?,
      )?;

      sys.set(
        "total_memory",
        lua.create_function(move |_, ()| Ok(sysinfo::System::new_all().total_memory()))?,
      )?;
      sys.set(
        "free_memory",
        lua.create_function(move |_, ()| Ok(sysinfo::System::new_all().free_memory()))?,
      )?;
      sys.set(
        "available_memory",
        lua.create_function(move |_, ()| Ok(sysinfo::System::new_all().available_memory()))?,
      )?;
      sys.set(
        "used_memory",
        lua.create_function(move |_, ()| Ok(sysinfo::System::new_all().used_memory()))?,
      )?;
      sys.set(
        "total_swap",
        lua.create_function(move |_, ()| Ok(sysinfo::System::new_all().total_swap()))?,
      )?;
      sys.set(
        "free_swap",
        lua.create_function(move |_, ()| Ok(sysinfo::System::new_all().free_swap()))?,
      )?;
      sys.set(
        "used_swap",
        lua.create_function(move |_, ()| Ok(sysinfo::System::new_all().used_swap()))?,
      )?;

      sys.set(
        "uptime",
        lua.create_function(move |_, ()| Ok(sysinfo::System::uptime()))?,
      )?;
      sys.set(
        "boot_time",
        lua.create_function(move |_, ()| Ok(sysinfo::System::boot_time()))?,
      )?;
      sys.set(
        "load_average",
        lua.create_function(move |_, ()| {
          let lavg = sysinfo::System::load_average();
          Ok(vec![lavg.one, lavg.five, lavg.fifteen])
        })?,
      )?;
      sys.set(
        "name",
        lua.create_function(move |_, ()| Ok(sysinfo::System::name()))?,
      )?;
      sys.set(
        "kernel_version",
        lua.create_function(move |_, ()| Ok(sysinfo::System::kernel_version()))?,
      )?;
      sys.set(
        "os_version",
        lua.create_function(move |_, ()| Ok(sysinfo::System::os_version()))?,
      )?;
      sys.set(
        "long_os_version",
        lua.create_function(move |_, ()| Ok(sysinfo::System::long_os_version()))?,
      )?;
      sys.set(
        "distribution_id",
        lua.create_function(move |_, ()| Ok(sysinfo::System::distribution_id()))?,
      )?;
      sys.set(
        "distribution_id_like",
        lua.create_function(move |_, ()| Ok(sysinfo::System::distribution_id_like()))?,
      )?;
      sys.set(
        "kernel_long_version",
        lua.create_function(move |_, ()| Ok(sysinfo::System::kernel_long_version()))?,
      )?;
      sys.set(
        "host_name",
        lua.create_function(move |_, ()| Ok(sysinfo::System::host_name()))?,
      )?;
      sys.set(
        "cpu_arch",
        lua.create_function(move |_, ()| Ok(sysinfo::System::cpu_arch()))?,
      )?;
      sys.set(
        "physical_core_count",
        lua.create_function(move |_, ()| Ok(sysinfo::System::physical_core_count()))?,
      )?;

      Ok(sys)
    })
    .into();
}