# Threads

A simple thread manager for lulu. Not complete multithreading support but somewhat useful for non-blocking tasks.

## Methods

- **`threads.spawn`** (`(fn) -> ThreadHandle`): Spawn a new thread with your function
- **`threads.join`** (`ThreadHandle`): Joins threads to the main thread
- **`threads.sleep`** (`number`): Sleeps the current thread

```lua
using { lulib.threads }

local t = threads.spawn(function()
  threads.sleep(10)
  print('Done')
end)

threads.join(t)
```