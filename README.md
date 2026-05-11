# STM32F411CEU6 Black Pill

The **STM32F411CEU6** is a high-performance **Cortex-M4F-based microcontroller**.

<img src="img/blackpill_board.png" alt="STM32F411CEU6 Black Pill" width="600"/>

## Specifications

| **Feature**                   | **Details**                 |
|-------------------------------|-----------------------------|
| **CPU Core**                  | ARM Cortex-M4F (100MHz)     |
| **Floating-Point Unit (FPU)** | Yes (Single-precision)      |
| **Flash Memory**              | 512 KB                      |
| **RAM**                       | 128 KB                      |
| **GPIO Pins**                 | 37 (5V-tolerant)            |
| **PWM Outputs**               | 12 (Supports Motor Control) |
| **I2C Interfaces**            | 3                           |
| **SPI Interfaces**            | 3                           |
| **UART Interfaces**           | 3                           |
| **USB Support**               | Full-Speed USB 2.0          |
| **Bootloader Support**        | DFU (USB) & UART            |
| **Power Supply**              | 3.3V                        |
| **Target**                    | `thumbv7em-none-eabihf`     |

## Memory Layout (`memory.x`)

The `memory.x` linker script defines the **Flash and RAM addresses** for the STM32F411:

```text
MEMORY
{
  FLASH : ORIGIN = 0x08000000, LENGTH = 512K
  RAM   : ORIGIN = 0x20000000, LENGTH = 128K
}
```

# Programming the STM32F411 Black Pill

This project is written in **Rust** and is meant to be flashed to the STM32F411CEU6 Black Pill.

There are two common ways to program the board:

1. **ST-LINK V2 over SWD**: recommended for development and debugging.
2. **USB-C using DFU bootloader mode**: useful when flashing directly through the board USB-C port.

## Using ST-LINK V2 (SWD)

### 1. Install the Rust target

```shell
rustup target add thumbv7em-none-eabihf
```

### 2. Project dependencies

`Cargo.toml` should include:

```toml
[dependencies]
cortex-m = "0.7.7"
cortex-m-rt = "0.7.5"
panic-halt = "1.0.0"

# STM32F411 HAL (Hardware Abstraction Layer)
[dependencies.stm32f4xx-hal]
version = "0.22.1"
features = ["stm32f411"]
```

### 3. Cargo configuration

`.cargo/config.toml` should include:

```toml
[build]
target = "thumbv7em-none-eabihf"

[target.thumbv7em-none-eabihf]
runner = "probe-rs run --chip STM32F411CEU6"
rustflags = ["-C", "link-arg=-Tlink.x"]
```

### 4. Install `probe-rs` tools

`cargo-flash` is now part of `probe-rs`. Do **not** install it with:

```shell
cargo install cargo-flash
```

On Windows PowerShell, install the `probe-rs` tools with:

```powershell
irm https://github.com/probe-rs/probe-rs/releases/latest/download/probe-rs-tools-installer.ps1 | iex
```

After installation, close and reopen PowerShell, then check:

```powershell
probe-rs --version
cargo flash --version
```

### 5. ST-LINK driver on Windows

If this command:

```powershell
probe-rs list
```

returns:

```text
No debug probes were found.
```

but **STM32 STLink** appears in Windows Device Manager with a yellow warning triangle, the ST-LINK driver is missing or
incorrect.

Fix it with **Zadig**:

1. Open Zadig.
2. Go to **Options > List All Devices**.
3. Select **STM32 STLink**.
4. Select **WinUSB** as the driver.
5. Click **Install Driver** or **Replace Driver**.
6. Unplug and replug the ST-LINK.
7. Run `probe-rs list` again.

![img.png](img/zadig_stlink_driver.png)

A working ST-LINK usually appears with USB ID:

```text
[0]: STLink V2 -- 0483:3748:3A001E00182D343632525544 (ST-LINK)
```

### 6. Connect the ST-LINK to the Black Pill

Use SWD wiring:

```text
ST-LINK V2      Black Pill
3.3V       ->   3.3V
GND        ->   GND
SWDIO      ->   SWDIO / DIO
SWCLK      ->   SWCLK / CLK
NRST       ->   RST   (optional)
```

Use **3.3V**, not 5V, unless you are sure your board expects 5V on that pin.

### 7. Build and flash an example

For normal flashing, use `cargo flash`:

```shell
cargo flash [--example <example_name>] --chip STM32F411CEUx
```

For example:

```shell
cargo flash --example blink --chip STM32F411CEUx
```

```shell
cargo flash --example fade_led --chip STM32F411CEUx
```

The physical chip on the board is **STM32F411CEU6**, but `probe-rs` uses the generic chip database name
**STM32F411CEUx**. Using `STM32F411CEU6` may still work, but it can give a wildcard matching warning.

You can also use the configured `probe-rs` runner:

```shell
cargo run --example blink
```

This also flashes the firmware, but it keeps the `probe-rs` session attached after programming. For an embedded program
such as `blink`, this can look like the command is hanging because the firmware runs forever in a loop. Stop it with
`Ctrl + C` when needed.

## Using the USB-C port

This method flashes firmware directly through the Black Pill USB-C port, without an ST-LINK, using the STM32 built-in
DFU bootloader.

This method is different from the ST-LINK method above. `probe-rs list` only detects debug probes such as ST-LINK,
J-Link, or CMSIS-DAP. It does **not** list the Black Pill itself when it is connected only by USB-C.

### 1. Install `dfu-util`

On Windows, `dfu-util` can be installed with Chocolatey:

```powershell
choco install dfu-util -y
```

After installation, close and reopen PowerShell, then check:

```powershell
dfu-util --version
```

### 2. Build the Rust example

Build the example in release mode:

```powershell
cargo build --example blink --release
```

### 3. Convert the firmware to a `.bin` file

Install the Rust binary tools if they are not installed yet:

```powershell
rustup component add llvm-tools-preview
cargo install cargo-binutils
```

Then convert the compiled firmware to a raw binary file:

```powershell
cargo objcopy --example blink --release -- -O binary blink.bin
```

Check that the file exists:

```powershell
dir blink.bin
```

### 4. Put the Black Pill into DFU mode

Disconnect the ST-LINK and connect the Black Pill directly to the computer with a USB-C data cable.

Then put the board into bootloader mode:

```text
Hold BOOT0
Press and release RESET / NRST
Release BOOT0
```

Windows should detect an STM32 DFU bootloader device. In Zadig or Device Manager, it may appear as:

```text
STM32 BOOTLOADER
```

The STM32 DFU bootloader usually has USB ID:

```text
0483:df11
```

### 5. Fix the DFU driver on Windows if needed

Check whether `dfu-util` can see the board:

```powershell
dfu-util -l
```

If the output contains this error:

```text
Cannot open DFU device 0483:df11 ... LIBUSB_ERROR_NOT_SUPPORTED
```

then Windows sees the STM32 bootloader, but the USB driver is wrong for `dfu-util`.

Fix it with **Zadig**:

1. Keep the board connected in DFU mode.
2. Open Zadig.
3. Go to **Options > List All Devices**.
4. Select **STM32 BOOTLOADER** or the device with USB ID **0483:df11**.
5. Select **WinUSB** as the driver.
6. Click **Install Driver** or **Replace Driver**.
7. Unplug and replug the board.
8. Put the board into DFU mode again.
9. Run `dfu-util -l` again.

A working STM32 DFU device should show entries like:

```text
Found DFU: [0483:df11] ... alt=0, name="@Internal Flash  /0x08000000/..."
```

Use `alt=0` for internal Flash.

### 6. Flash the `.bin` file over USB-C

If there is more than one DFU-capable USB device connected, `dfu-util` may show an error such as:

```text
More than one DFU capable USB device found!
```

In that case, specify the STM32 DFU device explicitly with `-d 0483:df11`:

```powershell
dfu-util -d 0483:df11 -a 0 -s 0x08000000:leave -D blink.bin
```

If needed, also specify the serial number shown by `dfu-util -l` with `-S`:

```powershell
dfu-util -d 0483:df11 -S <serial_number> -a 0 -s 0x08000000:leave -D blink.bin
```

For example:

```powershell
dfu-util -d 0483:df11 -S 368C35523133 -a 0 -s 0x08000000:leave -D blink.bin
```

The command writes `blink.bin` to the start of internal Flash at `0x08000000` and then leaves DFU mode.

A warning like this is usually not a problem for this raw binary workflow:

```text
Warning: Invalid DFU suffix signature
```

### 7. Run the firmware

After flashing, the board should leave DFU mode and run the firmware. If the LED does not start blinking, press the
RESET button once.
