# Testing

Lulu includes a built-in test runner and a special `test!` macro to make testing your code simple and declarative.

## Writing Tests

Tests are defined inside a `test!` macro block. Each test is a named block of code.

```lua
-- tests/my_test.lua

test! {
  -- This is a test block named `addition`
  addition {
    -- The macro automatically catches assertion failures.
    assert(1 + 1 == 2, "1 + 1 should equal 2")
  }

  -- This test will fail
  subtraction {
    assert(2 - 1 == 0, "This assertion is incorrect and will fail")
  }
}
```

### How it Works

- The `test!` macro and its contents are **only compiled** when you run the `lulu test` command. When you run or build your project normally, the entire `test!` block is removed from the code, so it has zero impact on your production bundle.

## Running Tests

You run tests using the `lulu test` command, pointing it at a file containing `test!` blocks.

### Run All Tests in a File

```bash
lulu test tests/my_test.lua
```

Lulu will execute the file and report the results for each test block.

```
Finished test: addition
Test subtraction failed due to: This assertion is incorrect and will fail
```

### Run a Specific Test

You can run a single, specific test block by using the `-t` or `--test` flag.

```bash
lulu test tests/my_test.lua -t addition
```

This is useful for focusing on a single test while you are debugging.
