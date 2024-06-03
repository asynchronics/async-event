# [Unreleased]
* Add `unexpected_cfgs` flag in `Cargo.toml` and accordingly decrease MSRV to
  1.64 (???).

# 0.2.0 (2024-05-15)

* Make it possible to specify a deadline when waiting for an event ([#1]).
* Increase the MSRV to work around breakage introduced by the new `--check-cfg`
  being enabled by default.

*Note*: there are no API-breaking changes, the minor version was only increased
due to the new MSRV.

[#1]: https://github.com/asynchronics/async-event/pull/1

# 0.1.0 (2023-07-16)

Initial release
