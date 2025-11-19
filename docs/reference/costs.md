# Lulu runtime costs

Here, I laid most lulu concepts, each with their costs.

## Lulu Core Components
| Name            | Memory Cost (MB) | Performance Cost | Runs On           |
| --------------- | ---------------: | ---------------- | ----------------- |
| Compiler        |             ~2.5 | High             | Compile time only |
| Runtime VM      |             ~2.4 | High             | Always            |
| Package Manager |             ~1.0 | Low              | When called       |
| Bundler         |             ~0.2 | Low              | When called       |


## Modules
| Name      | Memory Cost (MB) | Performance Cost | Availability     |
| --------- | ---------------: | ---------------- | ---------------- |
| `std`     |             ~1.0 | Low              | Always available |
| `net`     |             ~4.0 | Medium           | Imported         |
| `sys`     |             ~2.0 | Medium           | Imported         |
| `serde`   |             ~1.0 | Low              | Imported         |
| `threads` |             ~0.4 | Low              | Imported         |
| `clap`    |             ~0.5 | Low              | Imported         |
| `kvdb`    |             ~2.0 | Medium           | Imported         |
|`archives` |             ~2.0 | Low              | Imported         |
| `tui`     |             ~2.0 | Medium           | Imported         |

## Compile time/Runtime Concepts
| Name               | Compilation Cost | Runtime Cost |
| ------------------ | ---------------: | -----------: |
| Macros (most)      |             High |         None |
| `cfg!` macro       |           Medium |         None |
| `import!` macro    |              Low |          Low |
| Classes            |             High |       Medium |
| Enums              |             High |          Low |
| Class Shorthands   |          Extreme |       Medium |
| Enum Shorthands    |          Extreme |       Medium |
| `=>` Shorthands    |             High |       Medium |
| Decorators         |             High |         High |
| Traits             |           Medium |         High |
| `+=`, `-=`, ...    |              Low |         None |
| `in local ...`     |             High |         High |
| `in do/if ...`     |             High |         High |
| `match!` macro     |             High |         High |
| `f""` formatter    |              Low |         None |
| `ptr` shorthands   |              Low |         None |
