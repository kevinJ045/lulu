# The Lua Runtime

Lulu provides a set of global functions and modules available to all your `.lua` scripts when they are run or bundled with Lulu. This "standard library" makes it easy to perform common tasks without needing external dependencies.

## Process Management

- **`argv`**: A global table (array) containing the command-line arguments passed to your script.
- **`exit(code)`**: Exits the program with the given status code (e.g., `exit(0)` for success, `exit(1)` for failure).
- **`exec(command, inherit_stdio)`**: Executes a shell command.
  - `command` (string): The command to run.
  - `inherit_stdio` (boolean): If `true`, the command will share the parent's stdin/stdout/stderr.
  - Returns the output of the command as a string.
- **`spawn(command, inherit_stdio)`**: Spawns a shell command.
  - `command` (string): The command to run.
  - `inherit_stdio` (boolean): If `true`, the command will share the parent's stdin/stdout/stderr.
  - Returns a IO buffer.

## Filesystem (FS)

- **`reads(path)`**: Reads the entire content of a file into a string.
- **`read(path)`**: Reads the entire content of a file into a [ByteArray](./helper-classes.md#bytearray).
- **`exists(path)`**: Returns `true` if a file or directory exists at the path.
- **`mkdir(...)`**: Recursively creates a directory.
- **`cp(source, destination)`**: Copies a single file.
- **`mv(source, destination)`**: Moves a file.
- **`rm(path)`**: Removes a file or directory.

## Regular Expressions (`re`)

A global module for regular expressions.

- **`re.exec(pattern, text)`**: Returns `true` if the pattern matches the text, `false` otherwise.
- **`re.match(pattern, text)`**: Returns a table of capture groups from the first match.
- **`re.replace(pattern, text, replacement)`**: Replaces matches in the text. The `replacement` can be a string (e.g., `"$0 $1"`) or a function that receives capture groups as arguments.

## Cryptography & Utilities

- **`crypto.sha256(string)`**: Computes the SHA-256 hash of a string.
- **`uuid.v4()`**: Generates a version 4 UUID.
- **`rand.from(min, max)`**: Returns a random integer between `min` and `max` (inclusive).
- **`range(start, finish)`**: Returns a table containing a sequence of numbers from `start` to `finish`.
- **`fprint(table)`**: A formatted print function for tables, useful for debugging.

## Module Environment

Within each module, Lulu also provides a few special variables:

- **`mod`**: A table containing information about the current module, including `mod.name` and `mod.conf`.
- **`current_path`**: The path of the currently executing script.
- **`lookup_dylib(name)`**: Finds a dynamic library in the project's `.lib` directory or the system path.

## Lulu cache manager

- **`setup_downloader(options)`**: Sets up a package manager instance to let you download things.
  - **`format`**: (`^D ^N ^P ^C kb / ^T kb`) The format of the downloader.
  - **`download_text`**: (`Downloading`) The prefix of the downloader.
  - **`progressbar_size`**: (`10`) The size of the downloader progressbar.
  - **`progressbar_colors`**: (`{r,g,b}, {r,g,b}`) Two colors for the progressbar gradient.
- **`async download_file(url)`**: Downloads the file into a cache and gives you the `cache` folder. If cached, won't download.
- **`async download_uncached(url)`**: Same as `download_file` but will download regardless of being in cache.
- **`require_cached(url)`**: Dynamically include a lulib from url instead of adding it [`dependencies`](/reference/configuration.md#dependencies).
  
