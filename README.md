# Demo application for HAL and application testing

Targeting a custom board with an ATSAMD51N20A and external 8 MHz crystal

## Examples

Some examples will use `SERCOM0`UART for printouts.

Different examples found in the `bin/` folder

By default, a `clockv2` minimal demo application is run

```shell
cargo run
```

Current default clocking abstraction `clockv1`:

```shell
cargo run --bin demov1 --features clockv1
```

ICM - Integrity Check Module

ICM code is upstreamed, so no special HAL is required

```shell
cargo run --bin icm --features clockv1
```

```shell
cargo run --bin aes --features hal-aes
```
