# Pathing

A binding of the rust `Path` for lulu.

## Methods

- **`pathing.new(string)`**: Creates a new `Path`.
  - **`Path:join(string)`**: Returns a new `Path` with the joined result.
  - **`Path:append(string)`**: Returns the self `Path` but appends the provided string.
  - **`Path:to_string()`**: Returns the path as string.
  - **`Path:exists()`**: Checks if the path exists.
  - **`Path:is_file()`**: Checks if the path is a file.
  - **`Path:is_dir()`**: Checks if the path is a dir.
  - **`Path:filename()`**: Gets the filename.
  - **`Path:extension()`**: Gets the file extension.
  - **`Path:stem()`**: Gets the file stem.
  - **`Path:parent()`**: Gets the file parent dir.
  - **`Path:components()`**: Gets the components of the path.
  - **`Path:list()`**: Gets all files in the path.
  - **`Path:ensure_dir()`**: Creates the folder if does not exist.
  - **`Path:ensure_file(string)`**: Creates the file with provided content if does not exist.
  - **`Path:ensure(string?)`**: If provided a string then it will create a file in the specified path if that path does not exist, otherwise will create a folder.
- **`pathing.root()`**: Returns `Path` of the root of the system.
- **`pathing.appdata()`**: Returns `Path` of the appdata of the system.
- **`pathing.cache()`**: Returns `Path` of the cache path of the system.
- **`pathing.program_files()`**: Returns `Path` of the program files folder of the system.
- **`pathing.temp()`**: Returns `Path` of the temp folder of the system.

## Usage

```lua
using {
  lulib.pathing
}

local path = pathing.appdata()
  :join("my-app")
  :ensure()

print(path)
```