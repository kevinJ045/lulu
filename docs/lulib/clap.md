# Clap

A binding for a rust cli-parser(clap crate) for lulu.

## Methods

- **`clap.(Command/Subcommand)({ name, version, about })`**: Creates a new command.
    ```lua
    using { lulib.clap }
    
    local cmd = clap.Command({
      name = "myapp",
      version = "1.0",
      about = "Example CLI"
    })
    ```
    -   `:arg(name, opts)` – Adds a positional or named argument.

        `opts` is an optional table with the following keys:
        
        `short` (`string`) – Short option, e.g., "b" for -b.

        `long` (`string`) – Long option, e.g., "build" for --build.

        `help` (`string`) – Help text for the argument.

        `default` (`string`) – Default value if not provided.

        `required` (`boolean`) – Whether the argument is required.

        `trailing` (`boolean`) – Whether this argument collects all remaining inputs. (if you use this, your arg name should end with "*")

        ```lua
        cmd:arg("file", { default = ".", help = "Input file" })
        ```
    -   `:flag(name, opts)` – Adds a boolean flag.

        `opts` opts can include short, long, and help. Flags are either true (present) or false (absent).

        ```lua
        cmd:flag("build", { short = "b", long = "build", help = "Build project" })
        ```
    -   `:subcommand(subcommand)` – Adds a subcommand. Subcommands are themselves Command objects.

        ```lua
        local run = clap.Subcommand({ name = "run", about = "Run the project" })
        run:arg("file", { help = "File to run" })
        cmd:subcommand(run)
        ```
    -   `:parse(args)` – Parses a list of arguments.
        
        Returns a Lua table containing the parsed arguments and subcommands. Arguments that allow multiple values are returned as arrays.

        ```lua
        local result = cmd:parse({ "file.txt", "-b" })
        print(result.file)   -- "file.txt"
        print(result.build)  -- true
        ```