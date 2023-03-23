# Des

## About

Des is a simple tray application that spawns dummy processes to imitate hostile environment for viruses.
Those process names are identical to debugging tools, antiviruses and virtual machines.
Virus must believe that it is being debugged.

## Goals

* minimal dependency list;
* no unwraps or panics;

## How to compile

Assuming windows platform and PowerShell as a command line tool:
```
# compile the stub (dummy process)
cargo build --bin des-stub --release

# get SHA512 hash of the stub
Get-FileHash -Algorithm SHA512 -LiteralPath target\release\des-stub.exe | Select-Object -ExpandProperty Hash

# manually update the variable STUB_HASH in release.rs !!!

# compile resident app
cargo build --features "logger" --bin des-resident --release
```

## License

### Application GPLv3

### Icon - CC-BY Chenyu Wang