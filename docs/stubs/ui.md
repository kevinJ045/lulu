# UI Stub

A simple stub made for making simple UI capable binaries with lulu using rust(egui library).

## Setup

To get started, you have to use the `lulu-ui` stub from [github](https://github.com/kevinj045/lulu-ui-stub/).

-   **First**, setup your environment. You might need to install some native libraries to make this work.
    <!-- tabs:start -->


    #### **Linux/Arch**
    ```bash
    sudo pacman -S mesa libdrm libglvnd libxkbcommon cairo fontconfig freetype zlib \
                wayland wayland-protocols libdecor alsa-lib dbus ibus systemd libusb
    ```

    #### **Linux/Debian**
    ```bash
    sudo apt install libgl1-mesa-dev libdrm-dev libglvnd-dev libxkbcommon-dev libcairo2-dev \
                 libfontconfig1-dev libfreetype6-dev zlib1g-dev \
                 libwayland-dev wayland-protocols libdecor-0-dev \
                 libasound2-dev libdbus-1-dev ibus systemd libusb-1.0-0-dev
    ```

    #### **Linux/Fedora**
    ```bash
    sudo dnf install mesa-libGL mesa-libGLU libdrm libglvnd libxkbcommon cairo fontconfig freetype zlib \
                wayland-devel wayland-protocols-devel libdecor-devel \
                alsa-lib dbus ibus systemd libusb1-devel
    ```

    #### **Linux/Nix**
    A simple `flake.nix`.
    ```nix
    {
      inputs = {
        utils.url = "github:numtide/flake-utils";
      };

      outputs = { self, nixpkgs, utils }:
        utils.lib.eachDefaultSystem (system:
          let
            pkgs = nixpkgs.legacyPackages.${system};
            libraries = with pkgs; [

              # Core graphics + rendering
              mesa
              libdrm
              libGL
              libGLU
              cairo
              libglvnd
              libxkbcommon
              fontconfig
              freetype
              zlib

              # Wayland
              wayland
              wayland-protocols
              libdecor

              # Audio
              alsa-lib
              # pulseaudio
              # pipewire
              # pipewire.jack

              # Input / misc
              dbus
              ibus
              systemd
              libusb1
            ];
          in
          {
            devShell = pkgs.mkShell {
              buildInputs = libraries;
              nativeBuildInputs = [ pkgs.pkg-config ];

              shellHook = ''
                export LD_LIBRARY_PATH=${pkgs.lib.makeLibraryPath libraries}:$LD_LIBRARY_PATH
                echo "DevShell ready with full X11/Wayland/EGL/Vulkan/Audio stack"
              '';
            };
          }
        );
    }
    ```


    #### **Windows**

    You should be good to go for windows by default, no requirements or anything.


    #### **Darwin**

    Macos has all the libraries needed by default, so just pass into the next step.

    <!-- tabs:end -->
    
-   **Second**, add this to your [`build` function](../reference/build-environment.md).
    ```lua
    build = function()
      -- ...
      stubs {
        -- Windows binary
        windows = "https://github.com/kevinJ045/lulu-ui-stub/releases/download/v0.1.34/lulu-ui.exe",

        -- Linux x86_64 bin
        ["linux-x86_64"] = "https://github.com/kevinJ045/lulu-ui-stub/releases/download/v0.1.34/lulu-ui-linux-x86_64",

        -- Linux aarch64 bin (never tried this)
        ["linux-aarch64"] = "https://github.com/kevinJ045/lulu-ui-stub/releases/download/v0.1.34/lulu-ui-linux-aarch64",

        -- Darwin Executable (never tried this either)
        darwin = "https://github.com/kevinJ045/lulu-ui-stub/releases/download/v0.1.34/lulu-ui-darwin"
      }
      -- ...
    end
    ```

-   **Third**, Add a demo code into your `main.lua` as:
    ```lua
    local btn = ui.Button { text = "hi" }

    btn:into_root()
    ```

-   **Last**, Run the binary as:
    ```bash
      # -b for build
      lulu run -b
    ```

## Basics

Now that you've setup you're environment, here's a few basics about the lulu UI api.

### Elements

Lulu UI comes with a few basic elements, they are accessible either through `ui.ElementName` or `lml`.

```lua
local instance = ui.Button { text = "Click Me!" }
-- or
local instance = lml! {
  <button text="Click Me!" />
}

-- use as:
instance:into_root()
```

### Available Elements:
#### Text Elements
| Element          | Props                              | Description                                                                  |
| ---------------- | ---------------------------------- | ---------------------------------------------------------------------------- |
| **ColoredLabel** | `text: string`, `color: {r,g,b,a}` | Displays text in the given RGBA color.                                       |
| **Label**        | `text: string`                     | Displays normal body text.                                                   |
| **Heading**      | `text: string`                     | Large bold heading.                                                          |
| **Small**        | `text: string`                     | Small text variant.                                                          |
| **Weak**         | `text: string`                     | Muted/low-contrast text.                                                     |
| **Strong**       | `text: string`                     | Bold/strong emphasis.                                                        |
| **Monospace**    | `text: string`                     | Displays text in a monospace font (for code).                                |
| **Hyperlink**    | `text: string`, `url?: string`     | Clickable hyperlink. If `url` provided → `hyperlink_to`, else → `hyperlink`. |
| **Link**         | `text: string`                     | Styled like a link, but not necessarily clickable.                           |
| **Code**         | `text: string`                     | Displays inline code-like text.                                              |

#### Interactive Elements
| Element         | Props                                                                                                                                                                          | Description                                                                |
| --------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | -------------------------------------------------------------------------- |
| **Button**      | `text: string`, `style?: table`                                                                                                                                                | Simple clickable button.                                                   |
| **Checkbox**    | `text: string`, `checked: boolean`                                                                                                                                             | Toggles a boolean value. `checked` is automatically updated.               |
| **Dragvalue**   | `text: string`, `min: number`, `max: number`, `value: number`                                                                                                                  | Numeric value with draggable input.                                        |
| **Slider**      | `text: string`, `value: number`                                                                                                                                                | Slider input (alias of `drag_value`).                                      |
| **Combobox**    | `text: string`, `selected: string`, `values: table`, `render_item?: fn`                                                                                                        | Dropdown selector. Returns the selected item.                              |
| **Input**       | `value: string`, `placeholder: string`, `multiline: bool`, `interactive: bool`, `frame: bool`, `code_editor: bool`, `password: bool`, `clip_text: bool`, `cursor_at_end: bool` | Text input field. Supports multiline, password mode, or code-editor style. |
| **CodeEditor**  | `text: string`                                                                                                                                                                 | Text editor specialized for code. Two-way binding via `handle_change`.     |
| **ProgressBar** | `value: number`, `text: string`                                                                                                                                                | Displays progress between `0.0` and `1.0`.                                 |

#### Visual Elements
| Element       | Props         | Description                              |
| ------------- | ------------- | ---------------------------------------- |
| **Separator** | *(none)*      | Horizontal separator line.               |
| **Spinner**   | *(none)*      | Animated loading spinner.                |
| **Image**     | `src: string` | Displays an image from the given source. |

#### Layout Containers
| Element                 | Props                                                                                                                                               | Description                                                                                                              |
| ----------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------ |
| **Align**               | `layout: string`, `align: string`                                                                                                                   | Aligns children using layout direction (`"left_to_right"`, `"top_down"`) and alignment (`"start"`, `"center"`, `"end"`). |
| **HBox**                | *(none)*                                                                                                                                            | Horizontal container (`ui:horizontal`).                                                                                  |
| **VBox**                | *(none)*                                                                                                                                            | Vertical container (`ui:vertical`).                                                                                      |
| **VBoxCentered**        | *(none)*                                                                                                                                            | Vertically centered container.                                                                                           |
| **VBoxCenterJustified** | *(none)*                                                                                                                                            | Vertically centered and justified container.                                                                             |
| **ScrollArea**          | `stick_to_right: bool`, `stick_to_bottom: bool`, `horizontal: bool`, `auto_shrink: bool`, `vscroll: bool`, `drag_to_scroll: bool`, `animated: bool` | Scrollable container.                                                                                                    |
| **CollapsingHeader**    | `text: string`                                                                                                                                      | Collapsible header containing nested content.                                                                            |
| **Frame**               | `style: table`                                                                                                                                      | Framed block with custom style around children.                                                                          |

#### Drawing / Custom Rendering
| Element     | Props                           | Description                                                                                              |
| ----------- | ------------------------------- | -------------------------------------------------------------------------------------------------------- |
| **Painter** | `render: function(painter, ui)` | Provides low-level access to `ui:painter()` for custom drawing. You can use this for lines, shapes, etc. |


#### Styles and Scopes
| Element    | Props                  | Description                                              |
| ---------- | ---------------------- | -------------------------------------------------------- |
| **Style**  | *(none)*               | Applies local style overrides to child elements.         |
| **Scope**  | `render: function(ui)` | Executes a function with a fresh UI scope.               |
| **Handle** | `render: function(ui)` | Inline function element (directly runs render function). |

#### Dynamic / Data-Driven Elements
| Element   | Props                                                             | Description                                                                                                     |
| --------- | ----------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------- |
| **Each**  | `items: table or Vec`, `render: function(item, index, array, ui)` | Iterates over items and renders children dynamically. Useful for loops/lists.                                   |
| **VList** | `items: Vec or table`, `render: function(item, index, array)`     | Stateful list that automatically rerenders when items change (reactive). Uses `State` and `Vec` under the hood. |


### Simple Usage:
```lua
ui.VBox {
  children = {
    ui.Heading { text = "Settings" },
    ui.Separator(),
    ui.Checkbox { text = "Enable shadows", checked = true },
    ui.Slider { text = "Volume", value = 0.75 },
    ui.HBox {
      children = {
        ui.Button { text = "Apply" },
        ui.Button { text = "Cancel" },
      }
    }
  }
}
```

## Using with function components

Now, let's render our elements through a function.

```lua
-- using the namespace is not necessary, but it strips
-- the need for prefixing everything with `ui`.
() @namespace(ui) =>

  -- Now, this would normally be a simple function that would
  -- throw an error if you tried to access `self` and it's not 
  -- passed.
  (self) AppRoot =>
    return Button { text = "Click Me!" }
  end

  -- So, we make it into a component as such:
  (self) @Component AppRoot =>
    return Button { text = "Click Me!" }
  end

  -- Now render as:
  AppRoot({}):into_root()

  -- If you are making the root component,
  -- you can just use `AutoRender` to auto
  -- render the component without calling it.
  (self) @AutoRender @Component AppRoot =>
    return Button { text = "Click Me!" }
  end
end
```

## Using States

States are simple value containers that let you change values dynamically from your UI nest.

#### Simple Data:
```lua
(self) @AutoRender @Component AppRoot =>
  local clicked = State(0)

  return Button {
    text = clicked:format("Clicked: {}"),
    on_clicked = function()
      clicked:add(1)
    end    
  }
end
```

#### Binding to input:
```lua
(self) @AutoRender @Component AppRoot =>
  local inputText = State("val")

  return VBox {
    children = {
      Label { text = inputText },
      Input { value = inputText }
    }
  }
end
```
```lua
(self) @AutoRender @Component AppRoot =>
  local ch = State(false)

  return VBox {
    children = {
      Label { text = "This text will be hidden", inactive = ch },
      Checkbox { checked = ch, text = "Hide text" }
    }
  }
end
```

## Component States

Instead of declaring each state in your main function, you can make it Component-Wide by using `StatedComponent` decorator.

```lua
(self) @AutoRender @StatedComponent({
  clicked = 0
}) @Component AppRoot =>
  return Button {
    text = self.clicked:format("Clicked: {}"),
    on_clicked = function()
      self.clicked:add(1)
    end    
  }
end
```

## Component Values

You may want static component properties, and for that you can use the `ComponentValues` decorator.

```lua
(self) @AutoRender @StatedComponent({
  selected = "red"
}) @ComponentValues({
  colors = { red = "Red", blue = "Blue", green = "Green" }
}) @Component AppRoot =>
  return VBox {
    children = {
      Label { text = self.selected:inside(self.colors) },
      Combobox { values = self.colors, selected = self.selected }
    }
  }
end
```