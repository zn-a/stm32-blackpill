# STM32F411CEU6 Black Pill

The **STM32F411CEU6** is a high-performance **Cortex-M4F-based microcontroller**.

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

## Memory Layout (`memory.x`)

The `memory.x` linker script defines the **Flash and RAM addresses** for the STM32F411:

```
MEMORY
{
  FLASH : ORIGIN = 0x08000000, LENGTH = 512K
  RAM   : ORIGIN = 0x20000000, LENGTH = 128K
}
```

# Programming the STM32F411 Black Pill

## Using ST-LINK V2 (SWD)

1. Ensure the correct Rust target is installed:

    ```shell
    rustup target add thumbv7em-none-eabihf
    ```
   
2. `Cargo.toml` should include:

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
   
3. `.cargo/config.toml` should include:

    ```toml
   [build]
   target = "thumbv7em-none-eabihf"
   
   [target.thumbv7em-none-eabihf]
   runner = "probe-rs run --chip STM32F411CEU6"
   rustflags = ["-C", "link-arg=-Tlink.x"]
    ```
   
4. Install the necessary tool for flashing:

   ```shell
   cargo install cargo-flash
   ```

5. Use `cargo flash` to program the STM32 via ST-LINK:

   ```shell
   cargo flash [--example <example_name>] --chip STM32F411CEU6
   ```

## Using the USB-C port

This method flashes firmware directly via USB-C, without an ST-LINK, using DFU bootloader mode.
