# Lab_OS

An embedded OS for RISC-V made with Rust.

## Clone and configure

```
git clone git@github.com:trmckay/lab_os.git
cd lab_os
make init
```

## Run

### Docker

Make sure you have Docker installed and running. Then run `make run-docker`.

### Baremetal

Make sure you have:

* A Rust toolchain
* `qemu-system-riscv`

Then run: `make run`.

## Documentation

The documentation is continuously built and available at
[https://www.trmckay.com/lab_os](https://www.trmckay.com/lab_os/index.html).

The preferred method for building locally is with Docker. Use it with `make docs`.

## Extras

### Settings for `rust-analyzer`

`rust-analyzer` may complain about missing components for `riscv64gc`. To only lint
the targets we care about, add one or both of the following configurations:

* For VSCode in `.vscode/settings.json`:

  ```json
  {
      "rust-analyzer.checkonsave.alltargets": false,
      "rust-analyzer.checkonsave.extraargs": [
          "--target",
          "riscv64gc-unknown-none-elf"
      ]
  }
  ```

* For `coc.nvim` in `.vim/coc-settings.json`:

  ```json
  {
      "rust.target": "riscv64gc-unknown-none-elf",
      "rust.all_targets": false
  }
  ```
