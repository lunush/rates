# Rates
![Usage example](images/example.svg)
Rates is a scriptable CLI tool that brings currency exchange rates right into your terminal and supports 30+ fiat and 10K+ crypto currencies.

## Installation
### Cargo
If you have [Rust](https://rustup.rs/) installed, you can get the binary by
running:
```sh
cargo install rates
```

### Releases
Alternatively, you can download a binary for your system from the
[releases](https://github.com/lunush/rates/releases) page

## TODO
* Switch to Yahoo and Binance
* Add tests
* Add `--period` flag that takes a period e.g. day, week, month, quarter, year, max, etc. Defaults to 1 day.
* Add `--difference` flag that return difference for a given period of time. Works only if `--period` is greater than 2 days.
* Add `--max` and `--min` flags that return maximum and minimum value for a given period
* Add `--close`, `--adjusted-close`, and `--open` that return close, adjusted close, or open value for a given period
* Add aliaces for symbols, e.g. bitcoin = btc, gold = xau
* (maybe) Add volume subcommand that:
  - By default, accepts one value and returns its volume if available.

## License
Apache 2.0 or MIT, at your choice.
