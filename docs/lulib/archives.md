# Archive

An archive manager for zip and tar.

## Methods

- **`archive.zip.create(output_path, files_table)`**: Creates a `.zip` archive.
- **`archive.zip.extract(archive_path, destination_path)`**: Extracts a `.zip` archive.
- **`archive.tar.create(output_path, files_table)`**: Creates a `.tar.gz` archive.
- **`archive.tar.extract(archive_path, destination_path)`**: Extracts a `.tar.gz` archive.


```lua
using { lulib.archive }

archive.zip.create("./some.zip", { "./file.txt", "./file2.txt" })
```