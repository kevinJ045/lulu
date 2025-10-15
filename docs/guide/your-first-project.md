# Your First Project

Lulu treats your code as a project, centered around a config file. This makes managing your app's structure and dependencies somewhat clear and simple.

## Creating a New Project

The easiest way to start a new project is with the `lulu new` command.

```bash
lulu new -gi my-project
```

This will create a new directory named `my-project` with the following structure:

```
my-project/
├── .gitignore
├── lulu.conf.lua
└── main.lua
```

Let's look at the generated files:

- **`main.lua`**: This is the main entry point for your application. It starts with a simple `print("Hello, World!")`.
- **`lulu.conf.lua`**: This is the heart of your project, the project manifest. It tells Lulu everything it needs to know about your application.

## The Project Manifest

The generated `lulu.conf.lua` will look like this:

```lua
-- lulu.conf.lua

manifest = {
  name = "my-project",
  version = "1.0.0"
}

mods = {
  main = "main.lua"
}
```

- The `manifest` table contains metadata about your project.
- The `mods` table tells Lulu which Lua files are part of your project. Here, we've defined a single module named `main` that points to `main.lua`.

We will explore the manifest file in much more detail in the [Configuration reference page](../reference/configuration.md).

## Running Your Project

To run your new project, navigate into the directory and use the `lulu run` command:

```bash
cd my-project
lulu run main.lua
```

You should see `Hello, World!` printed to your console.

You're done here, you now have a project to start with!
